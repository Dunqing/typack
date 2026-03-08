//! Generate stage: transforms per-module ASTs and emits the bundled `.d.ts`
//! output.
//!
//! Coordinates the final pipeline stage: applies tree-shaking, semantic renames,
//! inline import rewriting, and namespace wrapping, then assembles per-module
//! codegen output into a single declaration file.

mod emit;
pub mod namespace;
mod output_assembler;
mod rewriter;
mod sourcemap;
mod types;

use std::collections::VecDeque;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::AstBuilder;
use oxc_ast::ast::{
    ExportDefaultDeclaration, ExportDefaultDeclarationKind, IdentifierReference, Statement,
    TSTypeName, TSTypeQuery,
};
use oxc_ast_visit::{Visit, VisitMut};
use oxc_codegen::{Codegen, CodegenOptions, IndentChar};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Scoping;
use oxc_span::{GetSpanMut, Ident, SPAN};
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::helpers::collect_decl_names;
use crate::link_stage::exports::{
    find_external_reexport_source, resolve_export_local_name, resolve_export_origin,
};
use crate::link_stage::{LinkStageOutput, NeededKindFlags, PerEntryLinkData, RenamePlan};
use crate::scan_stage::ScanResult;
use crate::types::{Module, ModuleIdx};
use namespace::{
    apply_namespace_wrap_renames, build_module_exports_cache, collect_declaration_names,
    collect_export_specifier, deconflict_namespace_wrap_names, pre_scan_namespace_info,
};
use output_assembler::OutputAssembler;
use rewriter::{InlineImportAndNamespaceRewriter, SemanticRenamer, ensure_declare_on_declaration};
use types::*;

/// Generate stage: produces the bundled `.d.ts` output.
pub struct GenerateStage<'a, 'b> {
    scan_result: &'b ScanResult<'a>,
    entry: EntryGenerateContext<'b>,
    cjs_default: bool,
    link_output: &'b LinkStageOutput,
    shared_output: &'b SharedGenerateOutput,
}

/// Output from generate stage.
pub struct GenerateOutput {
    pub code: String,
    pub map: Option<oxc_sourcemap::SourceMap>,
    pub warnings: Vec<OxcDiagnostic>,
}

pub fn build_shared_generate_output<'a>(
    scan_result: &ScanResult<'a>,
    allocator: &'a Allocator,
    sourcemap: bool,
    cwd: &Path,
    link_output: &LinkStageOutput,
) -> SharedGenerateOutput {
    let module_exports = build_module_exports_cache(scan_result);
    let mut modules = FxHashMap::default();
    for module in &scan_result.modules {
        let analysis = build_shared_module_analysis(module.idx, scan_result, link_output);
        let mut ns_name_map = FxHashMap::default();
        let statements = module
            .program
            .body
            .iter()
            .enumerate()
            .map(|(stmt_idx, stmt)| {
                prepare_statement_output(
                    module.idx,
                    stmt_idx,
                    stmt,
                    &analysis,
                    scan_result,
                    allocator,
                    sourcemap,
                    cwd,
                    link_output,
                    &mut ns_name_map,
                )
                .map(Arc::new)
            })
            .collect();
        modules.insert(module.idx, PreparedModule { analysis, statements });
    }
    SharedGenerateOutput { modules, module_exports }
}

#[cfg(test)]
mod test_stats {
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub(super) static SHARED_MODULE_PREPARES: AtomicUsize = AtomicUsize::new(0);
    pub(super) static SHARED_STATEMENT_PREPARES: AtomicUsize = AtomicUsize::new(0);

    pub(super) fn counts() -> (usize, usize) {
        (
            SHARED_MODULE_PREPARES.load(Ordering::SeqCst),
            SHARED_STATEMENT_PREPARES.load(Ordering::SeqCst),
        )
    }
}

impl<'a, 'b> GenerateStage<'a, 'b> {
    pub fn new(
        scan_result: &'b ScanResult<'a>,
        entry_idx: ModuleIdx,
        per_entry: &'b PerEntryLinkData,
        cjs_default: bool,
        link_output: &'b LinkStageOutput,
        shared_output: &'b SharedGenerateOutput,
    ) -> Self {
        Self {
            scan_result,
            entry: EntryGenerateContext { entry_idx, per_entry },
            cjs_default,
            link_output,
            shared_output,
        }
    }

