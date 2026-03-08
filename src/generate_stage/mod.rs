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
    ExportDefaultDeclarationKind, IdentifierReference, Statement, TSTypeName, TSTypeQuery,
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
    apply_namespace_wrap_renames, collect_declaration_names, collect_module_exports,
    deconflict_namespace_wrap_names, pre_scan_namespace_info,
};
use output_assembler::OutputAssembler;
use rewriter::{InlineImportAndNamespaceRewriter, SemanticRenamer, ensure_declare_on_declaration};
use types::*;

/// Output from generate stage.
pub struct GenerateOutput {
    pub code: String,
    pub map: Option<oxc_sourcemap::SourceMap>,
    pub warnings: Vec<OxcDiagnostic>,
}

/// Collect symbol kinds for the names declared by a `Declaration` node.
fn collect_decl_symbol_kinds(
    decl: &oxc_ast::ast::Declaration<'_>,
    scoping: &Scoping,
) -> Vec<(SymbolId, NeededKindFlags)> {
    let declared_kinds = declaration_needed_kinds(decl);
    if declared_kinds.is_empty() {
        return Vec::new();
    }
    let mut names = Vec::new();
    collect_decl_names(decl, &mut names);
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
    let plan_renames = rename_plan.module_symbol_renames(module.idx);

    // Fast path: if no renames needed at all, skip
    if plan_renames.is_none() && import_renames.is_empty() {
        return;
    }

    // Fast path: if only plan renames (no import renames), avoid cloning
    if import_renames.is_empty() {
        if let Some(renames) = plan_renames {
            let mut renamer = SemanticRenamer::new(allocator, &module.scoping, renames);
            renamer.visit_statements(body);
        }
        return;
    }

    // Merge both rename sources
    let mut renamed_symbols = plan_renames.cloned().unwrap_or_default();
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

// ─────────────────────────────────────────────────────────────────────────────
// Two-phase generate: process modules once, assemble per-entry
// ─────────────────────────────────────────────────────────────────────────────

/// Phase 0: Pre-compute per-entry data before module bodies are taken.
///
/// Must run while all module bodies are still available (before Phase 1).
pub fn precompute_entry_data(
    scan_result: &ScanResult<'_>,
    entry_idx: ModuleIdx,
    link_output: &LinkStageOutput,
) -> PrecomputedEntryData {
    let rename_plan = &link_output.rename_plan;
    let per_entry = build_per_entry_link_data(scan_result, entry_idx);

    let (mut namespace_wraps, namespace_aliases) =
        pre_scan_namespace_info(scan_result, entry_idx, &link_output.all_module_aliases);
    apply_namespace_wrap_renames(&mut namespace_wraps, rename_plan, scan_result);

    let mut helper_reserved_names = link_output.reserved_decl_names.clone();
    let mut warnings = Vec::new();
    deconflict_namespace_wrap_names(&mut namespace_wraps, &helper_reserved_names, &mut warnings);
    for wrap in namespace_wraps.values() {
        helper_reserved_names.insert(wrap.namespace_name.clone());
    }

    // Pre-compute star re-export data for entry module's `export * from "./internal"`.
    let mut star_reexport_data = FxHashMap::default();
    let entry = &scan_result.modules[entry_idx];
    for stmt in &entry.program.body {
        if let Statement::ExportAllDeclaration(export_all) = stmt
            && export_all.exported.is_none()
            && let Some(source_idx) =
                entry.resolve_internal_specifier(export_all.source.value.as_str())
        {
            star_reexport_data.entry(source_idx).or_insert_with(|| {
                let mut exports = Vec::new();
                let mut imports = Vec::new();
                let mut visited = FxHashSet::default();
                collect_module_exports(
                    source_idx,
                    &mut exports,
                    scan_result,
                    &mut visited,
                    Some(&mut imports),
                );
                for exp in &mut exports {
                    if let Some(new_name) =
                        rename_plan.resolve_name(&scan_result.modules[source_idx], &exp.local)
                    {
                        exp.local = new_name.to_string();
                    }
                }
                PrecomputedStarExport { exports, imports }
            });
        }
    }

    PrecomputedEntryData {
        namespace_wraps,
        namespace_aliases,
        helper_reserved_names,
        needed_symbol_kinds: per_entry.needed_names_plan.symbol_kinds,
        warnings,
        star_reexport_data,
    }
}

/// Phase 1: Process ALL modules into fragments, consuming AST bodies.
///
/// After this call, `scan_result.modules[*].program.body` is empty.
/// Module metadata (scoping, paths, export_import_info, etc.) remains accessible.
pub fn process_all_module_fragments<'a>(
    scan_result: &mut ScanResult<'a>,
    allocator: &'a Allocator,
    link_output: &LinkStageOutput,
    helper_reserved_names: &FxHashSet<String>,
    needed_modules: &FxHashSet<ModuleIdx>,
    sourcemap: bool,
    cwd: &Path,
) -> Vec<Option<ModuleFragments>> {
    let num_modules = scan_result.modules.len();
    let mut all_fragments: Vec<Option<ModuleFragments>> = Vec::with_capacity(num_modules);
    for module_idx_usize in 0..num_modules {
        let module_idx = ModuleIdx::from_usize(module_idx_usize);
        if !needed_modules.contains(&module_idx) {
            all_fragments.push(None);
            continue;
        }
        let fragments = process_single_module_fragments(
            module_idx,
            scan_result,
            allocator,
            link_output,
            helper_reserved_names,
            sourcemap,
            cwd,
        );
        all_fragments.push(fragments);
    }
    all_fragments
}

