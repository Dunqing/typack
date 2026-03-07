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
    .output
    .into_iter()
    .next()
    .expect("should have at least one output")
    .code
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "expected output to contain `{needle}`\noutput:\n{haystack}"
    );
}

fn normalize_newlines(text: &str) -> String {
    text.cow_replace("\r\n", "\n").into_owned()
}

#[test]
fn type_value_collisions_get_suffixed() {
    let project = TempProject::new("split_type_value");
    project.write_file("a.d.ts", "export interface Foo { v: string; }\n");
    project.write_file("b.d.ts", "export const Foo: number;\n");
    project.write_file(
        "index.d.ts",
        "export { Foo as TFoo } from \"./a\";\nexport { Foo as VFoo } from \"./b\";\n",
    );

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    // With a single name set, type-only and value-only names from different
    // modules collide just like same-space names.
    assert_contains(&output, "interface Foo");
    assert_contains(&output, "declare const Foo$1: number;");
    assert_contains(&output, "Foo as TFoo");
    assert_contains(&output, "Foo$1 as VFoo");
}

#[test]
fn same_space_collisions_still_suffix_deterministically() {
    let project = TempProject::new("same_space");
    project.write_file("a.d.ts", "export const Foo: number;\n");
    project.write_file("b.d.ts", "export const Foo: number;\n");
    project.write_file(
        "index.d.ts",
        "export { Foo as A } from \"./a\";\nexport { Foo as B } from \"./b\";\n",
    );

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    assert_contains(&output, "declare const Foo: number;");
    assert_contains(&output, "declare const Foo$1: number;");
    assert_contains(&output, "Foo as A");
    assert_contains(&output, "Foo$1 as B");
}

#[test]
fn class_collides_with_type_space_names() {
    let project = TempProject::new("class_type_collision");
    project.write_file("a.d.ts", "export declare class Foo {}\n");
    project.write_file("b.d.ts", "export interface Foo { x: number; }\n");
    project.write_file(
        "index.d.ts",
        "export { Foo as IFoo } from \"./b\";\nexport { Foo as CFoo } from \"./a\";\n",
    );

    let output = normalize_newlines(&bundle(&project, "index.d.ts"));
    assert_contains(&output, "interface Foo");
    assert_contains(&output, "declare class Foo$1");
    assert_contains(&output, "Foo as IFoo");
    assert_contains(&output, "Foo$1 as CFoo");
}
