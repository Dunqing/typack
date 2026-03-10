//! Per-module link metadata computation.
//!
//! Determines per-statement inclusion decisions and collects import resolution
//! data that the generate stage consumes. This moves the "what to include"
//! analysis out of generate and into link, aligning with Rolldown's architecture.

use oxc_ast::ast::{ExportDefaultDeclaration, ExportDefaultDeclarationKind, Statement};
use oxc_ast_visit::Visit;
use oxc_index::IndexVec;
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::link_stage::NeededKindFlags;
use crate::link_stage::exports::resolve_export_local_name;
use crate::scan_stage::ScanStageOutput;
use crate::types::ModuleIdx;

use super::types::{CanonicalNames, ModuleLinkMeta, StatementAction};

/// Compute only the `import_renames` map for a single module.
///
/// This is a lightweight helper that only resolves import-to-canonical-name
/// mappings without computing statement actions or detecting structural
/// mutations. Used by `pre_apply_global_renames` which only needs renames.
pub fn compute_import_renames(
    scan_result: &ScanStageOutput,
    module_idx: ModuleIdx,
    canonical_names: &CanonicalNames,
    default_export_names: &IndexVec<ModuleIdx, Option<String>>,
) -> FxHashMap<SymbolId, String> {
    let module = &scan_result.module_table[module_idx];
    let program_body = &scan_result.ast_table[module_idx].body;
    let mut import_renames = FxHashMap::default();

    for stmt in program_body {
        if let Statement::ImportDeclaration(import_decl) = stmt
            && let Some(specifiers) = &import_decl.specifiers
            && let Some(source_idx) =
                module.resolve_internal_specifier(import_decl.source.value.as_str())
        {
            let source_module = &scan_result.module_table[source_idx];
            collect_import_renames_from_specifiers(
                specifiers,
                source_module,
                canonical_names,
                default_export_names,
                &mut import_renames,
            );
        }
    }

    import_renames
}

/// Compute per-module link metadata for a single module.
///
/// Determines per-statement actions (include/skip/unwrap) and collects import
/// rename info, namespace aliases, and external namespace info. This is pure
/// link-stage data — no generate-stage dependency.
pub fn compute_module_link_meta(
    scan_result: &ScanStageOutput,
    module_idx: ModuleIdx,
    needed_symbol_kinds: Option<&FxHashMap<SymbolId, NeededKindFlags>>,
    canonical_names: &CanonicalNames,
    default_export_names: &IndexVec<ModuleIdx, Option<String>>,
) -> ModuleLinkMeta {
    let module = &scan_result.module_table[module_idx];
    let program_body = &scan_result.ast_table[module_idx].body;
    let mut meta = ModuleLinkMeta {
        statement_actions: Vec::with_capacity(program_body.len()),
        import_renames: FxHashMap::default(),
        ns_aliases: FxHashSet::default(),
        external_ns_info: FxHashMap::default(),
        reexported_import_names: FxHashSet::default(),
        needs_structural_mutation: false,
    };

    // Pre-scan: collect import renames, ns aliases, external ns info,
    // and reexported import names.
    for stmt in program_body {
        // Collect local names from `export { X }` (no source) for non-entry
        // modules. When X was imported from an external package, the import
        // must be preserved even though X isn't used in local declarations.
        if !module.is_entry
            && let Statement::ExportNamedDeclaration(decl) = stmt
            && decl.source.is_none()
            && decl.declaration.is_none()
        {
            for spec in &decl.specifiers {
                meta.reexported_import_names.insert(spec.local.name().to_string());
            }
        }
        if let Statement::ImportDeclaration(import_decl) = stmt
            && let Some(specifiers) = &import_decl.specifiers
        {
            if let Some(source_idx) =
                module.resolve_internal_specifier(import_decl.source.value.as_str())
            {
                // Internal import processing
                let source_module = &scan_result.module_table[source_idx];
                for spec in specifiers {
                    if let oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) =
                        spec
                        && let Some(symbol_id) = ns.local.symbol_id.get()
                    {
                        meta.ns_aliases.insert(symbol_id);
                    }
                }
                collect_import_renames_from_specifiers(
                    specifiers,
                    source_module,
                    canonical_names,
                    default_export_names,
                    &mut meta.import_renames,
                );
            } else {
                // External import — collect namespace specifiers
                for spec in specifiers {
                    if let oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) =
                        spec
                        && let Some(symbol_id) = ns.local.symbol_id.get()
                    {
                        meta.ns_aliases.insert(symbol_id);
                        meta.external_ns_info.insert(
                            symbol_id,
                            (import_decl.source.value.to_string(), ns.local.name.to_string()),
                        );
                    }
                }
            }
        }
    }

    // Determine per-statement actions.
    for stmt in program_body {
        let action = analyze_statement(stmt, module, needed_symbol_kinds);
        meta.statement_actions.push(action);
    }

    // Detect whether this module needs structural AST mutations beyond renames.
    meta.needs_structural_mutation = !meta.ns_aliases.is_empty()
        || has_internal_inline_imports(program_body, &meta.statement_actions, module);

    meta
}

