//! Tree-shaking analysis: determines which names from each module are needed
//! in the final bundle.

use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Expression, IdentifierReference,
    ImportDeclarationSpecifier, Statement, TSImportTypeQualifier, TSModuleDeclarationName,
    TSModuleReference, TSType, TSTypeName, TSTypeQuery, TSTypeQueryExprName,
};
use oxc_ast_visit::Visit;
use oxc_span::Ident;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::helpers::collect_statement_declaration_names;
use crate::scan_stage::ScanResult;
use crate::types::{ExportSource, ImportBindingKind, Module, ModuleIdx};

use super::exports::{
    collect_all_exported_names, collect_exported_names_from_program, resolve_export_local_name,
};
use super::types::{NeededKindFlags, NeededNamesPlan, NeededReason};

struct DeclarationNode {
    declared_names: Vec<String>,
    declared_root_symbols: FxHashMap<SymbolId, NeededKindFlags>,
    local_symbol_deps: FxHashMap<SymbolId, NeededKindFlags>,
    cross_module_deps: Vec<CrossModuleDep>,
    inline_import_deps: Vec<InlineImportDep>,
    is_always_retained: bool,
}

#[derive(Clone)]
struct CrossModuleDep {
    target_module_idx: ModuleIdx,
    target_name: Option<String>,
    reason: NeededReason,
}

#[derive(Clone)]
struct InlineImportDep {
    target_module_idx: ModuleIdx,
    target_name: Option<String>,
    kind: NeededKindFlags,
}

struct ModuleExpansion {
    expanded_names: FxHashSet<String>,
    needed_symbols: FxHashMap<SymbolId, NeededKindFlags>,
    cross_module_deps: Vec<CrossModuleDep>,
    inline_import_deps: Vec<InlineImportDep>,
}

pub fn build_needed_names(entry: &Module<'_>, scan_result: &ScanResult<'_>) -> NeededNamesPlan {
    let declaration_graphs = build_declaration_graphs(scan_result);
    let mut needed_names: FxHashMap<ModuleIdx, FxHashSet<String>> = FxHashMap::default();
    let mut whole_modules: FxHashSet<ModuleIdx> = FxHashSet::default();
    let mut reasons: FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>> = FxHashMap::default();
    let mut symbol_kinds: FxHashMap<ModuleIdx, Option<FxHashMap<SymbolId, NeededKindFlags>>> =
        FxHashMap::default();

    seed_entry_exports(entry, scan_result, &mut needed_names, &mut whole_modules, &mut reasons);
    propagate_entry_retained_dependencies(
        entry,
        declaration_graphs.get(&entry.idx).map_or(&[], Vec::as_slice),
        scan_result,
        &mut needed_names,
        &mut whole_modules,
        &mut reasons,
    );

    let mut changed = true;
    while changed {
        changed = false;
        changed |= propagate_needed_names(&mut needed_names, &mut reasons, scan_result);

        let current_needed = needed_names.clone();
        let current_whole = whole_modules.clone();
        let current_symbol_kinds = symbol_kinds.clone();
        let mut next_symbol_kinds: FxHashMap<
            ModuleIdx,
            Option<FxHashMap<SymbolId, NeededKindFlags>>,
        > = FxHashMap::default();
        let mut name_additions: Vec<(ModuleIdx, String, NeededReason)> = Vec::new();
        let mut whole_additions: Vec<(ModuleIdx, NeededReason)> = Vec::new();

        for module in &scan_result.modules {
            if module.is_entry {
                continue;
            }

            let direct_needed = current_needed.get(&module.idx);
            let is_whole = current_whole.contains(&module.idx) || module.has_augmentation;
            if direct_needed.is_none() && !is_whole {
                continue;
            }

            let expansion = expand_module_graph(
                module,
                declaration_graphs.get(&module.idx).map_or(&[], Vec::as_slice),
                direct_needed,
                is_whole,
                current_symbol_kinds.get(&module.idx).and_then(|entry| entry.as_ref()),
            );

            if is_whole {
                next_symbol_kinds.insert(module.idx, None);
            } else {
                next_symbol_kinds.insert(module.idx, Some(expansion.needed_symbols.clone()));
            }

            let before_names = current_needed.get(&module.idx);
            for name in &expansion.expanded_names {
                if before_names.is_none_or(|set| !set.contains(name.as_str())) {
                    name_additions.push((
                        module.idx,
                        name.clone(),
                        NeededReason::SemanticDependency,
                    ));
                }
            }

            for dep in expansion.cross_module_deps {
                match dep.target_name {
                    Some(name) => {
                        name_additions.push((dep.target_module_idx, name, dep.reason));
                    }
                    None => whole_additions.push((dep.target_module_idx, dep.reason)),
                }
            }

            for dep in expansion.inline_import_deps {
                if let Some(name) = dep.target_name {
                    name_additions.push((
                        dep.target_module_idx,
                        name,
                        NeededReason::InlineImportReference,
                    ));
                } else {
                    let _ = dep.kind;
                    whole_additions
                        .push((dep.target_module_idx, NeededReason::InlineImportReference));
                }
            }
        }

        symbol_kinds = next_symbol_kinds;

        for (module_idx, reason) in whole_additions {
            changed |= mark_module_whole_needed(
                &mut needed_names,
                &mut whole_modules,
                &mut reasons,
                module_idx,
                reason,
                scan_result,
            );
        }

        for (module_idx, name, reason) in name_additions {
            changed |= add_needed_name(&mut needed_names, &mut reasons, module_idx, &name, reason);
        }
    }

    let mut map: FxHashMap<ModuleIdx, Option<FxHashSet<SymbolId>>> = FxHashMap::default();
    for (&module_idx, names) in &needed_names {
        if whole_modules.contains(&module_idx) || scan_result.modules[module_idx].has_augmentation {
            map.insert(module_idx, None);
            symbol_kinds.insert(module_idx, None);
        } else {
            let module = &scan_result.modules[module_idx];
            let symbols = names
                .iter()
                .filter_map(|name| module.scoping.get_root_binding(Ident::from(name.as_str())))
                .collect();
            map.insert(module_idx, Some(symbols));
            symbol_kinds.entry(module_idx).or_insert_with(|| Some(FxHashMap::default()));
        }
    }

    NeededNamesPlan { map, symbol_kinds, reasons }
}

