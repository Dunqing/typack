#![cfg(feature = "cli")]

mod common;

use std::fs;
use std::process::Command;

use common::TempProject;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_typack"))
}

fn fixture_entry(name: &str) -> String {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = crate_dir.join("tests");

    // Check own fixtures first, then upstream (rolldown-plugin-dts)
    let own = tests_dir.join("fixtures").join(name).join("index.d.ts");
    if own.exists() {
        return own.to_string_lossy().to_string();
    }
    tests_dir
        .join("rolldown-plugin-dts")
        .join(name)
        .join("index.d.ts")
        .to_string_lossy()
        .to_string()
}

#[test]
fn no_args_exits_with_error() {
    let output = bin().output().expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ENTRY") || stderr.contains("entry") || stderr.contains("required"),
        "expected usage hint in stderr, got:\n{stderr}"
    );
}

#[test]
fn help_flag_exits_successfully() {
    let output = bin().arg("--help").output().expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--external"), "expected --external in help output");
    assert!(stdout.contains("--sourcemap"), "expected --sourcemap in help output");
    assert!(stdout.contains("--outfile"), "expected --outfile in help output");
    assert!(stdout.contains("--cwd"), "expected --cwd in help output");
    assert!(stdout.contains("--cjs-default"), "expected --cjs-default in help output");
    assert!(stdout.contains("ENTRY"), "expected ENTRY in help output");
}

#[test]
fn basic_entry_prints_to_stdout() {
    let output = bin().arg(fixture_entry("basic")).output().expect("failed to run binary");
    assert!(output.status.success(), "exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("export { Cls, bar, fn, foo };"),
        "expected exports in output:\n{stdout}"
    );
}

#[test]
fn missing_entry_file_exits_with_error() {
    let output = bin().arg("nonexistent/file.d.ts").output().expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot find entry file"),
        "expected error about missing file, got:\n{stderr}"
    );
}

#[test]
fn outfile_writes_to_disk() {
    let project = TempProject::new("cli_outfile");
    let outfile = project.root.join("output.d.ts");

    let output = bin()
        .arg("-o")
        .arg(&outfile)
        .arg(fixture_entry("basic"))
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let content = fs::read_to_string(&outfile).expect("outfile should exist");
    assert!(
        content.contains("export { Cls, bar, fn, foo };"),
        "expected exports in file:\n{content}"
    );

    // stdout should be empty when writing to file
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "stdout should be empty with --outfile, got:\n{stdout}");
}

#[test]
fn sourcemap_with_outfile_writes_map() {
    let project = TempProject::new("cli_sourcemap");
    let outfile = project.root.join("output.d.ts");
    let map_file = project.root.join("output.d.ts.map");

    let output = bin()
        .arg("--sourcemap")
        .arg("-o")
        .arg(&outfile)
        .arg(fixture_entry("basic"))
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(outfile.exists(), "output file should exist");
    assert!(map_file.exists(), "source map file should exist");

    let map_content = fs::read_to_string(&map_file).expect("map file should be readable");
    assert!(map_content.contains("\"mappings\""), "source map should contain mappings");
    assert!(map_content.contains("\"sources\""), "source map should contain sources");
}

#[test]
fn sourcemap_without_outfile_warns() {
    let output = bin()
        .arg("--sourcemap")
        .arg(fixture_entry("basic"))
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--sourcemap without --outfile")
            || stderr.contains("--sourcemap without --outfile/--outdir"),
        "expected warning about sourcemap without outfile, got:\n{stderr}"
    );
}

#[test]
fn external_flag_keeps_imports() {
    let project = TempProject::new("cli_external");
    project.write_file("lib.d.ts", "export interface Foo { x: number; }\n");
    project.write_file("index.d.ts", "export { Foo } from \"./lib\";\n");

    let entry = project.root.join("index.d.ts").to_string_lossy().to_string();
    let output = bin()
        .arg("--external")
        .arg("./lib")
        .arg("--cwd")
        .arg(&project.root)
        .arg(&entry)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("import { Foo } from \"./lib\";"),
        "expected external import preserved, got:\n{stdout}"
    );
    assert!(
        !stdout.contains("interface Foo"),
        "expected inlined declaration to be excluded, got:\n{stdout}"
    );
}

#[test]
fn multiple_external_flags() {
    let project = TempProject::new("cli_multi_external");
    project.write_file(
        "index.d.ts",
        "import { A } from \"pkg-a\";\nimport { B } from \"pkg-b\";\nexport interface Foo extends A, B {}\n",
    );

    let entry = project.root.join("index.d.ts").to_string_lossy().to_string();
    let output = bin()
        .arg("--external")
        .arg("pkg-a")
        .arg("--external")
        .arg("pkg-b")
        .arg(&entry)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"pkg-a\""), "expected pkg-a import, got:\n{stdout}");
    assert!(stdout.contains("\"pkg-b\""), "expected pkg-b import, got:\n{stdout}");
}

