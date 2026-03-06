//! Tree-shaking analysis: determines which names from each module are needed
//! in the final bundle.

use std::collections::hash_map::Entry;

use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Expression, IdentifierReference,
    ImportDeclarationSpecifier, Statement, TSImportTypeQualifier, TSModuleReference, TSType,
    TSTypeName, TSTypeQuery, TSTypeQueryExprName,
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

struct DeclarationDependency {
    declared_names: Vec<String>,
    declared_root_symbols: FxHashMap<SymbolId, NeededKindFlags>,
    referenced_root_symbols: FxHashMap<SymbolId, NeededKindFlags>,
}

/// Build a map of which names are needed from each module (tree-shaking).
///
/// Analyzes the entry module's re-exports to determine which names are needed
/// from each dependency. Returns a map where:
/// - `None` means all declarations are needed (e.g., `export * as ns from`)
/// - `Some(set)` means only the listed names are needed
/// - `Some(empty)` means nothing is needed (module should be tree-shaken out)
pub fn build_needed_names(entry: &Module<'_>, scan_result: &ScanResult<'_>) -> NeededNamesPlan {
    let mut needed = FxHashMap::default();
    let mut symbol_kinds = FxHashMap::default();
    let mut reasons: FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>> = FxHashMap::default();

    // Phase 1a: Process named source re-exports from the entry module.
    // e.g. `export { Foo } from "./mod"` or `export { Foo as Bar } from "./mod"`
    let entry_info = &entry.export_import_info;
    for export_entry in entry_info.named_exports.values() {
        if let ExportSource::SourceReexport { specifier, imported_name } = &export_entry.source
            && let Some(target_idx) = entry.resolve_internal_specifier(specifier)
        {
            let target_module = &scan_result.modules[target_idx];
            let name = resolve_export_local_name(target_module, imported_name)
                .unwrap_or_else(|| imported_name.clone());
            let slot = needed.entry(target_idx).or_insert_with(|| Some(FxHashSet::default()));
            if let Some(set) = slot {
                set.insert(name.clone());
                add_needed_reason(
                    &mut reasons,
                    target_idx,
                    &name,
                    NeededReason::EntryNamedReexport,
                );
            }
        }
    }

    // Phase 1b: Process star re-exports from the entry module.
    for star in &entry_info.star_reexports {
        if let Some(target_idx) = entry.resolve_internal_specifier(&star.specifier) {
            if star.alias.is_some() {
                // `export * as foo from "./mod"` — namespace re-export.
                add_namespace_requirement(&mut needed, &mut reasons, target_idx, scan_result);
            } else {
                // `export * from "./mod"` — compute the set of exported names
                let exported = collect_all_exported_names(target_idx, scan_result);
                let slot = needed.entry(target_idx).or_insert_with(|| Some(FxHashSet::default()));
                if let Some(set) = slot {
                    for name in exported {
                        set.insert(name.clone());
                        add_needed_reason(
                            &mut reasons,
                            target_idx,
                            &name,
                            NeededReason::EntryStarReexport,
                        );
                    }
                }
            }
        }
    }

    // Handle import-then-reexport patterns:
    //   import { X } from "./mod";
    //   export { X };
    // The above is equivalent to `export { X } from "./mod"` but the first loop
    // only handles the latter. Collect direct export names (no source) and match
    // them against internal import declarations.
    //
    // When an import declaration has at least one re-exported specifier, ALL
    // specifiers from that import are added to the needed set, because the
    // entry module keeps its full body and may use any imported name internally
    // (e.g., inside `declare global` blocks).
    {
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

        if !entry_exported_names.is_empty() {
            for stmt in &entry.program.body {
                if let Statement::ImportDeclaration(import_decl) = stmt
                    && let Some(target_idx) =
                        entry.resolve_internal_specifier(import_decl.source.value.as_str())
                    && let Some(specifiers) = &import_decl.specifiers
                {
                    // Check if any specifier in this import is re-exported
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

                    // Add ALL specifiers from this import to the needed set
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                let imported_name = s.imported.name().to_string();
                                let source_module = &scan_result.modules[target_idx];
                                let local_name =
                                    resolve_export_local_name(source_module, &imported_name)
                                        .unwrap_or(imported_name);
                                let entry = needed
                                    .entry(target_idx)
                                    .or_insert_with(|| Some(FxHashSet::default()));
                                if let Some(set) = entry {
                                    set.insert(local_name.clone());
                                    add_needed_reason(
                                        &mut reasons,
                                        target_idx,
                                        &local_name,
                                        NeededReason::EntryNamedReexport,
                                    );
                                }
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                                if let Entry::Vacant(slot) = needed.entry(target_idx) {
                                    slot.insert(None);
                                    for name in collect_all_exported_names(target_idx, scan_result)
                                    {
                                        add_needed_reason(
                                            &mut reasons,
                                            target_idx,
                                            &name,
                                            NeededReason::NamespaceRequirement,
                                        );
                                    }
                                }
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                                add_namespace_requirement(
                                    &mut needed,
                                    &mut reasons,
                                    target_idx,
                                    scan_result,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    let entry_decl_references = collect_referenced_root_symbols_in_declarations(entry);
    propagate_entry_declaration_import_references(
        entry,
        &entry_decl_references,
        &mut needed,
        &mut reasons,
        scan_result,
    );

    // Propagate needed names through intermediate modules.
    // If module M needs {"B"} and M does `export * from "./sub"`,
    // check if sub provides B and add sub → {"B"} if so.
    propagate_needed_names(&mut needed, &mut reasons, scan_result);

    // Fixpoint loop: expand semantically within each module, then propagate
    // cross-module import dependencies. Repeat until no new names are added.
    //
    // This handles the case where module A needs `Foo`, `Foo`'s declaration
    // references `Bar` (semantic expansion), `Bar` is imported from module B
    // (cross-module propagation), and module B then needs `Bar` expanded too.
    loop {
        // Expand each module's needed set semantically (root symbol dependency closure).
        for module in &scan_result.modules {
            let Some(Some(direct_needed)) = needed.get(&module.idx) else {
                continue;
            };
            let direct_needed = direct_needed.clone();
            let (expanded, expanded_symbol_kinds) =
                expand_needed_names_semantic(module, &direct_needed);
            for name in expanded.iter().filter(|name| !direct_needed.contains(*name)) {
                add_needed_reason(&mut reasons, module.idx, name, NeededReason::SemanticDependency);
            }
            symbol_kinds.insert(module.idx, Some(expanded_symbol_kinds));
            needed.insert(module.idx, Some(expanded));
        }

        // Propagate cross-module import dependencies: if a module's expanded
        // needed set includes names that are imports from internal modules,
        // add those to the source module's needed set.
        let mut changed = propagate_import_dependencies(&mut needed, &mut reasons, scan_result);
        changed |= propagate_entry_declaration_import_references(
            entry,
            &entry_decl_references,
            &mut needed,
            &mut reasons,
            scan_result,
        );
        changed |= propagate_inline_import_references(&mut needed, &mut reasons, scan_result);

        if !changed {
            break;
        }

        // Re-propagate through re-export chains since new modules may have entries.
        propagate_needed_names(&mut needed, &mut reasons, scan_result);
    }

    for (&module_idx, needed_names) in &needed {
        match needed_names {
            None => {
                symbol_kinds.insert(module_idx, None);
            }
            Some(_) => {
                symbol_kinds.entry(module_idx).or_insert_with(|| Some(FxHashMap::default()));
            }
        }
    }

    NeededNamesPlan { map: needed, symbol_kinds, reasons }
}

fn add_namespace_requirement(
    needed: &mut FxHashMap<ModuleIdx, Option<FxHashSet<String>>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    target_idx: ModuleIdx,
    scan_result: &ScanResult<'_>,
) {
    let skip_default_local = matches!(needed.get(&target_idx), Some(Some(set)) if !set.is_empty());
    let default_local = skip_default_local
        .then(|| resolve_export_local_name(&scan_result.modules[target_idx], "default"))
        .flatten();

    let exported = collect_all_exported_names(target_idx, scan_result);
    let entry = needed.entry(target_idx).or_insert_with(|| Some(FxHashSet::default()));
    if let Some(set) = entry {
        for name in exported {
            if default_local.as_deref() == Some(name.as_str()) {
                continue;
            }
            set.insert(name.clone());
            add_needed_reason(reasons, target_idx, &name, NeededReason::NamespaceRequirement);
        }
    }
}

fn add_needed_reason(
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    module_idx: ModuleIdx,
    name: &str,
    reason: NeededReason,
) {
    reasons.entry((module_idx, name.to_string())).or_default().insert(reason);
}

fn add_partial_needed_name(
    needed: &mut FxHashMap<ModuleIdx, Option<FxHashSet<String>>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    module_idx: ModuleIdx,
    name: &str,
    reason: NeededReason,
) -> bool {
    let entry = needed.entry(module_idx).or_insert_with(|| Some(FxHashSet::default()));
    let inserted = entry.as_mut().is_some_and(|set| set.insert(name.to_string()));
    add_needed_reason(reasons, module_idx, name, reason);
    inserted
}

fn propagate_entry_declaration_import_references(
    entry: &Module<'_>,
    entry_decl_references: &FxHashSet<SymbolId>,
    needed: &mut FxHashMap<ModuleIdx, Option<FxHashSet<String>>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    scan_result: &ScanResult<'_>,
) -> bool {
    if entry_decl_references.is_empty() {
        return false;
    }

    let mut changed = false;

    for stmt in &entry.program.body {
        if let Statement::ImportDeclaration(import_decl) = stmt
            && let Some(target_idx) =
                entry.resolve_internal_specifier(import_decl.source.value.as_str())
            && let Some(specifiers) = &import_decl.specifiers
        {
            for spec in specifiers {
                match spec {
                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        let Some(symbol_id) = s.local.symbol_id.get() else {
                            continue;
                        };
                        if !entry_decl_references.contains(&symbol_id) {
                            continue;
                        }

                        let imported_name = s.imported.name().to_string();
                        let source_module = &scan_result.modules[target_idx];
                        let local_name = resolve_export_local_name(source_module, &imported_name)
                            .unwrap_or(imported_name);
                        changed |= add_partial_needed_name(
                            needed,
                            reasons,
                            target_idx,
                            &local_name,
                            NeededReason::CrossModuleImportDependency,
                        );
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                        let Some(symbol_id) = s.local.symbol_id.get() else {
                            continue;
                        };
                        if !entry_decl_references.contains(&symbol_id) {
                            continue;
                        }

                        let source_module = &scan_result.modules[target_idx];
                        let local_name = resolve_export_local_name(source_module, "default")
                            .unwrap_or_else(|| "default".to_string());
                        changed |= add_partial_needed_name(
                            needed,
                            reasons,
                            target_idx,
                            &local_name,
                            NeededReason::CrossModuleImportDependency,
                        );
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                        let Some(symbol_id) = s.local.symbol_id.get() else {
                            continue;
                        };
                        if !entry_decl_references.contains(&symbol_id) {
                            continue;
                        }

                        let before_len = needed
                            .get(&target_idx)
                            .and_then(|entry| entry.as_ref())
                            .map_or(0, FxHashSet::len);
                        add_namespace_requirement(needed, reasons, target_idx, scan_result);
                        let after_len = needed
                            .get(&target_idx)
                            .and_then(|entry| entry.as_ref())
                            .map_or(0, FxHashSet::len);
                        if after_len > before_len {
                            changed = true;
                        }
                    }
                }
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

        // For import bindings, record with the requested kind regardless of
        // whether the import is type-only. The actual kind resolution happens
        // at the source module during cross-module propagation. This handles
        // cases like `typeof Foo` referencing a type-only import of a value
        // declaration (`import { type Foo }` where Foo is `declare const`).
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
            .and_modify(|flags| *flags = flags.union(kind))
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

fn collect_referenced_root_symbols_in_declarations(module: &Module<'_>) -> FxHashSet<SymbolId> {
    let root_scope_id = module.scoping.root_scope_id();
    let root_symbols: FxHashSet<SymbolId> =
        module.scoping.get_bindings(root_scope_id).values().copied().collect();
    let mut references = FxHashSet::default();

    for stmt in &module.program.body {
        let mut declared_names = Vec::new();
        collect_statement_declaration_names(stmt, &mut declared_names);
        // Also visit module augmentation blocks (`declare module "..."`) which
        // have string literal names and thus produce no declared_names, but
        // can still reference imported types that need to be pulled in.
        let is_module_augmentation = is_module_augmentation_stmt(stmt);
        if declared_names.is_empty() && !is_module_augmentation {
            continue;
        }

        let mut collector = RootReferenceCollector::new(&module.scoping, &root_symbols);
        collector.visit_statement(stmt);
        references.extend(collector.finish().into_keys());
    }

    references
}

/// Check if a statement is a module augmentation (`declare module "..."`)
/// with a string literal name (as opposed to a namespace declaration with
/// an identifier name like `declare module Foo { ... }`).
fn is_module_augmentation_stmt(stmt: &Statement<'_>) -> bool {
    let module_decl = match stmt {
        Statement::TSModuleDeclaration(decl) => Some(decl.as_ref()),
        Statement::ExportNamedDeclaration(export_decl) => {
            if let Some(Declaration::TSModuleDeclaration(decl)) = &export_decl.declaration {
                Some(decl.as_ref())
            } else {
                None
            }
        }
        _ => None,
    };
    module_decl.is_some_and(|decl| decl.id.is_string_literal())
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

fn collect_declaration_dependencies(module: &Module<'_>) -> Vec<DeclarationDependency> {
    let root_scope_id = module.scoping.root_scope_id();
    let root_symbols: FxHashSet<SymbolId> =
        module.scoping.get_bindings(root_scope_id).values().copied().collect();

    let mut dependencies = Vec::new();
    for stmt in &module.program.body {
        let mut declared_names = Vec::new();
        collect_statement_declaration_names(stmt, &mut declared_names);
        if declared_names.is_empty() {
            continue;
        }
        let declared_kinds = statement_declaration_needed_kinds(stmt);
        if declared_kinds.is_empty() {
            continue;
        }

        let declared_root_symbols: FxHashMap<SymbolId, NeededKindFlags> = declared_names
            .iter()
            .filter_map(|name| {
                let symbol_id = module.scoping.get_root_binding(Ident::from(name.as_str()))?;
                let kinds = declared_kinds.intersection(NeededKindFlags::from_symbol_flags(
                    module.scoping.symbol_flags(symbol_id),
                ));
                (!kinds.is_empty()).then_some((symbol_id, kinds))
            })
            .collect();

        let mut collector = RootReferenceCollector::new(&module.scoping, &root_symbols);
        collector.visit_statement(stmt);
        let mut referenced_root_symbols = collector.finish();

        // Compatibility: when a needed namespace declaration defines names that shadow
        // root bindings, retain those root declarations as well.
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
                            .and_modify(|existing| *existing = existing.union(kinds))
                            .or_insert(kinds);
                    }
                }
            }
        }

        dependencies.push(DeclarationDependency {
            declared_names,
            declared_root_symbols,
            referenced_root_symbols,
        });
    }
    dependencies
}

fn expand_needed_names_semantic(
    module: &Module<'_>,
    direct: &FxHashSet<String>,
) -> (FxHashSet<String>, FxHashMap<SymbolId, NeededKindFlags>) {
    let dependencies = collect_declaration_dependencies(module);

    let mut needed_names: FxHashSet<String> = FxHashSet::default();
    let mut needed_symbols: FxHashMap<SymbolId, NeededKindFlags> = FxHashMap::default();

    for name in direct {
        if let Some(symbol_id) = module.scoping.get_root_binding(Ident::from(name.as_str())) {
            let kinds = NeededKindFlags::ALL.intersection(NeededKindFlags::from_symbol_flags(
                module.scoping.symbol_flags(symbol_id),
            ));
            if !kinds.is_empty() {
                needed_symbols
                    .entry(symbol_id)
                    .and_modify(|existing| *existing = existing.union(kinds))
                    .or_insert(kinds);
            }
        } else {
            // Preserve direct unresolved names, but do not perform non-semantic expansion.
            needed_names.insert(name.clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        for dep in &dependencies {
            let declaration_is_needed =
                dep.declared_root_symbols.iter().any(|(symbol_id, decl_kinds)| {
                    needed_symbols
                        .get(symbol_id)
                        .is_some_and(|needed_kinds| needed_kinds.intersects(*decl_kinds))
                });
            if !declaration_is_needed {
                continue;
            }

            needed_names.extend(dep.declared_names.iter().cloned());

            for (dep_symbol_id, dep_kinds) in &dep.referenced_root_symbols {
                let entry = needed_symbols.entry(*dep_symbol_id).or_insert(NeededKindFlags::NONE);
                let merged = entry.union(*dep_kinds);
                if merged != *entry {
                    *entry = merged;
                    changed = true;
                }
            }
        }
    }

    // Collect all symbols covered by declaration entries.
    let mut declared_symbols: FxHashSet<SymbolId> = FxHashSet::default();
    for dep in &dependencies {
        declared_symbols.extend(dep.declared_root_symbols.keys().copied());
    }

    // For needed symbols not covered by declarations (e.g., import bindings),
    // resolve their names from root scope bindings so they can be propagated
    // cross-module by `propagate_import_dependencies`.
    let root_scope_id = module.scoping.root_scope_id();
    for (name, &sym) in module.scoping.get_bindings(root_scope_id) {
        if needed_symbols.contains_key(&sym) && !declared_symbols.contains(&sym) {
            needed_names.insert(name.to_string());
        }
    }

    (needed_names, needed_symbols)
}

/// Propagate needed names transitively through re-export chains.
///
/// For each module in the map that has specific needed names (Some(set)),
/// check if those names come from `export * from "..."` or `export { ... } from "..."`
/// re-exports, and propagate to the source modules.
fn propagate_needed_names(
    needed: &mut FxHashMap<ModuleIdx, Option<FxHashSet<String>>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    scan_result: &ScanResult<'_>,
) {
    use std::collections::VecDeque;

    let mut queue: VecDeque<ModuleIdx> = needed.keys().copied().collect();

    while let Some(module_idx) = queue.pop_front() {
        let module_needed = match needed.get(&module_idx) {
            Some(Some(set)) => set.clone(),
            Some(None) | None => continue, // all needed, nothing to refine
        };

        if module_needed.is_empty() {
            continue;
        }

        // Use pre-computed export/import maps instead of walking the AST.
        let module = &scan_result.modules[module_idx];
        let info = &module.export_import_info;

        // Locally declared export names (from `export <declaration>`)
        let locally_declared: FxHashSet<&str> =
            info.declared_export_names.iter().map(String::as_str).collect();

        // Collect star re-export targets
        let star_sources: Vec<ModuleIdx> = info
            .star_reexports
            .iter()
            .filter_map(|star| module.resolve_internal_specifier(&star.specifier))
            .collect();

        // Collect named re-exports: (target_idx, [(local_in_target, exported_name)])
        let mut named_reexports: Vec<(ModuleIdx, Vec<(String, String)>)> = Vec::new();
        // Group source re-exports by specifier → target module
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

        // For names not locally declared, they must come from re-exports.
        // Propagate to sub-modules.
        let mut unresolved: FxHashSet<String> = module_needed
            .iter()
            .filter(|n| !locally_declared.contains(n.as_str()))
            .cloned()
            .collect();

        // Check named re-exports: `export { X } from "./sub"`
        for (sub_path, specs) in &named_reexports {
            let matching: FxHashSet<String> = specs
                .iter()
                .filter(|(_, exported)| unresolved.contains(exported))
                .map(|(local, _)| local.clone())
                .collect();
            if matching.is_empty() {
                // Named re-export source doesn't contribute any needed names;
                // mark as empty so generate_module knows nothing is needed.
                needed.entry(*sub_path).or_insert_with(|| Some(FxHashSet::default()));
            } else {
                for local in &matching {
                    add_needed_reason(
                        reasons,
                        *sub_path,
                        local,
                        NeededReason::PropagationNamedReexport,
                    );
                }
                for (_, exported) in specs {
                    unresolved.remove(exported);
                }
                let entry = needed.entry(*sub_path).or_insert_with(|| Some(FxHashSet::default()));
                let mut new_module = false;
                if let Some(set) = entry {
                    let before = set.len();
                    set.extend(matching);
                    new_module = set.len() > before;
                }
                if new_module {
                    queue.push_back(*sub_path);
                }
            }
        }

        // Check export * sources for remaining unresolved names,
        // and mark non-matching star sources as empty (not needed).
        for sub_path in &star_sources {
            // Look up sub-module to find which names it exports
            let sub_exports = collect_exported_names_from_program(*sub_path, scan_result);
            let matching: FxHashSet<String> =
                unresolved.intersection(&sub_exports).cloned().collect();
            if matching.is_empty() {
                // Star source doesn't contribute any needed names;
                // explicitly mark as empty so generate_module knows
                // nothing is needed from it.
                needed.entry(*sub_path).or_insert_with(|| Some(FxHashSet::default()));
            } else {
                for name in &matching {
                    add_needed_reason(
                        reasons,
                        *sub_path,
                        name,
                        NeededReason::PropagationStarReexport,
                    );
                }
                for name in &matching {
                    unresolved.remove(name);
                }
                let entry = needed.entry(*sub_path).or_insert_with(|| Some(FxHashSet::default()));
                let mut new_module = false;
                if let Some(set) = entry {
                    let before = set.len();
                    set.extend(matching);
                    new_module = set.len() > before;
                }
                if new_module {
                    queue.push_back(*sub_path);
                }
            }
        }
    }
}

/// Propagate needed names through import dependencies.
///
/// After semantic expansion, a module's needed set may include names that are
/// imports from internal modules (not local declarations). These need to be
/// propagated to the source modules so their declarations aren't tree-shaken.
///
/// Returns `true` if any new names were added to any module.
fn propagate_import_dependencies(
    needed: &mut FxHashMap<ModuleIdx, Option<FxHashSet<String>>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    scan_result: &ScanResult<'_>,
) -> bool {
    let mut changed = false;

    // Collect all propagation targets first to avoid borrow conflicts.
    let mut additions: Vec<(ModuleIdx, String)> = Vec::new();
    let mut all_needed: Vec<ModuleIdx> = Vec::new();

    for module in &scan_result.modules {
        let Some(Some(needed_set)) = needed.get(&module.idx) else {
            continue;
        };
        let needed_set = needed_set.clone();
        let info = &module.export_import_info;

        // Use pre-computed import maps instead of walking import declarations.
        for (local_name, binding) in &info.named_imports {
            if !needed_set.contains(local_name.as_str()) {
                continue;
            }
            let Some(target_idx) = module.resolve_internal_specifier(&binding.source_specifier)
            else {
                continue;
            };

            match &binding.kind {
                ImportBindingKind::Named(imported_name) => {
                    let target_module = &scan_result.modules[target_idx];
                    let local_in_target = resolve_export_local_name(target_module, imported_name)
                        .unwrap_or_else(|| imported_name.clone());
                    additions.push((target_idx, local_in_target));
                }
                ImportBindingKind::Default => {
                    if needed.get(&target_idx) == Some(&None) {
                        continue;
                    }
                    let target_module = &scan_result.modules[target_idx];
                    let local_in_target = resolve_export_local_name(target_module, "default")
                        .unwrap_or_else(|| "default".to_string());
                    additions.push((target_idx, local_in_target));
                }
                ImportBindingKind::Namespace => {
                    all_needed.push(target_idx);
                }
            }
        }
    }

    // Apply namespace (all-needed) additions
    for target_idx in all_needed {
        if needed.get(&target_idx) != Some(&None) {
            needed.insert(target_idx, None);
            changed = true;
        }
    }

    // Apply named additions
    for (target_idx, local_name) in additions {
        let entry = needed.entry(target_idx).or_insert_with(|| Some(FxHashSet::default()));
        if let Some(set) = entry
            && set.insert(local_name.clone())
        {
            changed = true;
            add_needed_reason(
                reasons,
                target_idx,
                &local_name,
                NeededReason::CrossModuleImportDependency,
            );
        }
    }

    changed
}

/// Extract the leftmost identifier from a `TSImportTypeQualifier`.
///
/// For `import("./dep").Missing.Sub`, returns `"Missing"`.
fn extract_qualifier_first_name(qualifier: &TSImportTypeQualifier<'_>) -> String {
    match qualifier {
        TSImportTypeQualifier::Identifier(ident) => ident.name.to_string(),
        TSImportTypeQualifier::QualifiedName(q) => extract_qualifier_first_name(&q.left),
    }
}

/// Visitor that collects inline import type references from AST nodes.
///
/// Finds `import("./dep").Qualifier` patterns and records which symbols are
/// needed from which target modules.
struct InlineImportRefCollector<'m, 'a, 'out> {
    module: &'m Module<'a>,
    scan_result: &'m ScanResult<'a>,
    additions: &'out mut Vec<(ModuleIdx, String)>,
    whole_needed: &'out mut Vec<ModuleIdx>,
}

impl<'a> Visit<'a> for InlineImportRefCollector<'_, 'a, '_> {
    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        if let TSType::TSImportType(import_type) = it
            && let Some(target_idx) =
                self.module.resolve_internal_specifier(import_type.source.value.as_str())
        {
            if let Some(qualifier) = &import_type.qualifier {
                let first_name = extract_qualifier_first_name(qualifier);
                let target_module = &self.scan_result.modules[target_idx];
                let local_name =
                    resolve_export_local_name(target_module, &first_name).unwrap_or(first_name);
                self.additions.push((target_idx, local_name));
            } else {
                self.whole_needed.push(target_idx);
            }
        }
        oxc_ast_visit::walk::walk_ts_type(self, it);
    }

    fn visit_ts_type_query(&mut self, it: &TSTypeQuery<'a>) {
        if let TSTypeQueryExprName::TSImportType(import_type) = &it.expr_name
            && let Some(target_idx) =
                self.module.resolve_internal_specifier(import_type.source.value.as_str())
        {
            if let Some(qualifier) = &import_type.qualifier {
                let first_name = extract_qualifier_first_name(qualifier);
                let target_module = &self.scan_result.modules[target_idx];
                let local_name =
                    resolve_export_local_name(target_module, &first_name).unwrap_or(first_name);
                self.additions.push((target_idx, local_name));
            } else {
                self.whole_needed.push(target_idx);
            }
        }
        oxc_ast_visit::walk::walk_ts_type_query(self, it);
    }

    fn visit_ts_import_equals_declaration(
        &mut self,
        decl: &oxc_ast::ast::TSImportEqualsDeclaration<'a>,
    ) {
        if let TSModuleReference::ExternalModuleReference(ext) = &decl.module_reference
            && let Some(target_idx) =
                self.module.resolve_internal_specifier(ext.expression.value.as_str())
        {
            self.whole_needed.push(target_idx);
        }
        oxc_ast_visit::walk::walk_ts_import_equals_declaration(self, decl);
    }
}

/// Propagate needed names through inline import type references.
///
/// When a module's retained declarations contain `import("./dep").Missing`,
/// this adds `Missing` to `./dep`'s needed set. For `import X = require("./dep")`,
/// it marks the target module as whole-needed.
///
/// Returns `true` if any new names were added.
fn propagate_inline_import_references(
    needed: &mut FxHashMap<ModuleIdx, Option<FxHashSet<String>>>,
    reasons: &mut FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
    scan_result: &ScanResult<'_>,
) -> bool {
    let mut changed = false;
    let mut additions: Vec<(ModuleIdx, String)> = Vec::new();
    let mut whole_needed: Vec<ModuleIdx> = Vec::new();

    for module in &scan_result.modules {
        if !module.is_entry && !needed.contains_key(&module.idx) {
            continue;
        }

        let mut collector = InlineImportRefCollector {
            module,
            scan_result,
            additions: &mut additions,
            whole_needed: &mut whole_needed,
        };
        collector.visit_program(&module.program);
    }

    for target_idx in whole_needed {
        if needed.get(&target_idx) != Some(&None) {
            needed.insert(target_idx, None);
            changed = true;
        }
    }

    for (target_idx, name) in additions {
        changed |= add_partial_needed_name(
            needed,
            reasons,
            target_idx,
            &name,
            NeededReason::InlineImportReference,
        );
    }

    changed
}
