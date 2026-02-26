mod common;

use cow_utils::CowUtils;
use typack::{TypackBundler, TypackOptions};

use common::TempProject;

fn bundle(project: &TempProject, entry: &str) -> String {
    TypackBundler::bundle(&TypackOptions {
        input: vec![project.root.join(entry).to_string_lossy().to_string()],
        cwd: project.root.clone(),
        ..Default::default()
    })
    .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"))
    .code
}

fn normalize_newlines(text: &str) -> String {
    text.cow_replace("\r\n", "\n").into_owned()
}

#[test]
fn keeps_transitive_root_symbol_dependencies() {
    let project = TempProject::new("transitive_root_symbol_deps");
    project.write_file("mod.d.ts", "type A = { a: number };\ntype B = A;\nexport type C = B;\n");
    project.write_file("index.d.ts", "export { C } from \"./mod\";\n");

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    assert!(output.contains("type A = {"), "expected transitive dependency A\n{output}");
    assert!(output.contains("type B = A;"), "expected transitive dependency B\n{output}");
    assert!(output.contains("type C = B;"), "expected exported type C\n{output}");
}

#[test]
fn does_not_infer_dependencies_from_substring_matches() {
    let project = TempProject::new("no_substring_false_positive");
    project.write_file(
        "mod.d.ts",
        "type Foo = number;\ntype FooBar = number;\nexport type UsesFooBar = FooBar;\n",
    );
    project.write_file("index.d.ts", "export { UsesFooBar } from \"./mod\";\n");

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    assert!(output.contains("type FooBar = number;"), "expected FooBar declaration\n{output}");
    assert!(
        output.contains("type UsesFooBar = FooBar;"),
        "expected UsesFooBar declaration\n{output}"
    );
    assert!(
        !output.contains("type Foo = number;"),
        "did not expect substring-inferred Foo\n{output}"
    );
}

#[test]
fn drops_value_declaration_from_merged_name_when_only_type_is_needed() {
    let project = TempProject::new("drops_value_side_of_merged_name");
    project.write_file(
        "mod.d.ts",
        "export interface Foo { value: string }\nexport declare const Foo: { runtime: true };\nexport type UsesFoo = Foo;\n",
    );
    project.write_file("index.d.ts", "export { UsesFoo } from \"./mod\";\n");

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    assert!(output.contains("interface Foo {"), "expected interface side of Foo\n{output}");
    assert!(output.contains("type UsesFoo = Foo;"), "expected UsesFoo declaration\n{output}");
    assert!(
        !output.contains("declare const Foo:"),
        "did not expect value-side declaration for Foo\n{output}"
    );
}

#[test]
fn keeps_interface_heritage_dependencies_in_partially_needed_modules() {
    let project = TempProject::new("interface_heritage_dependency");
    project.write_file(
        "mod.d.ts",
        "interface Base { base: string }\ninterface Derived extends Base { derived: string }\nexport { type Derived };\n",
    );
    project.write_file("index.d.ts", "export { type Derived } from \"./mod\";\n");

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    assert!(output.contains("interface Base {"), "expected Base dependency\n{output}");
    assert!(output.contains("interface Derived extends Base {"), "expected Derived\n{output}");
}

#[test]
fn keeps_local_export_specifier_dependencies_inside_namespaces() {
    let project = TempProject::new("namespace_export_specifier_dependency");
    project.write_file(
        "mod.d.ts",
        "declare const Referenced: { value: string };\ndeclare namespace Ns {\n  export { Referenced as Ref };\n}\nexport { Ns };\n",
    );
    project.write_file("index.d.ts", "export { Ns } from \"./mod\";\n");

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    assert!(
        output.contains("declare const Referenced: {"),
        "expected Referenced dependency\n{output}"
    );
    assert!(output.contains("declare namespace Ns {"), "expected namespace export\n{output}");
}