    /// Generate the bundled `.d.ts` output.
    pub fn generate(&self) -> GenerateOutput {
        let mut output = OutputAssembler::default();
        let mut acc = GenerateAcc::default();
        let rename_plan = &self.link_output.rename_plan;

        // Collect and deduplicate reference directives from all modules
        let mut seen_set: FxHashSet<&str> = FxHashSet::default();
        let mut unique_directives: Vec<&str> = Vec::new();
        for module in &self.scan_result.modules {
            for directive in &module.reference_directives {
                if seen_set.insert(directive.as_str()) {
                    unique_directives.push(directive.as_str());
                }
            }
        }
        for directive in &unique_directives {
            output.push_unmapped(format!("{directive}\n"));
        }

        // Pre-scan all modules for namespace import patterns
        let (mut namespace_wraps, namespace_aliases) = pre_scan_namespace_info(
            self.scan_result,
            self.entry.entry_idx,
            &self.link_output.all_module_aliases,
            &self.shared_output.module_exports,
        );

        // Keep namespace wrapper exports aligned with semantic renames.
        apply_namespace_wrap_renames(&mut namespace_wraps, rename_plan, self.scan_result);
        let mut helper_reserved_names = self.link_output.reserved_decl_names.clone();
        deconflict_namespace_wrap_names(
            &mut namespace_wraps,
            &helper_reserved_names,
            &mut acc.warnings,
        );
        for wrap in namespace_wraps.values() {
            helper_reserved_names.insert(wrap.namespace_name.clone());
        }

        let shared = GenerateSharedCtx {
            namespace_wraps: &namespace_wraps,
            namespace_aliases: &namespace_aliases,
            rename_plan,
            needed_symbol_kinds: &self.entry.per_entry.needed_names_plan.symbol_kinds,
            default_export_names: &self.link_output.default_export_names,
        };

        let mut module_outputs: VecDeque<ModuleOutput> = VecDeque::new();
        for module_idx_usize in (0..self.scan_result.modules.len()).rev() {
            let module_idx = ModuleIdx::from_usize(module_idx_usize);
            if let Some(module_output) = self.generate_module_ast(module_idx, &shared, &mut acc) {
                module_outputs.push_front(module_output);
            }
        }

        // Emit merged external imports before region markers
        let had_imports = !acc.imports.is_empty();
        let mut external_imports_output = String::new();
        emit::write_external_imports(&mut acc.imports, &mut external_imports_output);
        output.push_unmapped(external_imports_output);

        // Emit star re-exports after imports but before regions
        let mut star_exports_output = String::new();
        for star in &acc.star_exports {
            let type_str = if star.is_type_only { "type " } else { "" };
            writeln!(star_exports_output, "export {type_str}* from \"{}\";", star.source).unwrap();
        }
        output.push_unmapped(star_exports_output);

        let has_module_output = !acc.ns_wrapper_blocks.is_empty()
            || module_outputs.iter().any(|m| !m.fragments.is_empty());

        // Blank line between imports/star-exports and region markers
        if (had_imports || !acc.star_exports.is_empty()) && has_module_output {
            output.push_unmapped("\n");
        }

        if !acc.ns_wrapper_blocks.is_empty() {
            for block in std::mem::take(&mut acc.ns_wrapper_blocks) {
                output.push_unmapped(block);
            }
        }

        // Emit namespace-wrapped modules first, then regular modules.
        for module in module_outputs.iter().filter(|m| m.is_ns_wrapped) {
            if let Some(wrapper) = &module.namespace_wrapper {
                output.push_unmapped(wrapper.clone());
            }
            for fragment in &module.fragments {
                output.push_mapped(&fragment.code, fragment.map.clone());
            }
        }
        for module in module_outputs.iter().filter(|m| !m.is_ns_wrapped) {
            if module.fragments.is_empty() {
                continue;
            }
            output.push_unmapped(format!("//#region {}\n", module.relative_path));
            for fragment in &module.fragments {
                output.push_mapped(&fragment.code, fragment.map.clone());
            }
            output.push_unmapped("//#endregion\n");
        }

        // Note: entry module's own export local names are updated during
        // process_statement (for declarations and local re-exports), not here.
        // This avoids renaming re-export local names that already have correct
        // post-rename names from their source modules.

        // Consolidated export statement
        let final_exports = &acc.exports;

        if let Some(default_local) = self.cjs_default_export_local(final_exports) {
            output.push_unmapped(format!("export = {default_local};"));
        } else if !final_exports.is_empty() {
            let mut export_output = String::new();
            emit::write_export_statement(final_exports, &mut export_output);
            output.push_unmapped(export_output);
        } else if acc.has_any_export_statement && acc.star_exports.is_empty() {
            // Source had `export {}` with no actual exports — preserve the empty export
            output.push_unmapped("export { };");
        }

        let mut generated = output.finish();
        while generated.code.ends_with('\n') {
            generated.code.pop();
        }

        generated.warnings = acc.warnings;
        generated
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

    fn generate_module_ast(
        &self,
        module_idx: ModuleIdx,
        shared: &GenerateSharedCtx<'_>,
        acc: &mut GenerateAcc,
    ) -> Option<ModuleOutput> {
        let prepared_module = self
            .shared_output
            .modules
            .get(&module_idx)
            .expect("prepared module output should exist for every scanned module");
        let ns_wrap = shared.namespace_wraps.get(&module_idx);
        let module_has_augmentation = self.scan_result.modules[module_idx].has_augmentation;

        let (module_is_needed, module_needed): (
            bool,
            Option<FxHashMap<SymbolId, NeededKindFlags>>,
        ) = match shared.needed_symbol_kinds.get(&module_idx) {
            Some(entry) => (true, entry.clone()),
            None => (module_has_augmentation, None),
        };

        if !module_is_needed {
            return None;
        }

        // Phase 1: Read-only analysis — determine per-statement actions and
        // collect exports, imports, star exports directly into acc without
        // cloning any AST nodes.
        let exports_start = acc.exports.len();
        let imports_start = acc.imports.len();
        let analysis = self.analyze_module(module_idx, module_needed.as_ref(), shared, acc);

        let mut fragments = Vec::new();
        let mut referenced_names = FxHashSet::default();
        let mut external_ns_members: FxHashMap<String, FxHashSet<String>> = FxHashMap::default();

        for (i, action) in analysis.statement_actions.iter().enumerate() {
            if matches!(action, StatementAction::Skip) {
                continue;
            }
            let Some(prepared) = prepared_module.statements[i].as_ref() else {
                continue;
            };

            acc.imports.extend(prepared.imports.iter().cloned());
            for block in &prepared.ns_wrapper_blocks {
                if acc.seen_ns_wrapper_blocks.insert(block.clone()) {
                    acc.ns_wrapper_blocks.push(block.clone());
                }
            }
            merge_external_ns_members(&mut external_ns_members, &prepared.external_ns_members);
            referenced_names.extend(prepared.referenced_names.iter().cloned());
            acc.warnings.extend(prepared.warnings.iter().cloned());
            fragments.push(Arc::clone(prepared));
        }

        if fragments.is_empty() && ns_wrap.is_none() {
            return None;
        }

        convert_external_namespace_imports(
            &mut acc.imports,
            imports_start,
            &prepared_module.analysis.external_ns_info,
            &external_ns_members,
        );

        let entry_reexported_import_names = FxHashSet::default();
        let reexported_import_names = if module_idx == self.entry.entry_idx {
            &entry_reexported_import_names
        } else {
            &prepared_module.analysis.reexported_import_names
        };

        prune_unused_imports_by_name(
            &mut acc.imports,
            imports_start,
            &referenced_names,
            &acc.exports[exports_start..],
            reexported_import_names,
        );

        let namespace_wrapper = ns_wrap.map(|wrap| {
            let mut out = String::new();
            write!(out, "declare namespace {} {{\n  export {{ ", wrap.namespace_name).unwrap();
            for (i, exp) in wrap.export_names.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                if exp.local == exp.exported {
                    out.push_str(&exp.exported);
                } else {
                    write!(out, "{} as {}", exp.local, exp.exported).unwrap();
                }
            }
            out.push_str(" };\n}\n");
            out
        });

        Some(ModuleOutput {
            relative_path: self.scan_result.modules[module_idx].relative_path.clone(),
            is_ns_wrapped: ns_wrap.is_some(),
            namespace_wrapper,
            fragments,
        })
    }

