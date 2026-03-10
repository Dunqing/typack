//! Export name collection and resolution utilities.
//!
//! All functions use the pre-computed `export_import_info` maps on each module
//! instead of walking the AST body.

use rustc_hash::FxHashSet;

use crate::scan_stage::ScanStageOutput;
use crate::types::{ExportSource, Module, ModuleIdx};

/// Collect all exported names from a module, including both declaration exports
/// and specifier-based exports (`export { X }`, `export { X as Y }`).
/// Returns the LOCAL names (the names used in declarations within the module).
pub fn collect_all_exported_names(
    module_idx: ModuleIdx,
    scan_result: &ScanStageOutput<'_>,
) -> FxHashSet<String> {
    let info = &scan_result.module_table[module_idx].export_import_info;
    info.named_exports
        .iter()
        .map(|(exported_name, entry)| {
            // For source re-exports, the "local name" from the consumer's perspective
            // is the exported name (since the actual local name is in the source module).
            match &entry.source {
                ExportSource::SourceReexport { .. } => exported_name.clone(),
                _ => entry.local_name.clone(),
            }
        })
        // Filter out the synthetic "default" key — callers expect only declaration-level names
        .filter(|name| {
            name != "default"
                || info
                    .named_exports
                    .get("default")
                    .is_some_and(|e| !matches!(e.source, ExportSource::Default))
        })
        .collect()
}

/// Collect the PUBLIC exported names from a module.
///
/// Unlike [`collect_all_exported_names`], this returns the outward-facing export names,
/// so `export { Internal as Public }` contributes `Public`.
pub fn collect_public_exported_names(
    module_idx: ModuleIdx,
    scan_result: &ScanStageOutput<'_>,
) -> FxHashSet<String> {
    let info = &scan_result.module_table[module_idx].export_import_info;
    info.named_exports
        .keys()
        .filter(|name| {
            name.as_str() != "default"
                || info
                    .named_exports
                    .get("default")
                    .is_some_and(|e| !matches!(e.source, ExportSource::Default))
        })
        .cloned()
        .collect()
}

/// Trace an exported name through module re-export chains to find its
/// external source. Returns `Some((source, imported_name))` if the name
/// is a passthrough from an external package.
pub fn find_external_reexport_source(
    module_idx: ModuleIdx,
    exported_name: &str,
    scan_result: &ScanStageOutput<'_>,
) -> Option<(String, String)> {
    let module = &scan_result.module_table[module_idx];
    let info = &module.export_import_info;

    let entry = info.named_exports.get(exported_name)?;
    match &entry.source {
        ExportSource::SourceReexport { specifier, imported_name } => {
            if let Some(target_idx) = module.resolve_internal_specifier(specifier) {
                // Internal module — recurse with the imported name
                find_external_reexport_source(target_idx, imported_name, scan_result)
            } else {
                // External module — found the source
                Some((specifier.clone(), imported_name.clone()))
            }
        }
        _ => None,
    }
}

/// Resolve an exported name (potentially an alias) back to the local declaration name.
///
/// For `export { RolldownLog as a }`, calling with `exported_name = "a"` returns
/// `Some("RolldownLog")`. For `export interface Foo {}`, calling with `"Foo"` returns
/// `Some("Foo")`. Returns `None` if no matching export is found.
pub fn resolve_export_local_name(module: &Module<'_>, exported_name: &str) -> Option<String> {
    let entry = module.export_import_info.named_exports.get(exported_name)?;
    match &entry.source {
        ExportSource::LocalDeclaration | ExportSource::LocalReexport | ExportSource::Default => {
            Some(entry.local_name.clone())
        }
        // Source re-exports don't have a local declaration in this module
        ExportSource::SourceReexport { .. } => None,
    }
}

/// Resolve an exported name through internal re-export chains to the module and local name
/// that ultimately declares it.
pub fn resolve_export_origin(
    module_idx: ModuleIdx,
    exported_name: &str,
    scan_result: &ScanStageOutput<'_>,
) -> Option<(ModuleIdx, String)> {
    fn inner(
        module_idx: ModuleIdx,
        exported_name: &str,
        scan_result: &ScanStageOutput<'_>,
        seen: &mut FxHashSet<(ModuleIdx, String)>,
    ) -> Option<(ModuleIdx, String)> {
        if !seen.insert((module_idx, exported_name.to_string())) {
            return None;
        }

        let module = &scan_result.module_table[module_idx];
        if let Some(entry) = module.export_import_info.named_exports.get(exported_name) {
            return match &entry.source {
                ExportSource::LocalDeclaration
                | ExportSource::LocalReexport
                | ExportSource::Default => Some((module_idx, entry.local_name.clone())),
                ExportSource::SourceReexport { specifier, imported_name } => {
                    let target_idx = module.resolve_internal_specifier(specifier)?;
                    inner(target_idx, imported_name, scan_result, seen)
                }
            };
        }

        for star in &module.export_import_info.star_reexports {
            if star.alias.is_some() {
                continue;
            }
            let Some(target_idx) = module.resolve_internal_specifier(&star.specifier) else {
                continue;
            };
            if let Some(origin) = inner(target_idx, exported_name, scan_result, seen) {
                return Some(origin);
            }
        }

        None
    }

    inner(module_idx, exported_name, scan_result, &mut FxHashSet::default())
}

/// Resolve the name of a module's default export.
///
/// For `export default class Foo {}`, returns `Some("Foo")`.
/// For `export default function bar() {}`, returns `Some("bar")`.
pub fn resolve_default_export_name(
    module_idx: ModuleIdx,
    scan_result: &ScanStageOutput<'_>,
) -> Option<String> {
    let info = &scan_result.module_table[module_idx].export_import_info;
    let entry = info.named_exports.get("default")?;
    Some(entry.local_name.clone())
}
