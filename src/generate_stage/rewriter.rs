//! AST mutation passes for the generate stage.
//!
//! Implements `SemanticRenamer` (applies symbol renames from the link stage) and
//! `InlineImportAndNamespaceRewriter` (rewrites inline `import("...")` type
//! expressions to direct type references and flattens namespace alias member
//! accesses).

use std::fmt::Write;

use oxc_allocator::FromIn;
use oxc_ast::AstBuilder;
use oxc_ast::ast::{
    Declaration, Expression, TSImportTypeQualifier, TSType, TSTypeName, TSTypeQueryExprName,
};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_diagnostics::OxcDiagnostic;
use oxc_semantic::Scoping;
use oxc_span::{Ident, SPAN};
use oxc_syntax::symbol::SymbolId;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::link_stage::collect_all_exported_names;

use crate::scan_stage::ScanResult;
use crate::types::Module;

use super::namespace::{get_or_create_ns_name, sanitize_to_identifier};
use super::types::{ExternalImport, ImportSpecifier, ImportSpecifierKind};

pub(super) fn ensure_declare_on_declaration(decl: &mut Declaration<'_>) {
    match decl {
        Declaration::VariableDeclaration(d) => d.declare = true,
        Declaration::FunctionDeclaration(d) => d.declare = true,
        Declaration::ClassDeclaration(d) => d.declare = true,
        Declaration::TSEnumDeclaration(d) => d.declare = true,
        Declaration::TSModuleDeclaration(d) => d.declare = true,
        Declaration::TSTypeAliasDeclaration(_)
        | Declaration::TSInterfaceDeclaration(_)
        | Declaration::TSGlobalDeclaration(_)
        | Declaration::TSImportEqualsDeclaration(_) => {}
    }
}

pub(super) struct SemanticRenamer<'a, 's> {
    pub allocator: &'a oxc_allocator::Allocator,
    pub scoping: &'s Scoping,
    pub renamed_symbols: &'s FxHashMap<oxc_syntax::symbol::SymbolId, String>,
}

impl<'a> VisitMut<'a> for SemanticRenamer<'a, '_> {
    fn visit_binding_identifier(&mut self, ident: &mut oxc_ast::ast::BindingIdentifier<'a>) {
        if let Some(symbol_id) = ident.symbol_id.get()
            && let Some(new_name) = self.renamed_symbols.get(&symbol_id)
        {
            ident.name = Ident::from_in(new_name.as_str(), self.allocator);
        }
    }

    fn visit_identifier_reference(&mut self, ident: &mut oxc_ast::ast::IdentifierReference<'a>) {
        if let Some(reference_id) = ident.reference_id.get()
            && let Some(symbol_id) = self.scoping.get_reference(reference_id).symbol_id()
            && let Some(new_name) = self.renamed_symbols.get(&symbol_id)
        {
            ident.name = Ident::from_in(new_name.as_str(), self.allocator);
        }
    }
}

pub(super) struct InlineImportAndNamespaceRewriter<'a, 's> {
    pub ast: AstBuilder<'a>,
    pub module: &'s Module<'a>,
    pub imports: &'s mut Vec<ExternalImport>,
    pub ns_name_map: &'s mut FxHashMap<String, String>,
    pub scan_result: &'s ScanResult<'a>,
    pub ns_wrapper_output: &'s mut String,
    pub namespace_aliases: FxHashSet<SymbolId>,
    /// Map: external ns SymbolId → (source specifier, local name)
    pub external_ns_info: &'s FxHashMap<SymbolId, (String, String)>,
    /// Tracks which members are accessed per external specifier (built during visit).
    pub external_ns_members: FxHashMap<String, FxHashSet<String>>,
    pub helper_reserved_names: &'s FxHashSet<String>,
    pub warnings: &'s mut Vec<OxcDiagnostic>,
}