    /// Read-only analysis phase: walks the original AST without cloning,
    /// determines what action to take for each statement, and collects
    /// exports, imports, star exports, and import rename info.
    fn analyze_module(
        &self,
        module_idx: ModuleIdx,
        needed_symbol_kinds: Option<&FxHashMap<SymbolId, NeededKindFlags>>,
        shared: &GenerateSharedCtx<'_>,
        acc: &mut GenerateAcc,
    ) -> ModuleAnalysis {
        let module = &self.scan_result.modules[module_idx];
        let mut analysis =
            ModuleAnalysis { statement_actions: Vec::with_capacity(module.program.body.len()) };

        // Determine per-statement actions and collect exports/imports/star_exports
        // directly into acc.
        for stmt in &module.program.body {
            let action = self.analyze_statement(stmt, module, needed_symbol_kinds, shared, acc);
            analysis.statement_actions.push(action);
        }

        analysis
    }

    /// Determine the action for a single statement and collect any side-effect
    /// metadata (exports, imports, star exports) directly into `acc`.
    fn analyze_statement(
        &self,
        stmt: &Statement<'a>,
        module: &Module<'a>,
        needed_symbol_kinds: Option<&FxHashMap<SymbolId, NeededKindFlags>>,
        shared: &GenerateSharedCtx<'_>,
        acc: &mut GenerateAcc,
    ) -> StatementAction {
        match stmt {
            Statement::ExportNamedDeclaration(export_decl) => {
                acc.has_any_export_statement = true;
                if let Some(decl) = &export_decl.declaration {
                    if let Some(needed) = needed_symbol_kinds
                        && !declaration_matches_needed_kinds(decl, &module.scoping, needed)
                    {
                        return StatementAction::Skip;
                    }
                    if module.idx == self.entry.entry_idx {
                        let before_len = acc.exports.len();
                        collect_declaration_names(decl, &mut acc.exports);
                        for exp in &mut acc.exports[before_len..] {
                            if let Some(new_name) =
                                shared.rename_plan.resolve_name(module, &exp.local)
                            {
                                exp.local = new_name.to_string();
                            }
                        }
                    }
                    StatementAction::UnwrapExportDeclaration
                } else if let Some(source) = &export_decl.source {
                    let internal_source_idx =
                        module.resolve_internal_specifier(source.value.as_str());
                    if internal_source_idx.is_none() {
                        let specifiers: Vec<ImportSpecifier> = export_decl
                            .specifiers
                            .iter()
                            .map(|spec| {
                                let local_name = spec.local.name();
                                ImportSpecifier {
                                    local: spec.exported.name().to_string(),
                                    kind: if local_name == "default" {
                                        ImportSpecifierKind::Default
                                    } else {
                                        ImportSpecifierKind::Named(local_name.to_string())
                                    },
                                    preserve_if_unused: false,
                                }
                            })
                            .collect();
                        if !specifiers.is_empty() {
                            acc.imports.push(ExternalImport {
                                source: source.value.to_string(),
                                specifiers,
                                is_type_only: false,
                                side_effect_only: false,
                                from_reexport: true,
                            });
                        }
                    }
                    if module.idx == self.entry.entry_idx {
                        for spec in &export_decl.specifiers {
                            let exported_name = spec.exported.name().to_string();
                            if let Some(source_module_idx) = internal_source_idx {
                                let mut local_name = spec.local.name().to_string();
                                let mut local_module_idx = source_module_idx;
                                if let Some((origin_module_idx, resolved)) = resolve_export_origin(
                                    source_module_idx,
                                    &local_name,
                                    self.scan_result,
                                ) {
                                    local_name = resolved;
                                    local_module_idx = origin_module_idx;
                                } else if let Some((ext_source, imported_name)) =
                                    find_external_reexport_source(
                                        source_module_idx,
                                        &local_name,
                                        self.scan_result,
                                    )
                                {
                                    acc.imports.push(ExternalImport {
                                        source: ext_source,
                                        specifiers: vec![ImportSpecifier {
                                            local: exported_name.clone(),
                                            kind: if imported_name == "default" {
                                                ImportSpecifierKind::Default
                                            } else {
                                                ImportSpecifierKind::Named(imported_name)
                                            },
                                            preserve_if_unused: false,
                                        }],
                                        is_type_only: false,
                                        side_effect_only: false,
                                        from_reexport: true,
                                    });
                                    local_name.clone_from(&exported_name);
                                }
                                if local_name == "default"
                                    && let Some(name) =
                                        shared.default_export_names.get(&local_module_idx)
                                {
                                    local_name.clone_from(name);
                                }
                                if let Some(new_name) = shared.rename_plan.resolve_name(
                                    &self.scan_result.modules[local_module_idx],
                                    &local_name,
                                ) {
                                    local_name = new_name.to_string();
                                }
                                acc.exports.push(ExportedName {
                                    local: local_name,
                                    exported: exported_name,
                                    is_type_only: export_decl.export_kind.is_type()
                                        || spec.export_kind.is_type(),
                                });
                            } else {
                                acc.exports.push(ExportedName {
                                    local: exported_name.clone(),
                                    exported: exported_name,
                                    is_type_only: export_decl.export_kind.is_type()
                                        || spec.export_kind.is_type(),
                                });
                            }
                        }
                    }
                    StatementAction::Skip
                } else {
                    // Bare specifiers: `export { X, Y }`
                    if module.idx == self.entry.entry_idx {
                        for spec in &export_decl.specifiers {
                            let exported_name = spec.exported.name().to_string();
                            let spec_is_type =
                                export_decl.export_kind.is_type() || spec.export_kind.is_type();
                            let symbol_id =
                                module.scoping.get_root_binding(Ident::from(spec.local.name()));
                            if let Some(symbol_id) = symbol_id
                                && let Some(source_module_idx) =
                                    shared.namespace_aliases.get(&symbol_id)
                                && let Some(wrap) = shared.namespace_wraps.get(source_module_idx)
                            {
                                acc.exports.push(ExportedName {
                                    local: wrap.namespace_name.clone(),
                                    exported: exported_name,
                                    is_type_only: spec_is_type,
                                });
                            } else {
                                let before_len = acc.exports.len();
                                collect_export_specifier(spec, &mut acc.exports, spec_is_type);
                                for exp in &mut acc.exports[before_len..] {
                                    if let Some(new_name) =
                                        shared.rename_plan.resolve_name(module, &exp.local)
                                    {
                                        exp.local = new_name.to_string();
                                    }
                                }
                            }
                        }
                    }
                    StatementAction::Skip
                }
            }
            Statement::ExportDefaultDeclaration(export_default) => {
                acc.has_any_export_statement = true;
                if let Some(needed) = needed_symbol_kinds
                    && !export_default_declaration_matches_needed_kinds(
                        export_default,
                        &module.scoping,
                        needed,
                    )
                {
                    return StatementAction::Skip;
                }
                match &export_default.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func_decl) => {
                        let name =
                            func_decl.id.as_ref().map_or("export_default", |id| id.name.as_str());
                        if module.idx == self.entry.entry_idx {
                            acc.exports.push(ExportedName {
                                local: name.to_string(),
                                exported: "default".to_string(),
                                is_type_only: false,
                            });
                        }
                        StatementAction::UnwrapExportDefault
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => {
                        let name =
                            class_decl.id.as_ref().map_or("export_default", |id| id.name.as_str());
                        if module.idx == self.entry.entry_idx {
                            acc.exports.push(ExportedName {
                                local: name.to_string(),
                                exported: "default".to_string(),
                                is_type_only: false,
                            });
                        }
                        StatementAction::UnwrapExportDefault
                    }
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface_decl) => {
                        let name = iface_decl.id.name.as_str();
                        if module.idx == self.entry.entry_idx {
                            acc.exports.push(ExportedName {
                                local: name.to_string(),
                                exported: "default".to_string(),
                                is_type_only: false,
                            });
                        }
                        StatementAction::UnwrapExportDefault
                    }
                    ExportDefaultDeclarationKind::Identifier(id) => {
                        if module.idx == self.entry.entry_idx {
                            acc.exports.push(ExportedName {
                                local: id.name.to_string(),
                                exported: "default".to_string(),
                                is_type_only: false,
                            });
                        }
                        StatementAction::Skip
                    }
                    _ => StatementAction::Skip,
                }
            }
            Statement::ExportAllDeclaration(export_all) => {
                acc.has_any_export_statement = true;
                let internal_source_idx =
                    module.resolve_internal_specifier(export_all.source.value.as_str());
                if let Some(exported) = &export_all.exported {
                    let name = exported.name().to_string();
                    if let Some(source_module_idx) = internal_source_idx {
                        if module.idx == self.entry.entry_idx
                            && let Some(wrap) = shared.namespace_wraps.get(&source_module_idx)
                        {
                            acc.exports.push(ExportedName {
                                local: wrap.namespace_name.clone(),
                                exported: name,
                                is_type_only: export_all.export_kind.is_type(),
                            });
                        }
                    } else {
                        acc.imports.push(ExternalImport {
                            source: export_all.source.value.to_string(),
                            specifiers: vec![ImportSpecifier {
                                local: name.clone(),
                                kind: ImportSpecifierKind::Namespace,
                                preserve_if_unused: true,
                            }],
                            is_type_only: false,
                            side_effect_only: false,
                            from_reexport: true,
                        });
                        if module.idx == self.entry.entry_idx {
                            acc.exports.push(ExportedName {
                                local: name.clone(),
                                exported: name,
                                is_type_only: export_all.export_kind.is_type(),
                            });
                        }
                    }
                } else if let Some(source_module_idx) = internal_source_idx {
                    if module.idx == self.entry.entry_idx {
                        let before_len = acc.exports.len();
                        if let Some(cached_exports) =
                            self.shared_output.module_exports.get(&source_module_idx)
                        {
                            acc.exports.extend(cached_exports.export_names.iter().cloned());
                            acc.imports.extend(cached_exports.external_imports.iter().cloned());
                        }
                        for exp in &mut acc.exports[before_len..] {
                            if let Some(new_name) = shared.rename_plan.resolve_name(
                                &self.scan_result.modules[source_module_idx],
                                &exp.local,
                            ) {
                                exp.local = new_name.to_string();
                            }
                        }
                    }
                } else {
                    acc.star_exports.push(ExternalStarExport {
                        source: export_all.source.value.to_string(),
                        is_type_only: false,
                    });
                }
                StatementAction::Skip
            }
            Statement::ImportDeclaration(import_decl) => {
                if module.resolve_internal_specifier(import_decl.source.value.as_str()).is_some() {
                    return StatementAction::Skip;
                }
                let specifiers: Vec<ImportSpecifier> = import_decl
                    .specifiers
                    .iter()
                    .flatten()
                    .map(|spec| {
                        use oxc_ast::ast::ImportDeclarationSpecifier;
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => ImportSpecifier {
                                local: s.local.name.to_string(),
                                kind: ImportSpecifierKind::Named(s.imported.name().to_string()),
                                preserve_if_unused: false,
                            },
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                ImportSpecifier {
                                    local: s.local.name.to_string(),
                                    kind: ImportSpecifierKind::Default,
                                    preserve_if_unused: false,
                                }
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                ImportSpecifier {
                                    local: s.local.name.to_string(),
                                    kind: ImportSpecifierKind::Namespace,
                                    preserve_if_unused: !is_generated_external_namespace_helper(
                                        s.local.name.as_str(),
                                    ),
                                }
                            }
                        }
                    })
                    .collect();
                if specifiers.is_empty()
                    && !module
                        .resolved_external_specifiers
                        .contains(import_decl.source.value.as_str())
                {
                    return StatementAction::Skip;
                }
                acc.imports.push(ExternalImport {
                    source: import_decl.source.value.to_string(),
                    side_effect_only: specifiers.is_empty(),
                    specifiers,
                    is_type_only: false,
                    from_reexport: false,
                });
                StatementAction::Skip
            }
            Statement::TSNamespaceExportDeclaration(_) | Statement::TSExportAssignment(_) => {
                StatementAction::Skip
            }
            Statement::TSImportEqualsDeclaration(decl) => {
                if let oxc_ast::ast::TSModuleReference::ExternalModuleReference(ext) =
                    &decl.module_reference
                    && module.resolve_internal_specifier(ext.expression.value.as_str()).is_some()
                {
                    return StatementAction::Skip;
                }
                StatementAction::Include
            }
            _ => {
                if let Some(needed) = needed_symbol_kinds
                    && let Some(decl) = stmt.as_declaration()
                    && !declaration_matches_needed_kinds(decl, &module.scoping, needed)
                {
                    return StatementAction::Skip;
                }
                StatementAction::Include
            }
        }
    }
}

