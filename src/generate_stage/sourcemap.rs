//! Source map composition utilities.
//!
//! Composes the codegen source map (bundled -> `.d.ts`) with the input
//! `.d.ts.map` (`.d.ts` -> original `.ts`) to produce a final map from
//! bundled output directly to original TypeScript sources.

use std::path::{Path, PathBuf};

use oxc_sourcemap::{SourceMap, SourceMapBuilder};

/// Compose two sourcemaps: the `generated_map` (from codegen, mapping bundled
/// positions -> `.d.ts` positions) is traced through `input_map` (from the
/// `.d.ts.map` file, mapping `.d.ts` positions -> original `.ts` positions).
/// The result maps bundled positions directly to original `.ts` positions.
///
/// `module_abs_path` is the absolute path to the `.d.ts` file. `cwd` is the
/// bundler's working directory. Together they resolve the input map's source
/// paths (which may be relative to the `.d.ts.map` or absolute) into paths
/// relative to CWD.
pub(super) fn compose_sourcemaps(
    generated_map: &SourceMap,
    input_map: &SourceMap,
    module_abs_path: &Path,
    cwd: &Path,
) -> SourceMap {
    let lookup_table = input_map.generate_lookup_table();
    let mut builder = SourceMapBuilder::default();
    let module_dir = module_abs_path.parent().unwrap_or(Path::new(""));

    let resolve = |s: &str| -> PathBuf {
        let p = Path::new(s);
        if p.is_absolute() {
            resolve_absolute_path(p)
        } else {
            resolve_absolute_path(&module_dir.join(p))
        }
    };

    // Pre-resolve input map sources (path relative to CWD + content).
    let resolved: Vec<(String, String)> = input_map
        .get_sources()
        .enumerate()
        .map(|(idx, s)| {
            let abs = resolve(s.as_ref());
            let rel = diff_paths(&abs, cwd);
            let content = input_map
                .get_source_content(u32::try_from(idx).unwrap_or(u32::MAX))
                .map(|c| c.as_ref().to_string())
                .or_else(|| std::fs::read_to_string(&abs).ok())
                .unwrap_or_default();
            (rel, content)
        })
        .collect();

    for token in generated_map.get_tokens() {
        // Try to trace through the input map to find the original .ts position
        let composed = input_map
            .lookup_token(&lookup_table, token.get_src_line(), token.get_src_col())
            .and_then(|orig| {
                let (source, content) = resolved.get(orig.get_source_id()? as usize)?;
                let src_id = builder.add_source_and_content(source, content);
                let name_id = orig
                    .get_name_id()
                    .and_then(|id| input_map.get_name(id).map(|n| builder.add_name(n)));
                Some((orig.get_src_line(), orig.get_src_col(), Some(src_id), name_id))
            });

        if let Some((src_line, src_col, source_id, name_id)) = composed {
            builder.add_token(
                token.get_dst_line(),
                token.get_dst_col(),
                src_line,
                src_col,
                source_id,
                name_id,
            );
        } else {
            // Fallback: keep the original .d.ts mapping (or unmapped)
            let source_id = token.get_source_id().and_then(|sid| {
                let source = generated_map.get_source(sid)?;
                let content = generated_map.get_source_content(sid);
                let content = content.map_or("", AsRef::as_ref);
                Some(builder.add_source_and_content(source, content))
            });
            builder.add_token(
                token.get_dst_line(),
                token.get_dst_col(),
                token.get_src_line(),
                token.get_src_col(),
                source_id,
                None,
            );
        }
    }

    builder.into_sourcemap()
}

/// Resolve an absolute path by processing `.` and `..` components without
/// touching the filesystem. `..` at the filesystem root is a no-op.
fn resolve_absolute_path(path: &Path) -> PathBuf {
    let mut components: Vec<std::path::Component<'_>> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                // Don't pop past RootDir
                if let Some(std::path::Component::Normal(_)) = components.last() {
                    components.pop();
                }
            }
            other => components.push(other),
        }
    }
    components.into_iter().collect()
}

/// Compute a relative path from `base` to `target`. Both paths should be
/// absolute and already normalized (no `.` or `..` components).
fn diff_paths(target: &Path, base: &Path) -> String {
    let target_components: Vec<_> = target.components().collect();
    let base_components: Vec<_> = base.components().collect();

    let common =
        target_components.iter().zip(base_components.iter()).take_while(|(a, b)| a == b).count();

    let mut result = PathBuf::new();
    for _ in common..base_components.len() {
        result.push("..");
    }
    for comp in &target_components[common..] {
        result.push(comp);
    }
    result.to_string_lossy().to_string()
}
