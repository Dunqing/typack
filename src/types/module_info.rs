//! Pre-computed export and import information collected during the scan stage.
//!
//! These maps are populated once per module (from the parsed AST) and then used
//! throughout the link and generate stages, eliminating repeated AST walks.

use rustc_hash::FxHashMap;

/// How a name is exported from a module.
#[derive(Debug, Clone)]
pub enum ExportSource {
    /// `export interface Foo {}` — local declaration.
    LocalDeclaration,
    /// `export { X }` or `export { X as Y }` — local binding re-export (no source).
    LocalReexport,
    /// `export { X } from "./mod"` or `export { X as Y } from "./mod"`.
    SourceReexport {
        specifier: String,
        /// The imported name in the source module (the `X` in `export { X as Y } from`).
        imported_name: String,
    },
    /// `export default class Foo {}`, `export default function bar() {}`, etc.
    Default,
}

/// A single named export entry.
#[derive(Debug, Clone)]
pub struct ExportEntry {
    /// The local declaration name in this module (may differ from exported_name for aliases).
    pub local_name: String,
    /// How this export is sourced.
    pub source: ExportSource,
}

/// A star re-export: `export * from "./mod"` or `export * as ns from "./mod"`.
#[derive(Debug, Clone)]
pub struct StarReexport {
    /// The import specifier string.
    pub specifier: String,
    /// `Some("ns")` for `export * as ns from`, `None` for plain `export * from`.
    pub alias: Option<String>,
}

/// What kind of import binding this is.
#[derive(Debug, Clone)]
pub enum ImportBindingKind {
    /// `import { X }` or `import { X as Y }` — the imported (remote) name.
    Named(String),
    /// `import X from "..."` — default import.
    Default,
    /// `import * as X from "..."` — namespace import.
    Namespace,
}

/// A single import binding (local binding name → source info).
#[derive(Debug, Clone)]
pub struct ImportBinding {
    /// The import specifier string (e.g. `"./mod"` or `"react"`).
    pub source_specifier: String,
    /// What kind of import this is.
    pub kind: ImportBindingKind,
}

/// Pre-computed export/import information for a module.
#[derive(Debug, Default)]
pub struct ModuleExportImportInfo {
    /// Named exports: exported_name → export entry.
    ///
    /// Includes declaration exports (`export interface Foo {}`), specifier exports
    /// (`export { X as Y }`), source re-exports (`export { X } from "./mod"`), and
    /// default exports (keyed as `"default"`).
    pub named_exports: FxHashMap<String, ExportEntry>,

    /// Star re-export entries (order-preserving).
    pub star_reexports: Vec<StarReexport>,

    /// Import bindings: local_name → import info.
    ///
    /// Includes named imports (`import { X }`), default imports (`import X from`),
    /// and namespace imports (`import * as X from`).
    pub named_imports: FxHashMap<String, ImportBinding>,

    /// Locally declared exported names (from `export <declaration>` only).
    /// This is the subset of `named_exports` with `LocalDeclaration` source.
    pub declared_export_names: Vec<String>,
}