fn build_shared_module_analysis(
    module_idx: ModuleIdx,
    scan_result: &ScanResult<'_>,
    link_output: &LinkStageOutput,
) -> SharedModuleAnalysis {
    #[cfg(test)]
    test_stats::SHARED_MODULE_PREPARES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let module = &scan_result.modules[module_idx];
    let mut analysis = SharedModuleAnalysis {
        import_renames: FxHashMap::default(),
        ns_aliases: FxHashSet::default(),
        external_ns_info: FxHashMap::default(),
        reexported_import_names: FxHashSet::default(),
    };

    for stmt in &module.program.body {
        if let Statement::ExportNamedDeclaration(decl) = stmt
            && decl.source.is_none()
            && decl.declaration.is_none()
        {
            for spec in &decl.specifiers {
                analysis.reexported_import_names.insert(spec.local.name().to_string());
            }
        }

        if let Statement::ImportDeclaration(import_decl) = stmt
            && let Some(specifiers) = &import_decl.specifiers
        {
            if let Some(source_idx) =
                module.resolve_internal_specifier(import_decl.source.value.as_str())
            {
                let source_module = &scan_result.modules[source_idx];
                for spec in specifiers {
                    match spec {
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                            if let Some(symbol_id) = ns.local.symbol_id.get() {
                                analysis.ns_aliases.insert(symbol_id);
                            }
                        }
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                            let imported_alias = s.imported.name().to_string();
                            let local_name =
                                resolve_export_local_name(source_module, &imported_alias)
                                    .unwrap_or(imported_alias);
                            let resolved_imported = link_output
                                .rename_plan
                                .resolve_name(source_module, &local_name)
                                .map_or(local_name, ToString::to_string);
                            if s.local.name.as_str() != resolved_imported
                                && let Some(symbol_id) = s.local.symbol_id.get()
                            {
                                analysis.import_renames.insert(symbol_id, resolved_imported);
                            }
                        }
                        oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(def) => {
                            if let Some(mut actual_name) =
                                link_output.default_export_names.get(&source_module.idx).cloned()
                            {
                                if let Some(renamed) = link_output
                                    .rename_plan
                                    .resolve_name(source_module, &actual_name)
                                {
                                    actual_name = renamed.to_string();
                                }
                                if def.local.name.as_str() != actual_name
                                    && let Some(symbol_id) = def.local.symbol_id.get()
                                {
                                    analysis.import_renames.insert(symbol_id, actual_name);
                                }
                            }
                        }
                    }
                }
            } else {
                for spec in specifiers {
                    if let oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) =
                        spec
                        && let Some(symbol_id) = ns.local.symbol_id.get()
                    {
                        analysis.ns_aliases.insert(symbol_id);
                        analysis.external_ns_info.insert(
                            symbol_id,
                            (import_decl.source.value.to_string(), ns.local.name.to_string()),
                        );
                    }
                }
            }
        }
    }

    analysis
}

