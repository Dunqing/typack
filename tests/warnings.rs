mod common;

use typack::{TypackBundler, TypackOptions};

use common::TempProject;

fn bundle_warnings(project: &TempProject, entry: &str, external: Vec<String>) -> String {
    let result = TypackBundler::bundle(&TypackOptions {
        input: vec![project.root.join(entry).to_string_lossy().to_string()],
        external,
        cwd: project.root.clone(),
        ..Default::default()
    })
    .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));
    result.warnings.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
}

#[test]
fn warns_for_unresolved_bare_externalization() {
    let project = TempProject::new("warn_unresolved_bare");
    project.write_file(
        "index.d.ts",
        "import { ExternalType } from \"nonexistent-package\";\nexport interface Foo extends ExternalType {}\n",
    );

    let warnings = bundle_warnings(&project, "index.d.ts", Vec::new());
    assert!(
        warnings.contains("typack/externalized-bare-unresolved"),
        "expected unresolved bare warning, got:\n{warnings}"
    );
}

#[test]
fn warns_for_forced_external_override() {
    let project = TempProject::new("warn_forced_external");
    project.write_file("lib.d.ts", "export interface Foo { x: number; }\n");
    project.write_file("index.d.ts", "export { Foo } from \"./lib\";\n");

    let warnings = bundle_warnings(&project, "index.d.ts", vec!["./lib".to_string()]);
    assert!(
        warnings.contains("typack/forced-external-override"),
        "expected forced-external warning, got:\n{warnings}"
    );
}

#[test]
fn warns_for_namespace_name_deconflict() {
    let project = TempProject::new("warn_namespace_deconflict");
    project.write_file(
        "index.d.ts",
        "declare const pkg: number;\nexport type T = import(\"pkg\").Type;\nexport { pkg };\n",
    );

    let warnings = bundle_warnings(&project, "index.d.ts", Vec::new());
    assert!(
        warnings.contains("typack/namespace-name-deconflict"),
        "expected namespace deconflict warning, got:\n{warnings}"
    );
}
