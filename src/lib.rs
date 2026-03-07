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

use crate::generate_stage::GenerateStage;
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
    /// Per-entry bundled outputs.
    pub outputs: Vec<BundleOutput>,
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
    /// allocator.  Each entry then gets its own link + generate pass using
    /// `take_in` (zero-copy move) from the shared scan result.
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

        for &entry_idx in &entry_indices {
            let generated = {
                let mut stage = GenerateStage::new(
                    &mut scan_result,
                    entry_idx,
                    &allocator,
                    options.sourcemap,
                    options.cjs_default,
                    &options.cwd,
                );
                stage.generate()
            };
            all_warnings.extend(generated.warnings);
            all_outputs.push(BundleOutput { code: generated.code, map: generated.map });
        }

        Ok(BundleResult { outputs: all_outputs, warnings: all_warnings })
    }
}
