//! Data structures shared across generate-stage submodules.

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashMap;

pub(super) use crate::link_stage::ExportedName;

/// An import specifier collected from an external import.
pub(super) struct ImportSpecifier {
    pub(super) local: String,
    pub(super) kind: ImportSpecifierKind,
}

pub(super) enum ImportSpecifierKind {
    Namespace,
    Default,
    Named(String),
}

impl ImportSpecifierKind {
    pub(super) fn sort_key(&self) -> &str {
        match self {
            Self::Namespace => "*",
            Self::Default => "default",
            Self::Named(name) => name.as_str(),
        }
    }
}

/// An external import to be preserved in the output.
pub(super) struct ExternalImport {
    pub(super) source: String,
    pub(super) specifiers: Vec<ImportSpecifier>,
    pub(super) is_type_only: bool,
    pub(super) side_effect_only: bool,
    /// When `true`, this import was created from an `export { ... } from "external"`
    /// re-export and should not be pruned by the per-module tree-shaking filter.
    pub(super) from_reexport: bool,
}

/// An `export * from "mod"` to be preserved in the output.
pub(super) struct ExternalStarExport {
    pub(super) source: String,
    pub(super) is_type_only: bool,
}

pub(super) struct ModuleOutput {
    pub(super) relative_path: String,
    pub(super) is_ns_wrapped: bool,
    pub(super) namespace_wrapper: Option<String>,
    pub(super) code: String,
    pub(super) map: Option<oxc_sourcemap::SourceMap>,
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
}