fn seed_entry_exports(
    entry: &Module<'_>,
    scan_result: &ScanResult<'_>,
    needed_names: &mut FxHashMap<ModuleIdx, FxHashSet<String>>,
    whole_modules: &mut FxHashSet<ModuleIdx>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
) {
    let entry_info = &entry.export_import_info;

    for export_entry in entry_info.named_exports.values() {
        if let ExportSource::SourceReexport { specifier, imported_name } = &export_entry.source
            && let Some(target_idx) = entry.resolve_internal_specifier(specifier)
        {
            let target_module = &scan_result.modules[target_idx];
            let name = resolve_export_local_name(target_module, imported_name)
                .unwrap_or_else(|| imported_name.clone());
            add_needed_name(
                needed_names,
                reasons,
                target_idx,
                &name,
                NeededReason::EntryNamedReexport,
            );
        }
    }

    for star in &entry_info.star_reexports {
        if let Some(target_idx) = entry.resolve_internal_specifier(&star.specifier) {
            if star.alias.is_some() {
                add_namespace_requirement(needed_names, reasons, target_idx, scan_result);
            } else {
                for name in collect_all_exported_names(target_idx, scan_result) {
                    add_needed_name(
                        needed_names,
                        reasons,
                        target_idx,
                        &name,
                        NeededReason::EntryStarReexport,
                    );
                }
            }
        }
    }

    let mut entry_exported_names: FxHashSet<String> = FxHashSet::default();
    for stmt in &entry.program.body {
        match stmt {
            Statement::ExportNamedDeclaration(decl)
                if decl.source.is_none() && decl.declaration.is_none() =>
            {
                for spec in &decl.specifiers {
                    entry_exported_names.insert(spec.local.name().to_string());
                }
            }
            Statement::ExportDefaultDeclaration(decl) => {
                if let ExportDefaultDeclarationKind::Identifier(id) = &decl.declaration {
                    entry_exported_names.insert(id.name.to_string());
                }
            }
            _ => {}
        }
    }

    if entry_exported_names.is_empty() {
        return;
    }

    for stmt in &entry.program.body {
        if let Statement::ImportDeclaration(import_decl) = stmt
            && let Some(target_idx) =
                entry.resolve_internal_specifier(import_decl.source.value.as_str())
            && let Some(specifiers) = &import_decl.specifiers
        {
            let has_reexport = specifiers.iter().any(|spec| match spec {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    entry_exported_names.contains(s.local.name.as_str())
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    entry_exported_names.contains(s.local.name.as_str())
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    entry_exported_names.contains(s.local.name.as_str())
                }
            });

            if !has_reexport {
                continue;
            }

            for spec in specifiers {
                match spec {
                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        let imported_name = s.imported.name().to_string();
                        let target_module = &scan_result.modules[target_idx];
                        let local_name = resolve_export_local_name(target_module, &imported_name)
                            .unwrap_or(imported_name);
                        add_needed_name(
                            needed_names,
                            reasons,
                            target_idx,
                            &local_name,
                            NeededReason::EntryNamedReexport,
                        );
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                        mark_module_whole_needed(
                            needed_names,
                            whole_modules,
                            reasons,
                            target_idx,
                            NeededReason::NamespaceRequirement,
                            scan_result,
                        );
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                        add_namespace_requirement(needed_names, reasons, target_idx, scan_result);
                    }
                }
            }
        }
    }
}

