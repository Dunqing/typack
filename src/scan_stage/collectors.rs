//! AST visitors that extract dependency hints and metadata from parsed modules.

use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, ImportDeclarationSpecifier, Program, Statement,
    TSModuleDeclaration, TSModuleDeclarationName,
};
use oxc_ast_visit::{Visit, walk};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::helpers::collect_decl_names;
use crate::types::{
    ExportEntry, ExportSource, ImportBinding, ImportBindingKind, ModuleExportImportInfo,
    StarReexport,
};

#[derive(Default)]
pub(super) struct ModuleDependencyHints {
    pub eager: Vec<String>,
    pub side_effect: Vec<String>,
    pub external: FxHashSet<String>,
}

#[derive(Default)]
struct InlineImportSpecifierCollector {
    specifiers: Vec<String>,
    seen_specifiers: FxHashSet<String>,
}

impl<'a> Visit<'a> for InlineImportSpecifierCollector {
    fn visit_ts_import_type(&mut self, import_type: &oxc_ast::ast::TSImportType<'a>) {
        let specifier = import_type.source.value.to_string();
        if self.seen_specifiers.insert(specifier.clone()) {
            self.specifiers.push(specifier);
        }
        walk::walk_ts_import_type(self, import_type);
    }
}

pub(super) fn collect_module_dependency_hints(program: &Program<'_>) -> ModuleDependencyHints {
    let mut hints = ModuleDependencyHints::default();
    let mut seen_eager: FxHashSet<String> = FxHashSet::default();
    let mut seen_side_effect: FxHashSet<String> = FxHashSet::default();

    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(decl) => {
                let specifier = decl.source.value.to_string();
                let has_specifiers = decl.specifiers.as_ref().is_some_and(|s| !s.is_empty());
                if has_specifiers {
                    push_unique_specifier(&mut hints.eager, &mut seen_eager, specifier);
                } else {
                    push_unique_specifier(&mut hints.side_effect, &mut seen_side_effect, specifier);
                }
            }
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source {
                    push_unique_specifier(
                        &mut hints.eager,
                        &mut seen_eager,
                        source.value.to_string(),
                    );
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                push_unique_specifier(
                    &mut hints.eager,
                    &mut seen_eager,
                    decl.source.value.to_string(),
                );
            }
            // Handle top-level `import X = require("./mod")`
            Statement::TSImportEqualsDeclaration(decl) => {
                if let oxc_ast::ast::TSModuleReference::ExternalModuleReference(ext) =
                    &decl.module_reference
                {
                    push_unique_specifier(
                        &mut hints.eager,
                        &mut seen_eager,
                        ext.expression.value.to_string(),
                    );
                }
            }
            _ => {}
        }
    }

    let mut inline_imports = InlineImportSpecifierCollector::default();
    inline_imports.visit_program(program);
    for specifier in inline_imports.specifiers {
        push_unique_specifier(&mut hints.eager, &mut seen_eager, specifier);
    }

    for specifier in hints.eager.iter().chain(hints.side_effect.iter()) {
        if !specifier.starts_with('.') {
            hints.external.insert(specifier.clone());
        }
    }

    hints
}

fn push_unique_specifier(
    specifiers: &mut Vec<String>,
    seen: &mut FxHashSet<String>,
    specifier: String,
) {
    if seen.insert(specifier.clone()) {
        specifiers.push(specifier);
    }
}

pub(super) fn collect_leading_reference_directives(
    source: &str,
    program: &Program<'_>,
) -> Vec<String> {
    let mut directives = Vec::new();
    let mut cursor = 0usize;

    for comment in &program.comments {
        let start = comment.span.start as usize;
        let end = comment.span.end as usize;

        if start < cursor {
            continue;
        }

        if contains_non_whitespace(&source[cursor..start]) {
            break;
        }

        let trimmed = comment.span.source_text(source).trim();
        if trimmed.starts_with("/// <reference ") {
            directives.push(trimmed.to_string());
            cursor = end;
            continue;
        }

        if !trimmed.is_empty() {
            break;
        }

        cursor = end;
    }

    directives
}

fn contains_non_whitespace(text: &str) -> bool {
    text.chars().any(|ch| !ch.is_whitespace())
}

pub(super) fn has_top_level_augmentation(program: &Program<'_>) -> bool {
    program.body.iter().any(statement_has_top_level_augmentation)
}

fn statement_has_top_level_augmentation(stmt: &Statement<'_>) -> bool {
    match stmt {
        Statement::TSGlobalDeclaration(_) => true,
        Statement::TSModuleDeclaration(module_decl) => {
            module_declaration_is_augmentation(module_decl)
        }
        Statement::ExportNamedDeclaration(export_decl) => {
            export_decl.declaration.as_ref().is_some_and(declaration_has_top_level_augmentation)
        }
        _ => false,
    }
}