/// Process a single module into fragments, consuming its AST body.
fn process_single_module_fragments<'a>(
    module_idx: ModuleIdx,
    scan_result: &mut ScanResult<'a>,
    allocator: &'a Allocator,
    link_output: &LinkStageOutput,
    helper_reserved_names: &FxHashSet<String>,
    sourcemap: bool,
    cwd: &Path,
) -> Option<ModuleFragments> {
    let rename_plan = &link_output.rename_plan;
    let default_export_names = &link_output.default_export_names;
    let ast = AstBuilder::new(allocator);

    // ── Step 1: Import analysis (read body before taking it) ──────────────
    let mut ns_aliases: FxHashSet<SymbolId> = FxHashSet::default();
    let mut import_renames: FxHashMap<SymbolId, String> = FxHashMap::default();
    let mut external_ns_info: FxHashMap<SymbolId, (String, String)> = FxHashMap::default();
    let mut reexported_import_names: FxHashSet<String> = FxHashSet::default();
    {
        let module = &scan_result.modules[module_idx];
        for stmt in &module.program.body {
            if !module.is_entry
                && let Statement::ExportNamedDeclaration(decl) = stmt
                && decl.source.is_none()
                && decl.declaration.is_none()
            {
                for spec in &decl.specifiers {
                    reexported_import_names.insert(spec.local.name().to_string());
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
                    let source_module = &scan_result.modules[source_idx];
                    for spec in specifiers {
                        match spec {
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                                ns,
                            ) => {
                                if let Some(symbol_id) = ns.local.symbol_id.get() {
                                    ns_aliases.insert(symbol_id);
                                }
                            }
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                let imported_alias = s.imported.name().to_string();
                                let local_name =
                                    resolve_export_local_name(source_module, &imported_alias)
                                        .unwrap_or(imported_alias);
                                let resolved_imported = rename_plan
                                    .resolve_name(source_module, &local_name)
                                    .map_or(local_name, ToString::to_string);
                                if s.local.name.as_str() != resolved_imported
                                    && let Some(symbol_id) = s.local.symbol_id.get()
                                {
                                    import_renames.insert(symbol_id, resolved_imported);
                                }
                            }
                            oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(
                                def,
                            ) => {
                                if let Some(mut actual_name) =
                                    default_export_names.get(&source_module.idx).cloned()
                                {
                                    if let Some(renamed) =
                                        rename_plan.resolve_name(source_module, &actual_name)
                                    {
                                        actual_name = renamed.to_string();
                                    }
                                    if def.local.name.as_str() != actual_name
                                        && let Some(symbol_id) = def.local.symbol_id.get()
                                    {
                                        import_renames.insert(symbol_id, actual_name);
                                    }
                                }
                            }
                        }
                    }
                } else if !is_internal {
                    for spec in specifiers {
                        if let oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                            ns,
                        ) = spec
                            && let Some(symbol_id) = ns.local.symbol_id.get()
                        {
                            ns_aliases.insert(symbol_id);
                            external_ns_info.insert(
                                symbol_id,
                                (import_decl.source.value.to_string(), ns.local.name.to_string()),
                            );
                        }
                    }
                }
            }
        }
    }

    // ── Step 2: Take body and program metadata ───────────────────────────
    let input_body = {
        let empty = oxc_allocator::Vec::new_in(allocator);
        std::mem::replace(&mut scan_result.modules[module_idx].program.body, empty)
    };
    if input_body.is_empty() {
        return None;
    }

    let (program_span, source_type, source_text) = {
        let module = &scan_result.modules[module_idx];
        (module.program.span, module.program.source_type, module.source)
    };

    // Take comments, hashbang, directives
    let (raw_comments, _hashbang, _directives) = {
        let program = &mut scan_result.modules[module_idx].program;
        let c = std::mem::replace(&mut program.comments, ast.vec());
        let h = program.hashbang.take();
        let d = std::mem::replace(&mut program.directives, ast.vec());
        (c, h, d)
    };

    let reference_directives = &scan_result.modules[module_idx].reference_directives;
    let comments = ast.vec_from_iter(raw_comments.into_iter().filter(|comment| {
        let comment_text = comment.span.source_text(source_text).trim();
        if reference_directives.iter().any(|d| d == comment_text) {
            return false;
        }
        !(comment_text.starts_with("//# sourceMappingURL=")
            || comment_text.starts_with("//@ sourceMappingURL="))
    }));

    // ── Step 3: Process statements (no tree-shaking) ─────────────────────
    // Use a single ordered list to preserve source statement order.
    // Each entry records its body_index (Some for code-producing, None for metadata-only).
    let mut transformed_body = ast.vec();
    let mut ordered_metas: Vec<(StatementMeta, Option<usize>)> = Vec::new();

    for stmt in input_body {
        let module = &scan_result.modules[module_idx];
        let mut meta = StatementMeta::default();
        let body_len_before = transformed_body.len();
        process_statement_for_fragment(
            stmt,
            ast,
            &mut transformed_body,
            module,
            scan_result,
            rename_plan,
            default_export_names,
            allocator,
            &mut meta,
        );
        if transformed_body.len() > body_len_before {
            ordered_metas.push((meta, Some(body_len_before)));
        } else if meta.has_metadata() {
            ordered_metas.push((meta, None));
        }
    }

    if transformed_body.is_empty() && ordered_metas.is_empty() {
        return None;
    }

    // ── Step 4: Apply semantic renames ────────────────────────────────────
    {
        let module = &scan_result.modules[module_idx];
        apply_semantic_renames(
            module,
            allocator,
            rename_plan,
            &import_renames,
            &mut transformed_body,
        );

        // Update export metadata to reflect renames applied in Step 4.
        for (meta, body_index) in &mut ordered_metas {
            // Patch export names from own declarations (body metas only)
            if body_index.is_some() {
                for exp in &mut meta.export_names {
                    if let Some(new_name) = rename_plan.resolve_name(module, &exp.local) {
                        exp.local = new_name.to_string();
                    }
                    if let Some(symbol_id) =
                        module.scoping.get_root_binding(Ident::from(exp.local.as_str()))
                        && let Some(renamed) = import_renames.get(&symbol_id)
                    {
                        exp.local.clone_from(renamed);
                    }
                }
            }
            // Patch bare_reexport_specs in all metas
            for spec in &mut meta.bare_reexport_specs {
                if let Some(new_name) = rename_plan.resolve_name(module, &spec.local) {
                    spec.local = new_name.to_string();
                }
                if let Some(symbol_id) = spec.symbol_id
                    && let Some(renamed) = import_renames.get(&symbol_id)
                {
                    spec.local.clone_from(renamed);
                }
            }
        }
    }

    // ── Step 5: Run rewriter ─────────────────────────────────────────────
    let mut rewriter_imports: Vec<ExternalImport> = Vec::new();
    let mut ns_name_map: FxHashMap<String, String> = FxHashMap::default();
    let mut ns_wrapper_blocks = String::new();
    let mut warnings: Vec<OxcDiagnostic> = Vec::new();
    let external_ns_members = {
        let module = &scan_result.modules[module_idx];
        let mut rewriter = InlineImportAndNamespaceRewriter {
            ast,
            module,
            imports: &mut rewriter_imports,
            ns_name_map: &mut ns_name_map,
            scan_result,
            ns_wrapper_output: &mut ns_wrapper_blocks,
            namespace_aliases: ns_aliases,
            external_ns_info: &external_ns_info,
            external_ns_members: FxHashMap::default(),
            helper_reserved_names,
            warnings: &mut warnings,
        };
        for stmt in &mut transformed_body {
            rewriter.visit_statement(stmt);
        }
        rewriter.external_ns_members
    };

    // ── Step 6: External ns→named import conversion (in-place) ──────────
    // Convert external namespace imports to named imports in-place on fragment
    // metas, preserving source order. When a namespace is accessed via member
    // expressions (external_ns_members), replace the namespace specifier with
    // individual named specifiers.
    for (specifier, members) in &external_ns_members {
        let ns_local = external_ns_info
            .values()
            .find(|(spec, _)| spec == specifier)
            .map(|(_, local)| local.as_str());

        let Some(ns_local_name) = ns_local else {
            continue;
        };

        // Find the meta containing the namespace import for this specifier
        let meta_and_imp_idx = ordered_metas.iter_mut().find_map(|(meta, _)| {
            meta.imports
                .iter()
                .position(|i| {
                    i.source == *specifier
                        && i.specifiers.iter().any(|s| {
                            matches!(s.kind, ImportSpecifierKind::Namespace)
                                && s.local == ns_local_name
                        })
                })
                .map(|idx| (meta, idx))
        });

        if let Some((meta, idx)) = meta_and_imp_idx {
            // Remove namespace specifier
            meta.imports[idx].specifiers.retain(|s| {
                !(matches!(s.kind, ImportSpecifierKind::Namespace) && s.local == ns_local_name)
            });

            // Find or use the same import for named specifiers
            let target_idx = meta
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
                if !meta.imports[target_idx]
                    .specifiers
                    .iter()
                    .any(|s| matches!(&s.kind, ImportSpecifierKind::Named(n) if n == member))
                {
                    meta.imports[target_idx].specifiers.push(ImportSpecifier {
                        local: member.clone(),
                        kind: ImportSpecifierKind::Named(member.clone()),
                    });
                }
            }

            if meta.imports[idx].specifiers.is_empty() {
                meta.imports.remove(idx);
            }
        }
    }

    // ── Step 7: Per-statement codegen → fragments ────────────────────────
    let relative_path = scan_result.modules[module_idx].relative_path.clone();
    let input_sourcemap = &scan_result.modules[module_idx].input_sourcemap;

    let mut codegen_options = CodegenOptions {
        indent_char: IndentChar::Space,
        indent_width: 2,
        ..CodegenOptions::default()
    };
    if sourcemap {
        codegen_options.source_map_path = Some(PathBuf::from(&relative_path));
    }

    // ── Step 7a: Collect referenced names per-statement BEFORE codegen ───
    let mut per_stmt_referenced_names: Vec<FxHashSet<String>> =
        Vec::with_capacity(transformed_body.len());
    for stmt in &transformed_body {
        let mut collector = ReferencedNameCollector::new();
        collector.visit_statement(stmt);
        per_stmt_referenced_names.push(collector.finish());
    }

    // Convert transformed_body to Option<Statement> for indexed take
    let mut body_stmts: Vec<Option<Statement<'a>>> =
        transformed_body.into_iter().map(Some).collect();

    let mut fragments: Vec<DeclarationFragment> =
        Vec::with_capacity(body_stmts.len() + ordered_metas.len());

    // Build fragments in source order (body and metaonly interleaved)
    for (meta, body_index) in ordered_metas {
        if let Some(idx) = body_index {
            let stmt = body_stmts[idx].take().unwrap();
            let referenced_names = std::mem::take(&mut per_stmt_referenced_names[idx]);

            // Build mini-program for codegen
            let mini_body = ast.vec_from_iter(std::iter::once(stmt));
            let mini_comments = comments.clone_in(allocator);
            let mini_program = ast.program(
                program_span,
                source_type,
                source_text,
                mini_comments,
                None,
                ast.vec(),
                mini_body,
            );

            let codegen_return =
                Codegen::new().with_options(codegen_options.clone()).build(&mini_program);
            let code = codegen_return.code;
            let map = if sourcemap {
                match (codegen_return.map, input_sourcemap) {
                    (Some(codegen_map), Some(input_map)) => {
                        let module_path = &scan_result.modules[module_idx].path;
                        Some(sourcemap::compose_sourcemaps(
                            &codegen_map,
                            input_map,
                            module_path,
                            cwd,
                        ))
                    }
                    (map, _) => map,
                }
            } else {
                None
            };

            fragments.push(DeclarationFragment {
                code,
                map,
                defined_symbols: meta.defined_symbols,
                export_names: meta.export_names,
                imports: meta.imports,
                referenced_names,
                is_export_statement: meta.is_export_statement,
                star_exports: meta.star_exports,
                bare_reexport_specs: meta.bare_reexport_specs,
                internal_ns_reexports: meta.internal_ns_reexports,
            });
        } else {
            // Metadata-only fragment (no code output)
            fragments.push(DeclarationFragment {
                code: String::new(),
                map: None,
                defined_symbols: meta.defined_symbols,
                export_names: meta.export_names,
                imports: meta.imports,
                referenced_names: FxHashSet::default(),
                is_export_statement: meta.is_export_statement,
                star_exports: meta.star_exports,
                bare_reexport_specs: meta.bare_reexport_specs,
                internal_ns_reexports: meta.internal_ns_reexports,
            });
        }
    }

    Some(ModuleFragments {
        fragments,
        relative_path,
        rewriter_imports,
        ns_wrapper_blocks,
        reexported_import_names,
        external_ns_info,
        warnings,
    })
}

