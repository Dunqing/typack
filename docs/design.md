# Typack Design

Typack is a native TypeScript `.d.ts` declaration bundler built on [Oxc](https://github.com/oxc-project/oxc). It takes one or more `.d.ts` entry files, resolves their transitive imports, tree-shakes unused declarations, deconflicts colliding names, and emits a single bundled declaration file per entry point with optional source maps.

## Pipeline Overview

Typack follows Rolldown's three-stage architecture:

```
  .d.ts files
       |
       v
 ┌───────────┐
 │   Scan    │  Parse files, resolve imports, build module graph
 └─────┬─────┘
       │  ScanStageOutput (ModuleTable, AstTable, entry_points)
       v
 ┌───────────┐
 │   Link    │  Name deconfliction, tree-shaking, namespace planning
 └─────┬─────┘
       │  LinkStageOutput (global) + PerEntryLinkData (per entry)
       v
 ┌───────────┐
 │ Generate  │  Transform ASTs, rewrite imports, codegen bundled output
 └─────┬─────┘
       │
       v
  Bundled .d.ts + sourcemap (one per entry)
```

Each stage has well-defined inputs and outputs. Data flows forward only — no stage reaches back to modify an earlier stage's output (except generate's `TakeIn` optimization for single-entry, described below).

---

## Stage 1: Scan

**Location:** `src/scan_stage/`

The scan stage discovers all modules reachable from the entry points, parses them, and builds a topologically-sorted module graph.

### Algorithm

1. **BFS from entries.** Starting from each entry file, resolve import specifiers using Oxc's resolution and enqueue discovered modules.
2. **Classify specifiers.** Each import is classified as:
   - **Internal** — resolved to a `.d.ts` file in the project. Added to the module graph.
   - **External** — a bare specifier (e.g., `react`) or explicitly marked external. Preserved as-is in the output.
3. **Parse and analyze.** Each discovered module is parsed with Oxc's parser and semantic analysis (scoping, symbol resolution). Pre-computed metadata is collected:
   - `ModuleExportImportInfo` — maps of exported names, imported bindings, star re-exports
   - Reference directives (`/// <reference ...>`)
   - Augmentation detection (`declare module "..."` at top level)
4. **Topological sort.** Modules are reindexed so dependencies appear before dependents. This ordering is critical for correct code generation — a declaration must appear before any reference to it.

### Output

```rust
struct ScanStageOutput<'a> {
    module_table: ModuleTable<'a>,   // All modules, indexed by ModuleIdx
    ast_table: AstTable<'a>,         // Parsed ASTs, indexed by ModuleIdx
    entry_points: Vec<ModuleIdx>,
    warnings: Vec<OxcDiagnostic>,
}
```

`Module` contains per-module metadata: path, source text, scoping info, resolved specifier maps, export/import info, and flags (`is_entry`, `has_augmentation`).

`ModuleIdx` is an opaque 32-bit index into both tables, enabling O(1) cross-referencing.

---

## Stage 2: Link

**Location:** `src/link_stage/`

The link stage performs all cross-module analysis: which names collide, which symbols are needed, and how modules should be wrapped. It produces two kinds of output:

- **Global** (`LinkStageOutput`) — computed once, shared across all entries
- **Per-entry** (`PerEntryLinkData`) — computed separately for each entry point

### 2.1 Name Deconfliction

**Location:** `src/link_stage/rename.rs`

When multiple modules declare the same name (e.g., two modules both export `Options`), the link stage renames one to avoid collisions in the bundled output.

**Algorithm (two-pass):**

1. **Pass 1 — Exported names.** Process modules in entry re-export order. The entry's own re-exports determine priority: the first module to claim a name keeps it.
2. **Pass 2 — Non-exported names.** Process remaining modules in reverse topological order. Later modules keep their original names; earlier ones get `$1`, `$2`, etc. suffixes.

**Per-module mapping:**

```rust
struct CanonicalNames {
    per_module_symbols: IndexVec<ModuleIdx, FxHashMap<SymbolId, String>>,  // O(1) per-module lookup
    fallback_name_renames: FxHashMap<(ModuleIdx, String), String>,  // For declaration merging
    used_names: FxHashSet<String>,
}
```