#[test]
fn cwd_flag_resolves_entries() {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = crate_dir.join("tests").join("rolldown-plugin-dts").join("basic");

    let output = bin()
        .arg("--cwd")
        .arg(&fixture_dir)
        .arg(fixture_dir.join("index.d.ts").to_string_lossy().as_ref())
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("export { Cls, bar, fn, foo };"), "expected exports, got:\n{stdout}");
}

#[test]
fn invalid_cwd_exits_with_error() {
    let output = bin()
        .arg("--cwd")
        .arg("/nonexistent/directory")
        .arg("some-file.d.ts")
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot resolve --cwd"), "expected --cwd error, got:\n{stderr}");
}

#[test]
fn outfile_creates_parent_directories() {
    let project = TempProject::new("cli_nested_outfile");
    let outfile = project.root.join("nested").join("dir").join("output.d.ts");

    let output = bin()
        .arg("-o")
        .arg(&outfile)
        .arg(fixture_entry("basic"))
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(outfile.exists(), "nested output file should be created");
}

#[test]
fn bundle_error_exits_with_failure() {
    let project = TempProject::new("cli_bundle_error");
    project.write_file("index.d.ts", "export { Foo } from \"./missing\";\n");

    let entry = project.root.join("index.d.ts").to_string_lossy().to_string();
    let output = bin().arg(&entry).output().expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:"), "expected error diagnostic on stderr, got:\n{stderr}");
}

#[test]
fn outdir_single_entry() {
    let project = TempProject::new("cli_outdir_single");
    project.write_file("index.d.ts", "export declare function hello(): void;\n");
    let outdir = project.root.join("dist");

    let entry = project.root.join("index.d.ts").to_string_lossy().to_string();
    let output = bin()
        .arg("--outdir")
        .arg(&outdir)
        .arg("--cwd")
        .arg(&project.root)
        .arg(&entry)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let out_path = outdir.join("index.d.ts");
    assert!(out_path.exists(), "output file should exist at {}", out_path.display());
    let content = fs::read_to_string(&out_path).expect("output should be readable");
    assert!(content.contains("hello"), "expected hello in output:\n{content}");

    // stdout should be empty when writing to outdir
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "stdout should be empty with --outdir, got:\n{stdout}");
}

#[test]
fn outdir_preserves_relative_directory_structure() {
    let project = TempProject::new("cli_outdir_relative");
    let outdir = project.root.join("dist");

    // Create two entries in different subdirectories with the same filename
    project.write_file("src/foo/index.d.ts", "export declare function fooFn(): void;\n");
    project.write_file("src/bar/index.d.ts", "export declare function barFn(): void;\n");

    let entry_foo = project.root.join("src/foo/index.d.ts").to_string_lossy().to_string();
    let entry_bar = project.root.join("src/bar/index.d.ts").to_string_lossy().to_string();

    let output = bin()
        .arg("--outdir")
        .arg(&outdir)
        .arg("--cwd")
        .arg(&project.root)
        .arg(&entry_foo)
        .arg(&entry_bar)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Both files should exist in their respective subdirectories
    let foo_out = outdir.join("src/foo/index.d.ts");
    let bar_out = outdir.join("src/bar/index.d.ts");
    assert!(foo_out.exists(), "foo output should exist at {}", foo_out.display());
    assert!(bar_out.exists(), "bar output should exist at {}", bar_out.display());

    let foo_content = fs::read_to_string(&foo_out).expect("foo output should be readable");
    let bar_content = fs::read_to_string(&bar_out).expect("bar output should be readable");
    assert!(foo_content.contains("fooFn"), "foo output should contain fooFn:\n{foo_content}");
    assert!(bar_content.contains("barFn"), "bar output should contain barFn:\n{bar_content}");
}

#[test]
fn outdir_entry_outside_cwd_falls_back_to_filename() {
    let project = TempProject::new("cli_outdir_outside_cwd");
    let other = TempProject::new("cli_outdir_outside_other");
    other.write_file("lib.d.ts", "export declare const x: number;\n");

    let outdir = project.root.join("dist");
    let entry = other.root.join("lib.d.ts").to_string_lossy().to_string();

    let output = bin()
        .arg("--outdir")
        .arg(&outdir)
        .arg("--cwd")
        .arg(&project.root)
        .arg(&entry)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Should fall back to just the filename, not the full absolute path
    let out_path = outdir.join("lib.d.ts");
    assert!(out_path.exists(), "output should be at {}", out_path.display());
}

