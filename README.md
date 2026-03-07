# typack

**ty**(pe) + **pack** — pack your TypeScript declarations.

A native TypeScript declaration (`.d.ts`) bundler built on [Oxc](https://oxc.rs).

Bundles one or more `.d.ts` entry points into a single output file using a three-stage AST pipeline — no TypeScript compiler required.

**[Try it in the REPL →](https://typack.pages.dev)**

## Features

- **Fast** — native Rust implementation powered by the Oxc parser, no `tsc` dependency
- **Correct** — semantic rename using `SymbolId`-level analysis to avoid name collisions across modules
- **Tree-shaking** — only emits declarations that are reachable from entry points
- **Source maps** — optional source map composition back to original `.d.ts` sources
- **External packages** — configurable external specifiers preserved as imports in the output
- **CJS support** — optional `export =` syntax for CommonJS default exports
- **Node.js bindings** — pre-built N-API bindings for 8 platforms via npm

## How it works

The bundler runs a three-stage pipeline:

1. **Scan** — parse `.d.ts` files with `oxc_parser`, build semantic scoping via `oxc_semantic`, resolve imports with `oxc_resolver`, and produce a topologically sorted module graph
2. **Link** — analyze the module graph to build a rename plan (deconflicting names across modules) and a needed-names plan (tree-shaking unused declarations)
3. **Generate** — apply renames, rewrite inline `import()` types, wrap namespace imports, filter unused declarations, and emit the bundled output with optional source maps

## Rust

### Install

```toml
[dependencies]
typack = "0.1.0"
```

### Usage

```rust
use typack::{TypackBundler, TypackOptions};

let result = TypackBundler::bundle(&TypackOptions {
    input: vec!["types/index.d.ts".to_string()],
    cwd: std::env::current_dir().unwrap(),
    external: vec!["react".to_string()],
    sourcemap: true,
    ..Default::default()
});

match result {
    Ok(bundle) => {
        for output in &bundle.outputs {
            println!("{}", output.code);
            if let Some(map) = &output.map {
                // write source map to disk
                let _ = map;
            }
        }
        for warning in &bundle.warnings {
            eprintln!("warning: {warning}");
        }
    }
    Err(diagnostics) => {
        for diagnostic in diagnostics {
            eprintln!("error: {diagnostic}");
        }
    }
}
```

### API

**`TypackOptions`**

| Field         | Type          | Default | Description                                    |
| ------------- | ------------- | ------- | ---------------------------------------------- |
| `input`       | `Vec<String>` | `[]`    | Entry `.d.ts` file paths to bundle             |
| `external`    | `Vec<String>` | `[]`    | Module specifiers to keep as external imports  |
| `cwd`         | `PathBuf`     | `"."`   | Working directory for relative path resolution |
| `sourcemap`   | `bool`        | `false` | Generate source map (`.d.ts.map`)              |
| `cjs_default` | `bool`        | `false` | Emit `export =` for single default export      |

**`BundleResult`**

| Field      | Type                 | Description                         |
| ---------- | -------------------- | ----------------------------------- |
| `outputs`  | `Vec<BundleOutput>`  | Per-entry bundled outputs           |
| `warnings` | `Vec<OxcDiagnostic>` | Non-fatal warnings                  |

**`BundleOutput`**

| Field  | Type                | Description                         |
| ------ | ------------------- | ----------------------------------- |
| `code` | `String`            | The bundled `.d.ts` output          |
| `map`  | `Option<SourceMap>` | Source map (when `sourcemap: true`) |

## CLI

Requires the `cli` feature:

```bash
cargo run --features cli -- [OPTIONS] <ENTRY>...
```

### Options

```
<ENTRY>...                Entry .d.ts files to bundle
--external <SPEC>         Module specifiers to keep external (repeatable)
--cwd <DIR>               Working directory (default: current directory)
--sourcemap               Generate source map (.d.ts.map)
--cjs-default             Emit `export =` for single default export
-o, --outfile <PATH>      Write output to file instead of stdout
```

### Example

```bash
cargo run --features cli -- \
  --external react \
  --external react-dom \
  --sourcemap \
  -o dist/index.d.ts \
  types/index.d.ts
```

## Node.js

### Install

```bash
npm install typack
```

Requires Node.js >= 20. Pre-built binaries are available for:

| Platform      | Architectures       |
| ------------- | ------------------- |
| macOS         | `aarch64`, `x86_64` |
| Linux (glibc) | `aarch64`, `x86_64` |
| Linux (musl)  | `aarch64`, `x86_64` |
| Windows       | `aarch64`, `x86_64` |

### Usage

```js
import { bundle } from "typack";

const result = bundle({
  input: ["types/index.d.ts"],
  cwd: process.cwd(),
  external: ["react"],
  sourcemap: true,
});

console.log(result.code);

if (result.map) {
  // result.map is a JSON string
  fs.writeFileSync("dist/index.d.ts.map", result.map);
}

for (const warning of result.warnings) {
  console.warn(`warning: ${warning.message}`);
}
```

### TypeScript types

```typescript
interface BundleDtsOptions {
  input: Array<string>;
  external?: Array<string>;
  cwd?: string;
  sourcemap?: boolean;
  cjsDefault?: boolean;
}

interface BundleDtsResult {
  code: string;
  map?: string;
  warnings: Array<BundleDtsDiagnostic>;
}

interface BundleDtsDiagnostic {
  message: string;
  file?: string;
  span?: Array<number>;
  severity: string;
}
```

## Resolution

Module resolution uses `oxc_resolver` with the `"types"` export condition enabled, respecting `package.json` exports and conditional exports.

- **External specifiers** — specifiers in the `external` list are preserved in the output (exact match)
- **Unresolved relative specifiers** — fatal errors
- **Unresolved bare specifiers** — automatically externalized with a warning
- **Side-effect imports** — `import "pkg"` statements are preserved when the target contains global or module augmentations
- **Non-TS assets** — imports of CSS, images, etc. are silently skipped
- **Reference directives** — `/// <reference path="..." />` directives are collected and deduplicated

## Comparison with other tools

Several tools exist for bundling TypeScript declarations. Here's how they compare:

|                           | typack                | [rolldown-plugin-dts]  | [@microsoft/api-extractor] | [dts-bundle-generator] | [rollup-plugin-dts]   |
| ------------------------- | --------------------- | ---------------------- | -------------------------- | ---------------------- | --------------------- |
| **Language**              | Rust                  | TypeScript + Rust      | TypeScript                 | TypeScript             | TypeScript            |
| **Approach**              | Standalone bundler    | Rolldown plugin        | Standalone CLI/API         | Standalone CLI         | Rollup plugin         |
| **Input**                 | Pre-generated `.d.ts` | `.ts` source files     | Pre-generated `.d.ts`      | `.ts` source files     | Pre-generated `.d.ts` |
| **Generates `.d.ts`**     | No                    | Yes (tsc / oxc / tsgo) | No                         | Yes (in-memory tsc)    | No                    |
| **Requires tsc**          | No                    | Optional               | Yes (for input)            | Yes (bundled)          | Yes (for input)       |
| **Multiple entry points** | Yes                   | Yes                    | No                         | Yes                    | Yes                   |
| **Tree-shaking**          | Yes                   | Yes                    | Yes (via trimming)         | Yes                    | Yes                   |
| **Source maps**           | Yes                   | Yes                    | No                         | No                     | No                    |
| **Semantic rename**       | Yes (SymbolId)        | Yes (via typack)       | Yes                        | Yes                    | Yes                   |
| **API report / docs**     | No                    | No                     | Yes                        | No                     | No                    |
| **Release trimming**      | No                    | No                     | Yes (@alpha, @beta, etc.)  | No                     | No                    |
| **Status**                | Active                | Active                 | Active                     | Active                 | Maintenance mode      |

### How they differ

**[rolldown-plugin-dts]** is a Rolldown build plugin that orchestrates the full pipeline — it generates `.d.ts` files (via tsc, oxc, or tsgo), then calls **typack** under the hood to bundle them. Use it when you want a single build step that produces both `.js` and `.d.ts` output.

**[@microsoft/api-extractor]** is Microsoft's tool focused on API governance. Beyond bundling, it generates API reports and supports release trimming (`@public`, `@beta`, `@internal`). It requires pre-generated `.d.ts` input, supports only a single entry point, and does not produce source maps.

**[dts-bundle-generator]** compiles TypeScript in-memory and generates bundled declarations without writing intermediate files. It supports multiple entry points and fine-grained control over which packages to inline vs. import. It does not produce source maps.

**[rollup-plugin-dts]** is a Rollup plugin that bundles pre-generated `.d.ts` files using Rollup's module graph. It is currently in maintenance mode with no new feature development.

**typack** is a standalone, native bundler that operates directly on `.d.ts` ASTs. It does not generate declarations from `.ts` source — it expects pre-generated `.d.ts` files as input. Its focus is on speed, correctness (semantic-level rename deconfliction), and source map support.

[rolldown-plugin-dts]: https://github.com/sxzz/rolldown-plugin-dts
[@microsoft/api-extractor]: https://api-extractor.com/
[dts-bundle-generator]: https://github.com/timocov/dts-bundle-generator
[rollup-plugin-dts]: https://github.com/Swatinem/rollup-plugin-dts

## Development

### Prerequisites

- Rust 1.91.0+
- [just](https://just.systems) (task runner)

### Commands

```bash
just ready    # fmt + check + test + lint
just fmt      # cargo fmt
just check    # cargo check
just test     # cargo test
just lint     # cargo clippy
```

### Tests

- **Fixture tests** — small focused cases in `tests/fixtures/`, each with an `index.d.ts` input and `snapshot.d.ts` expected output
- **Real-world tests** — regression tests against declarations from [Rolldown](https://rolldown.rs), [Vue](https://vuejs.org), and [Vitest](https://vitest.dev)
- **Semantic tests** — targeted tests for rename deconfliction and tree-shaking logic

## Credits

The three-stage architecture (scan, link, generate) is inspired by [Rolldown](https://rolldown.rs). Test fixtures were adapted from [rolldown-plugin-dts](https://github.com/sxzz/rolldown-plugin-dts) and [rollup-plugin-dts](https://github.com/Swatinem/rollup-plugin-dts).

## License

[MIT](LICENSE)
