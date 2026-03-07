use std::{fs, path::PathBuf};

use cow_utils::CowUtils;
use typack::{TypackBundler, TypackOptions};

mod common;
use common::lenient_compare;

/// Test runner for fixtures ported from rolldown-plugin-dts.
///
/// These fixtures live under `tests/rolldown-plugin-dts/` and mirror the
/// upstream `rolldown-plugin-dts/tests/rollup-plugin-dts/` test suite.
/// Separating them makes it easy to track upstream changes and distinguish
/// our own conformance tests from the ported suite.
#[test]
fn rolldown_plugin_dts() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = crate_dir.join("tests").join("rolldown-plugin-dts");

    // Collect and sort fixture directories for deterministic order
    let mut dirs: Vec<PathBuf> = fs::read_dir(&fixture_dir)
        .unwrap()
        .filter_map(|e| {
            let path = e.unwrap().path();
            if path.is_dir() { Some(path) } else { None }
        })
        .collect();
    dirs.sort();

    let mut passed = 0;
    let mut strict_passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failures = Vec::new();

    for dir in &dirs {
        let entry = dir.join("index.d.ts");
        let snapshot = dir.join("snapshot.d.ts");
        let fixture_name = dir.file_name().unwrap().to_string_lossy().to_string();

        // Skip fixtures without standard entry point or snapshot
        if !entry.exists() || !snapshot.exists() {
            skipped += 1;
            continue;
        }

        let expected = fs::read_to_string(&snapshot)
            .unwrap_or_else(|e| panic!("Failed to read snapshot for {fixture_name}: {e}"));

        let result = TypackBundler::bundle(&TypackOptions {
            input: vec![entry.to_string_lossy().to_string()],
            cwd: crate_dir.clone(),
            ..Default::default()
        });

        let actual = match result {
            Ok(bundle) => bundle.output[0].code.clone(),
            Err(diagnostics) => {
                failed += 1;
                let msgs: Vec<String> =
                    diagnostics.iter().map(std::string::ToString::to_string).collect();
                failures.push(format!("{fixture_name}: error: {}", msgs.join(", ")));
                continue;
            }
        };

        // Strict comparison (exact match after newline normalization)
        let expected_norm = expected.cow_replace("\r\n", "\n");
        let actual_norm = actual.cow_replace("\r\n", "\n");
        if actual_norm == expected_norm {
            strict_passed += 1;
        }

        // Lenient comparison
        match lenient_compare(&expected, &actual) {
            Ok(()) => {
                passed += 1;
            }
            Err(diff) => {
                failed += 1;
                failures.push(format!("{fixture_name}:\n  {diff}"));
            }
        }
    }

    let total = passed + failed + skipped;

    assert!(
        failed == 0,
        "rolldown-plugin-dts conformance failed: {failed} fixtures failed (strict_passed={strict_passed}, total={total}, skipped={skipped})\n{}",
        failures.join("\n")
    );
}
