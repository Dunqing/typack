# Real-world snapshot tests

These fixtures are copied from the build output of real projects and serve as
regression guards for the DTS bundler.

## Adding a new project

1. **Generate `.d.ts` files** from the project's TypeScript source using `tsc`
   with `--declaration --emitDeclarationOnly`.

2. **Create a fixture directory** under `tests/real-world/<project>/`. Place
   entry-level declaration files directly in that directory and supporting
   (non-entry) files in subdirectories. The test runner discovers entries by
   scanning for `*.d.ts` / `*.d.mts` files at the top level of each project
   directory; subdirectories are treated as internal dependencies.

3. **Rewrite imports** so that relative imports from entries point into their
   subdirectory (e.g. `'./foo'` → `'./<pkg>/foo'`). Cross-package bare
   specifiers (e.g. `@vue/shared`) remain untouched and are treated as
   external by the bundler.

4. **Generate snapshots** by running the bundler CLI and piping stdout into the
   snapshot file (do **not** use `echo` which adds a trailing newline):

   ```sh
   cargo run --features cli -- tests/real-world/<project>/<name>.d.ts \
     > tests/real-world/<project>/<name>.snapshot.d.ts
   ```

5. **Run the test** to verify:

   ```sh
   cargo test real_world
   ```

   The test compares bundler output against each snapshot and also validates
   sourcemaps and semantic hygiene (no dead root-level bindings). If a
   snapshot file does not exist for an entry, that entry is skipped.

---

## rolldown

Copied from the [rolldown](https://github.com/rolldown/rolldown)
`packages/rolldown/dist/` directory. Contains both `.d.ts` (unhashed shared
chunks) and `.d.mts` (hashed shared chunks) entry points.

Entry files and their shared dependencies are stored as-is. Each
`<name>.snapshot.d.{ts,mts}` file is the expected bundler output for the
corresponding `<name>.d.{ts,mts}` entry.

To regenerate a snapshot:

```sh
cargo run --features cli -- tests/real-world/rolldown/<name>.d.mts \
  > tests/real-world/rolldown/<name>.snapshot.d.mts
```

## core

Copied from the [Vue core](https://github.com/vuejs/core) `tsc` output. Each
Vue package (`shared`, `reactivity`, `compiler-core`, etc.) becomes a separate
entry.

### How the fixture was generated

1. Run `tsc` in the Vue core repo to produce raw declaration files:

   ```sh
   cd /path/to/core
   npx tsc -p tsconfig.build.json --noCheck
   ```

   This writes individual `.d.ts` files into `temp/packages/<pkg>/src/`.

2. For each package, copy `src/*.d.ts` (excluding `index.d.ts`) into
   `tests/real-world/core/<pkg>/`, preserving subdirectory structure.

3. Copy each package's `src/index.d.ts` to `tests/real-world/core/<pkg>.d.ts`
   and rewrite relative imports from `'./<module>'` to `'./<pkg>/<module>'` so
   they resolve into the subdirectory.

4. Fix any cross-package *relative* imports that `tsc` emitted (e.g.
   `'../../../compiler-core/src'` in `compiler-sfc`) to use the bare specifier
   instead (e.g. `'@vue/compiler-core'`).

5. Generate snapshots:

   ```sh
   for entry in tests/real-world/core/*.d.ts; do
     stem=$(basename "$entry" .d.ts)
     cargo run --features cli -- "$entry" \
       > "tests/real-world/core/$stem.snapshot.d.ts"
   done
   ```

### Known issues

`runtime-core` and `server-renderer` are present as entries but have no
snapshot files (and are therefore skipped). They fail the semantic hygiene
check because the bundler emits aliased-export patterns that the validator
flags as unused root bindings:

- `runtime-core`: `createBaseVNode` declared locally, exported as
  `createElementVNode`
- `server-renderer`: `ssrIncludeBooleanAttr` imported and re-exported under
  an alias
