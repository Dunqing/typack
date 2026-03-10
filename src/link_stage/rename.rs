//! Name deconfliction: builds a rename plan for declarations that collide
//! across bundled modules.

use cow_utils::CowUtils;
use oxc_ast::ast::Statement;
use oxc_span::Ident;
use oxc_syntax::symbol::{SymbolFlags, SymbolId};
use rustc_hash::FxHashSet;

use crate::helpers::collect_statement_declaration_names;
use crate::scan_stage::ScanStageOutput;
use crate::types::{ExportSource, Module, ModuleIdx};

use super::types::CanonicalNames;

struct SymbolCandidate {
    module_idx: ModuleIdx,
    symbol_id: SymbolId,
    name: String,
}

struct ModuleRenameData {
    module_idx: ModuleIdx,
    exported_symbols: Vec<SymbolCandidate>,
    exported_fallback_names: Vec<String>,
    non_exported_symbols: Vec<SymbolCandidate>,
    non_exported_fallback_names: Vec<String>,
}

/// Build canonical names for conflicting declaration names across modules.
///
/// Uses a two-pass approach:
/// 1. Register exported names in entry re-export order (priority order).
/// 2. Register non-exported names in reverse module order.
pub fn build_canonical_names(scan_result: &ScanStageOutput<'_>) -> CanonicalNames {
    let mut canonical_names = CanonicalNames::with_module_count(scan_result.module_table.len());

    reserve_synthetic_helper_names(scan_result, &mut canonical_names.used_names);

    let mut module_data: Vec<ModuleRenameData> = Vec::new();
    for module in &scan_result.module_table {
        module_data.push(build_module_rename_data(module, &scan_result.ast_table[module.idx].body));
    }

    // Pass 1: Register exported names in entry re-export order.
    // The entry module's re-export order determines which module's exports get priority.
    let entry = &scan_result.module_table[scan_result.entry_points[0]];
    let entry_re_export_order =
        collect_re_export_module_order(entry, &scan_result.ast_table[entry.idx].body);
    let entry_re_export_set: FxHashSet<ModuleIdx> = entry_re_export_order.iter().copied().collect();

    // Process exported names from modules in entry re-export order
    for module_idx in &entry_re_export_order {
        if let Some(module_data) = module_data.iter().find(|data| data.module_idx == *module_idx) {
            register_symbol_candidates(
                &module_data.exported_symbols,
                scan_result,
                &mut canonical_names,
            );
            register_fallback_names(
                module_data.module_idx,
                &module_data.exported_fallback_names,
                scan_result,
                &mut canonical_names,
            );
        }
    }

    // Also register exported names from any modules not in entry re-export order.
    // Process in reverse module order so that later modules (closer to entry) keep
    // their original names while earlier modules get $N suffixes.
    for module_data in module_data.iter().rev() {
        if !entry_re_export_set.contains(&module_data.module_idx) {
            register_symbol_candidates(
                &module_data.exported_symbols,
                scan_result,
                &mut canonical_names,
            );
            register_fallback_names(
                module_data.module_idx,
                &module_data.exported_fallback_names,
                scan_result,
                &mut canonical_names,
            );
        }
    }

    // Pass 2: Register non-exported names in REVERSE module order.
    // This ensures that later modules (closer to entry) keep internal names,
    // while earlier modules get $1 suffixes.
    for module_data in module_data.iter().rev() {
        register_symbol_candidates(
            &module_data.non_exported_symbols,
            scan_result,
            &mut canonical_names,
        );
        register_fallback_names(
            module_data.module_idx,
            &module_data.non_exported_fallback_names,
            scan_result,
            &mut canonical_names,
        );
    }

    canonical_names
}

fn build_module_rename_data(module: &Module<'_>, body: &[Statement<'_>]) -> ModuleRenameData {
    let (exported_names, non_exported_names) = classify_declaration_names(module, body);
    let (exported_symbols, exported_fallback_names) =
        resolve_symbol_candidates(module, &exported_names);
    let (non_exported_symbols, non_exported_fallback_names) =
        resolve_symbol_candidates(module, &non_exported_names);

    ModuleRenameData {
        module_idx: module.idx,
        exported_symbols,
        exported_fallback_names,
        non_exported_symbols,
        non_exported_fallback_names,
    }
}

/// Returns true if the symbol has flags that participate in name collision
/// (type, value, or namespace space).
fn symbol_participates_in_naming(flags: SymbolFlags) -> bool {
    flags.intersects(SymbolFlags::Type | SymbolFlags::Value | SymbolFlags::Namespace)
}

fn resolve_symbol_candidates(
    module: &Module<'_>,
    names: &[String],
) -> (Vec<SymbolCandidate>, Vec<String>) {
    let mut symbol_candidates = Vec::new();
    let mut fallback_names = Vec::new();
    let mut seen_symbols: FxHashSet<SymbolId> = FxHashSet::default();
    let mut seen_fallback_names: FxHashSet<String> = FxHashSet::default();

    for name in names {
        if let Some(symbol_id) = module.scoping.get_root_binding(Ident::from(name.as_str())) {
            if !seen_symbols.insert(symbol_id) {
                continue;
            }
            if symbol_participates_in_naming(module.scoping.symbol_flags(symbol_id)) {
                symbol_candidates.push(SymbolCandidate {
                    module_idx: module.idx,
                    symbol_id,
                    name: name.clone(),
                });
            } else if seen_fallback_names.insert(name.clone()) {
                fallback_names.push(name.clone());
            }
        } else if seen_fallback_names.insert(name.clone()) {
            fallback_names.push(name.clone());
        }
    }

    (symbol_candidates, fallback_names)
}

