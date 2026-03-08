//! A native TypeScript `.d.ts` declaration bundler built on Oxc.
//!
//! Implements a three-stage pipeline (Scan, Link, Generate) that parses `.d.ts`
//! files, resolves imports, applies tree-shaking and rename deconfliction, and
//! emits one bundled declaration file per entry point with optional source maps.

mod generate_stage;
mod helpers;
mod link_stage;
mod options;
mod scan_stage;
mod types;

#[cfg(feature = "cli")]
pub mod cli;

pub use options::TypackOptions;

use rustc_hash::FxHashSet;

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;

use crate::generate_stage::{assemble_entry, precompute_entry_data, process_all_module_fragments};
use crate::link_stage::{build_link_stage_output, build_rename_plan};
use crate::scan_stage::ScanStage;

/// A single bundled output for one entry point.
pub struct BundleOutput {
    /// The bundled `.d.ts` output code.
    pub code: String,
    /// Source map mapping bundled output back to original `.d.ts` sources.
    /// Only present when `options.sourcemap` is true.
    pub map: Option<oxc_sourcemap::SourceMap>,
}

/// Result of bundling `.d.ts` files.
///
/// Contains one [`BundleOutput`] per entry point.
pub struct BundleResult {
    /// Per-entry bundled output.
    pub output: Vec<BundleOutput>,
    /// Non-fatal warnings (e.g., unused exports, suspicious patterns).
    pub warnings: Vec<OxcDiagnostic>,
}

/// A native DTS bundler that operates directly on `.d.ts` ASTs.
///
/// Two-phase pipeline: modules are processed once into code fragments
/// (Phase 1), then assembled per-entry (Phase 2). This eliminates
/// redundant `clone_in` work in multi-entry builds.
pub struct TypackBundler;

impl TypackBundler {
    /// Bundle `.d.ts` files, producing one output per entry point.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a list of `OxcDiagnostic` when fatal errors occur,
    /// such as parse failures or unresolvable import specifiers.
    pub fn bundle(options: &TypackOptions) -> Result<BundleResult, Vec<OxcDiagnostic>> {
        let allocator = Allocator::default();
        let mut scan_result = ScanStage::new(options, &allocator).scan()?;
        let mut all_warnings = std::mem::take(&mut scan_result.warnings);
        let mut all_outputs = Vec::with_capacity(options.input.len());
        let entry_indices = scan_result.entry_indices.clone();
        let rename_plan = build_rename_plan(&scan_result);
        let link_output = build_link_stage_output(&scan_result, rename_plan);
        all_warnings.extend_from_slice(&link_output.warnings);

        // Phase 0: Pre-compute per-entry data while module bodies are still available.
        let precomputed: Vec<_> = entry_indices
            .iter()
            .map(|&entry_idx| precompute_entry_data(&scan_result, entry_idx, &link_output))
            .collect();

        // Merge all helper_reserved_names across entries for Phase 1.
        // Also compute the set of modules needed by any entry.
        let mut merged_helper_reserved_names: FxHashSet<String> =
            link_output.reserved_decl_names.clone();
        let mut needed_modules: FxHashSet<crate::types::ModuleIdx> = FxHashSet::default();
        for pre in &precomputed {
            merged_helper_reserved_names.extend(pre.helper_reserved_names.iter().cloned());
            needed_modules.extend(pre.needed_module_indices());
        }
        // Modules with augmentations are always needed
        for (i, module) in scan_result.modules.iter().enumerate() {
            if module.has_augmentation {
                needed_modules.insert(crate::types::ModuleIdx::from_usize(i));
            }
        }

        // Phase 1: Process all modules into fragments (consumes AST bodies).
        let module_fragments = process_all_module_fragments(
            &mut scan_result,
            &allocator,
            &link_output,
            &merged_helper_reserved_names,
            &needed_modules,
            options.sourcemap,
            &options.cwd,
        );

        // Phase 2: Assemble per-entry output from fragments.
        for (i, &entry_idx) in entry_indices.iter().enumerate() {
            let generated = assemble_entry(
                entry_idx,
                &module_fragments,
                &precomputed[i],
                &scan_result,
                options.sourcemap,
                options.cjs_default,
            );
            all_warnings.extend(generated.warnings);
            all_outputs.push(BundleOutput { code: generated.code, map: generated.map });
        }

        Ok(BundleResult { output: all_outputs, warnings: all_warnings })
    }
}
