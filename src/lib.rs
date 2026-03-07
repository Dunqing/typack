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
    /// The entry file path that produced this output.
    pub entry: String,
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
    /// Returns `Ok` with per-entry outputs + warnings, or `Err` with fatal
    /// diagnostics (e.g., parse errors, unresolvable imports).
    ///
    /// # Errors
    ///
    /// Returns `Err` with a list of `OxcDiagnostic` when fatal errors occur,
    /// such as parse failures or unresolvable import specifiers.
    pub fn bundle(options: &TypackOptions) -> Result<BundleResult, Vec<OxcDiagnostic>> {
        let mut all_outputs = Vec::new();
        let mut all_warnings = Vec::new();

        for entry in &options.input {
            let allocator = Allocator::default();
            let single_options = TypackOptions {
                input: vec![entry.clone()],
                external: options.external.clone(),
                cwd: options.cwd.clone(),
                sourcemap: options.sourcemap,
                cjs_default: options.cjs_default,
            };
            let mut scan_result = ScanStage::new(&single_options, &allocator).scan()?;
            let mut warnings = std::mem::take(&mut scan_result.warnings);
            let mut generated = GenerateStage::new(
                &mut scan_result,
                &allocator,
                options.sourcemap,
                options.cjs_default,
                &options.cwd,
            )
            .generate();
            warnings.append(&mut generated.warnings);
            all_warnings.extend(warnings);
            all_outputs.push(BundleOutput {
                entry: entry.clone(),
                code: generated.code,
                map: generated.map,
            });
        }

        Ok(BundleResult { outputs: all_outputs, warnings: all_warnings })
    }
}