/// Collect import renames from a set of import specifiers against a resolved
/// internal source module. Shared by both `compute_import_renames` and
/// `compute_module_link_meta`.
fn collect_import_renames_from_specifiers(
    specifiers: &oxc_allocator::Vec<'_, oxc_ast::ast::ImportDeclarationSpecifier<'_>>,
    source_module: &crate::types::Module<'_>,
    canonical_names: &CanonicalNames,
    default_export_names: &IndexVec<ModuleIdx, Option<String>>,
    import_renames: &mut FxHashMap<SymbolId, String>,
) {
    for spec in specifiers {
        match spec {
            oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                // Namespace specifiers don't produce import renames
            }
            oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                let imported_alias = s.imported.name().to_string();
                let local_name = resolve_export_local_name(source_module, &imported_alias)
                    .unwrap_or(imported_alias);
                let resolved_imported = canonical_names
                    .resolve_name(source_module, &local_name)
                    .map_or(local_name, ToString::to_string);
                if s.local.name.as_str() != resolved_imported
                    && let Some(symbol_id) = s.local.symbol_id.get()
                {
                    import_renames.insert(symbol_id, resolved_imported);
                }
            }
            oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(def) => {
                if let Some(mut actual_name) = default_export_names[source_module.idx].clone() {
                    if let Some(renamed) = canonical_names.resolve_name(source_module, &actual_name)
                    {
                        actual_name = renamed.to_string();
                    }
                    if def.local.name.as_str() != actual_name
                        && let Some(symbol_id) = def.local.symbol_id.get()
                    {
                        import_renames.insert(symbol_id, actual_name);
                    }
                }
            }
        }
    }
}

/// Determine the action for a single statement (inclusion decision only).
///
/// This is the pure link-stage version: it decides include/skip/unwrap based
/// on needed symbols and module structure, but does NOT collect exports/imports
/// into GenerateAcc (that happens in generate stage's `collect_module_outputs`).
fn analyze_statement<'a>(
    stmt: &Statement<'a>,
    module: &crate::types::Module<'a>,
    needed_symbol_kinds: Option<&FxHashMap<SymbolId, NeededKindFlags>>,
) -> StatementAction {
    match stmt {
        Statement::ExportNamedDeclaration(export_decl) => {
            if let Some(decl) = &export_decl.declaration {
                if let Some(needed) = needed_symbol_kinds
                    && !declaration_matches_needed_kinds(decl, &module.scoping, needed)
                {
                    return StatementAction::Skip;
                }
                StatementAction::UnwrapExportDeclaration
            } else {
                // Bare specifiers or re-exports with source — always skip
                // (metadata collected during generate's output collection).
                StatementAction::Skip
            }
        }
        Statement::ExportDefaultDeclaration(export_default) => {
            if let Some(needed) = needed_symbol_kinds
                && !export_default_declaration_matches_needed_kinds(
                    export_default,
                    &module.scoping,
                    needed,
                )
            {
                return StatementAction::Skip;
            }
            match &export_default.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(_)
                | ExportDefaultDeclarationKind::ClassDeclaration(_)
                | ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                    StatementAction::UnwrapExportDefault
                }
                _ => StatementAction::Skip,
            }
        }
        Statement::ExportAllDeclaration(_) => StatementAction::Skip,
        Statement::ImportDeclaration(import_decl) => {
            if module.resolve_internal_specifier(import_decl.source.value.as_str()).is_some() {
                return StatementAction::Skip;
            }
            // External imports: the statement itself is skipped (imports are
            // collected during generate's output collection phase).
            StatementAction::Skip
        }
        Statement::TSNamespaceExportDeclaration(_) | Statement::TSExportAssignment(_) => {
            StatementAction::Skip
        }
        Statement::TSImportEqualsDeclaration(decl) => {
            if let oxc_ast::ast::TSModuleReference::ExternalModuleReference(ext) =
                &decl.module_reference
                && module.resolve_internal_specifier(ext.expression.value.as_str()).is_some()
            {
                return StatementAction::Skip;
            }
            StatementAction::Include
        }
        _ => {
            if let Some(needed) = needed_symbol_kinds
                && let Some(decl) = stmt.as_declaration()
                && !declaration_matches_needed_kinds(decl, &module.scoping, needed)
            {
                return StatementAction::Skip;
            }
            StatementAction::Include
        }
    }
}

