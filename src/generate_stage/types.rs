//! Data structures shared across generate-stage submodules.

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashMap;

use crate::types::ModuleIdx;

pub(super) use crate::link_stage::{
    ExportedName, ExternalImport, ImportSpecifier, ImportSpecifierKind,
};

/// An `export * from "mod"` to be preserved in the output.
pub(super) struct ExternalStarExport {
    pub(super) source: String,
    pub(super) is_type_only: bool,
}

pub(super) struct GenerateAcc {
    pub(super) exports: Vec<ExportedName>,
    pub(super) imports: Vec<ExternalImport>,
    pub(super) star_exports: Vec<ExternalStarExport>,
    pub(super) has_any_export_statement: bool,
    pub(super) ns_name_map: FxHashMap<String, String>,
    pub(super) ns_wrapper_blocks: String,
    pub(super) has_kept_inline_exports: bool,
    pub(super) warnings: Vec<OxcDiagnostic>,
    /// Start index of imports for the current module (set before render_module).
    pub(super) module_imports_start: usize,
    /// Start index of exports for the current module (set before render_module).
    pub(super) module_exports_start: usize,
    /// Entry module index for this generation pass.
    pub(super) entry_idx: ModuleIdx,
    /// Whether entry module declarations can keep `export` inline
    /// (true when entry has no renames and is not namespace-wrapped).
    pub(super) keep_entry_exports_inline: bool,
}

impl GenerateAcc {
    pub(super) fn new(entry_idx: ModuleIdx, keep_entry_exports_inline: bool) -> Self {
        Self {
            entry_idx,
            keep_entry_exports_inline,
            exports: Vec::new(),
            imports: Vec::new(),
            star_exports: Vec::new(),
            has_any_export_statement: false,
            ns_name_map: FxHashMap::default(),
            ns_wrapper_blocks: String::new(),
            has_kept_inline_exports: false,
            warnings: Vec::new(),
            module_imports_start: 0,
            module_exports_start: 0,
        }
    }

    /// Whether the given module's export declarations should keep the `export` keyword.
    #[inline]
    pub(super) fn should_keep_export_inline(&self, module_idx: ModuleIdx) -> bool {
        self.keep_entry_exports_inline && module_idx == self.entry_idx
    }
}
