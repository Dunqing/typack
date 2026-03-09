//! Output collection phase: walks the original AST using pre-computed
//! per-statement actions from the link stage, and collects exports, imports,
//! star exports into GenerateAcc.

use oxc_ast::ast::{ExportDefaultDeclarationKind, Statement};
use rustc_hash::FxHashSet;

use super::namespace::{
    collect_declaration_names, collect_export_specifier, collect_module_exports,
};
use super::types::{
    ExportedName, ExternalImport, ExternalStarExport, GenerateAcc, ImportSpecifier,
    ImportSpecifierKind,
};
use crate::link_stage::exports::{find_external_reexport_source, resolve_export_origin};
use crate::link_stage::{LinkStageOutput, ModuleLinkMeta, PerEntryLinkData, StatementAction};
use crate::scan_stage::ScanStageOutput;
use crate::types::ModuleIdx;

/// Output collection phase: walks the original AST using pre-computed
/// per-statement actions from the link stage, and collects exports, imports,
/// star exports directly into `acc`.
pub(super) fn collect_module_outputs(
    scan_result: &ScanStageOutput,
    module_idx: ModuleIdx,
    meta: &ModuleLinkMeta,
    per_entry: &PerEntryLinkData,
    link_output: &LinkStageOutput,
    acc: &mut GenerateAcc,
) {
    let module = &scan_result.module_table[module_idx];

    for (i, stmt) in scan_result.ast_table[module_idx].body.iter().enumerate() {
        collect_statement_outputs(
            scan_result,
            stmt,
            &meta.statement_actions[i],
            module,
            per_entry,
            link_output,
            acc,
        );
    }
}

/// Collect output metadata (exports, imports, star exports) for a single
/// statement, using the pre-computed action from the link stage.
fn collect_statement_outputs<'a>(
    scan_result: &ScanStageOutput,
    stmt: &Statement<'a>,
    action: &StatementAction,
    module: &crate::types::Module<'a>,
    per_entry: &PerEntryLinkData,
    link_output: &LinkStageOutput,
    acc: &mut GenerateAcc,
) {
    match stmt {
        Statement::ExportNamedDeclaration(export_decl) => {
            acc.has_any_export_statement = true;
            if let Some(decl) = &export_decl.declaration {
                if matches!(action, StatementAction::Skip) {
                    return;
                }
                if module.is_entry {
                    let before_len = acc.exports.len();
                    collect_declaration_names(decl, &mut acc.exports);
                    for exp in &mut acc.exports[before_len..] {
                        if let Some(new_name) =
                            link_output.canonical_names.resolve_name(module, &exp.local)
                        {
                            exp.local = new_name.to_string();
                        }
                    }
                }
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
                                    link_output.default_export_names.get(&local_module_idx)
                            {
                                local_name.clone_from(name);
                            }
                            if let Some(new_name) = link_output.canonical_names.resolve_name(
                                &scan_result.module_table[local_module_idx],
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
            } else {
                // Bare specifiers: `export { X, Y }`
                if module.is_entry {
                    for spec in &export_decl.specifiers {
                        let exported_name = spec.exported.name().to_string();
                        let spec_is_type =
                            export_decl.export_kind.is_type() || spec.export_kind.is_type();
                        let symbol_id = module
                            .scoping
                            .get_root_binding(oxc_span::Ident::from(spec.local.name()));
                        if let Some(symbol_id) = symbol_id
                            && let Some(source_module_idx) =
                                per_entry.namespace_aliases.get(&symbol_id)
                            && let Some(wrap) = per_entry.namespace_wraps.get(source_module_idx)
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
                                    link_output.canonical_names.resolve_name(module, &exp.local)
                                {
                                    exp.local = new_name.to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
        Statement::ExportDefaultDeclaration(export_default) => {
            acc.has_any_export_statement = true;
            // Only skip output collection when the declaration was tree-shaken
            // (not needed by needed_symbol_kinds). The Identifier variant always
            // returns Skip (nothing to clone), but its export still needs collecting.
            if matches!(action, StatementAction::Skip)
                && !matches!(
                    export_default.declaration,
                    ExportDefaultDeclarationKind::Identifier(_)
                )
            {
                return;
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
                }
                ExportDefaultDeclarationKind::Identifier(id) => {
                    if module.is_entry {
                        acc.exports.push(ExportedName {
                            local: id.name.to_string(),
                            exported: "default".to_string(),
                            is_type_only: false,
                        });
                    }
                }
                _ => {}
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
                        && let Some(wrap) = per_entry.namespace_wraps.get(&source_module_idx)
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
                        scan_result,
                        &mut visited,
                        Some(&mut star_external_imports),
                    );
                    acc.imports.extend(star_external_imports);
                    for exp in &mut acc.exports[before_len..] {
                        if let Some(new_name) = link_output
                            .canonical_names
                            .resolve_name(&scan_result.module_table[source_module_idx], &exp.local)
                        {
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
        }
        Statement::ImportDeclaration(import_decl) => {
            if module.resolve_internal_specifier(import_decl.source.value.as_str()).is_some() {
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
            acc.imports.push(ExternalImport {
                source: import_decl.source.value.to_string(),
                side_effect_only: specifiers.is_empty(),
                specifiers,
                is_type_only: false,
                from_reexport: false,
            });
        }
        _ => {}
    }
}
