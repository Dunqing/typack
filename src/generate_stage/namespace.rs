//! Namespace wrapper generation for `export * as X from "..."` patterns.

use std::path::Path;

use cow_utils::CowUtils;
use oxc_ast::ast::{Declaration, ExportDefaultDeclarationKind, ExportSpecifier, Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Ident;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::helpers::collect_decl_names;
use crate::link_stage::RenamePlan;
use crate::scan_stage::ScanResult;
use crate::types::ModuleIdx;

use super::types::{
    ExportedName, ExternalImport, ImportSpecifier, ImportSpecifierKind, NamespaceWrapInfo,
};

/// Collect exported names from a declaration for the consolidated `export { ... }` statement.
///
/// Namespace (`TSModuleDeclaration`) and global declarations are excluded because
/// they retain their own `declare` keyword in the output and are not re-exported
/// via the consolidated statement.
pub(super) fn collect_declaration_names(decl: &Declaration<'_>, exports: &mut Vec<ExportedName>) {
    if matches!(
        decl,
        Declaration::TSModuleDeclaration(_)
            | Declaration::TSGlobalDeclaration(_)
            | Declaration::TSImportEqualsDeclaration(_)
    ) {
        return;
    }
    let mut names = Vec::new();
    collect_decl_names(decl, &mut names);
    for name in names {
        exports.push(ExportedName { local: name.clone(), exported: name, is_type_only: false });
    }
}

pub(super) fn collect_export_specifier(
    spec: &ExportSpecifier<'_>,
    exports: &mut Vec<ExportedName>,
    is_type_only: bool,
) {
    exports.push(ExportedName {
        local: spec.local.name().to_string(),
        exported: spec.exported.name().to_string(),
        is_type_only,
    });
}

/// Recursively collect all exported names from a module.
/// Handles `export * from "..."` transitively.
/// Uses stored ASTs from the scan result instead of re-parsing.
/// A `visited` set guards against infinite recursion from circular re-exports.
pub(super) fn collect_module_exports(
    module_idx: ModuleIdx,
    exports: &mut Vec<ExportedName>,
    scan_result: &ScanResult<'_>,
    visited: &mut FxHashSet<ModuleIdx>,
    mut external_imports: Option<&mut Vec<ExternalImport>>,
) {
    if !visited.insert(module_idx) {
        return;
    }
    let module = &scan_result.modules[module_idx];

    for stmt in &module.program.body {
        match stmt {
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(d) = &decl.declaration {
                    collect_declaration_names(d, exports);
                } else if let Some(source) = &decl.source {
                    // Re-export: `export { X } from "..."`
                    let source_is_external =
                        module.resolve_internal_specifier(source.value.as_str()).is_none();
                    for spec in &decl.specifiers {
                        if source_is_external {
                            // External: import creates the exported name as local binding
                            let exported = spec.exported.name().to_string();
                            exports.push(ExportedName {
                                local: exported.clone(),
                                exported: exported.clone(),
                                is_type_only: false,
                            });
                            if let Some(imports) = &mut external_imports {
                                imports.push(ExternalImport {
                                    source: source.value.to_string(),
                                    specifiers: vec![ImportSpecifier {
                                        local: exported,
                                        kind: ImportSpecifierKind::Named(
                                            spec.local.name().to_string(),
                                        ),
                                    }],
                                    is_type_only: false,
                                    side_effect_only: false,
                                    from_reexport: true,
                                });
                            }
                        } else {
                            collect_export_specifier(spec, exports, false);
                        }
                    }
                } else {
                    for spec in &decl.specifiers {
                        collect_export_specifier(spec, exports, false);
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export_default) => {
                match &export_default.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        if let Some(id) = &func.id {
                            exports.push(ExportedName {
                                local: id.name.to_string(),
                                exported: "default".to_string(),
                                is_type_only: false,
                            });
                        }
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        if let Some(id) = &class.id {
                            exports.push(ExportedName {
                                local: id.name.to_string(),
                                exported: "default".to_string(),
                                is_type_only: false,
                            });
                        }
                    }
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
                        exports.push(ExportedName {
                            local: iface.id.name.to_string(),
                            exported: "default".to_string(),
                            is_type_only: false,
                        });
                    }
                    _ => {}
                }
            }
            Statement::ExportAllDeclaration(export_all) => {
                // Transitive `export * from "..."` — collect from target
                if let Some(dep_idx) =
                    module.resolve_internal_specifier(export_all.source.value.as_str())
                {
                    collect_module_exports(
                        dep_idx,
                        exports,
                        scan_result,
                        visited,
                        external_imports.as_deref_mut(),
                    );
                }
            }
            _ => {}
        }
    }
}

