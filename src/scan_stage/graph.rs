//! Topological sorting of the module dependency graph with cycle detection.

use std::path::PathBuf;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::types::ModuleIdx;

pub(super) type PendingInternalEdges = FxHashMap<ModuleIdx, Vec<(String, PathBuf)>>;

pub(super) fn insert_pending_internal_edge(
    pending_internal_edges: &mut PendingInternalEdges,
    module_idx: ModuleIdx,
    specifier: &str,
    canonical_path: PathBuf,
) {
    let edges = pending_internal_edges.entry(module_idx).or_default();
    if edges.iter().any(|(existing_specifier, _)| existing_specifier == specifier) {
        return;
    }
    edges.push((specifier.to_string(), canonical_path));
}

/// Result of topological sort including detected cycles.
pub(super) struct TopologicalSortResult {
    /// Modules in dependency order (dependencies before dependents).
    pub order: Vec<ModuleIdx>,
    /// Detected circular dependencies. Each entry is a cycle as a list of module indices.
    pub cycles: Vec<Vec<ModuleIdx>>,
}

/// Topological sort of modules using DFS post-order traversal.
/// Returns modules in dependency order (dependencies before dependents),
/// plus any detected circular dependencies.
pub(super) fn topological_sort(
    path_to_idx: &FxHashMap<PathBuf, ModuleIdx>,
    pending_internal_edges: &PendingInternalEdges,
    entry_indices: &[ModuleIdx],
) -> TopologicalSortResult {
    let mut order = Vec::new();
    let mut visited = FxHashSet::default();
    let mut in_stack = FxHashSet::default();
    let mut cycles = Vec::new();
    for &entry_idx in entry_indices {
        topo_visit(
            path_to_idx,
            pending_internal_edges,
            entry_idx,
            &mut visited,
            &mut in_stack,
            &mut order,
            &mut cycles,
        );
    }
    TopologicalSortResult { order, cycles }
}

fn topo_visit(
    path_to_idx: &FxHashMap<PathBuf, ModuleIdx>,
    pending_internal_edges: &PendingInternalEdges,
    idx: ModuleIdx,
    visited: &mut FxHashSet<ModuleIdx>,
    in_stack: &mut FxHashSet<ModuleIdx>,
    order: &mut Vec<ModuleIdx>,
    cycles: &mut Vec<Vec<ModuleIdx>>,
) {
    if visited.contains(&idx) {
        return;
    }

    // Detect back-edge: if idx is already on the current DFS stack, we have a cycle.
    if !in_stack.insert(idx) {
        // Record the cycle (just the single back-edge target for now;
        // the full cycle path would require tracking the DFS stack).
        cycles.push(vec![idx]);
        return;
    }

    // Visit dependencies first
    if let Some(edges) = pending_internal_edges.get(&idx) {
        for (_, canonical_path) in edges {
            if let Some(&dep_idx) = path_to_idx.get(canonical_path) {
                topo_visit(
                    path_to_idx,
                    pending_internal_edges,
                    dep_idx,
                    visited,
                    in_stack,
                    order,
                    cycles,
                );
            }
        }
    }

    in_stack.remove(&idx);
    visited.insert(idx);
    order.push(idx);
}
