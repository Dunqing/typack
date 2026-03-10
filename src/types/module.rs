//! Per-module data collected during the scan stage.
//!
//! Defines [`Module`], which holds the parsed AST, scoping information,
//! resolved specifiers, and metadata for each `.d.ts` file in the module graph.

use std::path::PathBuf;

use oxc_index::define_index_type;
use oxc_semantic::Scoping;
use oxc_sourcemap::SourceMap;
use rustc_hash::{FxHashMap, FxHashSet};

use super::module_info::ModuleExportImportInfo;

define_index_type! {
    /// Index into the module table.
    pub struct ModuleIdx = u32;
}

/// Per-module data collected during the scan stage.
pub struct Module<'a> {
    /// Index of this module in the module table.
    pub idx: ModuleIdx,
    /// Absolute path to the source file.
    pub path: PathBuf,
    /// Path relative to CWD, used for region markers.
    pub relative_path: String,
    /// The raw source text, allocated in the shared arena.
    pub source: &'a str,
    /// Scope/symbol information from semantic analysis.
    pub scoping: Scoping,
    /// Leading `/// <reference ... />` directives collected during scan.
    pub reference_directives: Vec<String>,
    /// Whether this module contains top-level global/module augmentation declarations.
    pub has_augmentation: bool,
    /// Resolved internal specifiers for this module (specifier -> target module idx).
    pub resolved_internal_specifiers: FxHashMap<String, ModuleIdx>,
    /// External specifiers referenced by this module (imports/exports/inline imports).
    pub resolved_external_specifiers: FxHashSet<String>,
    /// Whether this module is an entry point.
    pub is_entry: bool,
    /// Pre-loaded input sourcemap from an adjacent `.d.ts.map` file, if present.
    /// Used to compose mappings back to original `.ts` sources.
    pub input_sourcemap: Option<SourceMap>,
    /// Pre-computed export/import information, collected once during scan.
    pub export_import_info: ModuleExportImportInfo,
}

impl Module<'_> {
    #[inline]
    pub fn resolve_internal_specifier(&self, specifier: &str) -> Option<ModuleIdx> {
        self.resolved_internal_specifiers.get(specifier).copied()
    }
}
