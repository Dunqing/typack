//! A native TypeScript `.d.ts` declaration bundler built on Oxc.
//!
//! Implements a three-stage pipeline (Scan, Link, Generate) that parses `.d.ts`
//! files, resolves imports, applies tree-shaking and rename deconfliction, and
//! emits a single bundled declaration file with optional source maps.

mod generate_stage;
mod helpers;
mod link_stage;
mod options;
mod scan_stage;
mod types;

pub use options::TypackOptions;

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;

use crate::generate_stage::GenerateStage;
use crate::scan_stage::ScanStage;

/// Result of bundling `.d.ts` files.
pub struct BundleResult {
    /// The bundled `.d.ts` output code.
    pub code: String,
    /// Source map mapping bundled output back to original `.d.ts` sources.
    /// Only present when `options.sourcemap` is true.
    pub map: Option<oxc_sourcemap::SourceMap>,
    /// Non-fatal warnings (e.g., unused exports, suspicious patterns).
    pub warnings: Vec<OxcDiagnostic>,
}

/// A native DTS bundler that operates directly on `.d.ts` ASTs.
///
/// Replaces the FakeJS transform/restore approach with a three-stage pipeline:
/// Scan → Link → Generate.
pub struct TypackBundler;

impl TypackBundler {
    /// Bundle `.d.ts` files into a single output.
    ///
    /// Returns `Ok` with code + warnings, or `Err` with fatal diagnostics
    /// (e.g., parse errors, unresolvable imports).
    ///
    /// # Errors
    ///
    /// Returns `Err` with a list of `OxcDiagnostic` when fatal errors occur,
    /// such as parse failures or unresolvable import specifiers.
    pub fn bundle(options: &TypackOptions) -> Result<BundleResult, Vec<OxcDiagnostic>> {
        let allocator = Allocator::default();
        let mut scan_result = ScanStage::new(options, &allocator).scan()?;
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
        Ok(BundleResult { code: generated.code, map: generated.map, warnings })
    }
}