#[test]
fn outdir_with_sourcemap() {
    let project = TempProject::new("cli_outdir_sourcemap");
    project.write_file("index.d.ts", "export declare function hello(): void;\n");
    let outdir = project.root.join("dist");

    let entry = project.root.join("index.d.ts").to_string_lossy().to_string();
    let output = bin()
        .arg("--outdir")
        .arg(&outdir)
        .arg("--sourcemap")
        .arg("--cwd")
        .arg(&project.root)
        .arg(&entry)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let out_path = outdir.join("index.d.ts");
    let map_path = outdir.join("index.d.ts.map");
    assert!(out_path.exists(), "output file should exist");
    assert!(map_path.exists(), "source map file should exist");

    let map_content = fs::read_to_string(&map_path).expect("map file should be readable");
    assert!(map_content.contains("\"mappings\""), "source map should contain mappings");
}

#[test]
fn outdir_multi_entry_with_sourcemap() {
    let project = TempProject::new("cli_outdir_multi_sourcemap");
    project.write_file("a.d.ts", "export declare const a: number;\n");
    project.write_file("b.d.ts", "export declare const b: string;\n");
    let outdir = project.root.join("dist");

    let entry_a = project.root.join("a.d.ts").to_string_lossy().to_string();
    let entry_b = project.root.join("b.d.ts").to_string_lossy().to_string();
    let output = bin()
        .arg("--outdir")
        .arg(&outdir)
        .arg("--sourcemap")
        .arg("--cwd")
        .arg(&project.root)
        .arg(&entry_a)
        .arg(&entry_b)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Each entry should have its own output and source map
    for name in &["a", "b"] {
        let out_path = outdir.join(format!("{name}.d.ts"));
        let map_path = outdir.join(format!("{name}.d.ts.map"));
        assert!(out_path.exists(), "{name}.d.ts should exist");
        assert!(map_path.exists(), "{name}.d.ts.map should exist");

        let map_content = fs::read_to_string(&map_path).expect("map file should be readable");
        assert!(map_content.contains("\"mappings\""), "{name} source map should contain mappings");
        assert!(map_content.contains("\"sources\""), "{name} source map should contain sources");
    }
}

#[test]
fn outfile_and_outdir_conflict() {
    let project = TempProject::new("cli_outfile_outdir_conflict");

    let output = bin()
        .arg("-o")
        .arg(project.root.join("out.d.ts"))
        .arg("--outdir")
        .arg(project.root.join("dist"))
        .arg(fixture_entry("basic"))
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--outfile and --outdir cannot be used together"),
        "expected conflict error, got:\n{stderr}"
    );
}

#[test]
fn outfile_with_multiple_entries_errors() {
    let project = TempProject::new("cli_outfile_multi");
    project.write_file("a.d.ts", "export declare const a: number;\n");
    project.write_file("b.d.ts", "export declare const b: number;\n");

    let entry_a = project.root.join("a.d.ts").to_string_lossy().to_string();
    let entry_b = project.root.join("b.d.ts").to_string_lossy().to_string();

    let output = bin()
        .arg("-o")
        .arg(project.root.join("out.d.ts"))
        .arg(&entry_a)
        .arg(&entry_b)
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--outfile cannot be used with multiple entries"),
        "expected multi-entry error, got:\n{stderr}"
    );
}

#[test]
fn warnings_printed_to_stderr() {
    let project = TempProject::new("cli_warnings");
    project.write_file(
        "index.d.ts",
        "import { ExternalType } from \"nonexistent-package\";\nexport interface Foo extends ExternalType {}\n",
    );

    let entry = project.root.join("index.d.ts").to_string_lossy().to_string();
    let output = bin().arg(&entry).output().expect("failed to run binary");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("warning:"), "expected warnings on stderr, got:\n{stderr}");
}

#[test]
fn ts_entry_bundles_declarations() {
    let project = TempProject::new("cli_ts_entry");
    project.write_file(
        "index.ts",
        "export interface Greeting { message: string; }\nexport function greet(name: string): string { return `Hello, ${name}`; }\n",
    );

    let entry = project.root.join("index.ts").to_string_lossy().to_string();
    let output = bin().arg(&entry).output().expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("interface Greeting"),
        "expected Greeting interface in output:\n{stdout}"
    );
    assert!(stdout.contains("function greet"), "expected greet declaration in output:\n{stdout}");
}

#[test]
fn tsconfig_flag_picks_up_strip_internal() {
    let project = TempProject::new("cli_tsconfig_flag");
    project.write_file(
        "tsconfig.build.json",
        r#"{ "compilerOptions": { "isolatedDeclarations": true, "stripInternal": true } }"#,
    );
    project.write_file(
        "index.ts",
        "export interface Public { name: string; }\n/** @internal */\nexport interface Private { secret: string; }\n",
    );

    let entry = project.root.join("index.ts").to_string_lossy().to_string();
    let output = bin()
        .arg("--tsconfig")
        .arg("tsconfig.build.json")
        .arg("--cwd")
        .arg(&project.root)
        .arg(&entry)
        .output()
        .expect("failed to run binary");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("interface Public"), "expected Public in output:\n{stdout}");
    assert!(!stdout.contains("Private"), "expected Private to be stripped:\n{stdout}");
}
