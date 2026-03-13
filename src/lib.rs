//! A native TypeScript `.d.ts` declaration bundler built on Oxc.
//!
//! Implements a three-stage pipeline (Scan, Link, Generate) that parses `.d.ts`
//! files (or `.ts` files via IsolatedDeclarations), resolves imports, applies
//! tree-shaking and rename deconfliction, and emits one bundled declaration file
//! per entry point with optional source maps.

mod generate_stage;
mod helpers;
mod link_stage;
mod options;
mod scan_stage;
mod types;

#[cfg(feature = "cli")]
pub mod cli;

pub use options::{GenerateOptions, TypackOptions};

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;

use crate::generate_stage::GenerateStage;
use crate::link_stage::{LinkStage, LinkStageOutput, NeededNamesCtx};
use crate::scan_stage::{ScanStage, ScanStageOutput};

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

/// Orchestrates the three-stage DTS bundling pipeline following Rolldown's
/// architecture: Scan → Link → Generate.
///
/// Owns the scan and link outputs, allowing the generate stage to be run
/// separately. Note that `generate` can only be called once because the
/// single-entry fast path takes ownership of AST statements via `TakeIn`.
pub struct Bundle<'a> {
    scan_output: ScanStageOutput<'a>,
    link_output: LinkStageOutput,
    allocator: &'a Allocator,
    generated: bool,
}

impl<'a> Bundle<'a> {
    /// Create a new bundle by running the scan and link stages.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a list of `OxcDiagnostic` when fatal errors occur,
    /// such as parse failures or unresolvable import specifiers.
    pub fn new(
        options: &TypackOptions,
        allocator: &'a Allocator,
    ) -> Result<Self, Vec<OxcDiagnostic>> {
        let scan_output = ScanStage::new(options, allocator).scan()?;
        let link_stage = LinkStage::new(&scan_output);
        let link_output = link_stage.link();
        Ok(Self { scan_output, link_output, allocator, generated: false })
    }

    /// Run the generate stage, producing one output per entry point.
    ///
    /// Takes `&mut self` because the single-entry fast path takes ownership of
    /// AST statements (via `TakeIn`) to avoid cloning. This means `generate`
    /// can only be called once.
    ///
    /// # Panics
    ///
    /// Panics if called more than once on the same `Bundle` instance.
    pub fn generate(&mut self, options: &GenerateOptions) -> BundleResult {
        assert!(!self.generated, "Bundle::generate can only be called once");
        self.generated = true;

        let mut all_warnings = std::mem::take(&mut self.scan_output.warnings);
        all_warnings.extend(std::mem::take(&mut self.link_output.warnings));

        // Pre-compute data that needs to read ast_table before we hand out &mut access.
        let unique_directives = collect_unique_directives(&self.scan_output);
        let needed_names_ctx = NeededNamesCtx::new(&self.scan_output);
        let entry_count = self.scan_output.entry_points.len();

        let mut stage = GenerateStage::new(
            &mut self.scan_output,
            self.allocator,
            options.sourcemap,
            options.cjs_default,
            &options.cwd,
            &self.link_output,
            unique_directives,
            needed_names_ctx,
        );
        let mut all_outputs = Vec::with_capacity(entry_count);
        for generated in stage.generate_all() {
            all_warnings.extend(generated.warnings);
            all_outputs.push(BundleOutput { code: generated.code, map: generated.map });
        }

        BundleResult { output: all_outputs, warnings: all_warnings }
    }
}

fn collect_unique_directives(scan_output: &ScanStageOutput) -> Vec<String> {
    let mut seen_set: rustc_hash::FxHashSet<&str> = rustc_hash::FxHashSet::default();
    let mut unique_directives = Vec::new();
    for module in &scan_output.module_table {
        for directive in &module.reference_directives {
            if seen_set.insert(directive.as_str()) {
                unique_directives.push(directive.clone());
            }
        }
    }
    unique_directives
}

/// Convenience wrapper for the bundling pipeline.
pub struct TypackBundler;

impl TypackBundler {
    /// Bundle `.d.ts` files, producing one output per entry point.
    ///
    /// This is a convenience method that creates a [`Bundle`] and immediately
    /// generates output. For more control over the pipeline stages, use
    /// [`Bundle::new`] and [`Bundle::generate`] directly.
    ///
    /// # Errors
    ///
    /// Returns `Err` with a list of `OxcDiagnostic` when fatal errors occur,
    /// such as parse failures or unresolvable import specifiers.
    pub fn bundle(options: &TypackOptions) -> Result<BundleResult, Vec<OxcDiagnostic>> {
        let allocator = Allocator::default();
        let mut bundle = Bundle::new(options, &allocator)?;
        Ok(bundle.generate(&GenerateOptions::from_typack_options(options)))
    }
}
