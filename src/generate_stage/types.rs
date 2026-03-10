//! Data structures shared across generate-stage submodules.

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashMap;

pub(super) use crate::link_stage::{
    ExportedName, ExternalImport, ImportSpecifier, ImportSpecifierKind,
};

/// An `export * from "mod"` to be preserved in the output.
pub(super) struct ExternalStarExport {
    pub(super) source: String,
    pub(super) is_type_only: bool,
}

#[derive(Default)]
pub(super) struct GenerateAcc {
    pub(super) exports: Vec<ExportedName>,
    pub(super) imports: Vec<ExternalImport>,
    pub(super) star_exports: Vec<ExternalStarExport>,
    pub(super) has_any_export_statement: bool,
    pub(super) ns_name_map: FxHashMap<String, String>,
    pub(super) ns_wrapper_blocks: String,
    pub(super) warnings: Vec<OxcDiagnostic>,
    /// Start index of imports for the current module (set before render_module).
    pub(super) module_imports_start: usize,
    /// Start index of exports for the current module (set before render_module).
    pub(super) module_exports_start: usize,
}