fn propagate_entry_retained_dependencies(
    _entry: &Module<'_>,
    nodes: &[DeclarationNode],
    scan_result: &ScanResult<'_>,
    needed_names: &mut FxHashMap<ModuleIdx, FxHashSet<String>>,
    whole_modules: &mut FxHashSet<ModuleIdx>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
) {
    for node in nodes {
        for dep in &node.cross_module_deps {
            match &dep.target_name {
                Some(name) => {
                    add_needed_name(needed_names, reasons, dep.target_module_idx, name, dep.reason);
                }
                None => {
                    mark_module_whole_needed(
                        needed_names,
                        whole_modules,
                        reasons,
                        dep.target_module_idx,
                        dep.reason,
                        scan_result,
                    );
                }
            }
        }

        for dep in &node.inline_import_deps {
            match &dep.target_name {
                Some(name) => {
                    add_needed_name(
                        needed_names,
                        reasons,
                        dep.target_module_idx,
                        name,
                        NeededReason::InlineImportReference,
                    );
                }
                None => {
                    mark_module_whole_needed(
                        needed_names,
                        whole_modules,
                        reasons,
                        dep.target_module_idx,
                        NeededReason::InlineImportReference,
                        scan_result,
                    );
                }
            }
        }
    }
}

fn add_namespace_requirement(
    needed_names: &mut FxHashMap<ModuleIdx, FxHashSet<String>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    target_idx: ModuleIdx,
    scan_result: &ScanResult<'_>,
) -> bool {
    let mut changed = false;
    for name in collect_all_exported_names(target_idx, scan_result) {
        changed |= add_needed_name(
            needed_names,
            reasons,
            target_idx,
            &name,
            NeededReason::NamespaceRequirement,
        );
    }
    changed
}

fn mark_module_whole_needed(
    needed_names: &mut FxHashMap<ModuleIdx, FxHashSet<String>>,
    whole_modules: &mut FxHashSet<ModuleIdx>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    module_idx: ModuleIdx,
    reason: NeededReason,
    scan_result: &ScanResult<'_>,
) -> bool {
    let mut changed = whole_modules.insert(module_idx);
    for name in collect_all_exported_names(module_idx, scan_result) {
        changed |= add_needed_name(needed_names, reasons, module_idx, &name, reason);
    }
    changed
}

fn add_needed_name(
    needed_names: &mut FxHashMap<ModuleIdx, FxHashSet<String>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    module_idx: ModuleIdx,
    name: &str,
    reason: NeededReason,
) -> bool {
    let inserted = needed_names.entry(module_idx).or_default().insert(name.to_string());
    add_needed_reason(reasons, module_idx, name, reason);
    inserted
}

fn add_needed_reason(
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    module_idx: ModuleIdx,
    name: &str,
    reason: NeededReason,
) {
    reasons.entry((module_idx, name.to_string())).or_default().insert(reason);
}

fn build_declaration_graphs(
    scan_result: &ScanResult<'_>,
) -> FxHashMap<ModuleIdx, Vec<DeclarationNode>> {
    scan_result
        .modules
        .iter()
        .map(|module| (module.idx, collect_declaration_nodes(module, scan_result)))
        .collect()
}