fn declaration_has_top_level_augmentation(decl: &Declaration<'_>) -> bool {
    match decl {
        Declaration::TSGlobalDeclaration(_) => true,
        Declaration::TSModuleDeclaration(module_decl) => {
            module_declaration_is_augmentation(module_decl)
        }
        _ => false,
    }
}

fn module_declaration_is_augmentation(module_decl: &TSModuleDeclaration<'_>) -> bool {
    matches!(module_decl.id, TSModuleDeclarationName::StringLiteral(_))
}

/// Collect pre-computed export/import information from a parsed program.
///
/// This walks the AST body **once** and populates structured maps that the link
/// and generate stages can query in O(1) instead of repeatedly traversing statements.
pub(super) fn collect_export_import_info(program: &Program<'_>) -> ModuleExportImportInfo {
    let mut named_exports: FxHashMap<String, ExportEntry> = FxHashMap::default();
    let mut star_reexports: Vec<StarReexport> = Vec::new();
    let mut named_imports: FxHashMap<String, ImportBinding> = FxHashMap::default();
    let mut declared_export_names: Vec<String> = Vec::new();

    for stmt in &program.body {
        match stmt {
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(d) = &decl.declaration {
                    // `export interface Foo {}`, `export declare function bar(): void`, etc.
                    let mut names = Vec::new();
                    collect_decl_names(d, &mut names);
                    for name in names {
                        declared_export_names.push(name.clone());
                        named_exports.insert(
                            name.clone(),
                            ExportEntry {
                                local_name: name,
                                source: ExportSource::LocalDeclaration,
                            },
                        );
                    }
                } else if let Some(source) = &decl.source {
                    // `export { X } from "./mod"` or `export { X as Y } from "./mod"`
                    let specifier = source.value.to_string();
                    for spec in &decl.specifiers {
                        let exported_name = spec.exported.name().to_string();
                        let imported_name = spec.local.name().to_string();
                        named_exports.insert(
                            exported_name.clone(),
                            ExportEntry {
                                local_name: exported_name,
                                source: ExportSource::SourceReexport {
                                    specifier: specifier.clone(),
                                    imported_name,
                                },
                            },
                        );
                    }
                } else {
                    // `export { X }` or `export { X as Y }` — local re-export (no source)
                    for spec in &decl.specifiers {
                        let exported_name = spec.exported.name().to_string();
                        let local_name = spec.local.name().to_string();
                        named_exports.insert(
                            exported_name,
                            ExportEntry { local_name, source: ExportSource::LocalReexport },
                        );
                    }
                }
            }
            Statement::ExportAllDeclaration(decl) => {
                let alias = decl.exported.as_ref().map(|e| e.name().to_string());
                star_reexports
                    .push(StarReexport { specifier: decl.source.value.to_string(), alias });
            }
            Statement::ExportDefaultDeclaration(default_decl) => {
                let local_name = match &default_decl.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        func.id.as_ref().map(|id| id.name.to_string())
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        class.id.as_ref().map(|id| id.name.to_string())
                    }
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
                        Some(iface.id.name.to_string())
                    }
                    ExportDefaultDeclarationKind::Identifier(id) => Some(id.name.to_string()),
                    _ => None,
                };
                if let Some(name) = local_name {
                    named_exports.insert(
                        "default".to_string(),
                        ExportEntry { local_name: name, source: ExportSource::Default },
                    );
                }
            }
            Statement::ImportDeclaration(import_decl) => {
                let source_specifier = import_decl.source.value.to_string();
                if let Some(specifiers) = &import_decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                let local_name = s.local.name.to_string();
                                let imported_name = s.imported.name().to_string();
                                named_imports.insert(
                                    local_name,
                                    ImportBinding {
                                        source_specifier: source_specifier.clone(),
                                        kind: ImportBindingKind::Named(imported_name),
                                    },
                                );
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                let local_name = s.local.name.to_string();
                                named_imports.insert(
                                    local_name,
                                    ImportBinding {
                                        source_specifier: source_specifier.clone(),
                                        kind: ImportBindingKind::Default,
                                    },
                                );
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                let local_name = s.local.name.to_string();
                                named_imports.insert(
                                    local_name,
                                    ImportBinding {
                                        source_specifier: source_specifier.clone(),
                                        kind: ImportBindingKind::Namespace,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    ModuleExportImportInfo { named_exports, star_reexports, named_imports, declared_export_names }
}
