//! Generate stage: transforms per-module ASTs and emits the bundled `.d.ts`
//! output.
//!
//! Coordinates the final pipeline stage: applies tree-shaking, semantic renames,
//! inline import rewriting, and namespace wrapping, then assembles per-module
//! codegen output into a single declaration file.

mod analysis;
mod emit;
mod finalizer;
pub mod namespace;
mod render_module;
mod source_joiner;
mod sourcemap;
pub mod types;

use std::collections::VecDeque;
use std::fmt::Write;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_diagnostics::OxcDiagnostic;

use crate::link_stage::{LinkStageOutput, NeededNamesCtx, build_per_entry_link_data};
use crate::scan_stage::ScanStageOutput;
use crate::types::ModuleIdx;
use render_module::{RenderedModule, render_module};
use source_joiner::SourceJoiner;
use types::*;

/// Generate stage: produces the bundled `.d.ts` output.
pub struct GenerateStage<'a, 'b> {
    scan_result: &'b mut ScanStageOutput<'a>,
    allocator: &'a Allocator,
    sourcemap: bool,
    cjs_default: bool,
    cwd: &'b Path,
    link_output: &'b LinkStageOutput,
    /// Pre-computed unique reference directives (shared across entries).
    unique_directives: Vec<String>,
    /// Pre-computed declaration graphs and root names.
    needed_names_ctx: NeededNamesCtx,
}

/// Output from generate stage.
pub struct GenerateOutput {
    pub code: String,
    pub map: Option<oxc_sourcemap::SourceMap>,
    pub warnings: Vec<OxcDiagnostic>,
}

impl<'a, 'b> GenerateStage<'a, 'b> {
    pub fn new(
        scan_result: &'b mut ScanStageOutput<'a>,
        allocator: &'a Allocator,
        sourcemap: bool,
        cjs_default: bool,
        cwd: &'b Path,
        link_output: &'b LinkStageOutput,
        unique_directives: Vec<String>,
        needed_names_ctx: NeededNamesCtx,
    ) -> Self {
        Self {
            scan_result,
            allocator,
            sourcemap,
            cjs_default,
            cwd,
            link_output,
            unique_directives,
            needed_names_ctx,
        }
    }

    /// Generate the bundled `.d.ts` output for all entries.
    pub fn generate_all(&mut self) -> Vec<GenerateOutput> {
        let single_entry = self.scan_result.entry_points.len() == 1;

        // For multi-entry: pre-apply global renames to all module ASTs so that
        // rename-only modules (no structural mutations) can skip cloning entirely.
        if !single_entry {
            self.pre_apply_global_renames();
        }

        self.scan_result
            .entry_points
            .clone()
            .iter()
            .map(|&entry_idx| self.generate_entry(entry_idx, single_entry))
            .collect()
    }

    /// Pre-apply canonical name renames and import renames permanently to all
    /// module ASTs. Since renames are global (identical across entries), this is
    /// safe to do once before the entry loop.
    fn pre_apply_global_renames(&mut self) {
        use crate::generate_stage::finalizer::RenameApplier;
        use oxc_ast_visit::VisitMut;

        for module_idx_usize in 0..self.scan_result.module_table.len() {
            let module_idx = ModuleIdx::from_usize(module_idx_usize);
            let module = &self.scan_result.module_table[module_idx];

            // Reuse link stage's compute_module_link_meta to get import_renames,
            // avoiding duplicated import rename logic.
            let meta = crate::link_stage::module_meta::compute_module_link_meta(
                self.scan_result,
                module_idx,
                None,
                &self.link_output.canonical_names,
                &self.link_output.default_export_names,
            );
            let merged_renames = render_module::build_merged_renames(
                &self.link_output.canonical_names,
                module_idx,
                &meta.import_renames,
            );
            if merged_renames.is_empty() {
                continue;
            }

            let mut applier = RenameApplier {
                allocator: self.allocator,
                scoping: &module.scoping,
                renamed_symbols: &merged_renames,
            };
            applier.visit_statements(&mut self.scan_result.ast_table[module_idx].body);
        }
    }