fn collect_declaration_nodes(
    module: &Module<'_>,
    scan_result: &ScanResult<'_>,
) -> Vec<DeclarationNode> {
    let root_scope_id = module.scoping.root_scope_id();
    let root_symbols: FxHashSet<SymbolId> =
        module.scoping.get_bindings(root_scope_id).values().copied().collect();
    let root_symbol_names: FxHashMap<SymbolId, String> = module
        .scoping
        .get_bindings(root_scope_id)
        .into_iter()
        .map(|(name, &symbol_id)| (symbol_id, name.to_string()))
        .collect();

    let mut nodes = Vec::new();

    for stmt in &module.program.body {
        let is_always_retained = statement_is_always_retained(stmt);
        let mut declared_names = Vec::new();
        collect_statement_declaration_names(stmt, &mut declared_names);
        if declared_names.is_empty()
            && let Statement::TSImportEqualsDeclaration(import_equals) = stmt
        {
            declared_names.push(import_equals.id.name.to_string());
        }
        if declared_names.is_empty() && !is_always_retained {
            continue;
        }

        let declared_kinds = statement_declaration_needed_kinds(stmt);
        if declared_kinds.is_empty() && !is_always_retained {
            continue;
        }

        let declared_root_symbols: FxHashMap<SymbolId, NeededKindFlags> = if declared_kinds
            .is_empty()
        {
            FxHashMap::default()
        } else {
            declared_names
                .iter()
                .filter_map(|name| {
                    let symbol_id = module.scoping.get_root_binding(Ident::from(name.as_str()))?;
                    let kinds = declared_kinds.intersection(NeededKindFlags::from_symbol_flags(
                        module.scoping.symbol_flags(symbol_id),
                    ));
                    (!kinds.is_empty()).then_some((symbol_id, kinds))
                })
                .collect()
        };

        let mut collector = RootReferenceCollector::new(&module.scoping, &root_symbols);
        collector.visit_statement(stmt);
        let mut referenced_root_symbols = collector.finish();

        let module_decl = match stmt {
            Statement::ExportNamedDeclaration(export_decl) => {
                match export_decl.declaration.as_ref() {
                    Some(Declaration::TSModuleDeclaration(module_decl)) => Some(module_decl),
                    _ => None,
                }
            }
            Statement::TSModuleDeclaration(module_decl) => Some(module_decl),
            _ => None,
        };
        if let Some(module_decl) = module_decl
            && let Some(scope_id) = module_decl.scope_id.get()
        {
            for name in module.scoping.get_bindings(scope_id).keys() {
                if let Some(root_symbol_id) =
                    module.scoping.get_root_binding(Ident::from(name.as_str()))
                {
                    let kinds = NeededKindFlags::from_symbol_flags(
                        module.scoping.symbol_flags(root_symbol_id),
                    );
                    if !kinds.is_empty() {
                        referenced_root_symbols
                            .entry(root_symbol_id)
                            .and_modify(|existing: &mut NeededKindFlags| {
                                *existing = existing.union(kinds);
                            })
                            .or_insert(kinds);
                    }
                }
            }
        }

        let mut local_symbol_deps = FxHashMap::default();
        let mut cross_module_deps = Vec::new();
        for (symbol_id, kind) in referenced_root_symbols {
            if module.scoping.symbol_flags(symbol_id).is_import() {
                let Some(local_name) = root_symbol_names.get(&symbol_id) else {
                    continue;
                };
                let Some(binding) =
                    module.export_import_info.named_imports.get(local_name.as_str())
                else {
                    local_symbol_deps
                        .entry(symbol_id)
                        .and_modify(|existing: &mut NeededKindFlags| {
                            *existing = existing.union(kind);
                        })
                        .or_insert(kind);
                    continue;
                };
                let Some(target_module_idx) =
                    module.resolve_internal_specifier(&binding.source_specifier)
                else {
                    continue;
                };

                let target_name = match &binding.kind {
                    ImportBindingKind::Named(imported_name) => {
                        let target_module = &scan_result.modules[target_module_idx];
                        Some(
                            resolve_export_local_name(target_module, imported_name)
                                .unwrap_or_else(|| imported_name.clone()),
                        )
                    }
                    ImportBindingKind::Default => {
                        let target_module = &scan_result.modules[target_module_idx];
                        Some(
                            resolve_export_local_name(target_module, "default")
                                .unwrap_or_else(|| "default".to_string()),
                        )
                    }
                    ImportBindingKind::Namespace => None,
                };
                cross_module_deps.push(CrossModuleDep {
                    target_module_idx,
                    target_name,
                    reason: NeededReason::CrossModuleImportDependency,
                });
            } else {
                local_symbol_deps
                    .entry(symbol_id)
                    .and_modify(|existing| {
                        *existing = existing.union(kind);
                    })
                    .or_insert(kind);
            }
        }

        nodes.push(DeclarationNode {
            declared_names,
            declared_root_symbols,
            local_symbol_deps,
            cross_module_deps,
            inline_import_deps: collect_inline_import_deps(stmt, module, scan_result),
            is_always_retained,
        });
    }

    nodes
}

