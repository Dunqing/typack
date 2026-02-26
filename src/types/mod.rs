//! Core data types shared across all pipeline stages.

mod module;
mod module_info;

pub use module::{Module, ModuleIdx};
pub use module_info::{
    ExportEntry, ExportSource, ImportBinding, ImportBindingKind, ModuleExportImportInfo,
    StarReexport,
};
