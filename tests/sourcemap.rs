use std::path::PathBuf;

use typack::{TypackBundler, TypackOptions};

mod common;

fn bundle_fixture_with_sourcemap(fixture: &str) -> (String, oxc_sourcemap::SourceMap) {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = crate_dir.join("tests");
    // Check own fixtures first, then upstream (rolldown-plugin-dts)
    let own = tests_dir.join("fixtures").join(fixture).join("index.d.ts");
    let entry = if own.exists() {
        own
    } else {
        tests_dir.join("rolldown-plugin-dts").join(fixture).join("index.d.ts")
    };
    let result = TypackBundler::bundle(&TypackOptions {
        input: vec![entry.to_string_lossy().to_string()],
        cwd: crate_dir,
        sourcemap: true,
        ..Default::default()
    })
    .unwrap_or_else(|diagnostics| panic!("bundle failed: {diagnostics:?}"));

    let output = result.outputs.into_iter().next().expect("should have at least one output");
    let map = output.map.expect("sourcemap should be present when `sourcemap: true`");
    (output.code, map)
}

fn find_generated_position(code: &str, needle: &str) -> (u32, u32) {
    for (line_index, line) in code.lines().enumerate() {
        if let Some(col) = line.find(needle) {
            return (
                u32::try_from(line_index).expect("line index should fit in u32"),
                u32::try_from(col).expect("column should fit in u32"),
            );
        }
    }
    panic!("did not find `{needle}` in generated bundle");
}

#[test]
fn sourcemap_includes_sources_and_contents() {
    let (_code, map) = bundle_fixture_with_sourcemap("import-referenced-interface");

    let sources: Vec<&str> = map.get_sources().map(std::convert::AsRef::as_ref).collect();
    assert!(!sources.is_empty(), "expected sourcemap sources to be populated");
    assert!(sources.iter().any(|source| source.ends_with("bar.d.ts")));
    assert!(sources.iter().any(|source| source.ends_with("index.d.ts")));

    // Sources should be relative paths (not absolute)
    for source in &sources {
        assert!(!source.starts_with('/'), "sourcemap source should be relative, got: {source}");
    }

    let source_contents: Vec<Option<&str>> =
        map.get_source_contents().map(|content| content.map(std::convert::AsRef::as_ref)).collect();
    assert!(
        source_contents.iter().all(Option::is_some),
        "expected all sourcesContent entries to be present"
    );
}

#[test]
fn sourcemap_maps_ast_emitted_lines_and_skips_synthetic_lines() {
    let (code, map) = bundle_fixture_with_sourcemap("import-referenced-interface");
    let lines: Vec<&str> = code.lines().collect();

    let (bar_line, _bar_col) = find_generated_position(&code, "interface Bar");
    let (foo_line, _foo_col) = find_generated_position(&code, "interface Foo");
    let (region_line, _region_col) = find_generated_position(&code, "//#region");

    let mut bar_line_is_mapped_to_bar = false;
    let mut foo_line_is_mapped_to_index = false;
    let mut region_line_has_mapping = false;

    for token in map.get_source_view_tokens() {
        let generated_line = token.get_dst_line() as usize;
        if generated_line == region_line as usize {
            region_line_has_mapping = true;
        }
        let Some(source) = token.get_source().map(std::convert::AsRef::as_ref) else {
            continue;
        };
        if generated_line == bar_line as usize && source.ends_with("bar.d.ts") {
            bar_line_is_mapped_to_bar = true;
        }
        if generated_line == foo_line as usize && source.ends_with("index.d.ts") {
            foo_line_is_mapped_to_index = true;
        }
    }

    assert!(
        bar_line_is_mapped_to_bar,
        "expected mapped token on `interface Bar` line to resolve to bar.d.ts"
    );
    assert!(
        foo_line_is_mapped_to_index,
        "expected mapped token on `interface Foo` line to resolve to index.d.ts"
    );
    assert!(!region_line_has_mapping, "synthetic region marker lines should be unmapped");

    assert!(lines.get(bar_line as usize).is_some_and(|line| line.contains("interface Bar")));
    assert!(lines.get(foo_line as usize).is_some_and(|line| line.contains("interface Foo")));
}

#[test]
fn sourcemap_composition_traces_to_original_ts_sources() {
    let (_code, map) = bundle_fixture_with_sourcemap("sourcemap-composition");

    let sources: Vec<&str> = map.get_sources().map(std::convert::AsRef::as_ref).collect();

    // When .d.ts.map files exist, sources should trace back to .ts files
    assert!(
        sources.iter().any(|s| s.ends_with("mod.ts")),
        "expected a source ending with mod.ts, got: {sources:?}"
    );
    assert!(
        sources.iter().any(|s| s.ends_with("index.ts")),
        "expected a source ending with index.ts, got: {sources:?}"
    );

    // Should NOT contain .d.ts sources (those are intermediate)
    for source in &sources {
        assert!(
            !source.ends_with(".d.ts"),
            "source should not be .d.ts after composition, got: {source}"
        );
    }

    // sourcesContent should contain the original .ts content
    let source_contents: Vec<Option<&str>> =
        map.get_source_contents().map(|c| c.map(std::convert::AsRef::as_ref)).collect();
    assert!(
        source_contents.iter().any(|c| c.is_some_and(|s| s.contains("export const foo"))),
        "expected sourcesContent to contain original .ts content"
    );
}