/// Per-statement metadata collected during Phase 1 processing.
#[derive(Default)]
struct StatementMeta {
    defined_symbols: Vec<(SymbolId, NeededKindFlags)>,
    export_names: Vec<ExportedName>,
    imports: Vec<ExternalImport>,
    is_export_statement: bool,
    star_exports: Vec<ExternalStarExport>,
    bare_reexport_specs: Vec<BareReexportSpec>,
    internal_ns_reexports: Vec<InternalNsReexport>,
    internal_star_reexport_exports: Vec<ExportedName>,
}

impl StatementMeta {
    fn has_metadata(&self) -> bool {
        !self.export_names.is_empty()
            || !self.imports.is_empty()
            || self.is_export_statement
            || !self.star_exports.is_empty()
            || !self.bare_reexport_specs.is_empty()
            || !self.internal_ns_reexports.is_empty()
            || !self.internal_star_reexport_exports.is_empty()
    }
}

/// Process a single statement for fragment creation.
///
/// Unlike `process_statement_ast`, this does NOT tree-shake (no needed_symbol_kinds check)
/// and stores export metadata in `StatementMeta` for deferred assembly.
fn process_statement_for_fragment<'a>(
    stmt: Statement<'a>,
    ast: AstBuilder<'a>,
    output: &mut oxc_allocator::Vec<'a, Statement<'a>>,
    module: &Module<'a>,
    scan_result: &ScanResult<'a>,
    rename_plan: &RenamePlan,
    default_export_names: &FxHashMap<ModuleIdx, String>,
    allocator: &'a Allocator,
    meta: &mut StatementMeta,
) {
    match stmt {
        Statement::ExportNamedDeclaration(mut export_decl) => {
            meta.is_export_statement = true;
            if let Some(mut decl) = export_decl.declaration.take() {
                // Collect defined symbols for tree-shaking
                let decl_symbols = collect_decl_symbol_kinds(&decl, &module.scoping);
                meta.defined_symbols.extend(decl_symbols);

                ensure_declare_on_declaration(&mut decl);
                decl.span_mut().start = export_decl.span.start;

                // Collect export metadata (always, not just for entry modules)
                if module.is_entry {
                    collect_declaration_names(&decl, &mut meta.export_names);
                }
                output.push(Statement::from(decl));
            } else if let Some(source) = &export_decl.source {
                let internal_source_idx = module.resolve_internal_specifier(source.value.as_str());
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
                        meta.imports.push(ExternalImport {
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
                            if let Some((origin_module_idx, resolved)) =
                                resolve_export_origin(source_module_idx, &local_name, scan_result)
                            {
                                local_name = resolved;
                                local_module_idx = origin_module_idx;
                            } else if let Some((ext_source, imported_name)) =
                                find_external_reexport_source(
                                    source_module_idx,
                                    &local_name,
                                    scan_result,
                                )
                            {
                                meta.imports.push(ExternalImport {
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
                                && let Some(name) = default_export_names.get(&local_module_idx)
                            {
                                local_name.clone_from(name);
                            }
                            if let Some(new_name) = rename_plan
                                .resolve_name(&scan_result.modules[local_module_idx], &local_name)
                            {
                                local_name = new_name.to_string();
                            }
                            meta.export_names.push(ExportedName {
                                local: local_name,
                                exported: exported_name,
                                is_type_only: export_decl.export_kind.is_type()
                                    || spec.export_kind.is_type(),
                            });
                        } else {
                            meta.export_names.push(ExportedName {
                                local: exported_name.clone(),
                                exported: exported_name,
                                is_type_only: export_decl.export_kind.is_type()
                                    || spec.export_kind.is_type(),
                            });
                        }
                    }
                }
            } else if module.is_entry {
                // Bare `export { X }` — needs namespace resolution at assembly time
                for spec in &export_decl.specifiers {
                    let exported_name = spec.exported.name().to_string();
                    let spec_is_type =
                        export_decl.export_kind.is_type() || spec.export_kind.is_type();
                    let symbol_id = module.scoping.get_root_binding(Ident::from(spec.local.name()));
                    meta.bare_reexport_specs.push(BareReexportSpec {
                        symbol_id,
                        local: spec.local.name().to_string(),
                        exported: exported_name,
                        is_type_only: spec_is_type,
                    });
                }
            }
        }
        Statement::ExportDefaultDeclaration(export_default) => {
            meta.is_export_statement = true;
            // Collect defined symbols
            match &export_default.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                    if let Some(id) = &func.id
                        && let Some(symbol_id) =
                            module.scoping.get_root_binding(Ident::from(id.name.as_str()))
                    {
                        meta.defined_symbols.push((symbol_id, NeededKindFlags::VALUE));
                    }
                }
                ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                    if let Some(id) = &class.id
                        && let Some(symbol_id) =
                            module.scoping.get_root_binding(Ident::from(id.name.as_str()))
                    {
                        meta.defined_symbols.push((symbol_id, NeededKindFlags::ALL));
                    }
                }
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
                    if let Some(symbol_id) =
                        module.scoping.get_root_binding(Ident::from(iface.id.name.as_str()))
                    {
                        meta.defined_symbols.push((symbol_id, NeededKindFlags::TYPE));
                    }
                }
                _ => {}
            }

            // Clone the declaration to unwrap it from the export.
            // This is the only remaining clone_in — for export default declarations.
            let declaration = export_default.declaration.clone_in_with_semantic_ids(allocator);
            match declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(mut func_decl) => {
                    func_decl.span.start = export_default.span.start;
                    let name = if let Some(id) = &func_decl.id {
                        id.name.to_string()
                    } else {
                        func_decl.id = Some(ast.binding_identifier(SPAN, "export_default"));
                        "export_default".to_string()
                    };
                    func_decl.declare = true;
                    output.push(Statement::FunctionDeclaration(func_decl));
                    if module.is_entry {
                        meta.export_names.push(ExportedName {
                            local: name,
                            exported: "default".to_string(),
                            is_type_only: false,
                        });
                    }
                }
                ExportDefaultDeclarationKind::ClassDeclaration(mut class_decl) => {
                    class_decl.span.start = export_default.span.start;
                    let name = if let Some(id) = &class_decl.id {
                        id.name.to_string()
                    } else {
                        class_decl.id = Some(ast.binding_identifier(SPAN, "export_default"));
                        "export_default".to_string()
                    };
                    class_decl.declare = true;
                    output.push(Statement::ClassDeclaration(class_decl));
                    if module.is_entry {
                        meta.export_names.push(ExportedName {
                            local: name,
                            exported: "default".to_string(),
                            is_type_only: false,
                        });
                    }
                }
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(mut iface_decl) => {
                    iface_decl.span.start = export_default.span.start;
                    let name = iface_decl.id.name.to_string();
                    output.push(Statement::TSInterfaceDeclaration(iface_decl));
                    if module.is_entry {
                        meta.export_names.push(ExportedName {
                            local: name,
                            exported: "default".to_string(),
                            is_type_only: false,
                        });
                    }
                }
                kind => {
                    if module.is_entry
                        && let ExportDefaultDeclarationKind::Identifier(id) = kind
                    {
                        meta.export_names.push(ExportedName {
                            local: id.name.to_string(),
                            exported: "default".to_string(),
                            is_type_only: false,
                        });
                    }
                }
            }
        }
        Statement::ExportAllDeclaration(export_all) => {
            meta.is_export_statement = true;
            let internal_source_idx =
                module.resolve_internal_specifier(export_all.source.value.as_str());
            if let Some(exported) = &export_all.exported {
                let name = exported.name().to_string();
                if let Some(source_module_idx) = internal_source_idx {
                    if module.is_entry {
                        // Needs namespace_wraps at assembly time
                        meta.internal_ns_reexports.push(InternalNsReexport {
                            source_module_idx,
                            exported_name: name,
                            is_type_only: export_all.export_kind.is_type(),
                        });
                    }
                } else {
                    meta.imports.push(ExternalImport {
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
                        meta.export_names.push(ExportedName {
                            local: name.clone(),
                            exported: name,
                            is_type_only: export_all.export_kind.is_type(),
                        });
                    }
                }
            } else if let Some(source_module_idx) = internal_source_idx {
                if module.is_entry {
                    // Pre-computed data will be used at assembly time.
                    // For now, store the source_module_idx as a marker.
                    // The actual exports/imports are in PrecomputedEntryData.star_reexport_data.
                    // We store an empty internal_star_reexport here as a placeholder.
                    // Assembly will look up PrecomputedEntryData.star_reexport_data[source_module_idx].
                    meta.internal_ns_reexports.push(InternalNsReexport {
                        source_module_idx,
                        exported_name: String::new(), // empty = plain star, not named
                        is_type_only: false,
                    });
                }
            } else {
                meta.star_exports.push(ExternalStarExport {
                    source: export_all.source.value.to_string(),
                    is_type_only: false,
                });
            }
        }
        Statement::ImportDeclaration(import_decl) => {
            if module.resolve_internal_specifier(import_decl.source.value.as_str()).is_some() {
                // Internal import — drop
                return;
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
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => ImportSpecifier {
                            local: s.local.name.to_string(),
                            kind: ImportSpecifierKind::Default,
                        },
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
                && !module.resolved_external_specifiers.contains(import_decl.source.value.as_str())
            {
                return;
            }
            meta.imports.push(ExternalImport {
                source: import_decl.source.value.to_string(),
                side_effect_only: specifiers.is_empty(),
                specifiers,
                is_type_only: false,
                from_reexport: false,
            });
        }
        Statement::TSNamespaceExportDeclaration(_) | Statement::TSExportAssignment(_) => {}
        Statement::TSImportEqualsDeclaration(decl) => {
            if let oxc_ast::ast::TSModuleReference::ExternalModuleReference(ext) =
                &decl.module_reference
                && module.resolve_internal_specifier(ext.expression.value.as_str()).is_some()
            {
                return;
            }
            output.push(Statement::TSImportEqualsDeclaration(decl));
        }
        _ => {
            // Collect defined symbols for tree-shaking
            if let Some(decl) = stmt.as_declaration() {
                let decl_symbols = collect_decl_symbol_kinds(decl, &module.scoping);
                meta.defined_symbols.extend(decl_symbols);
            }
            output.push(stmt);
        }
    }
}

