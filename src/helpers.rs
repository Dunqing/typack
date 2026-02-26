//! Shared AST helper functions for extracting declaration names.

use oxc_ast::ast::{BindingPattern, Declaration, ExportDefaultDeclarationKind, Statement};

/// Collect top-level binding names introduced by a `Declaration` node.
///
/// This extracts the names that a declaration introduces into the module scope,
/// used for rename/conflict resolution when bundling multiple `.d.ts` modules.
///
/// Only simple `BindingIdentifier` patterns are collected from variable declarations
/// (destructuring patterns are not expanded) since `.d.ts` files rarely use them.
pub fn collect_decl_names(decl: &Declaration<'_>, names: &mut Vec<String>) {
    match decl {
        Declaration::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                    names.push(id.name.to_string());
                }
            }
        }
        Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                names.push(id.name.to_string());
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                names.push(id.name.to_string());
            }
        }
        Declaration::TSTypeAliasDeclaration(alias) => {
            names.push(alias.id.name.to_string());
        }
        Declaration::TSInterfaceDeclaration(iface) => {
            names.push(iface.id.name.to_string());
        }
        Declaration::TSEnumDeclaration(enum_decl) => {
            names.push(enum_decl.id.name.to_string());
        }
        Declaration::TSModuleDeclaration(module_decl) => {
            if let oxc_ast::ast::TSModuleDeclarationName::Identifier(id) = &module_decl.id {
                names.push(id.name.to_string());
            }
        }
        // `TSGlobalDeclaration` augments the global scope without introducing a module-level name.
        // `TSImportEqualsDeclaration` (e.g. `import Foo = require("./bar")`) is treated as an
        // import rather than a declaration — it gets resolved and its target inlined during the
        // scan/link stages, so the name never appears in the bundled output.
        Declaration::TSGlobalDeclaration(_) | Declaration::TSImportEqualsDeclaration(_) => {}
    }
}

/// Collect declaration names from a single statement.
///
/// This handles both exported and non-exported declarations at the statement level,
/// covering `export { ... }` wrappers, `export default`, and bare ambient declarations
/// that appear directly as statements in `.d.ts` files.
pub fn collect_statement_declaration_names(stmt: &Statement<'_>, names: &mut Vec<String>) {
    match stmt {
        Statement::ExportNamedDeclaration(export_decl) => {
            if let Some(decl) = &export_decl.declaration {
                collect_decl_names(decl, names);
            }
        }
        Statement::ExportDefaultDeclaration(export_default) => match &export_default.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    names.push(id.name.to_string());
                }
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    names.push(id.name.to_string());
                }
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
                names.push(iface.id.name.to_string());
            }
            _ => {}
        },
        // Non-exported declarations (ambient declarations in .d.ts)
        Statement::TSTypeAliasDeclaration(alias) => {
            names.push(alias.id.name.to_string());
        }
        Statement::TSInterfaceDeclaration(iface) => {
            names.push(iface.id.name.to_string());
        }
        Statement::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                    names.push(id.name.to_string());
                }
            }
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                names.push(id.name.to_string());
            }
        }
        Statement::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                names.push(id.name.to_string());
            }
        }
        Statement::TSEnumDeclaration(enum_decl) => {
            names.push(enum_decl.id.name.to_string());
        }
        Statement::TSModuleDeclaration(module_decl) => {
            if let oxc_ast::ast::TSModuleDeclarationName::Identifier(id) = &module_decl.id {
                names.push(id.name.to_string());
            }
        }
        _ => {}
    }
}