fn expand_module_graph(
    module: &Module<'_>,
    nodes: &[DeclarationNode],
    direct_needed: Option<&FxHashSet<String>>,
    is_whole: bool,
    seeded_symbols: Option<&FxHashMap<SymbolId, NeededKindFlags>>,
) -> ModuleExpansion {
    let mut expanded_names = direct_needed.cloned().unwrap_or_default();
    let mut needed_symbols = seeded_symbols.cloned().unwrap_or_default();
    let mut cross_module_deps = Vec::new();
    let mut inline_import_deps = Vec::new();
    let mut activated = vec![false; nodes.len()];

    if !is_whole {
        for name in direct_needed.into_iter().flatten() {
            if let Some(symbol_id) = module.scoping.get_root_binding(Ident::from(name.as_str())) {
                if needed_symbols.contains_key(&symbol_id) {
                    continue;
                }
                let kinds = NeededKindFlags::ALL.intersection(NeededKindFlags::from_symbol_flags(
                    module.scoping.symbol_flags(symbol_id),
                ));
                if !kinds.is_empty() {
                    needed_symbols
                        .entry(symbol_id)
                        .and_modify(|existing| *existing = existing.union(kinds))
                        .or_insert(kinds);
                }
            }
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        for (index, node) in nodes.iter().enumerate() {
            let node_is_active = is_whole
                || node.is_always_retained
                || node.declared_root_symbols.iter().any(|(symbol_id, decl_kinds)| {
                    needed_symbols
                        .get(symbol_id)
                        .is_some_and(|needed_kinds| needed_kinds.intersects(*decl_kinds))
                });
            if !node_is_active {
                continue;
            }

            if !activated[index] {
                activated[index] = true;
                expanded_names.extend(node.declared_names.iter().cloned());
                cross_module_deps.extend(node.cross_module_deps.iter().cloned());
                inline_import_deps.extend(node.inline_import_deps.iter().cloned());
            }

            for (dep_symbol_id, dep_kinds) in &node.local_symbol_deps {
                let entry = needed_symbols.entry(*dep_symbol_id).or_insert(NeededKindFlags::NONE);
                let merged = entry.union(*dep_kinds);
                if merged != *entry {
                    *entry = merged;
                    changed = true;
                }
            }
        }
    }

    let mut declared_symbols: FxHashSet<SymbolId> = FxHashSet::default();
    for node in nodes {
        declared_symbols.extend(node.declared_root_symbols.keys().copied());
    }

    let root_scope_id = module.scoping.root_scope_id();
    for (name, &symbol_id) in module.scoping.get_bindings(root_scope_id) {
        if needed_symbols.contains_key(&symbol_id) && !declared_symbols.contains(&symbol_id) {
            expanded_names.insert(name.to_string());
        }
    }

    ModuleExpansion { expanded_names, needed_symbols, cross_module_deps, inline_import_deps }
}

fn propagate_needed_names(
    needed: &mut FxHashMap<ModuleIdx, FxHashSet<String>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    scan_result: &ScanResult<'_>,
) -> bool {
    use std::collections::VecDeque;

    let mut changed = false;
    let mut queue: VecDeque<ModuleIdx> = needed.keys().copied().collect();

    while let Some(module_idx) = queue.pop_front() {
        let module_needed = match needed.get(&module_idx) {
            Some(set) => set.clone(),
            None => continue,
        };

        if module_needed.is_empty() {
            continue;
        }

        let module = &scan_result.modules[module_idx];
        let info = &module.export_import_info;
        let locally_declared: FxHashSet<&str> =
            info.declared_export_names.iter().map(String::as_str).collect();

        let star_sources: Vec<ModuleIdx> = info
            .star_reexports
            .iter()
            .filter_map(|star| module.resolve_internal_specifier(&star.specifier))
            .collect();

        let mut named_reexports: Vec<(ModuleIdx, Vec<(String, String)>)> = Vec::new();
        let mut reexport_groups: FxHashMap<ModuleIdx, Vec<(String, String)>> = FxHashMap::default();
        for (exported_name, entry) in &info.named_exports {
            if let ExportSource::SourceReexport { specifier, imported_name } = &entry.source
                && let Some(target_idx) = module.resolve_internal_specifier(specifier)
            {
                let target_module = &scan_result.modules[target_idx];
                let local_name = resolve_export_local_name(target_module, imported_name)
                    .unwrap_or_else(|| imported_name.clone());
                reexport_groups
                    .entry(target_idx)
                    .or_default()
                    .push((local_name, exported_name.clone()));
            }
        }
        named_reexports.extend(reexport_groups);

        let mut unresolved: FxHashSet<String> = module_needed
            .iter()
            .filter(|name| !locally_declared.contains(name.as_str()))
            .cloned()
            .collect();

        for (sub_module_idx, specs) in &named_reexports {
            let matching: FxHashSet<String> = specs
                .iter()
                .filter(|(_, exported)| unresolved.contains(exported))
                .map(|(local, _)| local.clone())
                .collect();

            if matching.is_empty() {
                continue;
            }

            for local in &matching {
                add_needed_reason(
                    reasons,
                    *sub_module_idx,
                    local,
                    NeededReason::PropagationNamedReexport,
                );
            }
            for (_, exported) in specs {
                unresolved.remove(exported);
            }

            let entry = needed.entry(*sub_module_idx).or_default();
            let before = entry.len();
            entry.extend(matching);
            if entry.len() > before {
                changed = true;
                queue.push_back(*sub_module_idx);
            }
        }

        for sub_module_idx in &star_sources {
            let sub_exports = collect_exported_names_from_program(*sub_module_idx, scan_result);
            let matching: FxHashSet<String> =
                unresolved.intersection(&sub_exports).cloned().collect();
            if matching.is_empty() {
                continue;
            }

            for name in &matching {
                add_needed_reason(
                    reasons,
                    *sub_module_idx,
                    name,
                    NeededReason::PropagationStarReexport,
                );
                unresolved.remove(name);
            }

            let entry = needed.entry(*sub_module_idx).or_default();
            let before = entry.len();
            entry.extend(matching);
            if entry.len() > before {
                changed = true;
                queue.push_back(*sub_module_idx);
            }
        }
    }

    changed
}

struct RootReferenceCollector<'s> {
    scoping: &'s oxc_semantic::Scoping,
    root_symbols: &'s FxHashSet<SymbolId>,
    referenced_symbols: FxHashMap<SymbolId, NeededKindFlags>,
}

