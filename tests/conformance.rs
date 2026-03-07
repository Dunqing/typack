use std::{fs, path::PathBuf};

use cow_utils::CowUtils;
use typack::{TypackBundler, TypackOptions};

mod common;
use common::lenient_compare;

#[test]
fn conformance() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = crate_dir.join("tests").join("fixtures");

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
        let fixture_name = dir.file_name().unwrap().to_string_lossy().to_string();
        let config_path = dir.join("config.json");

        // Multi-entry fixtures have a config.json
        if config_path.exists() {
            let config: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(&config_path)
                    .unwrap_or_else(|e| panic!("Failed to read config for {fixture_name}: {e}")),
            )
            .unwrap_or_else(|e| panic!("Failed to parse config for {fixture_name}: {e}"));

            let entries: Vec<String> = config["entries"]
                .as_array()
                .unwrap_or_else(|| panic!("{fixture_name}: config.json must have 'entries' array"))
                .iter()
                .map(|v| dir.join(v.as_str().unwrap()).to_string_lossy().to_string())
                .collect();

            // Collect expected snapshots per entry
            let entry_stems: Vec<String> = config["entries"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let name = v.as_str().unwrap();
                    // "index.d.ts" -> "index", "a.d.ts" -> "a"
                    name.strip_suffix(".d.ts").unwrap_or(name).to_string()
                })
                .collect();

            let snapshots: Vec<Option<String>> = entry_stems
                .iter()
                .map(|stem| {
                    let snap = dir.join(format!("snapshot-{stem}.d.ts"));
                    if snap.exists() {
                        Some(fs::read_to_string(&snap).unwrap_or_else(|e| {
                            panic!("Failed to read snapshot for {fixture_name}/{stem}: {e}")
                        }))
                    } else {
                        None
                    }
                })
                .collect();

            if snapshots.iter().all(Option::is_none) {
                skipped += 1;
                continue;
            }

            let result = TypackBundler::bundle(&TypackOptions {
                input: entries,
                cwd: crate_dir.clone(),
                ..Default::default()
            });

            let output = match result {
                Ok(bundle) => bundle.output,
                Err(diagnostics) => {
                    failed += 1;
                    let msgs: Vec<String> =
                        diagnostics.iter().map(std::string::ToString::to_string).collect();
                    failures.push(format!("{fixture_name}: error: {}", msgs.join(", ")));
                    continue;
                }
            };

            if output.len() != entry_stems.len() {
                failed += 1;
                failures.push(format!(
                    "{fixture_name}: expected {} outputs, got {}",
                    entry_stems.len(),
                    output.len()
                ));
                continue;
            }

            let mut fixture_ok = true;
            for (i, (entry_output, snapshot)) in output.iter().zip(&snapshots).enumerate() {
                let stem = &entry_stems[i];
                let Some(expected) = snapshot else { continue };

                let expected_norm = expected.cow_replace("\r\n", "\n");
                let actual_norm = entry_output.code.cow_replace("\r\n", "\n");
                if actual_norm == expected_norm {
                    strict_passed += 1;
                }

                match lenient_compare(expected, &entry_output.code) {
                    Ok(()) => {}
                    Err(diff) => {
                        fixture_ok = false;
                        failures.push(format!("{fixture_name}/{stem}:\n  {diff}"));
                    }
                }
            }

            if fixture_ok {
                passed += 1;
            } else {
                failed += 1;
            }
            continue;
        }

        // Single-entry fixtures: index.d.ts + snapshot.d.ts
        let entry = dir.join("index.d.ts");
        let snapshot = dir.join("snapshot.d.ts");

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
        "DTS Bundler conformance failed: {failed} fixtures failed (strict_passed={strict_passed}, total={total}, skipped={skipped})\n{}",
        failures.join("\n")
    );
}
