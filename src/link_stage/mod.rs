//! Link stage: analyzes the module graph to plan tree-shaking and name
//! deconfliction.

pub mod exports;
mod needed_names;
mod rename;
pub mod resolved_exports;
mod types;
mod warnings;

use rustc_hash::FxHashMap;

use crate::scan_stage::ScanResult;
use crate::types::ModuleIdx;

pub use exports::{collect_all_exported_names, resolve_default_export_name};
pub use needed_names::build_needed_names;
pub use rename::build_rename_plan;
pub use resolved_exports::build_resolved_exports;
pub use types::NeededKindFlags;
pub use types::{LinkOutput, NeededNamesPlan, RenamePlan};

use warnings::collect_link_warnings;

pub fn build_link_output(scan_result: &ScanResult<'_>) -> LinkOutput {
    let rename_plan = build_rename_plan(scan_result);
    let needed_names_plan = build_merged_needed_names(scan_result);

    let mut default_export_names: FxHashMap<ModuleIdx, String> = FxHashMap::default();
    for module in &scan_result.modules {
        if let Some(name) = resolve_default_export_name(module.idx, scan_result) {
            default_export_names.insert(module.idx, name);
        }
    }

    let mut warnings = collect_link_warnings(&rename_plan, scan_result);
    warnings.extend(build_resolved_exports(scan_result));

    LinkOutput { rename_plan, needed_names_plan, default_export_names, warnings }
}