fn prepare_statement_output<'a>(
    module_idx: ModuleIdx,
    stmt_idx: usize,
    stmt: &Statement<'a>,
    analysis: &SharedModuleAnalysis,
    scan_result: &ScanResult<'a>,
    allocator: &'a Allocator,
    sourcemap: bool,
    cwd: &Path,
    link_output: &LinkStageOutput,
    ns_name_map: &mut FxHashMap<String, String>,
) -> Option<PreparedStatementOutput> {
    let module = &scan_result.modules[module_idx];
    let action = prepared_statement_action(stmt, module)?;
    #[cfg(test)]
    test_stats::SHARED_STATEMENT_PREPARES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let ast = AstBuilder::new(allocator);
    let mut transformed_body = ast.vec();

    match action {
        StatementAction::Skip => return None,
        StatementAction::Include => {
            let stmt = module.program.body[stmt_idx].clone_in_with_semantic_ids(allocator);
            transformed_body.push(stmt);
        }
        StatementAction::UnwrapExportDeclaration => {
            let Statement::ExportNamedDeclaration(export) = &module.program.body[stmt_idx] else {
                unreachable!()
            };
            let mut decl =
                export.declaration.as_ref().unwrap().clone_in_with_semantic_ids(allocator);
            ensure_declare_on_declaration(&mut decl);
            decl.span_mut().start = export.span.start;
            transformed_body.push(Statement::from(decl));
        }
        StatementAction::UnwrapExportDefault => {
            let Statement::ExportDefaultDeclaration(export_default) =
                &module.program.body[stmt_idx]
            else {
                unreachable!()
            };
            let declaration = export_default.declaration.clone_in_with_semantic_ids(allocator);
            match declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(mut func_decl) => {
                    func_decl.span.start = export_default.span.start;
                    if func_decl.id.is_none() {
                        func_decl.id = Some(ast.binding_identifier(SPAN, "export_default"));
                    }
                    func_decl.declare = true;
                    transformed_body.push(Statement::FunctionDeclaration(func_decl));
                }
                ExportDefaultDeclarationKind::ClassDeclaration(mut class_decl) => {
                    class_decl.span.start = export_default.span.start;
                    if class_decl.id.is_none() {
                        class_decl.id = Some(ast.binding_identifier(SPAN, "export_default"));
                    }
                    class_decl.declare = true;
                    transformed_body.push(Statement::ClassDeclaration(class_decl));
                }
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(mut iface_decl) => {
                    iface_decl.span.start = export_default.span.start;
                    transformed_body.push(Statement::TSInterfaceDeclaration(iface_decl));
                }
                _ => return None,
            }
        }
    }

    apply_semantic_renames(
        module,
        allocator,
        &link_output.rename_plan,
        &analysis.import_renames,
        &mut transformed_body,
    );

    let mut imports = Vec::new();
    let mut ns_wrapper_output = String::new();
    let mut warnings = Vec::new();
    let external_ns_members = {
        let mut rewriter = InlineImportAndNamespaceRewriter {
            ast,
            module,
            imports: &mut imports,
            ns_name_map,
            scan_result,
            ns_wrapper_output: &mut ns_wrapper_output,
            namespace_aliases: analysis.ns_aliases.clone(),
            external_ns_info: &analysis.external_ns_info,
            external_ns_members: FxHashMap::default(),
            helper_reserved_names: &link_output.reserved_decl_names,
            warnings: &mut warnings,
        };
        for stmt in &mut transformed_body {
            rewriter.visit_statement(stmt);
        }
        rewriter.external_ns_members
    };

    let referenced_names = {
        let mut collector = ReferencedNameCollector::new();
        for stmt in &transformed_body {
            collector.visit_statement(stmt);
        }
        collector.finish()
    };

    let source_text = module.source;
    let reference_directive_set =
        module.reference_directives.iter().cloned().collect::<FxHashSet<_>>();
    let raw_comments = module.program.comments.clone_in_with_semantic_ids(allocator);
    let comments = ast.vec_from_iter(raw_comments.into_iter().filter(|comment| {
        let comment_text = comment.span.source_text(source_text).trim();
        if reference_directive_set.contains(comment_text) {
            return false;
        }
        !(comment_text.starts_with("//# sourceMappingURL=")
            || comment_text.starts_with("//@ sourceMappingURL="))
    }));
    let hashbang = module.program.hashbang.clone_in_with_semantic_ids(allocator);
    let directives = module.program.directives.clone_in_with_semantic_ids(allocator);
    let program = ast.program(
        module.program.span,
        module.program.source_type,
        source_text,
        comments,
        hashbang,
        directives,
        transformed_body,
    );

    let mut codegen_options = CodegenOptions {
        indent_char: IndentChar::Space,
        indent_width: 2,
        ..CodegenOptions::default()
    };
    if sourcemap {
        codegen_options.source_map_path = Some(PathBuf::from(&module.relative_path));
    }
    let codegen_return = Codegen::new().with_options(codegen_options).build(&program);
    let map = if sourcemap {
        match (codegen_return.map, module.input_sourcemap.clone()) {
            (Some(codegen_map), Some(input_map)) => {
                Some(sourcemap::compose_sourcemaps(&codegen_map, &input_map, &module.path, cwd))
            }
            (map, _) => map,
        }
    } else {
        None
    };

    Some(PreparedStatementOutput {
        code: codegen_return.code,
        map,
        imports,
        ns_wrapper_blocks: split_namespace_wrapper_blocks(&ns_wrapper_output),
        external_ns_members,
        referenced_names,
        warnings,
    })
}