    /// Generate the bundled `.d.ts` output for a single entry.
    fn generate_entry(&mut self, entry_idx: ModuleIdx, single_entry: bool) -> GenerateOutput {
        let mut joiner = SourceJoiner::default();
        let mut acc = GenerateAcc::default();
        let per_entry = build_per_entry_link_data(
            self.scan_result,
            entry_idx,
            &self.needed_names_ctx,
            self.link_output,
        );

        for directive in &self.unique_directives {
            joiner.append_raw(format!("{directive}\n"));
        }

        acc.warnings.extend(per_entry.namespace_warnings.iter().cloned());

        let mut module_outputs: VecDeque<RenderedModule> = VecDeque::new();
        for module_idx_usize in (0..self.scan_result.module_table.len()).rev() {
            let module_idx = ModuleIdx::from_usize(module_idx_usize);

            let module_has_augmentation =
                self.scan_result.module_table[module_idx].has_augmentation;

            // Check if the module is needed: either it has pre-computed link meta,
            // or it has augmentation declarations.
            let meta = per_entry.module_metas.get(&module_idx);
            if meta.is_none() && !module_has_augmentation {
                continue;
            }

            // For augmentation-only modules without link meta, compute a minimal meta
            // on the fly (all statements included, no renames).
            let fallback_meta;
            let meta = if let Some(m) = meta {
                m
            } else {
                fallback_meta = crate::link_stage::module_meta::compute_module_link_meta(
                    self.scan_result,
                    module_idx,
                    None,
                    &self.link_output.canonical_names,
                    &self.link_output.default_export_names,
                );
                &fallback_meta
            };

            // Phase 1: Output collection — walk statements using pre-computed
            // actions and collect exports, imports, star exports into acc.
            acc.module_exports_start = acc.exports.len();
            acc.module_imports_start = acc.imports.len();
            analysis::collect_module_outputs(
                self.scan_result,
                module_idx,
                meta,
                &per_entry,
                self.link_output,
                &mut acc,
            );

            if let Some(rendered) = render_module(
                self.scan_result,
                self.allocator,
                module_idx,
                meta,
                &per_entry,
                self.link_output,
                self.sourcemap,
                self.cwd,
                &mut acc,
                single_entry,
            ) {
                module_outputs.push_front(rendered);
            }
        }

        // Emit merged external imports before region markers
        let had_imports = !acc.imports.is_empty();
        let mut external_imports_output = String::new();
        emit::write_external_imports(&mut acc.imports, &mut external_imports_output);
        joiner.append_raw(external_imports_output);

        // Emit star re-exports after imports but before regions
        let mut star_exports_output = String::new();
        for star in &acc.star_exports {
            let type_str = if star.is_type_only { "type " } else { "" };
            writeln!(star_exports_output, "export {type_str}* from \"{}\";", star.source).unwrap();
        }
        joiner.append_raw(star_exports_output);

        let has_module_output =
            !acc.ns_wrapper_blocks.is_empty() || module_outputs.iter().any(|m| !m.code.is_empty());

        // Blank line between imports/star-exports and region markers
        if (had_imports || !acc.star_exports.is_empty()) && has_module_output {
            joiner.append_raw("\n");
        }

        if !acc.ns_wrapper_blocks.is_empty() {
            joiner.append_raw(std::mem::take(&mut acc.ns_wrapper_blocks));
        }

        // Emit namespace-wrapped modules first, then regular modules.
        // Consume module_outputs by value to avoid cloning code/map strings.
        let (ns_wrapped, regular): (Vec<_>, Vec<_>) =
            module_outputs.into_iter().partition(|m| m.is_ns_wrapped);
        for module in ns_wrapped {
            if let Some(wrapper) = module.namespace_wrapper {
                joiner.append_raw(wrapper);
            }
            if !module.code.is_empty() {
                joiner.append_mapped(module.code, module.map);
            }
        }
        for module in regular {
            if module.code.is_empty() {
                continue;
            }
            joiner.append_raw(format!("//#region {}\n", module.relative_path));
            joiner.append_mapped(module.code, module.map);
            joiner.append_raw("//#endregion\n");
        }

        // Note: entry module's own export local names are updated during
        // process_statement (for declarations and local re-exports), not here.
        // This avoids renaming re-export local names that already have correct
        // post-rename names from their source modules.

        // Consolidated export statement
        let final_exports = &acc.exports;

        if let Some(default_local) = self.cjs_default_export_local(final_exports) {
            joiner.append_raw(format!("export = {default_local};"));
        } else if !final_exports.is_empty() {
            let mut export_output = String::new();
            emit::write_export_statement(final_exports, &mut export_output);
            joiner.append_raw(export_output);
        } else if acc.has_any_export_statement && acc.star_exports.is_empty() {
            // Source had `export {}` with no actual exports — preserve the empty export
            joiner.append_raw("export { };");
        }

        let (mut code, map) = joiner.join();
        while code.ends_with('\n') {
            code.pop();
        }

        GenerateOutput { code, map, warnings: acc.warnings }
    }

    fn cjs_default_export_local<'s>(&self, exports: &'s [ExportedName]) -> Option<&'s str> {
        if !self.cjs_default || exports.len() != 1 {
            return None;
        }
        let only = &exports[0];
        if only.exported == "default" && !only.is_type_only {
            return Some(only.local.as_str());
        }
        None
    }
}