/// Collect symbol kinds for the names declared by a `Declaration` node.
///
/// Uses `BindingIdentifier.symbol_id` directly instead of name-based lookups
/// so this remains correct even after AST renames have been applied.
fn collect_decl_symbol_kinds(
    decl: &oxc_ast::ast::Declaration<'_>,
    scoping: &Scoping,
) -> FxHashMap<SymbolId, NeededKindFlags> {
    let declared_kinds = declaration_needed_kinds(decl);
    if declared_kinds.is_empty() {
        return FxHashMap::default();
    }
    let mut symbol_ids = Vec::new();
    collect_decl_symbol_ids(decl, &mut symbol_ids);
    symbol_ids
        .into_iter()
        .filter_map(|symbol_id| {
            let kinds = declared_kinds
                .intersection(NeededKindFlags::from_symbol_flags(scoping.symbol_flags(symbol_id)));
            (!kinds.is_empty()).then_some((symbol_id, kinds))
        })
        .collect()
}

/// Collect `SymbolId`s directly from declaration binding identifiers.
///
/// Unlike `collect_decl_names` + `scoping.get_root_binding()`, this reads
/// `symbol_id` fields from the AST nodes, making it resilient to AST renames
/// applied during the multi-entry pre-rename pass.
fn collect_decl_symbol_ids(decl: &oxc_ast::ast::Declaration<'_>, ids: &mut Vec<SymbolId>) {
    match decl {
        oxc_ast::ast::Declaration::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                collect_binding_pattern_symbol_ids(&declarator.id, ids);
            }
        }
        oxc_ast::ast::Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id
                && let Some(symbol_id) = id.symbol_id.get()
            {
                ids.push(symbol_id);
            }
        }
        oxc_ast::ast::Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id
                && let Some(symbol_id) = id.symbol_id.get()
            {
                ids.push(symbol_id);
            }
        }
        oxc_ast::ast::Declaration::TSTypeAliasDeclaration(alias) => {
            if let Some(symbol_id) = alias.id.symbol_id.get() {
                ids.push(symbol_id);
            }
        }
        oxc_ast::ast::Declaration::TSInterfaceDeclaration(iface) => {
            if let Some(symbol_id) = iface.id.symbol_id.get() {
                ids.push(symbol_id);
            }
        }
        oxc_ast::ast::Declaration::TSEnumDeclaration(enum_decl) => {
            if let Some(symbol_id) = enum_decl.id.symbol_id.get() {
                ids.push(symbol_id);
            }
        }
        oxc_ast::ast::Declaration::TSModuleDeclaration(module_decl) => {
            if let oxc_ast::ast::TSModuleDeclarationName::Identifier(id) = &module_decl.id
                && let Some(symbol_id) = id.symbol_id.get()
            {
                ids.push(symbol_id);
            }
        }
        oxc_ast::ast::Declaration::TSGlobalDeclaration(_)
        | oxc_ast::ast::Declaration::TSImportEqualsDeclaration(_) => {}
    }
}

