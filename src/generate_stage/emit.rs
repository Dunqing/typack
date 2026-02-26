//! Import and export statement formatting for the bundled output.

use std::fmt::Write;

use rustc_hash::FxHashMap;

use super::types::{ExportedName, ExternalImport, ImportSpecifier, ImportSpecifierKind};

/// Write merged external import statements, sorted and deduplicated.
pub(super) fn write_external_imports(imports: &mut Vec<ExternalImport>, output: &mut String) {
    if imports.is_empty() {
        return;
    }

    // Merge imports from the same source module using a HashMap index for O(1) lookup
    let mut merged: Vec<ExternalImport> = Vec::new();
    let mut source_index: FxHashMap<String, usize> = FxHashMap::default();
    for imp in imports.drain(..) {
        if let Some(&idx) = source_index.get(&imp.source) {
            let existing = &mut merged[idx];
            // &= ensures the merged import is type-only only if ALL contributing
            // imports are type-only; any value-level import forces a regular import.
            existing.is_type_only &= imp.is_type_only;
            existing.side_effect_only &= imp.side_effect_only;
            for spec in imp.specifiers {
                if !existing
                    .specifiers
                    .iter()
                    .any(|s| s.local == spec.local && s.kind.sort_key() == spec.kind.sort_key())
                {
                    existing.specifiers.push(spec);
                }
            }
            if !existing.specifiers.is_empty() {
                existing.side_effect_only = false;
            }
        } else {
            source_index.insert(imp.source.clone(), merged.len());
            merged.push(imp);
        }
    }

    // Sort specifiers within each import alphabetically by imported name
    for imp in &mut merged {
        imp.specifiers.sort_by(|a, b| a.kind.sort_key().cmp(b.kind.sort_key()));
    }

    for imp in &merged {
        if imp.side_effect_only || imp.specifiers.is_empty() {
            writeln!(output, "import \"{}\";", imp.source).unwrap();
            continue;
        }

        let type_prefix = if imp.is_type_only { "type " } else { "" };

        // Emit each namespace specifier as a separate import statement,
        // since `import * as X` cannot be combined with named/default specifiers.
        for ns_spec in
            imp.specifiers.iter().filter(|s| matches!(s.kind, ImportSpecifierKind::Namespace))
        {
            writeln!(output, "import {type_prefix}* as {} from \"{}\";", ns_spec.local, imp.source)
                .unwrap();
        }

        // Emit default + named specifiers together (if any)
        let default_spec =
            imp.specifiers.iter().find(|s| matches!(s.kind, ImportSpecifierKind::Default));
        let named: Vec<&ImportSpecifier> = imp
            .specifiers
            .iter()
            .filter(|s| matches!(s.kind, ImportSpecifierKind::Named(_)))
            .collect();

        if default_spec.is_none() && named.is_empty() {
            continue;
        }

        let mut parts = Vec::new();
        if let Some(def) = default_spec {
            parts.push(def.local.clone());
        }
        if !named.is_empty() {
            let specs: Vec<String> = named
                .iter()
                .filter_map(|s| {
                    if let ImportSpecifierKind::Named(imported) = &s.kind {
                        Some(if s.local == *imported {
                            imported.clone()
                        } else {
                            format!("{imported} as {}", s.local)
                        })
                    } else {
                        None
                    }
                })
                .collect();
            parts.push(format!("{{ {} }}", specs.join(", ")));
        }
        writeln!(output, "import {type_prefix}{} from \"{}\";", parts.join(", "), imp.source)
            .unwrap();
    }
}

/// Write the consolidated export statement: `export { A, B, C };`
pub(super) fn write_export_statement(exports: &[ExportedName], output: &mut String) {
    let mut sorted: Vec<&ExportedName> = exports.iter().collect();
    sorted.sort_by(|a, b| a.exported.cmp(&b.exported));
    // Deduplicate by exported name (e.g., multiple overloads of same default export)
    sorted.dedup_by(|a, b| a.exported == b.exported);

    output.push_str("export { ");
    for (i, exp) in sorted.iter().enumerate() {
        if i > 0 {
            output.push_str(", ");
        }
        if exp.is_type_only {
            output.push_str("type ");
        }
        if exp.local == exp.exported {
            output.push_str(&exp.exported);
        } else {
            write!(output, "{} as {}", exp.local, exp.exported).unwrap();
        }
    }
    output.push_str(" };");
}
