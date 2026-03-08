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
use crate::link_stage::{LinkStageOutput, NeededKindFlags, RenamePlan, build_per_entry_link_data};
use crate::scan_stage::ScanResult;
use crate::types::{Module, ModuleIdx};
use namespace::{
    apply_namespace_wrap_renames, collect_declaration_names, collect_export_specifier,
    collect_module_exports, deconflict_namespace_wrap_names, pre_scan_namespace_info,
};
use output_assembler::OutputAssembler;
use rewriter::{InlineImportAndNamespaceRewriter, SemanticRenamer, ensure_declare_on_declaration};
use types::*;

/// Generate stage: produces the bundled `.d.ts` output.
pub struct GenerateStage<'a, 'b> {
    scan_result: &'b ScanResult<'a>,
    entry_indices: &'b [ModuleIdx],
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
        entry_indices: &'b [ModuleIdx],
        allocator: &'a Allocator,
        sourcemap: bool,
        cjs_default: bool,
        cwd: &'b Path,
        link_output: &'b LinkStageOutput,
    ) -> Self {
        Self { scan_result, entry_indices, allocator, sourcemap, cjs_default, cwd, link_output }
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

        self.entry_indices
            .iter()
            .map(|&entry_idx| self.generate_entry(entry_idx, &unique_directives))
            .collect()
    }

    /// Generate the bundled `.d.ts` output for a single entry.
    fn generate_entry(&self, entry_idx: ModuleIdx, unique_directives: &[&str]) -> GenerateOutput {
        let mut output = OutputAssembler::default();
        let mut acc = GenerateAcc::default();
        let rename_plan = &self.link_output.rename_plan;
        let per_entry = build_per_entry_link_data(self.scan_result, entry_idx);

        for directive in unique_directives {
            output.push_unmapped(format!("{directive}\n"));
        }

        // Pre-scan all modules for namespace import patterns
        let (mut namespace_wraps, namespace_aliases) = pre_scan_namespace_info(
            self.scan_result,
            entry_idx,
            &self.link_output.all_module_aliases,
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
            needed_symbol_kinds: &per_entry.needed_names_plan.symbol_kinds,
            default_export_names: &self.link_output.default_export_names,
            helper_reserved_names: &helper_reserved_names,
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
        shared: &GenerateSharedCtx<'_>,
        acc: &mut GenerateAcc,
    ) -> Option<ModuleOutput> {
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

        // Phase 2: Selective clone + transform — only clone statements that
        // survive tree-shaking, and for unwrapped exports clone only the inner
        // declaration (not the export wrapper).
        let ast = AstBuilder::new(self.allocator);
        let mut transformed_body = ast.vec();
        let module = &self.scan_result.modules[module_idx];

        for (i, action) in analysis.statement_actions.iter().enumerate() {
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
            shared.rename_plan,
            &analysis.import_renames,
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
                namespace_aliases: analysis.ns_aliases,
                external_ns_info: &analysis.external_ns_info,
                external_ns_members: FxHashMap::default(),
                helper_reserved_names: shared.helper_reserved_names,
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
            let ns_local = analysis
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
            &analysis.reexported_import_names,
            &analysis.external_ns_info,
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
        let mut analysis = ModuleAnalysis {
            statement_actions: Vec::with_capacity(module.program.body.len()),
            import_renames: FxHashMap::default(),
            ns_aliases: FxHashSet::default(),
            external_ns_info: FxHashMap::default(),
            reexported_import_names: FxHashSet::default(),
        };

        // Pre-scan: collect import renames, ns aliases, external ns info,
        // and reexported import names.
        for stmt in &module.program.body {
            // Collect local names from `export { X }` (no source) for non-entry
            // modules. When X was imported from an external package, the import
            // must be preserved even though X isn't used in local declarations.
            if !module.is_entry
                && let Statement::ExportNamedDeclaration(decl) = stmt
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
                let is_internal =
                    module.resolve_internal_specifier(import_decl.source.value.as_str()).is_some();

                if let Some(source_idx) =
                    module.resolve_internal_specifier(import_decl.source.value.as_str())
                {
                    // Internal import processing
                    let source_module = &self.scan_result.modules[source_idx];
                    for spec in specifiers {
                        match spec {
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                                ns,
                            ) => {
                                if let Some(symbol_id) = ns.local.symbol_id.get() {
                                    analysis.ns_aliases.insert(symbol_id);
                                }
                            }
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                let imported_alias = s.imported.name().to_string();
                                let local_name =
                                    resolve_export_local_name(source_module, &imported_alias)
                                        .unwrap_or(imported_alias);
                                let resolved_imported = shared
                                    .rename_plan
                                    .resolve_name(source_module, &local_name)
                                    .map_or(local_name, ToString::to_string);
                                if s.local.name.as_str() != resolved_imported
                                    && let Some(symbol_id) = s.local.symbol_id.get()
                                {
                                    analysis.import_renames.insert(symbol_id, resolved_imported);
                                }
                            }
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(
                                def,
                            ) => {
                                if let Some(mut actual_name) =
                                    shared.default_export_names.get(&source_module.idx).cloned()
                                {
                                    if let Some(renamed) =
                                        shared.rename_plan.resolve_name(source_module, &actual_name)
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
                } else if !is_internal {
                    // External import — collect namespace specifiers
                    for spec in specifiers {
                        if let oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                            ns,
                        ) = spec
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
                    if module.is_entry {
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
                    if module.is_entry {
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
                    if module.is_entry {
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
                        if module.is_entry {
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
                        if module.is_entry {
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
                        if module.is_entry {
                            acc.exports.push(ExportedName {
                                local: name.to_string(),
                                exported: "default".to_string(),
                                is_type_only: false,
                            });
                        }
                        StatementAction::UnwrapExportDefault
                    }
                    ExportDefaultDeclarationKind::Identifier(id) => {
                        if module.is_entry {
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
                        if module.is_entry
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
                            }],
                            is_type_only: false,
                            side_effect_only: false,
                            from_reexport: true,
                        });
                        if module.is_entry {
                            acc.exports.push(ExportedName {
                                local: name.clone(),
                                exported: name,
                                is_type_only: export_all.export_kind.is_type(),
                            });
                        }
                    }
                } else if let Some(source_module_idx) = internal_source_idx {
                    if module.is_entry {
                        let before_len = acc.exports.len();
                        let mut visited = FxHashSet::default();
                        let mut star_external_imports = Vec::new();
                        collect_module_exports(
                            source_module_idx,
                            &mut acc.exports,
                            self.scan_result,
                            &mut visited,
                            Some(&mut star_external_imports),
                        );
                        acc.imports.extend(star_external_imports);
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
                            },
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                ImportSpecifier {
                                    local: s.local.name.to_string(),
                                    kind: ImportSpecifierKind::Default,
                                }
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                ImportSpecifier {
                                    local: s.local.name.to_string(),
                                    kind: ImportSpecifierKind::Namespace,
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
