//! Data types for the link stage output.

use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexVec;
use oxc_syntax::symbol::{SymbolFlags, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::types::{Module, ModuleIdx};

/// What to do with each statement during the transform phase.
pub enum StatementAction {
    /// Skip this statement entirely (tree-shaken, consumed as metadata, or internal import).
    Skip,
    /// Move this statement as-is and include in output.
    Include,
    /// Move the inner declaration from an `export named`, add `declare`, adjust span.
    UnwrapExportDeclaration,
    /// Move the inner declaration from an `export default`, convert to named declaration.
    UnwrapExportDefault,
}

/// Per-module analysis results computed during the link stage.
/// Contains inclusion decisions and import resolution data
/// that the generate stage consumes.
pub struct ModuleLinkMeta {
    /// Per-statement actions (indexed by position in original body).
    pub statement_actions: Vec<StatementAction>,
    /// Import renames: local symbol → resolved name from source module.
    pub import_renames: FxHashMap<SymbolId, String>,
    /// Internal namespace alias symbols.
    pub ns_aliases: FxHashSet<SymbolId>,
    /// External namespace info: symbol → (source, local_name).
    pub external_ns_info: FxHashMap<SymbolId, (String, String)>,
    /// Names from re-exported imports that must survive pruning.
    pub reexported_import_names: FxHashSet<String>,
    /// Whether the module needs structural AST mutations beyond simple renames.
    /// True when the module has namespace aliases to strip or inline import
    /// types (`import("...")`) referencing internal modules that need rewriting.
    pub needs_structural_mutation: bool,
}

/// An import specifier collected from an external import.
pub struct ImportSpecifier {
    pub local: String,
    pub kind: ImportSpecifierKind,
}

pub enum ImportSpecifierKind {
    Namespace,
    Default,
    Named(String),
}

impl ImportSpecifierKind {
    pub fn sort_key(&self) -> &str {
        match self {
            Self::Namespace => "*",
            Self::Default => "default",
            Self::Named(name) => name.as_str(),
        }
    }
}

/// An external import to be preserved in the output.
pub struct ExternalImport {
    pub source: String,
    pub specifiers: Vec<ImportSpecifier>,
    pub is_type_only: bool,
    pub side_effect_only: bool,
    /// When `true`, this import was created from an `export { ... } from "external"`
    /// re-export and should not be pruned by the per-module tree-shaking filter.
    pub from_reexport: bool,
}

/// An exported name with optional rename info.
pub struct ExportedName {
    /// The local name (used in the declaration).
    pub local: String,
    /// The exported name (used in the export statement). Same as local unless renamed.
    pub exported: String,
    /// Whether this specifier should be emitted with `type` modifier.
    pub is_type_only: bool,
}

/// Info for creating a namespace wrapper around a module.
pub struct NamespaceWrapInfo {
    /// The namespace name, e.g. `foo_d_exports`.
    pub namespace_name: String,
    /// Exported names from the wrapped module (for the namespace export list).
    pub export_names: Vec<ExportedName>,
}

/// Canonical name mappings for resolving name conflicts across bundled modules.
///
/// When multiple modules declare names that collide, the link stage builds canonical
/// name mappings from old names to conflict-free alternatives (e.g. `Foo` → `Foo$1`).
#[derive(Default, Clone)]
pub struct CanonicalNames {
    /// Symbol-based renames grouped by module for O(1) per-module lookup.
    per_module_symbols: IndexVec<ModuleIdx, FxHashMap<SymbolId, String>>,
    /// Fallback name renames for when symbol resolution isn't possible
    /// (e.g. names from declaration merging).
    pub fallback_name_renames: FxHashMap<(ModuleIdx, String), String>,
    /// Names already claimed in the output scope. Used during rename planning
    /// to detect collisions and allocate `$N` suffixes.
    pub used_names: FxHashSet<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum NeededReason {
    EntryNamedReexport,
    EntryStarReexport,
    PropagationNamedReexport,
    PropagationStarReexport,
    SemanticDependency,
    NamespaceRequirement,
    CrossModuleImportDependency,
    InlineImportReference,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct NeededKindFlags(u8);

impl NeededKindFlags {
    pub const NONE: Self = Self(0);
    pub const VALUE: Self = Self(1 << 0);
    pub const TYPE: Self = Self(1 << 1);
    pub const ALL: Self = Self(Self::VALUE.0 | Self::TYPE.0);

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn from_symbol_flags(flags: SymbolFlags) -> Self {
        let mut kinds = Self::NONE;
        if flags.can_be_referenced_by_value() {
            kinds = kinds.union(Self::VALUE);
        }
        if flags.can_be_referenced_by_type() {
            kinds = kinds.union(Self::TYPE);
        }
        kinds
    }
}

/// Tracks which names from each non-entry module are actually needed in the bundle.
///
/// Names not in this plan are filtered out during generate to minimize output size.
#[derive(Default)]
pub struct NeededNamesPlan {
    /// Per-module needed symbols. `None` means all declarations are needed (e.g. entry module),
    /// `Some(set)` restricts to the given root-scope symbols, `Some(empty)` means nothing is needed.
    pub map: FxHashMap<ModuleIdx, Option<FxHashSet<SymbolId>>>,
    /// Per-module needed declaration spaces keyed by root symbol.
    /// `None` means all declarations are needed.
    pub symbol_kinds: FxHashMap<ModuleIdx, Option<FxHashMap<SymbolId, NeededKindFlags>>>,
    /// Diagnostic info: why each name was determined to be needed (for testing/debugging).
    /// Only read in tests via `reasons_for()`.
    #[cfg_attr(not(test), expect(dead_code))]
    pub reasons: FxHashMap<(ModuleIdx, String), FxHashSet<NeededReason>>,
}

impl NeededNamesPlan {
    #[cfg(test)]
    pub fn reasons_for(
        &self,
        module_idx: ModuleIdx,
        name: &str,
    ) -> Option<&FxHashSet<NeededReason>> {
        self.reasons.get(&(module_idx, name.to_string()))
    }

    /// Check whether a specific symbol is needed for a module.
    #[cfg(test)]
    pub fn contains_symbol(&self, module: &Module<'_>, name: &str) -> bool {
        let Some(entry) = self.map.get(&module.idx) else { return false };
        let Some(set) = entry else { return true }; // None = all needed
        let Some(symbol_id) = module.scoping.get_root_binding(oxc_span::Ident::from(name)) else {
            return false;
        };
        set.contains(&symbol_id)
    }
}

/// Global link-stage output computed once across all entries.
pub struct LinkStageOutput {
    pub canonical_names: CanonicalNames,
    pub default_export_names: IndexVec<ModuleIdx, Option<String>>,
    pub reserved_decl_names: FxHashSet<String>,
    pub all_module_aliases: IndexVec<ModuleIdx, FxHashMap<SymbolId, ModuleIdx>>,
    pub warnings: Vec<OxcDiagnostic>,
}

/// Per-entry link data containing the needed names plan and per-module link metadata.
pub struct PerEntryLinkData {
    #[expect(dead_code)]
    pub needed_names_plan: NeededNamesPlan,
    /// Pre-computed per-module analysis from the link stage.
    pub module_metas: IndexVec<ModuleIdx, Option<ModuleLinkMeta>>,
    /// Modules that need namespace wrappers for this entry.
    pub namespace_wraps: IndexVec<ModuleIdx, Option<NamespaceWrapInfo>>,
    /// Entry-level `import * as X` aliases: local symbol → source module.
    pub namespace_aliases: FxHashMap<SymbolId, ModuleIdx>,
    /// Reserved declaration names (from global + namespace wrap names).
    pub helper_reserved_names: FxHashSet<String>,
    /// Warnings produced during namespace deconfliction.
    pub namespace_warnings: Vec<OxcDiagnostic>,
}

impl CanonicalNames {
    pub fn with_module_count(n: usize) -> Self {
        Self {
            per_module_symbols: std::iter::repeat_with(FxHashMap::default).take(n).collect(),
            fallback_name_renames: FxHashMap::default(),
            used_names: FxHashSet::default(),
        }
    }

    pub fn resolve_symbol(&self, module_idx: ModuleIdx, symbol_id: SymbolId) -> Option<&str> {
        self.per_module_symbols[module_idx].get(&symbol_id).map(String::as_str)
    }

    pub fn resolve_name(&self, module: &Module<'_>, name: &str) -> Option<&str> {
        module
            .scoping
            .get_root_binding(oxc_span::Ident::from(name))
            .and_then(|symbol_id| self.resolve_symbol(module.idx, symbol_id))
            .or_else(|| {
                self.fallback_name_renames.get(&(module.idx, name.to_string())).map(String::as_str)
            })
    }

    /// Get all symbol renames for a specific module (O(1) lookup).
    pub fn module_symbol_renames(
        &self,
        module_idx: ModuleIdx,
    ) -> Option<&FxHashMap<SymbolId, String>> {
        let map = &self.per_module_symbols[module_idx];
        if map.is_empty() { None } else { Some(map) }
    }

    /// Insert a symbol rename for a specific module.
    pub fn insert_symbol_rename(
        &mut self,
        module_idx: ModuleIdx,
        symbol_id: SymbolId,
        new_name: String,
    ) {
        self.per_module_symbols[module_idx].insert(symbol_id, new_name);
    }
}