impl<'s> RootReferenceCollector<'s> {
    fn new(scoping: &'s oxc_semantic::Scoping, root_symbols: &'s FxHashSet<SymbolId>) -> Self {
        Self { scoping, root_symbols, referenced_symbols: FxHashMap::default() }
    }

    fn finish(self) -> FxHashMap<SymbolId, NeededKindFlags> {
        self.referenced_symbols
    }

    fn record_symbol(&mut self, symbol_id: SymbolId, kind: NeededKindFlags) {
        if !self.root_symbols.contains(&symbol_id) {
            return;
        }

        let flags = self.scoping.symbol_flags(symbol_id);
        let kind = if flags.is_import() {
            kind
        } else {
            let supported = NeededKindFlags::from_symbol_flags(flags);
            kind.intersection(supported)
        };
        if kind.is_empty() {
            return;
        }

        self.referenced_symbols
            .entry(symbol_id)
            .and_modify(|existing| *existing = existing.union(kind))
            .or_insert(kind);
    }

    fn record_root_binding(&mut self, name: &str, kind: NeededKindFlags) {
        if let Some(symbol_id) = self.scoping.get_root_binding(Ident::from(name)) {
            self.record_symbol(symbol_id, kind);
        }
    }

    fn record_identifier_reference(
        &mut self,
        ident: &IdentifierReference<'_>,
        kind: NeededKindFlags,
    ) {
        if let Some(reference_id) = ident.reference_id.get()
            && let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id()
        {
            self.record_symbol(symbol_id, kind);
            return;
        }

        self.record_root_binding(ident.name.as_str(), kind);
    }

