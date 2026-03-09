//! Resolved exports map: follows re-export chains to determine the origin
//! of each exported name, and detects star export ambiguity.
//!
//! Inspired by `oxc_module_graph`'s `build_resolved_exports` algorithm.

use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use crate::scan_stage::ScanStageOutput;
use crate::types::{ExportSource, ModuleIdx};

/// The resolved origin of an exported name.
#[derive(Debug, Clone)]
struct ResolvedExport {
    /// The module that originally declares this name.
    origin_module: ModuleIdx,
    /// The local declaration name in the origin module.
    local_name: String,
    /// Whether this export is potentially ambiguous (provided by multiple star sources).
    potentially_ambiguous: bool,
}

/// Per-module resolved exports: export_name → origin info.
type ModuleResolvedExports = FxHashMap<String, ResolvedExport>;

/// Build resolved exports for all modules in the module graph.
///
/// For each module, follows re-export chains (`export { X } from` and `export * from`)
/// to determine where each exported name ultimately comes from. Detects ambiguity
/// when multiple `export *` sources provide the same name.
pub fn build_resolved_exports(scan_result: &ScanStageOutput<'_>) -> Vec<OxcDiagnostic> {
    let module_count = scan_result.module_table.len();
    let mut resolved: IndexVec<ModuleIdx, Option<ModuleResolvedExports>> =
        std::iter::repeat_n(None, module_count).collect();
    let mut warnings: Vec<OxcDiagnostic> = Vec::new();

    // Process modules in topological order (dependencies before dependents).
    // This ensures that when we resolve a re-export chain, the source module's
    // resolved exports are already available.
    for module in &scan_result.module_table {
        let info = &module.export_import_info;
        let mut module_resolved: ModuleResolvedExports = FxHashMap::default();

        // Phase 1: Initialize from local declarations and local re-exports.
        for (exported_name, entry) in &info.named_exports {
            match &entry.source {
                ExportSource::LocalDeclaration | ExportSource::Default => {
                    module_resolved.insert(
                        exported_name.clone(),
                        ResolvedExport {
                            origin_module: module.idx,
                            local_name: entry.local_name.clone(),
                            potentially_ambiguous: false,
                        },
                    );
                }
                ExportSource::LocalReexport => {
                    // `export { X as Y }` — the local name is a binding in this module.
                    // Resolve it: it might be an import that chains further.
                    module_resolved.insert(
                        exported_name.clone(),
                        ResolvedExport {
                            origin_module: module.idx,
                            local_name: entry.local_name.clone(),
                            potentially_ambiguous: false,
                        },
                    );
                }
                ExportSource::SourceReexport { specifier, imported_name } => {
                    // Follow the chain to the source module's resolved exports.
                    if let Some(target_idx) = module.resolve_internal_specifier(specifier)
                        && let Some(target_resolved) = resolved[target_idx].as_ref()
                        && let Some(origin) = target_resolved.get(imported_name)
                    {
                        module_resolved.insert(
                            exported_name.clone(),
                            ResolvedExport {
                                origin_module: origin.origin_module,
                                local_name: origin.local_name.clone(),
                                potentially_ambiguous: origin.potentially_ambiguous,
                            },
                        );
                    }
                    // If not found in resolved, it may come from a star re-export
                    // of the target module. We'll pick it up below if needed.
                    // If specifier doesn't resolve internally, it's external — skip.
                }
            }
        }

        // Phase 2: Propagate star re-exports.
        // For each `export * from "./sub"`, merge the sub-module's resolved exports
        // into this module's map. Skip names that already exist (local exports take
        // precedence over star exports). Detect ambiguity when multiple star sources
        // provide the same name.
        let mut star_contributed_names: FxHashMap<String, ModuleIdx> = FxHashMap::default();

        for star in &info.star_reexports {
            // Only handle plain `export * from` (not `export * as ns from`).
            if star.alias.is_some() {
                continue;
            }

            let Some(target_idx) = module.resolve_internal_specifier(&star.specifier) else {
                continue;
            };
            let Some(target_resolved) = resolved[target_idx].as_ref() else {
                continue;
            };

            for (name, origin) in target_resolved {
                // Skip "default" — `export *` doesn't re-export default.
                if name == "default" {
                    continue;
                }

                // Local/named exports (from Phase 1) take precedence over star exports.
                // Only skip if this name was NOT contributed by a previous star source.
                if module_resolved.contains_key(name) && !star_contributed_names.contains_key(name)
                {
                    continue;
                }

                // Check for ambiguity: another star source already provides this name.
                if let Some(&prev_source) = star_contributed_names.get(name)
                    && prev_source != target_idx
                {
                    // Ambiguous: mark as such and emit a warning.
                    if let Some(existing) = module_resolved.get_mut(name) {
                        existing.potentially_ambiguous = true;
                    }
                    warnings.push(OxcDiagnostic::warn(format!(
                        "Ambiguous star export: \"{name}\" is provided by both module {} and module {} via `export *`",
                        scan_result.module_table[prev_source].relative_path,
                        scan_result.module_table[target_idx].relative_path,
                    )).with_help(format!(
                        "Add an explicit `export {{ {name} }} from \"...\"` to resolve the ambiguity",
                    )));
                    continue;
                }

                star_contributed_names.insert(name.clone(), target_idx);
                module_resolved.insert(
                    name.clone(),
                    ResolvedExport {
                        origin_module: origin.origin_module,
                        local_name: origin.local_name.clone(),
                        potentially_ambiguous: origin.potentially_ambiguous,
                    },
                );
            }
        }

        // Phase 3: Resolve named source re-exports that weren't found in Phase 1
        // (they may come from the source's star re-exports, which are now resolved).
        for (exported_name, entry) in &info.named_exports {
            if module_resolved.contains_key(exported_name) {
                continue;
            }
            if let ExportSource::SourceReexport { specifier, imported_name } = &entry.source
                && let Some(target_idx) = module.resolve_internal_specifier(specifier)
                && let Some(target_resolved) = resolved[target_idx].as_ref()
                && let Some(origin) = target_resolved.get(imported_name)
            {
                module_resolved.insert(
                    exported_name.clone(),
                    ResolvedExport {
                        origin_module: origin.origin_module,
                        local_name: origin.local_name.clone(),
                        potentially_ambiguous: origin.potentially_ambiguous,
                    },
                );
            }
        }

        resolved[module.idx] = Some(module_resolved);
    }

    warnings
}