impl<'a> InlineImportAndNamespaceRewriter<'a, '_> {
    fn ensure_external_namespace_import(&mut self, specifier: &str) -> String {
        if let Some(existing_name) = self
            .imports
            .iter()
            .find(|imp| imp.source == specifier)
            .and_then(|imp| {
                imp.specifiers
                    .iter()
                    .find(|spec| matches!(spec.kind, ImportSpecifierKind::Namespace))
            })
            .map(|spec| spec.local.clone())
        {
            self.ns_name_map.entry(specifier.to_string()).or_insert_with(|| existing_name.clone());
            return existing_name;
        }

        let ns_name = get_or_create_ns_name(
            specifier,
            self.ns_name_map,
            self.helper_reserved_names,
            self.warnings,
            sanitize_to_identifier,
        );

        self.imports.push(ExternalImport {
            source: specifier.to_string(),
            specifiers: vec![ImportSpecifier {
                local: ns_name.clone(),
                kind: ImportSpecifierKind::Namespace,
            }],
            is_type_only: false,
            side_effect_only: false,
            from_reexport: false,
        });

        ns_name
    }

    fn ensure_internal_typeof_namespace(&mut self, specifier: &str) -> String {
        let ns_name = get_or_create_ns_name(
            specifier,
            self.ns_name_map,
            self.helper_reserved_names,
            self.warnings,
            |spec| {
                if let Some(source_module_idx) = self.module.resolve_internal_specifier(spec) {
                    let stem = self.scan_result.modules[source_module_idx]
                        .path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    format!("{}_exports", sanitize_to_identifier(&stem))
                } else {
                    format!("{}_exports", sanitize_to_identifier(spec))
                }
            },
        );

        let namespace_decl_head = format!("declare namespace {ns_name} {{");
        if self.ns_wrapper_output.contains(&namespace_decl_head) {
            return ns_name;
        }

        if let Some(source_module_idx) = self.module.resolve_internal_specifier(specifier) {
            let export_names = collect_all_exported_names(source_module_idx, self.scan_result);
            if !export_names.is_empty() {
                let mut sorted_names: Vec<&String> = export_names.iter().collect();
                sorted_names.sort();
                write!(self.ns_wrapper_output, "{namespace_decl_head}\n  export {{ ").unwrap();
                for (index, name) in sorted_names.iter().enumerate() {
                    if index > 0 {
                        self.ns_wrapper_output.push_str(", ");
                    }
                    self.ns_wrapper_output.push_str(name);
                }
                self.ns_wrapper_output.push_str(" };\n}\n");
            }
        }

        ns_name
    }

    fn rewrite_import_type_to_type_name(
        &mut self,
        import_type: &oxc_ast::ast::TSImportType<'a>,
    ) -> Option<TSTypeName<'a>> {
        let specifier = import_type.source.value.as_str();
        let is_internal = self.module.resolve_internal_specifier(specifier).is_some();
        if !is_internal && !specifier.starts_with('.') {
            debug_assert!(self.module.resolved_external_specifiers.contains(specifier));
        }

        if is_internal {
            let mut segments = Vec::new();
            collect_import_qualifier_segments(import_type.qualifier.as_ref()?, &mut segments);
            return build_type_name_from_segments(self.ast, &segments);
        }

        let namespace_name = self.ensure_external_namespace_import(specifier);
        let mut segments = vec![namespace_name];
        if let Some(qualifier) = import_type.qualifier.as_ref() {
            collect_import_qualifier_segments(qualifier, &mut segments);
        }
        build_type_name_from_segments(self.ast, &segments)
    }

    fn rewrite_import_type_to_query_expr(
        &mut self,
        import_type: &oxc_ast::ast::TSImportType<'a>,
    ) -> Option<TSTypeQueryExprName<'a>> {
        let specifier = import_type.source.value.as_str();
        let is_internal = self.module.resolve_internal_specifier(specifier).is_some();
        if !is_internal && !specifier.starts_with('.') {
            debug_assert!(self.module.resolved_external_specifiers.contains(specifier));
        }

        let type_name = if is_internal {
            if let Some(qualifier) = import_type.qualifier.as_ref() {
                let mut segments = Vec::new();
                collect_import_qualifier_segments(qualifier, &mut segments);
                build_type_name_from_segments(self.ast, &segments)?
            } else {
                let ns_name = self.ensure_internal_typeof_namespace(specifier);
                self.ast.ts_type_name_identifier_reference(
                    SPAN,
                    Ident::from_in(ns_name.as_str(), self.ast.allocator),
                )
            }
        } else {
            let namespace_name = self.ensure_external_namespace_import(specifier);
            let mut segments = vec![namespace_name];
            if let Some(qualifier) = import_type.qualifier.as_ref() {
                collect_import_qualifier_segments(qualifier, &mut segments);
            }
            build_type_name_from_segments(self.ast, &segments)?
        };

        Some(type_name_to_query_expr_name(type_name))
    }
}

