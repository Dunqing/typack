//! Generate stage: transforms per-module ASTs and emits the bundled `.d.ts`
//! output.
//!
//! Coordinates the final pipeline stage: applies tree-shaking, semantic renames,
//! inline import rewriting, and namespace wrapping, then assembles per-module
//! codegen output into a single declaration file.

mod analysis;
mod emit;
pub mod namespace;
mod output_assembler;
mod rewriter;
mod sourcemap;
mod types;

use std::collections::VecDeque;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::AstBuilder;
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, IdentifierReference, Statement, TSTypeName, TSTypeQuery,
};
use oxc_ast_visit::{Visit, VisitMut};
use oxc_codegen::{Codegen, CodegenOptions, IndentChar};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpanMut, SPAN};
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::link_stage::{
    LinkStageOutput, NeededNamesCtx, PerEntryLinkData, RenamePlan, StatementAction,
    build_per_entry_link_data,
};
use crate::scan_stage::ScanResult;
use crate::types::{Module, ModuleIdx};
use output_assembler::OutputAssembler;
use rewriter::{InlineImportAndNamespaceRewriter, SemanticRenamer, ensure_declare_on_declaration};
use types::*;

/// Generate stage: produces the bundled `.d.ts` output.
pub struct GenerateStage<'a, 'b> {
    scan_result: &'b ScanResult<'a>,
    allocator: &'a Allocator,
    sourcemap: bool,
    cjs_default: bool,
    cwd: &'b Path,
    link_output: &'b LinkStageOutput,
}

/// Output from generate stage.
pub struct GenerateOutput {
    pub code: String,
    pub map: Option<oxc_sourcemap::SourceMap>,
    pub warnings: Vec<OxcDiagnostic>,
}

impl<'a, 'b> GenerateStage<'a, 'b> {
    pub fn new(
        scan_result: &'b ScanResult<'a>,
        allocator: &'a Allocator,
        sourcemap: bool,
        cjs_default: bool,
        cwd: &'b Path,
        link_output: &'b LinkStageOutput,
    ) -> Self {
        Self { scan_result, allocator, sourcemap, cjs_default, cwd, link_output }
    }

    /// Generate the bundled `.d.ts` output for all entries.
    pub fn generate_all(&self) -> Vec<GenerateOutput> {
        // Collect and deduplicate reference directives from all modules (shared across entries)
        let mut seen_set: FxHashSet<&str> = FxHashSet::default();
        let mut unique_directives: Vec<&str> = Vec::new();
        for module in &self.scan_result.modules {
            for directive in &module.reference_directives {
                if seen_set.insert(directive.as_str()) {
                    unique_directives.push(directive.as_str());
                }
            }
        }

        // Precompute declaration graphs and root names once for all entries
        let needed_names_ctx = NeededNamesCtx::new(self.scan_result);

        self.scan_result
            .entry_indices
            .iter()
            .map(|&entry_idx| self.generate_entry(entry_idx, &unique_directives, &needed_names_ctx))
            .collect()
    }

