use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};
use typack::{TypackBundler, TypackOptions};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn real_world_dir() -> PathBuf {
    crate_dir().join("tests").join("real-world")
}

fn bench_fixtures_dir() -> PathBuf {
    crate_dir().join("benches").join("fixtures")
}

fn make_options(entries: &[PathBuf]) -> TypackOptions {
    TypackOptions {
        input: entries.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        cwd: crate_dir(),
        ..Default::default()
    }
}

fn bundle_real_world(c: &mut Criterion) {
    let rw = real_world_dir();

    // Small: vitest/utils (78 lines output, 5 dependency files)
    let vitest_utils = rw.join("vitest/utils.d.ts");

    // Medium: vue/reactivity (777 lines output, 12 dependency files)
    let vue_reactivity = rw.join("core/reactivity.d.ts");

    // Large: rolldown/experimental-index (5,398 lines output, many shared deps)
    let rolldown_exp = rw.join("rolldown/experimental-index.d.mts");

    // Multi-entry: 3 Vue core entries bundled together
    let vue_multi: Vec<PathBuf> = vec![
        rw.join("core/reactivity.d.ts"),
        rw.join("core/runtime-core.d.ts"),
        rw.join("core/compiler-core.d.ts"),
    ];

    let mut group = c.benchmark_group("bundle");

    group.bench_function("vitest_utils", |b| {
        let opts = make_options(std::slice::from_ref(&vitest_utils));
        b.iter(|| TypackBundler::bundle(&opts).unwrap());
    });

    group.bench_function("vue_reactivity", |b| {
        let opts = make_options(std::slice::from_ref(&vue_reactivity));
        b.iter(|| TypackBundler::bundle(&opts).unwrap());
    });

    group.bench_function("rolldown_experimental", |b| {
        let opts = make_options(std::slice::from_ref(&rolldown_exp));
        b.iter(|| TypackBundler::bundle(&opts).unwrap());
    });

    group.bench_function("vue_multi_entry", |b| {
        let opts = make_options(&vue_multi);
        b.iter(|| TypackBundler::bundle(&opts).unwrap());
    });

    group.finish();
}

fn bundle_ts_entry(c: &mut Criterion) {
    let ts_entry = bench_fixtures_dir().join("ts-entry").join("index.ts");

    let mut group = c.benchmark_group("bundle_ts");

    group.bench_function("ts_entry_isolated_declarations", |b| {
        let opts = make_options(std::slice::from_ref(&ts_entry));
        b.iter(|| TypackBundler::bundle(&opts).unwrap());
    });

    group.finish();
}

criterion_group!(benches, bundle_real_world, bundle_ts_entry);
criterion_main!(benches);