fn register_symbol_candidates(
    candidates: &[SymbolCandidate],
    scan_result: &ScanStageOutput<'_>,
    canonical_names: &mut CanonicalNames,
) {
    for candidate in candidates {
        let module = &scan_result.module_table[candidate.module_idx];
        let chosen_name = allocate_name(
            &candidate.name,
            &mut canonical_names.used_names,
            |candidate_name, is_original| {
                if is_original {
                    return true;
                }
                !has_nested_scope_binding(module, candidate_name)
            },
        );
        if chosen_name != candidate.name {
            canonical_names.insert_symbol_rename(
                candidate.module_idx,
                candidate.symbol_id,
                chosen_name,
            );
        }
    }
}

fn register_fallback_names(
    module_idx: ModuleIdx,
    names: &[String],
    scan_result: &ScanStageOutput<'_>,
    canonical_names: &mut CanonicalNames,
) {
    let module = &scan_result.module_table[module_idx];
    for name in names {
        let chosen_name =
            allocate_name(name, &mut canonical_names.used_names, |candidate_name, is_original| {
                if is_original {
                    return true;
                }
                !has_nested_scope_binding(module, candidate_name)
            });
        if chosen_name != *name {
            canonical_names.fallback_name_renames.insert((module_idx, name.clone()), chosen_name);
        }
    }
}

fn allocate_name<F>(
    base_name: &str,
    used_names: &mut FxHashSet<String>,
    is_candidate_name_available: F,
) -> String
where
    F: Fn(&str, bool) -> bool,
{
    if !used_names.contains(base_name) && is_candidate_name_available(base_name, true) {
        used_names.insert(base_name.to_string());
        return base_name.to_string();
    }

    let mut suffix = 1u32;
    loop {
        let candidate = format!("{base_name}${suffix}");
        if !used_names.contains(&candidate) && is_candidate_name_available(&candidate, false) {
            used_names.insert(candidate.clone());
            return candidate;
        }
        suffix = suffix.saturating_add(1);
    }
}

fn has_nested_scope_binding(module: &Module<'_>, name: &str) -> bool {
    let root_scope_id = module.scoping.root_scope_id();
    module
        .scoping
        .iter_bindings()
        .any(|(scope_id, bindings)| scope_id != root_scope_id && bindings.contains_key(name))
}

fn reserve_synthetic_helper_names(
    scan_result: &ScanStageOutput<'_>,
    used_names: &mut FxHashSet<String>,
) {
    used_names.insert("export_default".to_string());
    for module in &scan_result.module_table {
        let stem = module.path.file_stem().and_then(std::ffi::OsStr::to_str).unwrap_or("mod");
        let safe_stem = stem.cow_replace(&['-', '.'][..], "_");
        used_names.insert(format!("{safe_stem}_exports"));
        used_names.insert(format!("{safe_stem}_d_exports"));
    }
}

/// Classify a module's declaration names into exported and non-exported.
///
/// Uses the pre-computed `export_import_info` for the exported set (avoiding a full
/// AST walk), but still walks the AST for non-exported bare declarations since those
/// aren't tracked in the export/import maps.
fn classify_declaration_names(
    module: &Module<'_>,
    body: &[Statement<'_>],
) -> (Vec<String>, Vec<String>) {
    let info = &module.export_import_info;

    // Exported names: local declarations + default export local names.
    let mut seen_exported: FxHashSet<String> = FxHashSet::default();
    let mut exported: Vec<String> = Vec::new();

    // `declared_export_names` covers `export <declaration>` patterns.
    for name in &info.declared_export_names {
        if seen_exported.insert(name.clone()) {
            exported.push(name.clone());
        }
    }
    // Default export with a named declaration (e.g. `export default class Foo {}`).
    if let Some(entry) = info.named_exports.get("default")
        && matches!(entry.source, ExportSource::Default)
    {
        let name = &entry.local_name;
        if seen_exported.insert(name.clone()) {
            exported.push(name.clone());
        }
    }

    // Non-exported names: bare declarations not covered by exports.
    // Still requires an AST walk since these aren't tracked in export_import_info.
    let mut non_exported: Vec<String> = Vec::new();
    let mut seen_non_exported: FxHashSet<String> = FxHashSet::default();
    for stmt in body {
        // Skip export statements — those are already handled above.
        if matches!(
            stmt,
            Statement::ExportNamedDeclaration(_) | Statement::ExportDefaultDeclaration(_)
        ) {
            continue;
        }
        let mut names = Vec::new();
        collect_statement_declaration_names(stmt, &mut names);
        for name in names {
            if seen_non_exported.insert(name.clone()) {
                non_exported.push(name);
            }
        }
    }

    // Remove non-exported names that are also exported (same name used for both
    // type and value, e.g. `interface Stuff` + `const Stuff`). These should be
    // counted as a single name per module to avoid inflated $N suffixes.
    non_exported.retain(|name| !seen_exported.contains(name));

    (exported, non_exported)
}

/// Collect the order of modules as they appear in the entry's re-export statements.
///
/// This still walks the AST because the statement order determines rename priority,
/// and `FxHashMap` doesn't preserve insertion order.
fn collect_re_export_module_order(entry: &Module<'_>, body: &[Statement<'_>]) -> Vec<ModuleIdx> {
    let mut order = Vec::new();
    for stmt in body {
        let source_specifier = match stmt {
            Statement::ExportNamedDeclaration(decl) => {
                decl.source.as_ref().map(|s| s.value.to_string())
            }
            Statement::ExportAllDeclaration(decl) => Some(decl.source.value.to_string()),
            _ => None,
        };
        if let Some(specifier) = source_specifier
            && let Some(module_idx) = entry.resolve_internal_specifier(&specifier)
            && !order.contains(&module_idx)
        {
            order.push(module_idx);
        }
    }
    order
}
