//! Module specifier resolution for the scan stage.
//!
//! Resolves import specifiers to internal file paths or classifies them as
//! external, using the Oxc resolver with TypeScript condition names.

use std::path::{Path, PathBuf};

use oxc_diagnostics::OxcDiagnostic;
use oxc_resolver::{ResolveOptions, Resolver};
use rustc_hash::FxHashSet;

#[derive(Clone, Copy)]
pub(super) enum ExternalReason {
    ResolvedExternal,
    UnresolvedBare,
    ForcedExternal,
    ForcedExternalOverrideResolved,
}

pub(super) enum ResolvedSpecifier {
    Internal(PathBuf),
    External(ExternalReason),
    /// Non-TS asset import (CSS, images, etc.) — silently dropped from the graph.
    Skipped,
}

pub(super) fn maybe_push_resolution_warning(
    warnings: &mut Vec<OxcDiagnostic>,
    warning_dedup: &mut FxHashSet<(String, String, String)>,
    importer_path: &Path,
    specifier: &str,
    reason: ExternalReason,
) {
    let (code, message) = match reason {
        ExternalReason::UnresolvedBare => (
            "typack/externalized-bare-unresolved",
            format!(
                "typack/externalized-bare-unresolved: externalized unresolved bare specifier \"{specifier}\" from {}",
                importer_path.display()
            ),
        ),
        ExternalReason::ForcedExternalOverrideResolved => (
            "typack/forced-external-override",
            format!(
                "typack/forced-external-override: force-external override for \"{specifier}\" from {} despite successful resolution",
                importer_path.display()
            ),
        ),
        ExternalReason::ResolvedExternal | ExternalReason::ForcedExternal => return,
    };
    let key = (code.to_string(), importer_path.display().to_string(), specifier.to_string());
    if warning_dedup.insert(key) {
        warnings.push(OxcDiagnostic::warn(message));
    }
}

pub(super) fn create_dts_resolver() -> Resolver {
    let mut options = ResolveOptions::default();
    if !options.condition_names.iter().any(|name| name == "types") {
        options.condition_names.push("types".to_string());
    }
    Resolver::new(options)
}

pub(super) fn resolve_specifier_for_scan(
    resolver: &Resolver,
    importer_path: &Path,
    specifier: &str,
    forced_external: &FxHashSet<String>,
    explicit_internal_paths: &FxHashSet<PathBuf>,
) -> Result<ResolvedSpecifier, OxcDiagnostic> {
    if forced_external.contains(specifier) {
        let reason = match resolver.resolve_dts(importer_path, specifier) {
            Ok(_) => ExternalReason::ForcedExternalOverrideResolved,
            Err(_) => ExternalReason::ForcedExternal,
        };
        return Ok(ResolvedSpecifier::External(reason));
    }

    match resolver.resolve_dts(importer_path, specifier) {
        Ok(resolution) => {
            let resolved_path = resolution.into_path_buf();
            if should_bundle_resolved_target(&resolved_path, explicit_internal_paths) {
                Ok(ResolvedSpecifier::Internal(resolved_path))
            } else {
                Ok(ResolvedSpecifier::External(ExternalReason::ResolvedExternal))
            }
        }
        Err(error) => {
            // Skip non-TS asset imports (CSS, images, fonts, etc.) that the
            // resolver couldn't find a .d.ts sidecar for.
            if is_relative_specifier(specifier) && is_non_ts_asset(specifier) {
                return Ok(ResolvedSpecifier::Skipped);
            }
            if is_relative_specifier(specifier) {
                Err(OxcDiagnostic::error(format!(
                    "Cannot resolve relative specifier \"{specifier}\" from {}: {error}",
                    importer_path.display()
                )))
            } else {
                Ok(ResolvedSpecifier::External(ExternalReason::UnresolvedBare))
            }
        }
    }
}

fn should_bundle_resolved_target(
    resolved_path: &Path,
    explicit_internal_paths: &FxHashSet<PathBuf>,
) -> bool {
    if explicit_internal_paths.contains(resolved_path) {
        return true;
    }
    if path_has_component(resolved_path, "node_modules") {
        return false;
    }
    is_ts_file(resolved_path)
}

/// Returns `true` if the path has a TypeScript-family extension that we can parse.
fn is_ts_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Check .d.ts / .d.mts / .d.cts first (compound extensions)
    if name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts") {
        return true;
    }
    matches!(path.extension().and_then(|e| e.to_str()), Some("ts" | "mts" | "cts" | "tsx"))
}

fn path_has_component(path: &Path, component: &str) -> bool {
    path.components().any(|c| c.as_os_str() == std::ffi::OsStr::new(component))
}

fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with('.') || specifier.starts_with('/')
}

/// Check if a specifier refers to a non-TS asset (CSS, images, fonts, etc.)
/// that cannot be resolved to a declaration file.
fn is_non_ts_asset(specifier: &str) -> bool {
    const ASSET_EXTENSIONS: &[&str] = &[
        "css", "scss", "sass", "less", "styl", "stylus", "pcss", "postcss", "svg", "png", "jpg",
        "jpeg", "gif", "webp", "ico", "woff", "woff2", "eot", "ttf", "otf", "mp3", "mp4", "webm",
    ];
    Path::new(specifier)
        .extension()
        .is_some_and(|ext| ASSET_EXTENSIONS.iter().any(|e| ext.eq_ignore_ascii_case(e)))
}