/// Phase 2: Assemble per-entry output from pre-computed fragments.
pub fn assemble_entry(
    entry_idx: ModuleIdx,
    module_fragments: &[Option<ModuleFragments>],
    precomputed: &PrecomputedEntryData,
    scan_result: &ScanResult<'_>,
    sourcemap: bool,
    cjs_default: bool,
) -> GenerateOutput {
    let mut output = OutputAssembler::default();
    let mut exports: Vec<ExportedName> = Vec::new();
    let mut imports: Vec<ExternalImport> = Vec::new();
    let mut star_exports: Vec<ExternalStarExport> = Vec::new();
    let mut has_any_export_statement = false;
    let mut all_warnings: Vec<OxcDiagnostic> = Vec::new();
    let mut all_ns_wrapper_blocks = String::new();

    // Collect warnings from precomputation
    all_warnings.extend(precomputed.warnings.iter().cloned());

    // Collect and deduplicate reference directives
    let mut seen_set: FxHashSet<&str> = FxHashSet::default();
    let mut unique_directives: Vec<&str> = Vec::new();
    for module in &scan_result.modules {
        for directive in &module.reference_directives {
            if seen_set.insert(directive.as_str()) {
                unique_directives.push(directive.as_str());
            }
        }
    }
    for directive in &unique_directives {
        output.push_unmapped(format!("{directive}\n"));
    }

    // Process modules in reverse order (to match original ordering behavior)
    let mut module_outputs: VecDeque<ModuleOutput> = VecDeque::new();
    for module_idx_usize in (0..scan_result.modules.len()).rev() {
        let module_idx = ModuleIdx::from_usize(module_idx_usize);
        let ns_wrap = precomputed.namespace_wraps.get(&module_idx);
        let module_has_augmentation = scan_result.modules[module_idx].has_augmentation;

        let (module_is_needed, module_needed): (
            bool,
            Option<&FxHashMap<SymbolId, NeededKindFlags>>,
        ) = match precomputed.needed_symbol_kinds.get(&module_idx) {
            Some(entry) => (true, entry.as_ref()),
            None => (module_has_augmentation, None),
        };

        if !module_is_needed {
            continue;
        }

        let Some(module_frags) = &module_fragments[module_idx_usize] else {
            continue;
        };

        // Select fragments based on tree-shaking
        let mut selected_code = String::new();
        let mut selected_maps: Vec<(oxc_sourcemap::SourceMap, u32)> = Vec::new();
        let mut current_line_offset: u32 = 0;
        let mut module_referenced_names: FxHashSet<&str> = FxHashSet::default();
        let mut module_exported_locals: FxHashSet<String> = FxHashSet::default();

        for fragment in &module_frags.fragments {
            // Check if this fragment passes tree-shaking
            let should_include = if let Some(needed) = module_needed {
                if fragment.defined_symbols.is_empty() {
                    true // no defined symbols = always include (e.g., imports, type-only stmts)
                } else {
                    fragment.defined_symbols.iter().any(|(symbol_id, decl_kinds)| {
                        needed
                            .get(symbol_id)
                            .is_some_and(|needed_kinds| needed_kinds.intersects(*decl_kinds))
                    })
                }
            } else {
                true // no tree-shaking filter
            };

            if !should_include {
                continue;
            }

            if !fragment.code.is_empty() {
                if let Some(map) = &fragment.map {
                    selected_maps.push((map.clone(), current_line_offset));
                }
                current_line_offset +=
                    u32::try_from(fragment.code.match_indices('\n').count()).unwrap();
                selected_code.push_str(&fragment.code);
            }
            module_referenced_names.extend(fragment.referenced_names.iter().map(String::as_str));

            if fragment.is_export_statement {
                has_any_export_statement = true;
            }

            // Collect exports for entry module
            if scan_result.modules[module_idx].is_entry && module_idx == entry_idx {
                for e in &fragment.export_names {
                    module_exported_locals.insert(e.local.clone());
                    exports.push(e.clone());
                }

                // Resolve bare re-export specifiers with namespace info
                for spec in &fragment.bare_reexport_specs {
                    let local = if let Some(symbol_id) = spec.symbol_id
                        && let Some(source_module_idx) =
                            precomputed.namespace_aliases.get(&symbol_id)
                        && let Some(wrap) = precomputed.namespace_wraps.get(source_module_idx)
                    {
                        wrap.namespace_name.clone()
                    } else {
                        spec.local.clone()
                    };
                    module_exported_locals.insert(local.clone());
                    exports.push(ExportedName {
                        local,
                        exported: spec.exported.clone(),
                        is_type_only: spec.is_type_only,
                    });
                }

                // Resolve internal namespace re-exports
                for ns_reexport in &fragment.internal_ns_reexports {
                    if ns_reexport.exported_name.is_empty() {
                        // Plain star re-export: `export * from "./internal"`
                        // Use pre-computed data
                        if let Some(star_data) =
                            precomputed.star_reexport_data.get(&ns_reexport.source_module_idx)
                        {
                            exports.extend(star_data.exports.iter().cloned());
                            imports.extend(star_data.imports.iter().cloned());
                        }
                    } else if let Some(wrap) =
                        precomputed.namespace_wraps.get(&ns_reexport.source_module_idx)
                    {
                        // Named namespace re-export: `export * as X from "./internal"`
                        exports.push(ExportedName {
                            local: wrap.namespace_name.clone(),
                            exported: ns_reexport.exported_name.clone(),
                            is_type_only: ns_reexport.is_type_only,
                        });
                    }
                }
            }

            // Collect imports from this fragment
            imports.extend(fragment.imports.iter().cloned());

            // Collect star exports from this fragment
            star_exports.extend(fragment.star_exports.iter().cloned());
        }

        // Add rewriter imports for this module
        imports.extend(module_frags.rewriter_imports.iter().cloned());

        // Import pruning: drop unused external namespace helpers
        if !module_frags.external_ns_info.is_empty() {
            for (specifier, local_name) in module_frags.external_ns_info.values() {
                if module_referenced_names.contains(local_name.as_str())
                    || module_exported_locals.contains(local_name.as_str())
                    || !is_generated_external_namespace_helper(local_name)
                {
                    continue;
                }
                if let Some(idx) = imports.iter().position(|i| {
                    i.source == *specifier
                        && i.specifiers.iter().any(|s| {
                            matches!(s.kind, ImportSpecifierKind::Namespace)
                                && s.local == *local_name
                        })
                }) {
                    imports[idx].specifiers.retain(|s| {
                        !(matches!(s.kind, ImportSpecifierKind::Namespace)
                            && s.local == *local_name)
                    });
                    if imports[idx].specifiers.is_empty() {
                        imports.remove(idx);
                    }
                }
            }
        }

        // Import pruning: drop unreferenced named/default imports
        let imports_start = imports.len().saturating_sub(
            module_frags.rewriter_imports.len()
                + module_frags.fragments.iter().map(|f| f.imports.len()).sum::<usize>(),
        );
        for import in &mut imports[imports_start..] {
            if import.from_reexport {
                continue;
            }
            import.specifiers.retain(|specifier| match specifier.kind {
                ImportSpecifierKind::Namespace => true,
                ImportSpecifierKind::Default | ImportSpecifierKind::Named(_) => {
                    module_referenced_names.contains(specifier.local.as_str())
                        || module_exported_locals.contains(specifier.local.as_str())
                        || module_frags.reexported_import_names.contains(specifier.local.as_str())
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

        // Collect warnings
        all_warnings.extend(module_frags.warnings.iter().cloned());

        // Accumulate ns_wrapper_blocks
        if !module_frags.ns_wrapper_blocks.is_empty() {
            all_ns_wrapper_blocks.push_str(&module_frags.ns_wrapper_blocks);
        }

        if selected_code.is_empty() && ns_wrap.is_none() {
            continue;
        }

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

        // Combine maps for this module
        let combined_map = if sourcemap && !selected_maps.is_empty() {
            let mut builder = oxc_sourcemap::ConcatSourceMapBuilder::default();
            for (map, line_offset) in &selected_maps {
                builder.add_sourcemap(map, *line_offset);
            }
            Some(builder.into_sourcemap())
        } else {
            None
        };

        module_outputs.push_front(ModuleOutput {
            relative_path: module_frags.relative_path.clone(),
            is_ns_wrapped: ns_wrap.is_some(),
            namespace_wrapper,
            code: selected_code,
            map: combined_map,
        });
    }

    // Emit merged external imports
    let had_imports = !imports.is_empty();
    let mut external_imports_output = String::new();
    emit::write_external_imports(&mut imports, &mut external_imports_output);
    output.push_unmapped(external_imports_output);

    // Emit star re-exports
    let mut star_exports_output = String::new();
    for star in &star_exports {
        let type_str = if star.is_type_only { "type " } else { "" };
        writeln!(star_exports_output, "export {type_str}* from \"{}\";", star.source).unwrap();
    }
    output.push_unmapped(star_exports_output);

    let has_module_output =
        !all_ns_wrapper_blocks.is_empty() || module_outputs.iter().any(|m| !m.code.is_empty());

    if (had_imports || !star_exports.is_empty()) && has_module_output {
        output.push_unmapped("\n");
    }

    if !all_ns_wrapper_blocks.is_empty() {
        output.push_unmapped(all_ns_wrapper_blocks);
    }

    // Emit namespace-wrapped modules first, then regular modules
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

    // Consolidated export statement
    if let Some(default_local) = cjs_default_export_local(cjs_default, &exports) {
        output.push_unmapped(format!("export = {default_local};"));
    } else if !exports.is_empty() {
        let mut export_output = String::new();
        emit::write_export_statement(&exports, &mut export_output);
        output.push_unmapped(export_output);
    } else if has_any_export_statement && star_exports.is_empty() {
        output.push_unmapped("export { };");
    }

    let mut generated = output.finish();
    while generated.code.ends_with('\n') {
        generated.code.pop();
    }
    generated.warnings = all_warnings;
    generated
}

fn cjs_default_export_local(cjs_default: bool, exports: &[ExportedName]) -> Option<&str> {
    if !cjs_default || exports.len() != 1 {
        return None;
    }
    let only = &exports[0];
    if only.exported == "default" && !only.is_type_only {
        return Some(only.local.as_str());
    }
    None
}