    fn record_value_type_name(&mut self, type_name: &TSTypeName<'_>) {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                self.record_identifier_reference(ident, NeededKindFlags::VALUE);
            }
            TSTypeName::QualifiedName(name) => {
                self.record_value_type_name(&name.left);
            }
            TSTypeName::ThisExpression(expr) => {
                self.visit_this_expression(expr);
            }
        }
    }

    fn record_type_expression(&mut self, expression: &Expression<'_>) {
        match expression {
            Expression::Identifier(ident) => {
                self.record_identifier_reference(ident, NeededKindFlags::TYPE);
            }
            Expression::StaticMemberExpression(member) => {
                self.record_type_expression(&member.object);
            }
            _ => {}
        }
    }
}

impl<'a> Visit<'a> for RootReferenceCollector<'_> {
    fn visit_export_named_declaration(&mut self, decl: &oxc_ast::ast::ExportNamedDeclaration<'a>) {
        if let Some(declaration) = &decl.declaration {
            self.visit_declaration(declaration);
        } else if decl.source.is_none() {
            for specifier in &decl.specifiers {
                if let Some(name) = specifier.local.identifier_name() {
                    self.record_root_binding(name.as_str(), NeededKindFlags::ALL);
                }
            }
        }
    }

    fn visit_export_default_declaration(
        &mut self,
        decl: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if let ExportDefaultDeclarationKind::Identifier(ident) = &decl.declaration {
            self.record_root_binding(ident.name.as_str(), NeededKindFlags::ALL);
        } else {
            oxc_ast_visit::walk::walk_export_default_declaration(self, decl);
        }
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.record_identifier_reference(ident, NeededKindFlags::VALUE);
    }

    fn visit_ts_interface_heritage(&mut self, heritage: &oxc_ast::ast::TSInterfaceHeritage<'a>) {
        self.record_type_expression(&heritage.expression);
        if let Some(type_arguments) = &heritage.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    fn visit_ts_class_implements(&mut self, implements: &oxc_ast::ast::TSClassImplements<'a>) {
        self.visit_ts_type_name(&implements.expression);
        if let Some(type_arguments) = &implements.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    fn visit_ts_type_name(&mut self, type_name: &TSTypeName<'a>) {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                self.record_identifier_reference(ident, NeededKindFlags::TYPE);
            }
            TSTypeName::QualifiedName(name) => {
                self.visit_ts_type_name(&name.left);
            }
            TSTypeName::ThisExpression(expr) => {
                self.visit_this_expression(expr);
            }
        }
    }

    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        if let Some(type_name) = ty.expr_name.as_ts_type_name() {
            self.record_value_type_name(type_name);
            if let Some(type_arguments) = &ty.type_arguments {
                self.visit_ts_type_parameter_instantiation(type_arguments);
            }
        } else {
            oxc_ast_visit::walk::walk_ts_type_query(self, ty);
        }
    }
}

fn declaration_needed_kinds(decl: &Declaration<'_>) -> NeededKindFlags {
    match decl {
        Declaration::VariableDeclaration(_) | Declaration::FunctionDeclaration(_) => {
            NeededKindFlags::VALUE
        }
        Declaration::ClassDeclaration(_)
        | Declaration::TSEnumDeclaration(_)
        | Declaration::TSModuleDeclaration(_)
        | Declaration::TSImportEqualsDeclaration(_) => NeededKindFlags::ALL,
        Declaration::TSTypeAliasDeclaration(_) | Declaration::TSInterfaceDeclaration(_) => {
            NeededKindFlags::TYPE
        }
        Declaration::TSGlobalDeclaration(_) => NeededKindFlags::NONE,
    }
}

fn statement_declaration_needed_kinds(stmt: &Statement<'_>) -> NeededKindFlags {
    match stmt {
        Statement::ExportNamedDeclaration(export_decl) => {
            export_decl.declaration.as_ref().map_or(NeededKindFlags::NONE, declaration_needed_kinds)
        }
        Statement::ExportDefaultDeclaration(export_default) => match &export_default.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(_) => NeededKindFlags::VALUE,
            ExportDefaultDeclarationKind::ClassDeclaration(_) => NeededKindFlags::ALL,
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => NeededKindFlags::TYPE,
            _ => NeededKindFlags::NONE,
        },
        _ => stmt.as_declaration().map_or(NeededKindFlags::NONE, declaration_needed_kinds),
    }
}