fn prepared_statement_action(stmt: &Statement<'_>, module: &Module<'_>) -> Option<StatementAction> {
    match stmt {
        Statement::ExportNamedDeclaration(export_decl) => {
            if export_decl.declaration.is_some() {
                Some(StatementAction::UnwrapExportDeclaration)
            } else {
                None
            }
        }
        Statement::ExportDefaultDeclaration(export_default) => match &export_default.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(_)
            | ExportDefaultDeclarationKind::ClassDeclaration(_)
            | ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                Some(StatementAction::UnwrapExportDefault)
            }
            _ => None,
        },
        Statement::ImportDeclaration(import_decl) => {
            module.resolve_internal_specifier(import_decl.source.value.as_str())?;
            None
        }
        Statement::TSNamespaceExportDeclaration(_) | Statement::TSExportAssignment(_) => None,
        Statement::TSImportEqualsDeclaration(decl) => {
            if let oxc_ast::ast::TSModuleReference::ExternalModuleReference(ext) =
                &decl.module_reference
                && module.resolve_internal_specifier(ext.expression.value.as_str()).is_some()
            {
                return None;
            }
            Some(StatementAction::Include)
        }
        _ => Some(StatementAction::Include),
    }
}

fn split_namespace_wrapper_blocks(output: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in output.lines() {
        // `InlineImportAndNamespaceRewriter::ensure_internal_typeof_namespace` and
        // `ensure_external_namespace_import` emit wrapper blocks with `declare namespace`
        // at column 0. This splitter relies on that exact block boundary format.
        if line.starts_with("declare namespace ") && !current.is_empty() {
            blocks.push(std::mem::take(&mut current));
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

fn merge_external_ns_members(
    target: &mut FxHashMap<String, FxHashSet<String>>,
    incoming: &FxHashMap<String, FxHashSet<String>>,
) {
    for (specifier, members) in incoming {
        target.entry(specifier.clone()).or_default().extend(members.iter().cloned());
    }
}

fn convert_external_namespace_imports(
    imports: &mut Vec<ExternalImport>,
    imports_start: usize,
    external_ns_info: &FxHashMap<SymbolId, (String, String)>,
    external_ns_members: &FxHashMap<String, FxHashSet<String>>,
) {
    for (specifier, members) in external_ns_members {
        let ns_local = external_ns_info
            .values()
            .find(|(spec, _)| spec == specifier)
            .map(|(_, local)| local.as_str());

        let ns_imp_idx = ns_local.and_then(|ns_local_name| {
            imports[imports_start..].iter().position(|i| {
                i.source == *specifier
                    && i.specifiers.iter().any(|s| {
                        matches!(s.kind, ImportSpecifierKind::Namespace) && s.local == ns_local_name
                    })
            })
        });

        if let Some(relative_idx) = ns_imp_idx {
            let idx = imports_start + relative_idx;
            let ns_local_name = ns_local.unwrap();
            imports[idx].specifiers.retain(|s| {
                !(matches!(s.kind, ImportSpecifierKind::Namespace) && s.local == ns_local_name)
            });

            let target_idx = imports[imports_start..]
                .iter()
                .position(|i| {
                    i.source == *specifier
                        && i.specifiers
                            .iter()
                            .any(|s| matches!(s.kind, ImportSpecifierKind::Named(_)))
                })
                .map_or(idx, |relative| imports_start + relative);

            for member in members {
                if !imports[target_idx]
                    .specifiers
                    .iter()
                    .any(|s| matches!(&s.kind, ImportSpecifierKind::Named(n) if n == member))
                {
                    imports[target_idx].specifiers.push(ImportSpecifier {
                        local: member.clone(),
                        kind: ImportSpecifierKind::Named(member.clone()),
                        preserve_if_unused: false,
                    });
                }
            }

            if imports[idx].specifiers.is_empty() {
                imports.remove(idx);
            }
        }
    }
}

/// Prune unused imports after entry assembly selects the module fragments to emit.
fn prune_unused_imports_by_name(
    imports: &mut Vec<ExternalImport>,
    imports_start: usize,
    referenced_names: &FxHashSet<String>,
    module_exports: &[ExportedName],
    reexported_import_names: &FxHashSet<String>,
) {
    let module_exported_locals: FxHashSet<String> =
        module_exports.iter().map(|export| export.local.clone()).collect();

    for import in &mut imports[imports_start..] {
        if import.from_reexport {
            continue;
        }
        import.specifiers.retain(|specifier| match specifier.kind {
            ImportSpecifierKind::Namespace => {
                specifier.preserve_if_unused
                    || referenced_names.contains(specifier.local.as_str())
                    || module_exported_locals.contains(specifier.local.as_str())
                    || reexported_import_names.contains(specifier.local.as_str())
            }
            ImportSpecifierKind::Default | ImportSpecifierKind::Named(_) => {
                referenced_names.contains(specifier.local.as_str())
                    || module_exported_locals.contains(specifier.local.as_str())
                    || reexported_import_names.contains(specifier.local.as_str())
            }
        });
    }
    let mut import_idx = imports_start;
    while import_idx < imports.len() {
        if imports[import_idx].side_effect_only || !imports[import_idx].specifiers.is_empty() {
            import_idx += 1;
        } else {
            imports.remove(import_idx);
        }
    }
}

/// Collect symbol kinds for the names declared by a `Declaration` node.
fn collect_decl_symbol_kinds(
    decl: &oxc_ast::ast::Declaration<'_>,
    scoping: &Scoping,
) -> FxHashMap<SymbolId, NeededKindFlags> {
    let mut names = Vec::new();
    collect_decl_names(decl, &mut names);
    let declared_kinds = declaration_needed_kinds(decl);
    if declared_kinds.is_empty() {
        return FxHashMap::default();
    }
    names
        .iter()
        .filter_map(|name| {
            let symbol_id = scoping.get_root_binding(Ident::from(name.as_str()))?;
            let kinds = declared_kinds
                .intersection(NeededKindFlags::from_symbol_flags(scoping.symbol_flags(symbol_id)));
            (!kinds.is_empty()).then_some((symbol_id, kinds))
        })
        .collect()
}

fn declaration_needed_kinds(decl: &oxc_ast::ast::Declaration<'_>) -> NeededKindFlags {
    use oxc_ast::ast::Declaration;

    match decl {
        Declaration::VariableDeclaration(_) | Declaration::FunctionDeclaration(_) => {
            NeededKindFlags::VALUE
        }
        Declaration::ClassDeclaration(_)
        | Declaration::TSEnumDeclaration(_)
        | Declaration::TSModuleDeclaration(_)
        | Declaration::TSImportEqualsDeclaration(_) => NeededKindFlags::ALL,
        Declaration::TSTypeAliasDeclaration(_) | Declaration::TSInterfaceDeclaration(_) => {
            NeededKindFlags::TYPE
        }
        Declaration::TSGlobalDeclaration(_) => NeededKindFlags::NONE,
    }
}

fn declaration_matches_needed_kinds(
    decl: &oxc_ast::ast::Declaration<'_>,
    scoping: &Scoping,
    needed: &FxHashMap<SymbolId, NeededKindFlags>,
) -> bool {
    let decl_symbols = collect_decl_symbol_kinds(decl, scoping);
    decl_symbols.is_empty()
        || decl_symbols.iter().any(|(symbol_id, decl_kinds)| {
            needed.get(symbol_id).is_some_and(|needed_kinds| needed_kinds.intersects(*decl_kinds))
        })
}

fn export_default_declaration_matches_needed_kinds(
    export_default: &ExportDefaultDeclaration<'_>,
    scoping: &Scoping,
    needed: &FxHashMap<SymbolId, NeededKindFlags>,
) -> bool {
    let (name, decl_kinds) = match &export_default.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
            let Some(id) = &func.id else {
                return true;
            };
            (id.name.as_str(), NeededKindFlags::VALUE)
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            let Some(id) = &class.id else {
                return true;
            };
            (id.name.as_str(), NeededKindFlags::ALL)
        }
        ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
            (iface.id.name.as_str(), NeededKindFlags::TYPE)
        }
        _ => return true,
    };

    let Some(symbol_id) = scoping.get_root_binding(Ident::from(name)) else {
        return true;
    };
    let decl_kinds = decl_kinds
        .intersection(NeededKindFlags::from_symbol_flags(scoping.symbol_flags(symbol_id)));
    decl_kinds.is_empty() || needed.get(&symbol_id).is_some_and(|k| k.intersects(decl_kinds))
}