Renames are stored grouped by module, giving O(1) per-module lookup without a separate indexing step. The fallback handles cases where symbol resolution isn't possible (e.g., names from declaration merging across interfaces).

### 2.2 Tree-Shaking

**Location:** `src/link_stage/needed_names.rs`

Determines which symbols from each non-entry module are actually needed in the bundle.

**Algorithm (fixpoint propagation):**

1. **Seed.** Mark the entry module's exported symbols as needed.
2. **Expand.** For each needed symbol in a module, use that module's _declaration graph_ to find local dependencies (other declarations referenced in the same module).
3. **Propagate.** When a dependency crosses module boundaries (e.g., `import { X } from "./other"`), mark the target symbol as needed in the target module.
4. **Repeat** until no new symbols are discovered (fixpoint).

**Declaration graph.** Pre-computed per module in `NeededNamesCtx`. Each declaration node records:

- Which root-scope symbols it declares
- Which local symbols it references (local dependencies)
- Which cross-module imports it references (cross-module dependencies)
- Which `import("...")` type references it contains (inline import dependencies)

**Kind tracking.** Symbols are tracked with `NeededKindFlags` (VALUE, TYPE, or both). A class used only as a type (`typeof Foo`) needs only its TYPE declaration space. This enables finer-grained tree-shaking than name-only tracking.

### 2.3 Per-Module Link Metadata

**Location:** `src/link_stage/module_meta.rs`

For each module included in an entry's bundle, `compute_module_link_meta` determines:

| Field                                         | Purpose                                                                                     |
| --------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `statement_actions: Vec<StatementAction>`     | Per-statement decision: `Skip`, `Include`, `UnwrapExportDeclaration`, `UnwrapExportDefault` |
| `import_renames: FxHashMap<SymbolId, String>` | Local symbol → resolved canonical name from source module                                   |
| `ns_aliases: FxHashSet<SymbolId>`             | `import * as X` symbols that need stripping                                                 |
| `external_ns_info`                            | External namespace imports for member-access rewriting                                      |
| `needs_structural_mutation: bool`             | Whether the module needs AST structural changes beyond renames                              |

`StatementAction` is the key decision type:

- **Skip** — Tree-shaken, or metadata-only (e.g., import declarations, bare `export { }` specifiers). Collected for metadata but not emitted.
- **Include** — Emit as-is.
- **UnwrapExportDeclaration** — Extract the inner declaration from `export const X = ...` and add `declare`.
- **UnwrapExportDefault** — Convert `export default class Foo` to `declare class Foo`.

### 2.4 Namespace Planning

**Location:** `src/link_stage/namespace.rs`

When a module uses `import * as X from "./mod"` and re-exports `X`, the bundler wraps the source module's exports in a `declare namespace X { ... }` block. The link stage plans which modules need wrapping and deconflicts namespace names against other declarations.

### 2.5 Link Output

```
LinkStageOutput (global)          PerEntryLinkData (per entry)
├── canonical_names               ├── needed_names_plan
├── default_export_names          ├── module_metas (per-module analysis)
├── all_module_aliases            ├── namespace_wraps
├── reserved_decl_names           ├── namespace_aliases
└── warnings                      ├── helper_reserved_names
                                  └── namespace_warnings
```

---

## Stage 3: Generate

**Location:** `src/generate_stage/`

The generate stage transforms each module's AST according to the link stage decisions and assembles the final bundled output.

### 3.1 Per-Module Processing

Each module goes through two phases:

**Phase 1: Output Collection** (`analysis.rs`)

Walks the original AST using pre-computed `StatementAction` decisions. Collects metadata into `GenerateAcc`:

- `exports` — names for the consolidated `export { ... }` statement
- `imports` — external imports to merge and emit
- `star_exports` — `export * from "external"` passthrough

This phase reads but does not modify the AST.

**Phase 2: Rendering** (`render_module.rs`)

Clones or takes ownership of surviving AST statements, applies transformations, and runs codegen.

The transformation is performed by `DtsFinalizer` (`finalizer.rs`), a single `VisitMut` pass that handles:

- **Renaming** — Apply canonical name renames and import renames to binding identifiers and identifier references
- **Inline import rewriting** — Replace `import("./internal")` type references with namespace-qualified names
- **Namespace alias stripping** — Remove `import * as X` prefix from member accesses (`X.Foo` → `Foo`)
- **External namespace recording** — Track member accesses on external namespace imports for later conversion to named imports