pub(super) fn apply_namespace_wrap_renames(
    namespace_wraps: &mut FxHashMap<ModuleIdx, NamespaceWrapInfo>,
    rename_plan: &RenamePlan,
    scan_result: &ScanResult<'_>,
) {
    for (module_idx, wrap) in namespace_wraps.iter_mut() {
        let module = &scan_result.modules[*module_idx];
        for export_name in &mut wrap.export_names {
            if let Some(renamed) = rename_plan.resolve_name(module, &export_name.local) {
                export_name.local = renamed.to_string();
            }
        }
    }
}

pub(super) fn collect_reserved_decl_names(
    scan_result: &ScanResult<'_>,
    rename_plan: &RenamePlan,
) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    for module in &scan_result.modules {
        for stmt in &module.program.body {
            if let Statement::ExportDefaultDeclaration(default_decl) = stmt {
                match &default_decl.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        if let Some(id) = &func.id {
                            let name = rename_plan
                                .resolve_name(module, id.name.as_str())
                                .unwrap_or(id.name.as_str());
                            names.insert(name.to_string());
                        }
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        if let Some(id) = &class.id {
                            let name = rename_plan
                                .resolve_name(module, id.name.as_str())
                                .unwrap_or(id.name.as_str());
                            names.insert(name.to_string());
                        }
                    }
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
                        let name = rename_plan
                            .resolve_name(module, iface.id.name.as_str())
                            .unwrap_or(iface.id.name.as_str());
                        names.insert(name.to_string());
                    }
                    _ => {}
                }
            } else {
                let maybe_decl = if let Statement::ExportNamedDeclaration(export_decl) = stmt {
                    export_decl.declaration.as_ref()
                } else {
                    stmt.as_declaration()
                };
                if let Some(decl) = maybe_decl {
                    let mut decl_names = Vec::new();
                    collect_decl_names(decl, &mut decl_names);
                    for name in decl_names {
                        let resolved = rename_plan.resolve_name(module, &name).unwrap_or(&name);
                        names.insert(resolved.to_string());
                    }
                }
            }
        }
    }
    names
}

pub(super) fn deconflict_namespace_wrap_names(
    namespace_wraps: &mut FxHashMap<ModuleIdx, NamespaceWrapInfo>,
    reserved_names: &FxHashSet<String>,
    warnings: &mut Vec<OxcDiagnostic>,
) {
    let mut used_names = reserved_names.clone();
    let mut sorted_keys: Vec<ModuleIdx> = namespace_wraps.keys().copied().collect();
    sorted_keys.sort();
    for module_idx in sorted_keys {
        let Some(wrap) = namespace_wraps.get_mut(&module_idx) else {
            continue;
        };
        let base_name = wrap.namespace_name.clone();
        if used_names.insert(base_name.clone()) {
            continue;
        }

        let mut suffix = 1;
        loop {
            let candidate = format!("{base_name}${suffix}");
            if used_names.insert(candidate.clone()) {
                warnings.push(OxcDiagnostic::warn(format!(
                    "typack/namespace-name-deconflict: renamed namespace wrapper \"{base_name}\" to \"{candidate}\" to avoid collision"
                )));
                wrap.namespace_name = candidate;
                break;
            }
            suffix += 1;
        }
    }
}

