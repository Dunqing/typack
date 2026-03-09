//! Per-module link metadata computation.
//!
//! Determines per-statement inclusion decisions and collects import resolution
//! data that the generate stage consumes. This moves the "what to include"
//! analysis out of generate and into link, aligning with Rolldown's architecture.

use oxc_ast::ast::{ExportDefaultDeclaration, ExportDefaultDeclarationKind, Statement};
use oxc_semantic::Scoping;
use oxc_span::Ident;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::helpers::collect_decl_names;
use crate::link_stage::NeededKindFlags;
use crate::link_stage::exports::resolve_export_local_name;
use crate::scan_stage::ScanResult;
use crate::types::ModuleIdx;

use super::types::{ModuleLinkMeta, RenamePlan, StatementAction};

/// Compute per-module link metadata for a single module.
///
/// Determines per-statement actions (include/skip/unwrap) and collects import
/// rename info, namespace aliases, and external namespace info. This is pure
/// link-stage data — no generate-stage dependency.
pub fn compute_module_link_meta(
    scan_result: &ScanResult,
    module_idx: ModuleIdx,
    needed_symbol_kinds: Option<&FxHashMap<SymbolId, NeededKindFlags>>,
    rename_plan: &RenamePlan,
    default_export_names: &FxHashMap<ModuleIdx, String>,
) -> ModuleLinkMeta {
    let module = &scan_result.modules[module_idx];
    let mut meta = ModuleLinkMeta {
        statement_actions: Vec::with_capacity(module.program.body.len()),
        import_renames: FxHashMap::default(),
        ns_aliases: FxHashSet::default(),
        external_ns_info: FxHashMap::default(),
        reexported_import_names: FxHashSet::default(),
    };

    // Pre-scan: collect import renames, ns aliases, external ns info,
    // and reexported import names.
    for stmt in &module.program.body {
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
                let source_module = &scan_result.modules[source_idx];
                for spec in specifiers {
                    match spec {
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                            if let Some(symbol_id) = ns.local.symbol_id.get() {
                                meta.ns_aliases.insert(symbol_id);
                            }
                        }
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                            let imported_alias = s.imported.name().to_string();
                            let local_name =
                                resolve_export_local_name(source_module, &imported_alias)
                                    .unwrap_or(imported_alias);
                            let resolved_imported = rename_plan
                                .resolve_name(source_module, &local_name)
                                .map_or(local_name, ToString::to_string);
                            if s.local.name.as_str() != resolved_imported
                                && let Some(symbol_id) = s.local.symbol_id.get()
                            {
                                meta.import_renames.insert(symbol_id, resolved_imported);
                            }
                        }
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(def) => {
                            if let Some(mut actual_name) =
                                default_export_names.get(&source_module.idx).cloned()
                            {
                                if let Some(renamed) =
                                    rename_plan.resolve_name(source_module, &actual_name)
                                {
                                    actual_name = renamed.to_string();
                                }
                                if def.local.name.as_str() != actual_name
                                    && let Some(symbol_id) = def.local.symbol_id.get()
                                {
                                    meta.import_renames.insert(symbol_id, actual_name);
                                }
                            }
                        }
                    }
                }
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
    for stmt in &module.program.body {
        let action = analyze_statement(stmt, module, needed_symbol_kinds);
        meta.statement_actions.push(action);
    }

    meta
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
fn collect_decl_symbol_kinds(
    decl: &oxc_ast::ast::Declaration<'_>,
    scoping: &Scoping,
) -> FxHashMap<SymbolId, NeededKindFlags> {
    let mut names = Vec::new();
    collect_decl_names(decl, &mut names);
    let declared_kinds = declaration_needed_kinds(decl);
    if declared_kinds.is_empty() {
        return FxHashMap::default();
    }
    names
        .iter()
        .filter_map(|name| {
            let symbol_id = scoping.get_root_binding(Ident::from(name.as_str()))?;
            let kinds = declared_kinds
                .intersection(NeededKindFlags::from_symbol_flags(scoping.symbol_flags(symbol_id)));
            (!kinds.is_empty()).then_some((symbol_id, kinds))
        })
        .collect()
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

fn export_default_declaration_matches_needed_kinds(
    export_default: &ExportDefaultDeclaration<'_>,
    scoping: &Scoping,
    needed: &FxHashMap<SymbolId, NeededKindFlags>,
) -> bool {
    let (name, decl_kinds) = match &export_default.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            let Some(id) = &func.id else {
                return true;
            };
            (id.name.as_str(), NeededKindFlags::VALUE)
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            let Some(id) = &class.id else {
                return true;
            };
            (id.name.as_str(), NeededKindFlags::ALL)
        }
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
            (iface.id.name.as_str(), NeededKindFlags::TYPE)
        }
        _ => return true,
    };

    let Some(symbol_id) = scoping.get_root_binding(Ident::from(name)) else {
        return true;
    };
    let decl_kinds = decl_kinds
        .intersection(NeededKindFlags::from_symbol_flags(scoping.symbol_flags(symbol_id)));
    decl_kinds.is_empty() || needed.get(&symbol_id).is_some_and(|k| k.intersects(decl_kinds))
}