/// Apply semantic renames to the transformed AST body.
///
/// Merges two rename sources into a single symbol map:
/// 1. Symbol renames from the link stage (conflict resolution, e.g. `Foo` → `Foo$1`).
/// 2. Import renames (mapping local import bindings to their resolved names in
///    source modules).
fn apply_semantic_renames<'a>(
    module: &Module<'a>,
    allocator: &'a Allocator,
    rename_plan: &RenamePlan,
    import_renames: &FxHashMap<SymbolId, String>,
    body: &mut oxc_allocator::Vec<'a, Statement<'a>>,
) {
    let mut renamed_symbols =
        rename_plan.module_symbol_renames(module.idx).cloned().unwrap_or_default();

    renamed_symbols.extend(import_renames.iter().map(|(k, v)| (*k, v.clone())));

    if !renamed_symbols.is_empty() {
        let mut renamer = SemanticRenamer::new(allocator, &module.scoping, &renamed_symbols);
        renamer.visit_statements(body);
    }
}

struct ReferencedNameCollector {
    referenced_names: FxHashSet<String>,
}

impl ReferencedNameCollector {
    fn new() -> Self {
        Self { referenced_names: FxHashSet::default() }
    }

    fn finish(self) -> FxHashSet<String> {
        self.referenced_names
    }