/// Pre-scan all modules to identify namespace import/export patterns.
///
/// Returns:
/// - `namespace_wraps`: modules that need namespace wrappers (module idx → wrap info)
/// - `namespace_aliases`: `import * as X` aliases in the entry (local name → module idx)
pub(super) fn pre_scan_namespace_info(
    scan_result: &ScanResult<'_>,
) -> (FxHashMap<ModuleIdx, NamespaceWrapInfo>, FxHashMap<SymbolId, ModuleIdx>) {
    let entry = &scan_result.modules[scan_result.entry_idx];

    let mut namespace_wraps: FxHashMap<ModuleIdx, NamespaceWrapInfo> = FxHashMap::default();
    // Entry-level namespace aliases: `import * as X` SymbolId → source module idx
    let mut namespace_aliases: FxHashMap<SymbolId, ModuleIdx> = FxHashMap::default();
    // Per-module namespace aliases: (module idx, SymbolId) → target module idx
    let mut all_module_aliases: FxHashMap<(ModuleIdx, SymbolId), ModuleIdx> = FxHashMap::default();
    let mut re_exported_names: Vec<SymbolId> = Vec::new();

    // Scan all modules for namespace aliases (using stored ASTs)
    for module in &scan_result.modules {
        for stmt in &module.program.body {
            if let Statement::ImportDeclaration(import_decl) = stmt
                && let Some(specifiers) = &import_decl.specifiers
            {
                for spec in specifiers {
                    if let oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) =
                        spec
                        && let Some(target_idx) =
                            module.resolve_internal_specifier(import_decl.source.value.as_str())
                        && let Some(symbol_id) = ns.local.symbol_id.get()
                    {
                        all_module_aliases.insert((module.idx, symbol_id), target_idx);
                        if module.is_entry {
                            namespace_aliases.insert(symbol_id, target_idx);
                        }
                    }
                }
            }
        }
    }

    // Scan entry for export patterns (using stored AST)
    for stmt in &entry.program.body {
        match stmt {
            // `export * as X from "./internal"`
            Statement::ExportAllDeclaration(export_all) => {
                if export_all.exported.is_some()
                    && let Some(target_idx) =
                        entry.resolve_internal_specifier(export_all.source.value.as_str())
                {
                    let ns_name = derive_namespace_name(&scan_result.modules[target_idx].path);
                    let mut export_names = Vec::new();
                    let mut visited = FxHashSet::default();
                    collect_module_exports(
                        target_idx,
                        &mut export_names,
                        scan_result,
                        &mut visited,
                        None,
                    );
                    namespace_wraps.insert(
                        target_idx,
                        NamespaceWrapInfo { namespace_name: ns_name, export_names },
                    );
                }
            }
            // Collect re-exported local names
            Statement::ExportNamedDeclaration(export_decl) => {
                if export_decl.source.is_none() && export_decl.declaration.is_none() {
                    for spec in &export_decl.specifiers {
                        if let Some(symbol_id) =
                            entry.scoping.get_root_binding(Ident::from(spec.local.name()))
                        {
                            re_exported_names.push(symbol_id);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Mark namespace-imported modules that are re-exported as needing wrapping
    for name in &re_exported_names {
        if let Some(target_idx) = namespace_aliases.get(name)
            && !namespace_wraps.contains_key(target_idx)
        {
            let ns_name = derive_namespace_name(&scan_result.modules[*target_idx].path);
            let mut export_names = Vec::new();
            let mut visited = FxHashSet::default();
            collect_module_exports(*target_idx, &mut export_names, scan_result, &mut visited, None);
            namespace_wraps
                .insert(*target_idx, NamespaceWrapInfo { namespace_name: ns_name, export_names });
        }
    }

    // Resolve namespace aliases in export lists: if a namespace wrapper's export
    // references another namespace alias, replace with the target namespace name.
    // First pass: collect new wraps needed for alias targets.
    let wrap_keys: Vec<ModuleIdx> = namespace_wraps.keys().copied().collect();
    let mut new_wraps: Vec<(ModuleIdx, NamespaceWrapInfo)> = Vec::new();
    for module_idx in &wrap_keys {
        let wrap = &namespace_wraps[module_idx];
        for exp in &wrap.export_names {
            let symbol_key = scan_result.modules[*module_idx]
                .scoping
                .get_root_binding(Ident::from(exp.local.as_str()));
            if let Some(symbol_id) = symbol_key
                && let Some(target_idx) = all_module_aliases.get(&(*module_idx, symbol_id))
                && !namespace_wraps.contains_key(target_idx)
                && !new_wraps.iter().any(|(idx, _)| idx == target_idx)
            {
                let ns_name = derive_namespace_name(&scan_result.modules[*target_idx].path);
                let mut target_exports = Vec::new();
                let mut visited = FxHashSet::default();
                collect_module_exports(
                    *target_idx,
                    &mut target_exports,
                    scan_result,
                    &mut visited,
                    None,
                );
                new_wraps.push((
                    *target_idx,
                    NamespaceWrapInfo { namespace_name: ns_name, export_names: target_exports },
                ));
            }
        }
    }
    for (path, info) in new_wraps {
        namespace_wraps.insert(path, info);
    }

    // Second pass: update export lists to use namespace names for aliases.
    let wrap_keys: Vec<ModuleIdx> = namespace_wraps.keys().copied().collect();
    for module_idx in &wrap_keys {
        let wrap = &namespace_wraps[module_idx];
        let mut updated_exports = Vec::new();
        for exp in &wrap.export_names {
            let symbol_key = scan_result.modules[*module_idx]
                .scoping
                .get_root_binding(Ident::from(exp.local.as_str()));
            if let Some(symbol_id) = symbol_key
                && let Some(target_idx) = all_module_aliases.get(&(*module_idx, symbol_id))
                && let Some(target_wrap) = namespace_wraps.get(target_idx)
            {
                updated_exports.push(ExportedName {
                    local: target_wrap.namespace_name.clone(),
                    exported: exp.exported.clone(),
                    is_type_only: exp.is_type_only,
                });
            } else {
                updated_exports.push(ExportedName {
                    local: exp.local.clone(),
                    exported: exp.exported.clone(),
                    is_type_only: exp.is_type_only,
                });
            }
        }
        namespace_wraps.get_mut(module_idx).unwrap().export_names = updated_exports;
    }

    (namespace_wraps, namespace_aliases)
}

/// Derive a namespace name from a module's file path.
/// e.g., `foo.d.ts` → `foo_d_exports`, `namespace.d.ts` → `namespace_d_exports`
/// Hyphens and dots in filenames are replaced with underscores.
pub(super) fn derive_namespace_name(path: &Path) -> String {
    let filename = path.file_name().unwrap().to_string_lossy();
    let stem = if let Some(s) = filename.strip_suffix(".d.ts") {
        s
    } else if let Some(s) = filename.strip_suffix(".ts") {
        s
    } else {
        &filename
    };
    let safe_stem = stem.cow_replace('-', "_");
    let safe_stem = safe_stem.cow_replace('.', "_");
    format!("{safe_stem}_d_exports")
}

/// Sanitize a string to be a valid JavaScript identifier.
/// Replaces non-alphanumeric chars (except `_`) with `_`.
pub(super) fn sanitize_to_identifier(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect()
}

/// Get or create a unique namespace name for a specifier.
/// Uses `default_name_fn` to generate the base name if not already in the map.
/// Appends numeric suffixes (0, 1, 2, ...) to resolve conflicts.
pub(super) fn get_or_create_ns_name(
    specifier: &str,
    ns_name_map: &mut FxHashMap<String, String>,
    reserved_names: &FxHashSet<String>,
    warnings: &mut Vec<OxcDiagnostic>,
    default_name_fn: impl FnOnce(&str) -> String,
) -> String {
    if let Some(name) = ns_name_map.get(specifier) {
        return name.clone();
    }

    let base_name = default_name_fn(specifier);
    let used_names: FxHashSet<&str> = ns_name_map.values().map(String::as_str).collect();

    let name = if used_names.contains(base_name.as_str()) || reserved_names.contains(&base_name) {
        let mut i = 0;
        loop {
            let candidate = format!("{base_name}{i}");
            if !used_names.contains(candidate.as_str()) && !reserved_names.contains(&candidate) {
                warnings.push(OxcDiagnostic::warn(format!(
                    "typack/namespace-name-deconflict: renamed namespace helper \"{base_name}\" for \"{specifier}\" to avoid collision"
                )));
                break candidate;
            }
            i += 1;
        }
    } else {
        base_name
    };

    ns_name_map.insert(specifier.to_string(), name.clone());
    name
}
