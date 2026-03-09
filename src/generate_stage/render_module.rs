//! Per-module rendering: clone, finalize, codegen.

use std::fmt::Write;
use std::path::PathBuf;

use oxc_allocator::{Allocator, CloneIn, TakeIn};
use oxc_ast::AstBuilder;
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, IdentifierReference, Statement, TSTypeName, TSTypeQuery,
};
use oxc_ast_visit::{Visit, VisitMut};
use oxc_codegen::{Codegen, CodegenOptions, IndentChar};
use oxc_span::{GetSpanMut, SPAN};
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::link_stage::{
    CanonicalNames, LinkStageOutput, ModuleLinkMeta, PerEntryLinkData, StatementAction,
};
use crate::scan_stage::ScanStageOutput;
use crate::types::ModuleIdx;

use super::finalizer::{DtsFinalizer, ensure_declare_on_declaration};
use super::sourcemap;
use super::types::*;

pub(super) struct RenderedModule {
    pub relative_path: String,
    pub is_ns_wrapped: bool,
    pub namespace_wrapper: Option<String>,
    pub code: String,
    pub map: Option<oxc_sourcemap::SourceMap>,
}

pub(super) fn render_module<'a>(
    scan_result: &mut ScanStageOutput<'a>,
    allocator: &'a Allocator,
    module_idx: ModuleIdx,
    meta: &ModuleLinkMeta,
    per_entry: &PerEntryLinkData,
    link_output: &LinkStageOutput,
    sourcemap_enabled: bool,
    cwd: &std::path::Path,
    acc: &mut GenerateAcc,
    single_entry: bool,
) -> Option<RenderedModule> {
    let ns_wrap = per_entry.namespace_wraps.get(&module_idx);
    let ast = AstBuilder::new(allocator);
    let module = &scan_result.module_table[module_idx];

    // Phase 2: Selective clone + transform — only clone statements that
    // survive tree-shaking, and for unwrapped exports clone only the inner
    // declaration (not the export wrapper).
    let mut transformed_body = ast.vec();

    if single_entry {
        // Single-entry fast path: take ownership of AST nodes (no cloning).
        for (i, action) in meta.statement_actions.iter().enumerate() {
            match action {
                StatementAction::Skip => {}
                StatementAction::Include => {
                    let stmt = scan_result.ast_table[module_idx].body[i].take_in(allocator);
                    transformed_body.push(stmt);
                }
                StatementAction::UnwrapExportDeclaration => {
                    let stmt = scan_result.ast_table[module_idx].body[i].take_in(allocator);
                    let Statement::ExportNamedDeclaration(export) = stmt else { unreachable!() };
                    let export_span_start = export.span.start;
                    let mut decl = export.unbox().declaration.unwrap();
                    ensure_declare_on_declaration(&mut decl);
                    decl.span_mut().start = export_span_start;
                    transformed_body.push(Statement::from(decl));
                }
                StatementAction::UnwrapExportDefault => {
                    let stmt = scan_result.ast_table[module_idx].body[i].take_in(allocator);
                    let Statement::ExportDefaultDeclaration(export_default) = stmt else {
                        unreachable!()
                    };
                    let export_default = export_default.unbox();
                    let span_start = export_default.span.start;
                    match export_default.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(mut func_decl) => {
                            func_decl.span.start = span_start;
                            if func_decl.id.is_none() {
                                func_decl.id = Some(ast.binding_identifier(SPAN, "export_default"));
                            }
                            func_decl.declare = true;
                            transformed_body.push(Statement::FunctionDeclaration(func_decl));
                        }
                        ExportDefaultDeclarationKind::ClassDeclaration(mut class_decl) => {
                            class_decl.span.start = span_start;
                            if class_decl.id.is_none() {
                                class_decl.id =
                                    Some(ast.binding_identifier(SPAN, "export_default"));
                            }
                            class_decl.declare = true;
                            transformed_body.push(Statement::ClassDeclaration(class_decl));
                        }
                        ExportDefaultDeclarationKind::TSInterfaceDeclaration(mut iface_decl) => {
                            iface_decl.span.start = span_start;
                            transformed_body.push(Statement::TSInterfaceDeclaration(iface_decl));
                        }
                        _ => {}
                    }
                }
            }
        }
    } else {
        // Multi-entry path: clone statements (AST is shared across entries).
        for (i, action) in meta.statement_actions.iter().enumerate() {
            match action {
                StatementAction::Skip => {}
                StatementAction::Include => {
                    let stmt = scan_result.ast_table[module_idx].body[i]
                        .clone_in_with_semantic_ids(allocator);
                    transformed_body.push(stmt);
                }
                StatementAction::UnwrapExportDeclaration => {
                    let Statement::ExportNamedDeclaration(export) =
                        &scan_result.ast_table[module_idx].body[i]
                    else {
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
                        &scan_result.ast_table[module_idx].body[i]
                    else {
                        unreachable!()
                    };
                    let declaration =
                        export_default.declaration.clone_in_with_semantic_ids(allocator);
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
    }

    if transformed_body.is_empty() && ns_wrap.is_none() {
        return None;
    }

    // For single-entry: compute and apply renames inline (full DtsFinalizer).
    // For multi-entry: renames are pre-applied, so only run DtsFinalizer when
    // structural mutations are needed (inline import rewriting, namespace alias
    // stripping). Skip the entire traversal for rename-only modules.
    let external_ns_members = if single_entry || meta.needs_structural_mutation {
        let merged_renames = if single_entry {
            build_merged_renames(&link_output.canonical_names, module_idx, &meta.import_renames)
        } else {
            FxHashMap::default()
        };
        let mut finalizer = DtsFinalizer {
            ast,
            allocator,
            scoping: &module.scoping,
            renamed_symbols: &merged_renames,
            module,
            imports: &mut acc.imports,
            ns_name_map: &mut acc.ns_name_map,
            scan_result,
            ns_wrapper_output: &mut acc.ns_wrapper_blocks,
            namespace_aliases: &meta.ns_aliases,
            external_ns_info: &meta.external_ns_info,
            external_ns_members: FxHashMap::default(),
            helper_reserved_names: &per_entry.helper_reserved_names,
            warnings: &mut acc.warnings,
        };
        finalizer.visit_statements(&mut transformed_body);
        finalizer.external_ns_members
    } else {
        FxHashMap::default()
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
                        matches!(s.kind, ImportSpecifierKind::Namespace) && s.local == ns_local_name
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
    let imports_start = acc.module_imports_start;
    let exports_start = acc.module_exports_start;
    prune_unused_imports(
        &mut acc.imports,
        imports_start,
        &transformed_body,
        &acc.exports[exports_start..],
        &meta.reexported_import_names,
        &meta.external_ns_info,
    );

    let (relative_path, program_span, source_type, source_text, reference_directive_set) = {
        let module = &scan_result.module_table[module_idx];
        let program = &scan_result.ast_table[module_idx];
        (
            module.relative_path.clone(),
            program.span,
            program.source_type,
            module.source,
            module.reference_directives.iter().cloned().collect::<FxHashSet<String>>(),
        )
    };

    let (comments, hashbang, directives) = if single_entry {
        // Single-entry: take ownership instead of cloning.
        let program = &mut scan_result.ast_table[module_idx];
        let raw_comments = program.comments.take_in(allocator);
        let comments = ast.vec_from_iter(raw_comments.into_iter().filter(|comment| {
            let comment_text = comment.span.source_text(source_text).trim();
            if reference_directive_set.contains(comment_text) {
                return false;
            }
            !(comment_text.starts_with("//# sourceMappingURL=")
                || comment_text.starts_with("//@ sourceMappingURL="))
        }));
        let hashbang = program.hashbang.take();
        let directives = program.directives.take_in(allocator);
        (comments, hashbang, directives)
    } else {
        let program = &scan_result.ast_table[module_idx];
        let raw_comments = program.comments.clone_in_with_semantic_ids(allocator);
        let comments = ast.vec_from_iter(raw_comments.into_iter().filter(|comment| {
            let comment_text = comment.span.source_text(source_text).trim();
            if reference_directive_set.contains(comment_text) {
                return false;
            }
            !(comment_text.starts_with("//# sourceMappingURL=")
                || comment_text.starts_with("//@ sourceMappingURL="))
        }));
        let hashbang = program.hashbang.clone_in_with_semantic_ids(allocator);
        let directives = program.directives.clone_in_with_semantic_ids(allocator);
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

    let mut codegen_options = CodegenOptions {
        indent_char: IndentChar::Space,
        indent_width: 2,
        ..CodegenOptions::default()
    };
    if sourcemap_enabled {
        codegen_options.source_map_path = Some(PathBuf::from(&relative_path));
    }
    let codegen_return = Codegen::new().with_options(codegen_options).build(&program);
    let code = codegen_return.code;
    let map = if sourcemap_enabled {
        match codegen_return.map {
            Some(codegen_map) => {
                if let Some(input_map) = &scan_result.module_table[module_idx].input_sourcemap {
                    let module_path = &scan_result.module_table[module_idx].path;
                    Some(sourcemap::compose_sourcemaps(&codegen_map, input_map, module_path, cwd))
                } else {
                    Some(codegen_map)
                }
            }
            None => None,
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

    Some(RenderedModule {
        relative_path,
        is_ns_wrapped: ns_wrap.is_some(),
        namespace_wrapper,
        code,
        map,
    })
}

/// Build merged rename map combining symbol renames from link stage
/// and import renames from module meta.
pub(super) fn build_merged_renames(
    canonical_names: &CanonicalNames,
    module_idx: ModuleIdx,
    import_renames: &FxHashMap<SymbolId, String>,
) -> FxHashMap<SymbolId, String> {
    let mut renamed_symbols = canonical_names.module_symbol_renames(module_idx);
    renamed_symbols.extend(import_renames.iter().map(|(k, v)| (*k, v.clone())));
    renamed_symbols
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
    let module_exported_locals: FxHashSet<&str> =
        module_exports.iter().map(|export| export.local.as_str()).collect();

    // Drop TypeScript-generated external namespace helpers whose only
    // usages were inside tree-shaken declarations. Keep user-authored
    // namespace imports (for example `import * as http`) to preserve the
    // plugin's current snapshot semantics.
    if !external_ns_info.is_empty() {
        for (specifier, local_name) in external_ns_info.values() {
            if referenced_names.contains(local_name.as_str())
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

struct ReferencedNameCollector<'a> {
    referenced_names: FxHashSet<&'a str>,
}

impl<'a> ReferencedNameCollector<'a> {
    fn new() -> Self {
        Self { referenced_names: FxHashSet::default() }
    }

    fn finish(self) -> FxHashSet<&'a str> {
        self.referenced_names
    }

    fn record_name(&mut self, name: &'a str) {
        self.referenced_names.insert(name);
    }

    fn record_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.record_name(ident.name.as_str());
    }

    fn record_value_type_name(&mut self, type_name: &TSTypeName<'a>) {
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

impl<'a> Visit<'a> for ReferencedNameCollector<'a> {
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
