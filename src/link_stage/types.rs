//! Data types for the link stage output.

use oxc_diagnostics::OxcDiagnostic;
use oxc_syntax::symbol::{SymbolFlags, SymbolId};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::types::{Module, ModuleIdx};

/// Rename plan for resolving name conflicts across bundled modules.
///
/// When multiple modules declare names that collide, the link stage builds a rename
/// plan mapping old names to conflict-free alternatives (e.g. `Foo` → `Foo$1`).
#[derive(Default)]
pub struct RenamePlan {
    /// Renames keyed by (module, symbol). Uses `SymbolId` for precise renaming
    /// that respects scoping and avoids false matches.
    pub symbol_renames: FxHashMap<ModuleIdx, FxHashMap<SymbolId, String>>,
    /// Renames keyed by name string (fallback). Used when a name couldn't be
    /// resolved to a semantic symbol (e.g. names from declaration merging).
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

pub struct LinkOutput {
    pub rename_plan: RenamePlan,
    pub needed_names_plan: NeededNamesPlan,
    pub default_export_names: FxHashMap<ModuleIdx, String>,
    pub warnings: Vec<OxcDiagnostic>,
}

impl RenamePlan {
    pub fn resolve_symbol(&self, module_idx: ModuleIdx, symbol_id: SymbolId) -> Option<&str> {
        self.symbol_renames.get(&module_idx)?.get(&symbol_id).map(String::as_str)
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

    /// Get all symbol renames for a specific module.
    pub fn module_symbol_renames(
        &self,
        module_idx: ModuleIdx,
    ) -> Option<&FxHashMap<SymbolId, String>> {
        self.symbol_renames.get(&module_idx)
    }

    /// Insert a symbol rename for a specific module.
    pub fn insert_symbol_rename(
        &mut self,
        module_idx: ModuleIdx,
        symbol_id: SymbolId,
        new_name: String,
    ) {
        self.symbol_renames.entry(module_idx).or_default().insert(symbol_id, new_name);
    }
}
