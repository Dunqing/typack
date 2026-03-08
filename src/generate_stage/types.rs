//! Data structures shared across generate-stage submodules.

use oxc_diagnostics::OxcDiagnostic;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::link_stage::{NeededKindFlags, RenamePlan};
use crate::types::ModuleIdx;

/// An exported name with optional rename info.
pub(super) struct ExportedName {
    /// The local name (used in the declaration).
    pub(super) local: String,
    /// The exported name (used in the export statement). Same as local unless renamed.
    pub(super) exported: String,
    /// Whether this specifier should be emitted with `type` modifier.
    pub(super) is_type_only: bool,
}

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

/// Info for creating a namespace wrapper around a module.
pub(super) struct NamespaceWrapInfo {
    /// The namespace name, e.g. `foo_d_exports`.
    pub(super) namespace_name: String,
    /// Exported names from the wrapped module (for the namespace export list).
    pub(super) export_names: Vec<ExportedName>,
}

pub(super) struct ModuleOutput {
    pub(super) relative_path: String,
    pub(super) is_ns_wrapped: bool,
    pub(super) namespace_wrapper: Option<String>,
    pub(super) code: String,
    pub(super) map: Option<oxc_sourcemap::SourceMap>,
}

pub(super) struct GenerateSharedCtx<'s> {
    pub(super) namespace_wraps: &'s FxHashMap<ModuleIdx, NamespaceWrapInfo>,
    pub(super) namespace_aliases: &'s FxHashMap<SymbolId, ModuleIdx>,
    pub(super) rename_plan: &'s RenamePlan,
    pub(super) needed_symbol_kinds:
        &'s FxHashMap<ModuleIdx, Option<FxHashMap<SymbolId, NeededKindFlags>>>,
    pub(super) default_export_names: &'s FxHashMap<ModuleIdx, String>,
    pub(super) helper_reserved_names: &'s FxHashSet<String>,
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

/// What to do with each statement during the transform phase.
pub(super) enum StatementAction {
    /// Skip this statement entirely (tree-shaken, consumed as metadata, or internal import).
    Skip,
    /// Clone this statement as-is and include in output.
    Include,
    /// Clone the inner declaration from an `export named`, add `declare`, adjust span.
    UnwrapExportDeclaration,
    /// Clone the inner declaration from an `export default`, convert to named declaration.
    UnwrapExportDefault,
}

/// Result of the read-only analysis phase for one module.
///
/// Contains per-module data needed by the transform phase. Exports, imports,
/// and star exports are written directly into `GenerateAcc` during analysis
/// to avoid intermediate allocations.
pub(super) struct ModuleAnalysis {
    /// Per-statement actions (indexed by position in original body).
    pub(super) statement_actions: Vec<StatementAction>,
    /// Import renames: local symbol → resolved name from source module.
    pub(super) import_renames: FxHashMap<SymbolId, String>,
    /// Internal namespace alias symbols.
    pub(super) ns_aliases: FxHashSet<SymbolId>,
    /// External namespace info: symbol → (source, local_name).
    pub(super) external_ns_info: FxHashMap<SymbolId, (String, String)>,
    /// Names from re-exported imports that must survive pruning.
    pub(super) reexported_import_names: FxHashSet<String>,
}