    /// Generate the bundled `.d.ts` output for a single entry.
    fn generate_entry(
        &self,
        entry_idx: ModuleIdx,
        unique_directives: &[&str],
        needed_names_ctx: &NeededNamesCtx,
    ) -> GenerateOutput {
        let mut output = OutputAssembler::default();
        let mut acc = GenerateAcc::default();
        let per_entry = build_per_entry_link_data(
            self.scan_result,
            entry_idx,
            needed_names_ctx,
            self.link_output,
        );

        for directive in unique_directives {
            output.push_unmapped(format!("{directive}\n"));
        }

        acc.warnings.extend(per_entry.namespace_warnings.iter().cloned());

        let mut module_outputs: VecDeque<ModuleOutput> = VecDeque::new();
        for module_idx_usize in (0..self.scan_result.modules.len()).rev() {
            let module_idx = ModuleIdx::from_usize(module_idx_usize);
            if let Some(module_output) = self.generate_module_ast(module_idx, &per_entry, &mut acc)
            {
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

        let has_module_output =
            !acc.ns_wrapper_blocks.is_empty() || module_outputs.iter().any(|m| !m.code.is_empty());

        // Blank line between imports/star-exports and region markers
        if (had_imports || !acc.star_exports.is_empty()) && has_module_output {
            output.push_unmapped("\n");
        }

        if !acc.ns_wrapper_blocks.is_empty() {
            output.push_unmapped(std::mem::take(&mut acc.ns_wrapper_blocks));
        }

        // Emit namespace-wrapped modules first, then regular modules.
        for module in module_outputs.iter().filter(|m| m.is_ns_wrapped) {
            if let Some(wrapper) = &module.namespace_wrapper {
                output.push_unmapped(wrapper.clone());
            }
            if !module.code.is_empty() {
                output.push_mapped(&module.code, module.map.clone());
            }
        }
        for module in module_outputs.iter().filter(|m| !m.is_ns_wrapped) {
            if module.code.is_empty() {
                continue;
            }
            output.push_unmapped(format!("//#region {}\n", module.relative_path));
            output.push_mapped(&module.code, module.map.clone());
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
        per_entry: &PerEntryLinkData,
        acc: &mut GenerateAcc,
    ) -> Option<ModuleOutput> {
        let ns_wrap = per_entry.namespace_wraps.get(&module_idx);
        let module_has_augmentation = self.scan_result.modules[module_idx].has_augmentation;

        // Check if the module is needed: either it has pre-computed link meta,
        // or it has augmentation declarations.
        let meta = per_entry.module_metas.get(&module_idx);
        if meta.is_none() && !module_has_augmentation {
            return None;
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
                &self.link_output.rename_plan,
                &self.link_output.default_export_names,
            );
            &fallback_meta
        };

        // Phase 1: Output collection — walk statements using pre-computed
        // actions and collect exports, imports, star exports into acc.
        let exports_start = acc.exports.len();
        let imports_start = acc.imports.len();
        analysis::collect_module_outputs(
            self.scan_result,
            module_idx,
            meta,
            per_entry,
            self.link_output,
            acc,
        );

        // Phase 2: Selective clone + transform — only clone statements that
        // survive tree-shaking, and for unwrapped exports clone only the inner
        // declaration (not the export wrapper).
        let ast = AstBuilder::new(self.allocator);
        let mut transformed_body = ast.vec();
        let module = &self.scan_result.modules[module_idx];

        for (i, action) in meta.statement_actions.iter().enumerate() {
            match action {
                StatementAction::Skip => {}
                StatementAction::Include => {
                    let stmt = module.program.body[i].clone_in_with_semantic_ids(self.allocator);
                    transformed_body.push(stmt);
                }
                StatementAction::UnwrapExportDeclaration => {
                    let Statement::ExportNamedDeclaration(export) = &module.program.body[i] else {
                        unreachable!()
                    };
                    let mut decl = export
                        .declaration
                        .as_ref()
                        .unwrap()
                        .clone_in_with_semantic_ids(self.allocator);
                    ensure_declare_on_declaration(&mut decl);
                    decl.span_mut().start = export.span.start;
                    transformed_body.push(Statement::from(decl));
                }
                StatementAction::UnwrapExportDefault => {
                    let Statement::ExportDefaultDeclaration(export_default) =
                        &module.program.body[i]
                    else {
                        unreachable!()
                    };
                    let declaration =
                        export_default.declaration.clone_in_with_semantic_ids(self.allocator);
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
                                class_decl.id =
                                    Some(ast.binding_identifier(SPAN, "export_default"));
                            }
                            class_decl.declare = true;
                            transformed_body.push(Statement::ClassDeclaration(class_decl));
                        }
                        ExportDefaultDeclarationKind::TSInterfaceDeclaration(mut iface_decl) => {
                            iface_decl.span.start = export_default.span.start;
                            transformed_body.push(Statement::TSInterfaceDeclaration(iface_decl));
                        }
                        _ => {}
                    }
                }
            }
        }