impl<'a> VisitMut<'a> for InlineImportAndNamespaceRewriter<'a, '_> {
    fn visit_ts_type_name(&mut self, it: &mut TSTypeName<'a>) {
        // Record external namespace member access before stripping
        if let Some((root_symbol, member_name)) =
            extract_ns_member_access_type_name(it, &self.module.scoping)
            && let Some((specifier, _)) = self.external_ns_info.get(&root_symbol)
        {
            self.external_ns_members.entry(specifier.clone()).or_default().insert(member_name);
        }

        if let Some(rewritten) = strip_namespace_alias_type_name(
            self.ast,
            it,
            &self.namespace_aliases,
            &self.module.scoping,
        ) {
            *it = rewritten;
        }
        walk_mut::walk_ts_type_name(self, it);
    }

    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        // Record external namespace member access before stripping
        if let Some((root_symbol, member_name)) =
            extract_ns_member_access_expression(it, &self.module.scoping)
            && let Some((specifier, _)) = self.external_ns_info.get(&root_symbol)
        {
            self.external_ns_members.entry(specifier.clone()).or_default().insert(member_name);
        }

        if let Some(rewritten) = strip_namespace_alias_expression(
            self.ast,
            it,
            &self.namespace_aliases,
            &self.module.scoping,
        ) {
            *it = rewritten;
            walk_mut::walk_expression(self, it);
            return;
        }
        walk_mut::walk_expression(self, it);
    }

    fn visit_ts_type(&mut self, it: &mut TSType<'a>) {
        if let TSType::TSImportType(import_type) = it
            && let Some(type_name) = self.rewrite_import_type_to_type_name(import_type)
        {
            let type_arguments = import_type.type_arguments.take();
            *it = TSType::TSTypeReference(self.ast.alloc_ts_type_reference(
                import_type.span,
                type_name,
                type_arguments,
            ));
            walk_mut::walk_ts_type(self, it);
            return;
        }
        walk_mut::walk_ts_type(self, it);
    }

    fn visit_ts_type_query(&mut self, it: &mut oxc_ast::ast::TSTypeQuery<'a>) {
        let rewritten_expr_name = match &it.expr_name {
            TSTypeQueryExprName::TSImportType(import_type) => {
                self.rewrite_import_type_to_query_expr(import_type)
            }
            _ => None,
        };

        if let Some(expr_name) = rewritten_expr_name {
            it.expr_name = expr_name;
        }
        walk_mut::walk_ts_type_query(self, it);
    }
}

fn collect_import_qualifier_segments(qualifier: &TSImportTypeQualifier<'_>, out: &mut Vec<String>) {
    match qualifier {
        TSImportTypeQualifier::Identifier(ident) => out.push(ident.name.to_string()),
        TSImportTypeQualifier::QualifiedName(qualified) => {
            collect_import_qualifier_segments(&qualified.left, out);
            out.push(qualified.right.name.to_string());
        }
    }
}

fn collect_type_name_segments(type_name: &TSTypeName<'_>, out: &mut Vec<String>) -> bool {
    match type_name {
        TSTypeName::IdentifierReference(ident) => {
            out.push(ident.name.to_string());
            true
        }
        TSTypeName::QualifiedName(qualified) => {
            if !collect_type_name_segments(&qualified.left, out) {
                return false;
            }
            out.push(qualified.right.name.to_string());
            true
        }
        TSTypeName::ThisExpression(_) => false,
    }
}

fn build_type_name_from_segments<'a>(
    ast: AstBuilder<'a>,
    segments: &[String],
) -> Option<TSTypeName<'a>> {
    let mut iter = segments.iter();
    let first = iter.next()?;
    let mut type_name =
        ast.ts_type_name_identifier_reference(SPAN, Ident::from_in(first.as_str(), ast.allocator));
    for segment in iter {
        type_name = ast.ts_type_name_qualified_name(
            SPAN,
            type_name,
            ast.identifier_name(SPAN, Ident::from_in(segment.as_str(), ast.allocator)),
        );
    }
    Some(type_name)
}

