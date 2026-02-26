mod common;

use typack::{TypackBundler, TypackOptions};

use common::TempProject;

fn bundle(
    project: &TempProject,
    entry: &str,
    external: Vec<String>,
) -> Result<String, Vec<oxc_diagnostics::OxcDiagnostic>> {
    TypackBundler::bundle(&TypackOptions {
        input: vec![project.root.join(entry).to_string_lossy().to_string()],
        external,
        cwd: project.root.clone(),
        ..Default::default()
    })
    .map(|result| result.code)
}

#[test]
fn unresolved_relative_specifier_is_fatal() {
    let project = TempProject::new("unresolved_relative");
    project.write_file("index.d.ts", "export { Foo } from \"./missing\";\n");

    let diagnostics = bundle(&project, "index.d.ts", Vec::new())
        .expect_err("bundle should fail for unresolved relative specifier");

    let messages = diagnostics.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    assert!(
        messages.contains("Cannot resolve relative specifier \"./missing\""),
        "expected unresolved-relative diagnostic, got:\n{messages}"
    );
    assert!(messages.contains("index.d.ts"), "expected importer path in diagnostic:\n{messages}");
}

#[test]
fn external_side_effect_import_is_preserved() {
    let project = TempProject::new("external_side_effect");
    project.write_file(
        "index.d.ts",
        "import \"pkg-side-effect\";\nexport interface Foo { value: string; }\n",
    );

    let output = bundle(&project, "index.d.ts", Vec::new())
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(
        output.contains("import \"pkg-side-effect\";"),
        "expected side-effect import to be preserved, output:\n{output}"
    );
}

#[test]
fn force_external_precedence_wins_over_resolvable_relative() {
    let project = TempProject::new("force_external");
    project.write_file("lib.d.ts", "export interface Foo { x: number; }\n");
    project.write_file("index.d.ts", "export { Foo } from \"./lib\";\n");

    let output = bundle(&project, "index.d.ts", vec!["./lib".to_string()])
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(
        output.contains("import { Foo } from \"./lib\";"),
        "expected forced-external import to be emitted, output:\n{output}"
    );
    assert!(
        !output.contains("interface Foo"),
        "expected internal module to be excluded after force-external, output:\n{output}"
    );
}

#[test]
fn unresolved_bare_specifier_is_externalized() {
    let project = TempProject::new("bare_unresolved");
    project.write_file(
        "index.d.ts",
        "import { ExternalType } from \"nonexistent-package\";\nexport interface Foo extends ExternalType {}\n",
    );

    let output = bundle(&project, "index.d.ts", Vec::new())
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(
        output.contains("import { ExternalType } from \"nonexistent-package\";"),
        "expected unresolved bare specifier to remain external import, output:\n{output}"
    );
}