### 3.2 AST Ownership Strategy

The key performance challenge: the AST is shared across entries, but transformations are destructive.

**Single-entry fast path (90%+ of usage):**

When there is only one entry point, the AST will never be read again. The generate stage takes ownership of statements using Oxc's `TakeIn` trait (`mem::replace` with a zero placeholder) — O(1) per node, no deep copy.

```
StatementAction::Include      →  body[i].take_in(allocator)
UnwrapExportDeclaration       →  take statement, unbox(), extract declaration
UnwrapExportDefault           →  take statement, unbox(), match declaration kind
```

Comments, hashbang, and directives also use `take_in` / `.take()`.

**Multi-entry path:**

Renames are global (identical across entries), so they can be applied once before the entry loop via `RenameApplier` — a lightweight `VisitMut` that only handles identifiers.

For each entry, modules are then classified:

- `needs_structural_mutation == false` — Clone statements (renames pre-applied), skip `DtsFinalizer` entirely. No visitor traversal.
- `needs_structural_mutation == true` — Clone statements, run `DtsFinalizer` with empty rename map (only structural rewrites).

### 3.3 Output Assembly

For each entry, the generate stage assembles output in this order:

1. Reference directives (`/// <reference ...>`)
2. External imports (merged by source, deduplicated by specifier)
3. Star re-exports (`export * from "external"`)
4. Namespace wrapper blocks (`declare namespace X { ... }`)
5. Per-module code regions (reverse topological order, with `#region` markers)
6. Consolidated export statement (`export { A, B, C }`)

After assembly, unused imports are pruned: the `ReferencedNameCollector` visitor scans the transformed AST for identifier references, and import specifiers not referenced in the final output are removed.

### 3.4 Source Map Composition

When source maps are enabled, each module's codegen produces a source map mapping the bundled output back to the module's source text. If the module has an input source map (e.g., generated by `tsc`), the two maps are composed so the final map traces back to the original `.ts` source.

---

## Key Data Types

### Module Identity

| Type        | Description                                    |
| ----------- | ---------------------------------------------- |
| `ModuleIdx` | 32-bit index into `ModuleTable` and `AstTable` |
| `SymbolId`  | Oxc symbol identifier (scoping-aware)          |

### Export/Import Info (Pre-computed in Scan)

```rust
struct ModuleExportImportInfo {
    named_exports: FxHashMap<String, ExportEntry>,    // exported_name → source info
    star_reexports: Vec<StarReexport>,                // export * from "..."
    named_imports: FxHashMap<String, ImportBinding>,   // local_name → import binding
    declared_export_names: FxHashSet<String>,          // exports from local declarations only
}
```

### Generate Accumulator

```rust
struct GenerateAcc {
    exports: Vec<ExportedName>,           // For consolidated export { ... }
    imports: Vec<ExternalImport>,         // External imports to merge
    star_exports: Vec<ExternalStarExport>,
    ns_wrapper_blocks: String,            // Namespace declarations
    ns_name_map: FxHashMap<String, String>,
    warnings: Vec<OxcDiagnostic>,
    has_any_export_statement: bool,
    module_exports_start: usize,          // Partition index for current module
    module_imports_start: usize,
}
```

---

## Cross-Cutting Concerns

### Allocation

All module source text and parsed ASTs live in a shared arena allocator (`&'a Allocator`). The `'a` lifetime binds AST nodes to the allocator, ensuring they remain valid throughout the pipeline. Cloning AST nodes (`clone_in_with_semantic_ids`) allocates into the same arena.

### Error Handling

- **Fatal errors** (parse failures, unresolvable entries) — returned as `Err(Vec<OxcDiagnostic>)` from scan.
- **Warnings** (unresolved bare specifiers, namespace name conflicts, rename fallbacks) — accumulated in each stage's output and merged in `Bundle::generate`.

### Determinism

Output is deterministic for a given set of inputs:

- Module ordering is topological (dependency-driven, not filesystem-order)
- Name deconfliction uses a fixed priority order (entry re-export order, then reverse topological)
- Export statements are emitted in collection order (which follows AST visit order)