/// Recursively collect all `SymbolId`s from a `BindingPattern`.
///
/// Handles `BindingIdentifier` (leaf), `ObjectPattern`, `ArrayPattern`, and
/// `AssignmentPattern` so that destructuring declarations like
/// `const { x, y } = obj` or `const [a, b] = arr` are fully covered.
fn collect_binding_pattern_symbol_ids(
    pattern: &oxc_ast::ast::BindingPattern<'_>,
    ids: &mut Vec<SymbolId>,
) {
    match pattern {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => {
            if let Some(symbol_id) = id.symbol_id.get() {
                ids.push(symbol_id);
            }
        }
        oxc_ast::ast::BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_pattern_symbol_ids(&prop.value, ids);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_pattern_symbol_ids(&rest.argument, ids);
            }
        }
        oxc_ast::ast::BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_pattern_symbol_ids(elem, ids);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_pattern_symbol_ids(&rest.argument, ids);
            }
        }
        oxc_ast::ast::BindingPattern::AssignmentPattern(assign) => {
            collect_binding_pattern_symbol_ids(&assign.left, ids);
        }
    }
}

fn declaration_needed_kinds(decl: &oxc_ast::ast::Declaration<'_>) -> NeededKindFlags {
    use oxc_ast::ast::Declaration;

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

fn declaration_matches_needed_kinds(
    decl: &oxc_ast::ast::Declaration<'_>,
    scoping: &Scoping,
    needed: &FxHashMap<SymbolId, NeededKindFlags>,
) -> bool {
    let decl_symbols = collect_decl_symbol_kinds(decl, scoping);
    decl_symbols.is_empty()
        || decl_symbols.iter().any(|(symbol_id, decl_kinds)| {
            needed.get(symbol_id).is_some_and(|needed_kinds| needed_kinds.intersects(*decl_kinds))
        })
}

/// Uses `BindingIdentifier.symbol_id` directly instead of name-based lookups
/// so this remains correct even after AST renames have been applied.
fn export_default_declaration_matches_needed_kinds(
    export_default: &ExportDefaultDeclaration<'_>,
    scoping: &Scoping,
    needed: &FxHashMap<SymbolId, NeededKindFlags>,
) -> bool {
    let (symbol_id, decl_kinds) = match &export_default.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            let Some(id) = &func.id else {
                return true;
            };
            let Some(symbol_id) = id.symbol_id.get() else {
                return true;
            };
            (symbol_id, NeededKindFlags::VALUE)
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            let Some(id) = &class.id else {
                return true;
            };
            let Some(symbol_id) = id.symbol_id.get() else {
                return true;
            };
            (symbol_id, NeededKindFlags::ALL)
        }
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
            let Some(symbol_id) = iface.id.symbol_id.get() else {
                return true;
            };
            (symbol_id, NeededKindFlags::TYPE)
        }
        _ => return true,
    };

    let decl_kinds = decl_kinds
        .intersection(NeededKindFlags::from_symbol_flags(scoping.symbol_flags(symbol_id)));
    decl_kinds.is_empty() || needed.get(&symbol_id).is_some_and(|k| k.intersects(decl_kinds))
}

/// Check whether any included statement contains `TSImportType` nodes that
/// reference internal modules (requiring inline import rewriting).
fn has_internal_inline_imports(
    program_body: &oxc_allocator::Vec<'_, Statement<'_>>,
    actions: &[StatementAction],
    module: &crate::types::Module<'_>,
) -> bool {
    let mut detector = InlineImportDetector { module, found: false };
    for (i, stmt) in program_body.iter().enumerate() {
        if matches!(actions[i], StatementAction::Skip) {
            continue;
        }
        detector.visit_statement(stmt);
        if detector.found {
            return true;
        }
    }
    false
}

struct InlineImportDetector<'a, 'm> {
    module: &'m crate::types::Module<'a>,
    found: bool,
}

impl<'a> Visit<'a> for InlineImportDetector<'a, '_> {
    fn visit_ts_import_type(&mut self, it: &oxc_ast::ast::TSImportType<'a>) {
        let specifier = it.source.value.as_str();
        if self.module.resolve_internal_specifier(specifier).is_some() || specifier.starts_with('.')
        {
            self.found = true;
        }
        if !self.found {
            oxc_ast_visit::walk::walk_ts_import_type(self, it);
        }
    }
}
