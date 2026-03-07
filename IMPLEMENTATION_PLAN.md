# Task: Eliminate redundant work in multi-entry bundling pipeline

## Context

The current multi-entry loop re-computes global work (identical across all entries) inside the per-entry loop. For N entries, operations like `collect_link_warnings`, `build_resolved_exports`, `resolve_default_export_name` (all-modules loop), `collect_reserved_decl_names`, and the all-modules namespace alias scan each run N times producing identical results. The `RenamePlan` is also cloned N times.

Following Rolldown's architecture (Scan once → Link once globally → Generate per-chunk), we hoist all global work into a single link pass and make the generate stage only do per-entry work with shared references.

## Stages

### Stage 1: Split `LinkOutput` into global and per-entry structs

- **Goal**: Define `LinkStageOutput` (global, computed once) and rename existing `LinkOutput` to `PerEntryLinkData` (per-entry, keeps only `needed_names_plan`).
- **Depends on**: none
- **Parallel**: yes (with nothing)
- **Success criteria**: `cargo check` passes
- **Status**: Not Started
- **Files**: `src/link_stage/types.rs`, `src/link_stage/mod.rs` (update pub exports)

### Stage 2: Create `build_link_stage_output()` and refactor `build_link_output_for_entry`

- **Goal**: Add `pub fn build_link_stage_output(scan_result, rename_plan) -> LinkStageOutput` that computes all global data once (default_export_names, link warnings, resolved exports, reserved_decl_names, all_module_aliases). Refactor `build_link_output_for_entry` → `build_per_entry_link_data` to only do per-entry work (build_needed_names + compute_entry_needed_symbols), returning `PerEntryLinkData`. Make `collect_reserved_decl_names` in namespace.rs `pub(crate)`.
- **Depends on**: Stage 1
- **Parallel**: no
- **Success criteria**: `cargo check` passes
- **Files**: `src/link_stage/mod.rs`, `src/generate_stage/namespace.rs` (visibility change)

### Stage 3: Refactor `GenerateStage` to use shared `&LinkStageOutput`

- **Goal**: Change `GenerateStage` to take `&LinkStageOutput` by reference instead of owning a cloned `RenamePlan`. Change `scan_result` from `&mut` to `&` (the `&mut` at line 544 is unnecessary). Update `generate()` to call `build_per_entry_link_data`, use `self.link_output.*` for global data, clone `reserved_decl_names` into local for namespace deconfliction. Remove `dbg!` calls. Refactor `pre_scan_namespace_info` to accept pre-computed `all_module_aliases` instead of re-scanning all modules.
- **Depends on**: Stage 2
- **Parallel**: no
- **Success criteria**: `cargo check` passes
- **Files**: `src/generate_stage/mod.rs`, `src/generate_stage/namespace.rs`, `src/generate_stage/rewriter.rs`

### Stage 4: Update orchestration in `lib.rs` and verify

- **Goal**: Wire everything together: call `build_link_stage_output` once before loop, collect global warnings once, pass `&link_output` and `&scan_result` to `GenerateStage`. Delete old `build_link_output_for_entry`. Run full test suite.
- **Depends on**: Stage 3
- **Parallel**: no
- **Success criteria**: `cargo test` passes, `cargo clippy` clean
- **Files**: `src/lib.rs`, `src/link_stage/mod.rs` (cleanup dead code)
