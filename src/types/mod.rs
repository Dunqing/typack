//! Core data types shared across all pipeline stages.

mod ast_table;
mod module;
mod module_info;
mod module_table;
pub use ast_table::AstTable;
pub use module::{Module, ModuleIdx};
pub use module_info::{
    ExportEntry, ExportSource, ImportBinding, ImportBindingKind, ModuleExportImportInfo,
    StarReexport,
};
pub use module_table::ModuleTable;