fn strip_namespace_alias_type_name<'a>(
    ast: AstBuilder<'a>,
    type_name: &TSTypeName<'a>,
    namespace_aliases: &FxHashSet<SymbolId>,
    scoping: &Scoping,
) -> Option<TSTypeName<'a>> {
    let mut segments = Vec::new();
    if !collect_type_name_segments(type_name, &mut segments) || segments.len() < 2 {
        return None;
    }
    let root_symbol = resolve_type_name_root_symbol(type_name, scoping)?;
    if !namespace_aliases.contains(&root_symbol) {
        return None;
    }
    build_type_name_from_segments(ast, &segments[1..])
}

fn collect_expression_segments(expression: &Expression<'_>, out: &mut Vec<String>) -> bool {
    match expression {
        Expression::Identifier(ident) => {
            out.push(ident.name.to_string());
            true
        }
        Expression::StaticMemberExpression(member) => {
            if !collect_expression_segments(&member.object, out) {
                return false;
            }
            out.push(member.property.name.to_string());
            true
        }
        _ => false,
    }
}

fn build_expression_from_segments<'a>(
    ast: AstBuilder<'a>,
    segments: &[String],
) -> Option<Expression<'a>> {
    let mut iter = segments.iter();
    let first = iter.next()?;
    let mut expression =
        ast.expression_identifier(SPAN, Ident::from_in(first.as_str(), ast.allocator));
    for segment in iter {
        expression = Expression::StaticMemberExpression(ast.alloc_static_member_expression(
            SPAN,
            expression,
            ast.identifier_name(SPAN, Ident::from_in(segment.as_str(), ast.allocator)),
            false,
        ));
    }
    Some(expression)
}

fn strip_namespace_alias_expression<'a>(
    ast: AstBuilder<'a>,
    expression: &Expression<'a>,
    namespace_aliases: &FxHashSet<SymbolId>,
    scoping: &Scoping,
) -> Option<Expression<'a>> {
    let mut segments = Vec::new();
    if !collect_expression_segments(expression, &mut segments) || segments.len() < 2 {
        return None;
    }
    let root_symbol = resolve_expression_root_symbol(expression, scoping)?;
    if !namespace_aliases.contains(&root_symbol) {
        return None;
    }
    build_expression_from_segments(ast, &segments[1..])
}

fn resolve_type_name_root_symbol(
    type_name: &TSTypeName<'_>,
    scoping: &Scoping,
) -> Option<SymbolId> {
    match type_name {
        TSTypeName::IdentifierReference(ident) => {
            let ref_id = ident.reference_id.get()?;
            scoping.get_reference(ref_id).symbol_id()
        }
        TSTypeName::QualifiedName(qualified) => {
            resolve_type_name_root_symbol(&qualified.left, scoping)
        }
        TSTypeName::ThisExpression(_) => None,
    }
}

fn resolve_expression_root_symbol(expr: &Expression<'_>, scoping: &Scoping) -> Option<SymbolId> {
    match expr {
        Expression::Identifier(ident) => {
            let ref_id = ident.reference_id.get()?;
            scoping.get_reference(ref_id).symbol_id()
        }
        Expression::StaticMemberExpression(member) => {
            resolve_expression_root_symbol(&member.object, scoping)
        }
        _ => None,
    }
}

fn extract_ns_member_access_type_name(
    type_name: &TSTypeName<'_>,
    scoping: &Scoping,
) -> Option<(SymbolId, String)> {
    let mut segments = Vec::new();
    if !collect_type_name_segments(type_name, &mut segments) || segments.len() < 2 {
        return None;
    }
    let root_symbol = resolve_type_name_root_symbol(type_name, scoping)?;
    Some((root_symbol, segments[1].clone()))
}

fn extract_ns_member_access_expression(
    expression: &Expression<'_>,
    scoping: &Scoping,
) -> Option<(SymbolId, String)> {
    let mut segments = Vec::new();
    if !collect_expression_segments(expression, &mut segments) || segments.len() < 2 {
        return None;
    }
    let root_symbol = resolve_expression_root_symbol(expression, scoping)?;
    Some((root_symbol, segments[1].clone()))
}

fn type_name_to_query_expr_name(type_name: TSTypeName<'_>) -> TSTypeQueryExprName<'_> {
    match type_name {
        TSTypeName::IdentifierReference(ident) => TSTypeQueryExprName::IdentifierReference(ident),
        TSTypeName::QualifiedName(qualified) => TSTypeQueryExprName::QualifiedName(qualified),
        TSTypeName::ThisExpression(this_expr) => TSTypeQueryExprName::ThisExpression(this_expr),
    }
}
