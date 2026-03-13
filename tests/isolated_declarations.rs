mod common;

use typack::{TypackBundler, TypackOptions};

use common::TempProject;

fn bundle(
    project: &TempProject,
    entry: &str,
) -> Result<String, Vec<oxc_diagnostics::OxcDiagnostic>> {
    TypackBundler::bundle(&TypackOptions {
        input: vec![project.root.join(entry).to_string_lossy().to_string()],
        cwd: project.root.clone(),
        ..Default::default()
    })
    .map(|result| result.output[0].code.clone())
}

fn bundle_with_warnings(
    project: &TempProject,
    entry: &str,
) -> Result<(String, Vec<String>), Vec<oxc_diagnostics::OxcDiagnostic>> {
    TypackBundler::bundle(&TypackOptions {
        input: vec![project.root.join(entry).to_string_lossy().to_string()],
        cwd: project.root.clone(),
        ..Default::default()
    })
    .map(|result| {
        let warnings = result.warnings.iter().map(ToString::to_string).collect();
        (result.output[0].code.clone(), warnings)
    })
}

#[test]
fn ts_entry_point_generates_declarations() {
    let project = TempProject::new("ts_entry");
    project.write_file(
        "index.ts",
        r"
export interface Foo {
    name: string;
}

export function greet(name: string): string {
    return `Hello, ${name}!`;
}

export type Bar = Foo & { age: number };
",
    );

    let output = bundle(&project, "index.ts")
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(output.contains("interface Foo"), "expected Foo interface in output:\n{output}");
    assert!(
        output.contains("function greet"),
        "expected greet function declaration in output:\n{output}"
    );
    assert!(output.contains("type Bar"), "expected Bar type alias in output:\n{output}");
}

#[test]
fn ts_entry_with_dts_dependency() {
    let project = TempProject::new("ts_dts_mixed");
    project.write_file("types.d.ts", "export interface Base { id: number; }\n");
    project.write_file(
        "index.d.ts",
        r#"import { Base } from "./types";
export interface Extended extends Base {
    name: string;
}
export { Base };
"#,
    );

    let output = bundle(&project, "index.d.ts")
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(output.contains("interface Base"), "expected Base in output:\n{output}");
    assert!(output.contains("interface Extended"), "expected Extended in output:\n{output}");
}

#[test]
fn dts_entry_importing_ts_file() {
    let project = TempProject::new("dts_imports_ts");
    project.write_file(
        "lib.ts",
        r"
export interface Config {
    debug: boolean;
    port: number;
}

export function createConfig(): Config {
    return { debug: false, port: 3000 };
}
",
    );
    project.write_file(
        "index.d.ts",
        r#"import { Config } from "./lib";
export { Config };
export declare function init(config: Config): void;
"#,
    );

    let output = bundle(&project, "index.d.ts")
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(output.contains("interface Config"), "expected Config interface in output:\n{output}");
    assert!(output.contains("function init"), "expected init function in output:\n{output}");
}

#[test]
fn ts_isolated_declarations_error_surfaces() {
    let project = TempProject::new("ts_id_error");
    // Missing return type annotation — IsolatedDeclarations requires explicit annotations
    project.write_file(
        "index.ts",
        r"
export function compute(x: number) {
    return x * 2;
}
",
    );

    let diagnostics = bundle(&project, "index.ts")
        .expect_err("bundle should fail for missing return type annotation");

    let messages = diagnostics.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
    assert!(!messages.is_empty(), "expected IsolatedDeclarations error about missing return type");
}

#[test]
fn ts_entry_warns_without_isolated_declarations_in_tsconfig() {
    let project = TempProject::new("ts_no_tsconfig_warn");
    // No tsconfig.json in project
    project.write_file(
        "index.ts",
        r"
export interface Simple {
    value: string;
}
",
    );

    let (output, warnings) = bundle_with_warnings(&project, "index.ts")
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(output.contains("interface Simple"), "expected Simple in output:\n{output}");
    assert!(
        warnings.iter().any(|w| w.contains("isolatedDeclarations")),
        "expected warning about isolatedDeclarations not being enabled, warnings: {warnings:?}"
    );
}

#[test]
fn ts_entry_no_warn_with_isolated_declarations_tsconfig() {
    let project = TempProject::new("ts_with_tsconfig");
    project
        .write_file("tsconfig.json", r#"{ "compilerOptions": { "isolatedDeclarations": true } }"#);
    project.write_file(
        "index.ts",
        r"
export interface WithConfig {
    ready: boolean;
}
",
    );

    let (output, warnings) = bundle_with_warnings(&project, "index.ts")
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(output.contains("interface WithConfig"), "expected WithConfig in output:\n{output}");
    assert!(
        !warnings.iter().any(|w| w.contains("isolatedDeclarations")),
        "should not warn when isolatedDeclarations is enabled, warnings: {warnings:?}"
    );
}

#[test]
fn ts_strip_internal_excludes_internal_declarations() {
    let project = TempProject::new("ts_strip_internal");
    project.write_file(
        "tsconfig.json",
        r#"{ "compilerOptions": { "isolatedDeclarations": true, "stripInternal": true } }"#,
    );
    project.write_file(
        "index.ts",
        r"
export interface PublicApi {
    name: string;
}

/** @internal */
export interface InternalHelper {
    secret: string;
}
",
    );

    let output = bundle(&project, "index.ts")
        .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    assert!(output.contains("interface PublicApi"), "expected PublicApi in output:\n{output}");
    assert!(
        !output.contains("InternalHelper"),
        "expected InternalHelper to be stripped, output:\n{output}"
    );
}