fn build_merged_needed_names(scan_result: &ScanResult<'_>) -> NeededNamesPlan {
    debug_assert!(!scan_result.entry_indices.is_empty());
    let mut merged = NeededNamesPlan::default();

    for &entry_idx in &scan_result.entry_indices {
        let plan = build_needed_names(&scan_result.modules[entry_idx], scan_result);
        for (module_idx, incoming) in plan.map {
            match (merged.map.remove(&module_idx), incoming) {
                (Some(None), _) | (_, None) => {
                    merged.map.insert(module_idx, None);
                }
                (Some(Some(mut existing)), Some(incoming)) => {
                    existing.extend(incoming);
                    merged.map.insert(module_idx, Some(existing));
                }
                (None, Some(incoming)) => {
                    merged.map.insert(module_idx, Some(incoming));
                }
            }
        }
        for (module_idx, incoming) in plan.symbol_kinds {
            match (merged.symbol_kinds.remove(&module_idx), incoming) {
                (Some(None), _) | (_, None) => {
                    merged.symbol_kinds.insert(module_idx, None);
                }
                (Some(Some(mut existing)), Some(incoming)) => {
                    for (symbol_id, incoming_kind) in incoming {
                        existing
                            .entry(symbol_id)
                            .and_modify(|kind| *kind = kind.union(incoming_kind))
                            .or_insert(incoming_kind);
                    }
                    merged.symbol_kinds.insert(module_idx, Some(existing));
                }
                (None, Some(incoming)) => {
                    merged.symbol_kinds.insert(module_idx, Some(incoming));
                }
            }
        }
        for (key, incoming_reasons) in plan.reasons {
            merged.reasons.entry(key).or_default().extend(incoming_reasons);
        }
    }

    // Entry modules must stay whole even when they are pulled in as
    // dependencies of another entry in a multi-entry build.
    for &entry_idx in &scan_result.entry_indices {
        merged.map.insert(entry_idx, None);
        merged.symbol_kinds.insert(entry_idx, None);
    }

    merged
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use oxc_allocator::Allocator;

    use crate::options::TypackOptions;
    use crate::scan_stage::ScanStage;

    use super::types::NeededReason;
    use super::{RenamePlan, build_needed_names, collect_link_warnings};

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
                "typack_link_stage_{name}_{}_{}",
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

        fn scan(&self, entry: &str) -> crate::scan_stage::ScanResult<'_> {
            self.scan_many(&[entry])
        }

        fn scan_many(&self, entries: &[&str]) -> crate::scan_stage::ScanResult<'_> {
            let allocator = Box::leak(Box::new(Allocator::default()));
            let options = TypackOptions {
                input: entries
                    .iter()
                    .map(|entry| self.root.join(entry).to_string_lossy().to_string())
                    .collect(),
                cwd: self.root.clone(),
                ..Default::default()
            };
            ScanStage::new(&options, allocator)
                .scan()
                .unwrap_or_else(|diagnostics| panic!("scan failed: {diagnostics:?}"))
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn needed_reasons_track_entry_and_semantic_dependencies() {
        let project = TempProject::new("needed_reasons_semantic");
        project
            .write_file("mod.d.ts", "type A = { a: number };\ntype B = A;\nexport type C = B;\n");
        project.write_file("index.d.ts", "export { C } from \"./mod\";\n");

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mod_idx = entry.resolve_internal_specifier("./mod").expect("mod should resolve");

        let c_reasons = plan.reasons_for(mod_idx, "C").expect("C should have reasons");
        assert!(c_reasons.contains(&NeededReason::EntryNamedReexport));

        let b_reasons = plan.reasons_for(mod_idx, "B").expect("B should have reasons");
        assert!(b_reasons.contains(&NeededReason::SemanticDependency));

        let a_reasons = plan.reasons_for(mod_idx, "A").expect("A should have reasons");
        assert!(a_reasons.contains(&NeededReason::SemanticDependency));
    }

    #[test]
    fn needed_reasons_track_named_propagation() {
        let project = TempProject::new("needed_reasons_named_propagation");
        project.write_file("mod.d.ts", "export interface A { value: string }\n");
        project.write_file("mid.d.ts", "export { A } from \"./mod\";\n");
        project.write_file("index.d.ts", "export { A } from \"./mid\";\n");

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mid_idx = entry.resolve_internal_specifier("./mid").expect("mid should resolve");
        let mod_idx = scan_result.modules[mid_idx]
            .resolve_internal_specifier("./mod")
            .expect("mod should resolve");

        let mid_reasons = plan.reasons_for(mid_idx, "A").expect("A should have reasons in mid");
        assert!(mid_reasons.contains(&NeededReason::EntryNamedReexport));

        let mod_reasons = plan.reasons_for(mod_idx, "A").expect("A should have reasons in mod");
        assert!(mod_reasons.contains(&NeededReason::PropagationNamedReexport));
    }

    #[test]
    fn needed_reasons_track_entry_star_reexport() {
        let project = TempProject::new("needed_reasons_entry_star");
        project.write_file("mod.d.ts", "export interface A { value: string }\n");
        project.write_file("index.d.ts", "export * from \"./mod\";\n");

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mod_idx = entry.resolve_internal_specifier("./mod").expect("mod should resolve");
        let mod_reasons = plan.reasons_for(mod_idx, "A").expect("A should have reasons in mod");
        assert!(mod_reasons.contains(&NeededReason::EntryStarReexport));
    }

    #[test]
    fn needed_reasons_track_entry_declaration_import_references() {
        let project = TempProject::new("needed_reasons_entry_decl_import_refs");
        project.write_file(
            "mod.d.ts",
            "export interface Config { value: string }\nexport declare const runtime: unique symbol;\n",
        );
        project.write_file(
            "index.d.ts",
            "export { Config } from \"./mod\";\nimport { runtime } from \"./mod\";\nexport declare function foo(): typeof runtime;\n",
        );

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mod_idx = entry.resolve_internal_specifier("./mod").expect("mod should resolve");

        let config_reasons =
            plan.reasons_for(mod_idx, "Config").expect("Config should have reasons in mod");
        assert!(config_reasons.contains(&NeededReason::EntryNamedReexport));

        let runtime_reasons =
            plan.reasons_for(mod_idx, "runtime").expect("runtime should have reasons in mod");
        assert!(runtime_reasons.contains(&NeededReason::CrossModuleImportDependency));
    }

    #[test]
    fn entry_default_export_identifier_marks_imported_module_needed() {
        let project = TempProject::new("entry_default_export_identifier");
        project.write_file(
            "mod.d.ts",
            "declare class Foo { value: string }\nexport default Foo;\nexport interface Extra { ok: true }\n",
        );
        project.write_file("index.d.ts", "import Foo from \"./mod\";\nexport default Foo;\n");

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mod_idx = entry.resolve_internal_specifier("./mod").expect("mod should resolve");

        assert!(matches!(plan.map.get(&mod_idx), Some(None)));
    }

    #[test]
    fn semantic_dependencies_track_type_only_need_for_class_symbols() {
        let project = TempProject::new("semantic_dependencies_track_type_only_class_need");
        project.write_file(
            "mod.d.ts",
            "export declare class Foo { value: string }\nexport type UsesFoo = Foo;\n",
        );
        project.write_file("index.d.ts", "export { UsesFoo } from \"./mod\";\n");

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mod_idx = entry.resolve_internal_specifier("./mod").expect("mod should resolve");
        let mod_module = &scan_result.modules[mod_idx];
        let foo_symbol = mod_module
            .scoping
            .get_root_binding(oxc_span::Ident::from("Foo"))
            .expect("Foo symbol should exist");
        let foo_kinds = plan
            .symbol_kinds
            .get(&mod_idx)
            .and_then(|entry| entry.as_ref())
            .and_then(|kinds| kinds.get(&foo_symbol))
            .copied()
            .expect("Foo should have recorded needed kinds");

        assert_eq!(foo_kinds, super::NeededKindFlags::TYPE);
    }

    #[test]
    fn entry_declaration_references_refine_named_imports() {
        let project = TempProject::new("entry_decl_refs_refine_named_imports");
        project.write_file(
            "mod.d.ts",
            "export interface Shared { value: string }\nexport declare const shared: Shared;\n",
        );
        project.write_file(
            "index.d.ts",
            "import { Shared, shared } from \"./mod\";\nexport interface Input extends Shared { extra: string }\n",
        );

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mod_idx = entry.resolve_internal_specifier("./mod").expect("mod should resolve");

        let mod_module = &scan_result.modules[mod_idx];
        assert!(plan.contains_symbol(mod_module, "Shared"));
        assert!(!plan.contains_symbol(mod_module, "shared"));
    }

    #[test]
    fn merged_needed_names_keep_secondary_entries_whole() {
        let project = TempProject::new("merged_needed_names_keep_entries");
        project.write_file(
            "a.d.ts",
            "export interface A { value: B }\nimport type { B } from \"./b\";\n",
        );
        project.write_file(
            "b.d.ts",
            "export interface B { value: string }\nexport declare const x: B;\n",
        );

        let scan_result = project.scan_many(&["a.d.ts", "b.d.ts"]);
        let merged = super::build_merged_needed_names(&scan_result);
        let a_idx = scan_result
            .entry_indices
            .iter()
            .copied()
            .find(|&idx| scan_result.modules[idx].path.ends_with("a.d.ts"))
            .expect("a entry should exist");
        let b_idx =
            scan_result.modules[a_idx].resolve_internal_specifier("./b").expect("b should resolve");

        assert!(matches!(merged.map.get(&b_idx), Some(None)));
    }

    #[test]
    fn transitive_default_import_dependencies_refine_partially_needed_modules() {
        let project = TempProject::new("transitive_default_import_dependencies");
        project.write_file(
            "leaf.d.ts",
            "declare class Foo { value: string }\nexport default Foo;\nexport interface Marker { ok: true }\n",
        );
        project.write_file(
            "mid.d.ts",
            "export { Marker } from \"./leaf\";\nimport Foo from \"./leaf\";\nexport interface UsesFoo { value: Foo }\n",
        );
        project.write_file("index.d.ts", "export { Marker, UsesFoo } from \"./mid\";\n");

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let mid_idx = entry.resolve_internal_specifier("./mid").expect("mid should resolve");
        let leaf_idx = scan_result.modules[mid_idx]
            .resolve_internal_specifier("./leaf")
            .expect("leaf should resolve");

        let leaf_module = &scan_result.modules[leaf_idx];
        assert!(plan.contains_symbol(leaf_module, "Marker"));
        assert!(plan.contains_symbol(leaf_module, "Foo"));

        let foo_reasons =
            plan.reasons_for(leaf_idx, "Foo").expect("Foo should have reasons in leaf");
        assert!(foo_reasons.contains(&NeededReason::CrossModuleImportDependency));
    }

    #[test]
    fn entry_declaration_imports_refine_modules_that_become_partial_later() {
        let project = TempProject::new("entry_decl_imports_refine_later_partial_modules");
        project.write_file(
            "leaf.d.ts",
            "export interface A { a: string }\nexport interface B { b: string }\n",
        );
        project.write_file(
            "mid.d.ts",
            "import { B } from \"./leaf\";\nexport declare function g(value: B): void;\n",
        );
        project.write_file(
            "index.d.ts",
            "import { A } from \"./leaf\";\nexport declare function f(value: A): void;\nexport { g } from \"./mid\";\n",
        );

        let scan_result = project.scan("index.d.ts");
        let entry = &scan_result.modules[scan_result.entry_idx];
        let plan = build_needed_names(entry, &scan_result);
        let leaf_idx = entry.resolve_internal_specifier("./leaf").expect("leaf should resolve");

        let leaf_module = &scan_result.modules[leaf_idx];
        assert!(plan.contains_symbol(leaf_module, "A"));
        assert!(plan.contains_symbol(leaf_module, "B"));

        let a_reasons = plan.reasons_for(leaf_idx, "A").expect("A should have reasons in leaf");
        assert!(a_reasons.contains(&NeededReason::CrossModuleImportDependency));
    }

    #[test]
    fn link_warnings_include_rename_fallback_code() {
        let project = TempProject::new("rename_fallback_warning_code");
        project.write_file("index.d.ts", "export interface Foo { value: string }\n");

        let scan_result = project.scan("index.d.ts");
        let mut rename_plan = RenamePlan::default();
        rename_plan
            .fallback_name_renames
            .insert((scan_result.entry_idx, "Foo".to_string()), "Foo$1".to_string());
        let warnings = collect_link_warnings(&rename_plan, &scan_result);
        let text = warnings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");

        assert!(
            text.contains("typack/rename-fallback"),
            "expected structured rename-fallback warning, got:\n{text}"
        );
    }

    #[test]
    fn circular_dependency_emits_warning() {
        let project = TempProject::new("circular_dependency");
        // a.d.ts imports from b.d.ts and b.d.ts imports from a.d.ts
        project
            .write_file("a.d.ts", "import { B } from \"./b\";\nexport interface A { value: B }\n");
        project
            .write_file("b.d.ts", "import { A } from \"./a\";\nexport interface B { value: A }\n");
        project
            .write_file("index.d.ts", "export { A } from \"./a\";\nexport { B } from \"./b\";\n");

        let scan_result = project.scan("index.d.ts");
        let has_cycle_warning =
            scan_result.warnings.iter().any(|w| w.to_string().contains("Circular dependency"));
        assert!(
            has_cycle_warning,
            "expected circular dependency warning, got: {:?}",
            scan_result.warnings.iter().map(ToString::to_string).collect::<Vec<_>>()
        );
    }
}