struct InlineImportCollector<'m, 'a> {
    module: &'m Module<'a>,
    scan_result: &'m ScanResult<'a>,
    deps: Vec<InlineImportDep>,
}

impl InlineImportCollector<'_, '_> {
    fn record_target(
        &mut self,
        target_module_idx: ModuleIdx,
        target_name: Option<String>,
        kind: NeededKindFlags,
    ) {
        self.deps.push(InlineImportDep { target_module_idx, target_name, kind });
    }
}

impl<'a> Visit<'a> for InlineImportCollector<'_, 'a> {
    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        if let TSType::TSImportType(import_type) = it
            && let Some(target_module_idx) =
                self.module.resolve_internal_specifier(import_type.source.value.as_str())
        {
            let target_name = import_type.qualifier.as_ref().map(|qualifier| {
                let exported_name = extract_qualifier_first_name(qualifier);
                let target_module = &self.scan_result.modules[target_module_idx];
                resolve_export_local_name(target_module, &exported_name).unwrap_or(exported_name)
            });
            self.record_target(target_module_idx, target_name, NeededKindFlags::TYPE);
        }
        oxc_ast_visit::walk::walk_ts_type(self, it);
    }

    fn visit_ts_type_query(&mut self, it: &TSTypeQuery<'a>) {
        if let TSTypeQueryExprName::TSImportType(import_type) = &it.expr_name
            && let Some(target_module_idx) =
                self.module.resolve_internal_specifier(import_type.source.value.as_str())
        {
            let target_name = import_type.qualifier.as_ref().map(|qualifier| {
                let exported_name = extract_qualifier_first_name(qualifier);
                let target_module = &self.scan_result.modules[target_module_idx];
                resolve_export_local_name(target_module, &exported_name).unwrap_or(exported_name)
            });
            self.record_target(target_module_idx, target_name, NeededKindFlags::VALUE);
        }
        oxc_ast_visit::walk::walk_ts_type_query(self, it);
    }

    fn visit_ts_import_equals_declaration(
        &mut self,
        decl: &oxc_ast::ast::TSImportEqualsDeclaration<'a>,
    ) {
        if let TSModuleReference::ExternalModuleReference(ext) = &decl.module_reference
            && let Some(target_module_idx) =
                self.module.resolve_internal_specifier(ext.expression.value.as_str())
        {
            self.record_target(target_module_idx, None, NeededKindFlags::ALL);
        }
        oxc_ast_visit::walk::walk_ts_import_equals_declaration(self, decl);
    }
}

fn extract_qualifier_first_name(qualifier: &TSImportTypeQualifier<'_>) -> String {
    match qualifier {
        TSImportTypeQualifier::Identifier(ident) => ident.name.to_string(),
        TSImportTypeQualifier::QualifiedName(name) => extract_qualifier_first_name(&name.left),
    }
}

fn collect_inline_import_deps(
    stmt: &Statement<'_>,
    module: &Module<'_>,
    scan_result: &ScanResult<'_>,
) -> Vec<InlineImportDep> {
    let mut collector = InlineImportCollector { module, scan_result, deps: Vec::new() };
    collector.visit_statement(stmt);
    collector.deps
}

fn statement_is_always_retained(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::TSGlobalDeclaration(_) => true,
        Statement::TSModuleDeclaration(module_decl) => {
            module_declaration_is_augmentation(module_decl)
        }
        Statement::ExportNamedDeclaration(export_decl) => {
            export_decl.declaration.as_ref().is_some_and(declaration_is_always_retained)
        }
        _ => false,
    }
}

fn declaration_is_always_retained(decl: &Declaration<'_>) -> bool {
    match decl {
        Declaration::TSGlobalDeclaration(_) => true,
        Declaration::TSModuleDeclaration(module_decl) => {
            module_declaration_is_augmentation(module_decl)
        }
        _ => false,
    }
}

fn module_declaration_is_augmentation(module_decl: &oxc_ast::ast::TSModuleDeclaration<'_>) -> bool {
    matches!(module_decl.id, TSModuleDeclarationName::StringLiteral(_))
}