    fn record_name(&mut self, name: &str) {
        self.referenced_names.insert(name.to_string());
    }

    fn record_identifier_reference(&mut self, ident: &IdentifierReference<'_>) {
        self.record_name(ident.name.as_str());
    }

    fn record_value_type_name(&mut self, type_name: &TSTypeName<'_>) {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                self.record_identifier_reference(ident);
            }
            TSTypeName::QualifiedName(name) => {
                self.record_value_type_name(&name.left);
            }
            TSTypeName::ThisExpression(expr) => {
                self.visit_this_expression(expr);
            }
        }
    }
}

impl<'a> Visit<'a> for ReferencedNameCollector {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.record_identifier_reference(ident);
    }

    fn visit_ts_type_name(&mut self, type_name: &TSTypeName<'a>) {
        match type_name {
            TSTypeName::IdentifierReference(ident) => {
                self.record_identifier_reference(ident);
            }
            TSTypeName::QualifiedName(name) => {
                self.visit_ts_type_name(&name.left);
            }
            TSTypeName::ThisExpression(expr) => {
                self.visit_this_expression(expr);
            }
        }
    }

    fn visit_ts_type_query(&mut self, ty: &TSTypeQuery<'a>) {
        if let Some(type_name) = ty.expr_name.as_ts_type_name() {
            self.record_value_type_name(type_name);
            if let Some(type_arguments) = &ty.type_arguments {
                self.visit_ts_type_parameter_instantiation(type_arguments);
            }
        } else {
            oxc_ast_visit::walk::walk_ts_type_query(self, ty);
        }
    }
}

fn is_generated_external_namespace_helper(name: &str) -> bool {
    name.starts_with('_') && name.chars().any(|ch| ch.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{TypackBundler, TypackOptions};

    use super::test_stats;

    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new(name: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("current time should be after unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "typack_generate_stage_{name}_{}_{}",
                std::process::id(),
                nanos
            ));
            fs::create_dir_all(&root).expect("temp project directory should be created");
            Self { root }
        }

        fn write_file(&self, relative_path: &str, content: &str) {
            let path = self.root.join(relative_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("parent directory should be created");
            }
            fs::write(path, content).expect("fixture file should be written");
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn shared_prepare_runs_once_per_scanned_module_for_multi_entry_dep_is_entry() {
        let project = TempProject::new("multi_entry_dep_is_entry");
        project.write_file(
            "index.d.ts",
            "import { add } from './utils';\nexport declare function greet(name: string): string;\nexport { add };\n",
        );
        project.write_file(
            "utils.d.ts",
            "export declare function add(a: number, b: number): number;\nexport declare function subtract(a: number, b: number): number;\n",
        );

        let (module_prepares_before, statement_prepares_before) = test_stats::counts();
        let result = TypackBundler::bundle(&TypackOptions {
            input: vec![
                project.root.join("index.d.ts").to_string_lossy().to_string(),
                project.root.join("utils.d.ts").to_string_lossy().to_string(),
            ],
            cwd: project.root.clone(),
            ..Default::default()
        });

        assert!(result.is_ok(), "bundle should succeed for multi-entry dep-is-entry fixture");
        let (module_prepares_after, statement_prepares_after) = test_stats::counts();
        let module_prepares = module_prepares_after - module_prepares_before;
        let statement_prepares = statement_prepares_after - statement_prepares_before;
        assert_eq!(module_prepares, 2, "shared preparation should visit each scanned module once");
        assert_eq!(
            statement_prepares, 3,
            "shared statement preparation should prepare each transformable top-level statement once"
        );
    }
}
