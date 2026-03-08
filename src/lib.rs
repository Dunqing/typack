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

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;

use crate::generate_stage::{GenerateStage, build_shared_generate_output};
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
/// Replaces the FakeJS transform/restore approach with a three-stage pipeline:
/// Scan → Link → Generate.
pub struct TypackBundler;

impl TypackBundler {
    /// Bundle `.d.ts` files, producing one output per entry point.
    ///
    /// All entries are scanned once into a shared module graph using a single
    /// allocator.  Each entry then gets its own link + generate pass that clones
    /// the required AST structures from the shared scan result.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a list of `OxcDiagnostic` when fatal errors occur,
    /// such as parse failures or unresolvable import specifiers.
    ///
    /// # Panics
    ///
    /// Panics if per-entry link data is missing for a known entry index.
    pub fn bundle(options: &TypackOptions) -> Result<BundleResult, Vec<OxcDiagnostic>> {
        let allocator = Allocator::default();
        let mut scan_result = ScanStage::new(options, &allocator).scan()?;
        let mut all_warnings = std::mem::take(&mut scan_result.warnings);
        let mut all_outputs = Vec::with_capacity(options.input.len());
        let entry_indices = scan_result.entry_indices.clone();
        let rename_plan = build_rename_plan(&scan_result);
        let link_output = build_link_stage_output(&scan_result, rename_plan);
        let shared_generate = build_shared_generate_output(
            &scan_result,
            &allocator,
            options.sourcemap,
            &options.cwd,
            &link_output,
        );
        all_warnings.extend(link_output.warnings.iter().cloned());

        for &entry_idx in &entry_indices {
            let per_entry = link_output
                .per_entry
                .get(&entry_idx)
                .expect("per-entry link data should exist for every entry");
            let generated = {
                let stage = GenerateStage::new(
                    &scan_result,
                    entry_idx,
                    per_entry,
                    options.cjs_default,
                    &link_output,
                    &shared_generate,
                );
                stage.generate()
            };
            all_warnings.extend(generated.warnings);
            all_outputs.push(BundleOutput { code: generated.code, map: generated.map });
        }

        Ok(BundleResult { output: all_outputs, warnings: all_warnings })
    }
}