        if transformed_body.is_empty() && ns_wrap.is_none() {
            return None;
        }

        // Apply semantic renames (symbol renames from link stage + import renames).
        apply_semantic_renames(
            module,
            self.allocator,
            &self.link_output.rename_plan,
            &meta.import_renames,
            &mut transformed_body,
        );

        // AST-level import-type rewriting and namespace alias flattening.
        let external_ns_members = {
            let mut rewriter = InlineImportAndNamespaceRewriter {
                ast,
                module,
                imports: &mut acc.imports,
                ns_name_map: &mut acc.ns_name_map,
                scan_result: self.scan_result,
                ns_wrapper_output: &mut acc.ns_wrapper_blocks,
                namespace_aliases: meta.ns_aliases.clone(),
                external_ns_info: &meta.external_ns_info,
                external_ns_members: FxHashMap::default(),
                helper_reserved_names: &per_entry.helper_reserved_names,
                warnings: &mut acc.warnings,
            };
            for stmt in &mut transformed_body {
                rewriter.visit_statement(stmt);
            }
            rewriter.external_ns_members
        };

        // Convert external namespace imports to named imports based on
        // member accesses recorded during the rewrite pass.
        for (specifier, members) in &external_ns_members {
            let ns_local = meta
                .external_ns_info
                .values()
                .find(|(spec, _)| spec == specifier)
                .map(|(_, local)| local.as_str());

            // Find the import entry that contains the namespace specifier
            let ns_imp_idx = ns_local.and_then(|ns_local_name| {
                acc.imports.iter().position(|i| {
                    i.source == *specifier
                        && i.specifiers.iter().any(|s| {
                            matches!(s.kind, ImportSpecifierKind::Namespace)
                                && s.local == ns_local_name
                        })
                })
            });

            if let Some(idx) = ns_imp_idx {
                // Remove the namespace specifier
                let ns_local_name = ns_local.unwrap();
                acc.imports[idx].specifiers.retain(|s| {
                    !(matches!(s.kind, ImportSpecifierKind::Namespace) && s.local == ns_local_name)
                });

                // Add named imports — either to an existing import with the
                // same source, or to the one we just modified
                let target_idx = acc
                    .imports
                    .iter()
                    .position(|i| {
                        i.source == *specifier
                            && i.specifiers
                                .iter()
                                .any(|s| matches!(s.kind, ImportSpecifierKind::Named(_)))
                    })
                    .unwrap_or(idx);

                for member in members {
                    if !acc.imports[target_idx]
                        .specifiers
                        .iter()
                        .any(|s| matches!(&s.kind, ImportSpecifierKind::Named(n) if n == member))
                    {
                        acc.imports[target_idx].specifiers.push(ImportSpecifier {
                            local: member.clone(),
                            kind: ImportSpecifierKind::Named(member.clone()),
                        });
                    }
                }

                // Remove the namespace import entry if it has no specifiers left
                if acc.imports[idx].specifiers.is_empty() {
                    acc.imports.remove(idx);
                }
            }
        }

        // Phase 3: Prune unused imports.
        prune_unused_imports(
            &mut acc.imports,
            imports_start,
            &transformed_body,
            &acc.exports[exports_start..],
            &meta.reexported_import_names,
            &meta.external_ns_info,
        );

        let (relative_path, program_span, source_type, source_text, reference_directive_set) = {
            let module = &self.scan_result.modules[module_idx];
            (
                module.relative_path.clone(),
                module.program.span,
                module.program.source_type,
                module.source,
                module.reference_directives.iter().cloned().collect::<FxHashSet<String>>(),
            )
        };

        let (comments, hashbang, directives) = {
            let module = &self.scan_result.modules[module_idx];
            let raw_comments = module.program.comments.clone_in_with_semantic_ids(self.allocator);
            let comments = ast.vec_from_iter(raw_comments.into_iter().filter(|comment| {
                let comment_text = comment.span.source_text(source_text).trim();
                if reference_directive_set.contains(comment_text) {
                    return false;
                }
                !(comment_text.starts_with("//# sourceMappingURL=")
                    || comment_text.starts_with("//@ sourceMappingURL="))
            }));
            let hashbang = module.program.hashbang.clone_in_with_semantic_ids(self.allocator);
            let directives = module.program.directives.clone_in_with_semantic_ids(self.allocator);
            (comments, hashbang, directives)
        };

        let program = ast.program(
            program_span,
            source_type,
            source_text,
            comments,
            hashbang,
            directives,
            transformed_body,
        );

        let input_sourcemap = self.scan_result.modules[module_idx].input_sourcemap.clone();

        let mut codegen_options = CodegenOptions {
            indent_char: IndentChar::Space,
            indent_width: 2,
            ..CodegenOptions::default()
        };
        if self.sourcemap {
            codegen_options.source_map_path = Some(PathBuf::from(&relative_path));
        }
        let codegen_return = Codegen::new().with_options(codegen_options).build(&program);
        let code = codegen_return.code;
        let map = if self.sourcemap {
            match (codegen_return.map, input_sourcemap) {
                (Some(codegen_map), Some(input_map)) => {
                    let module_path = &self.scan_result.modules[module_idx].path;
                    Some(sourcemap::compose_sourcemaps(
                        &codegen_map,
                        &input_map,
                        module_path,
                        self.cwd,
                    ))
                }
                (map, _) => map,
            }
        } else {
            None
        };

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
            relative_path,
            is_ns_wrapped: ns_wrap.is_some(),
            namespace_wrapper,
            code,
            map,
        })
    }
}

/// Prune unused imports after the transform and rewrite passes.
///
/// Drops external namespace helpers whose only usages were inside tree-shaken
/// declarations, and removes named/default import specifiers that are no longer
/// referenced in the transformed AST.
fn prune_unused_imports(
    imports: &mut Vec<ExternalImport>,
    imports_start: usize,
    transformed_body: &oxc_allocator::Vec<'_, Statement<'_>>,
    module_exports: &[ExportedName],
    reexported_import_names: &FxHashSet<String>,
    external_ns_info: &FxHashMap<SymbolId, (String, String)>,
) {
    let referenced_names = {
        let mut collector = ReferencedNameCollector::new();
        for stmt in transformed_body {
            collector.visit_statement(stmt);
        }
        collector.finish()
    };
    let module_exported_locals: FxHashSet<String> =
        module_exports.iter().map(|export| export.local.clone()).collect();

    // Drop TypeScript-generated external namespace helpers whose only
    // usages were inside tree-shaken declarations. Keep user-authored
    // namespace imports (for example `import * as http`) to preserve the
    // plugin's current snapshot semantics.
    if !external_ns_info.is_empty() {
        for (specifier, local_name) in external_ns_info.values() {
            if referenced_names.contains(local_name)
                || module_exported_locals.contains(local_name.as_str())
                || !is_generated_external_namespace_helper(local_name)
            {
                continue;
            }

            if let Some(idx) = imports.iter().position(|i| {
                i.source == *specifier
                    && i.specifiers.iter().any(|s| {
                        matches!(s.kind, ImportSpecifierKind::Namespace) && s.local == *local_name
                    })
            }) {
                imports[idx].specifiers.retain(|s| {
                    !(matches!(s.kind, ImportSpecifierKind::Namespace) && s.local == *local_name)
                });
                if imports[idx].specifiers.is_empty() {
                    imports.remove(idx);
                }
            }
        }
    }

    // Drop external named/default imports that no longer have any
    // remaining references after tree-shaking. Imports created from
    // `export { ... } from "ext"` re-exports are exempt.
    for import in &mut imports[imports_start..] {
        if import.from_reexport {
            continue;
        }
        import.specifiers.retain(|specifier| match specifier.kind {
            ImportSpecifierKind::Namespace => true,
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
