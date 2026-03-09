//! Generate-stage namespace helpers.
//!
//! Functions for sanitizing identifiers and creating unique namespace names
//! during the generate phase. Link-stage namespace planning functions live
//! in `crate::link_stage::namespace`.

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::{FxHashMap, FxHashSet};

// Re-export link-stage namespace functions used by generate-stage code.
pub use crate::link_stage::namespace::{
    collect_declaration_names, collect_export_specifier, collect_module_exports,
};

/// Sanitize a string to be a valid JavaScript identifier.
/// Replaces non-alphanumeric chars (except `_`) with `_`.
pub(super) fn sanitize_to_identifier(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect()
}

/// Get or create a unique namespace name for a specifier.
/// Uses `default_name_fn` to generate the base name if not already in the map.
/// Appends numeric suffixes (0, 1, 2, ...) to resolve conflicts.
pub(super) fn get_or_create_ns_name(
    specifier: &str,
    ns_name_map: &mut FxHashMap<String, String>,
    reserved_names: &FxHashSet<String>,
    warnings: &mut Vec<OxcDiagnostic>,
    default_name_fn: impl FnOnce(&str) -> String,
) -> String {
    if let Some(name) = ns_name_map.get(specifier) {
        return name.clone();
    }

    let base_name = default_name_fn(specifier);
    let used_names: FxHashSet<&str> = ns_name_map.values().map(String::as_str).collect();

    let name = if used_names.contains(base_name.as_str()) || reserved_names.contains(&base_name) {
        let mut i = 0;
        loop {
            let candidate = format!("{base_name}{i}");
            if !used_names.contains(candidate.as_str()) && !reserved_names.contains(&candidate) {
                warnings.push(OxcDiagnostic::warn(format!(
                    "typack/namespace-name-deconflict: renamed namespace helper \"{base_name}\" for \"{specifier}\" to avoid collision"
                )));
                break candidate;
            }
            i += 1;
        }
    } else {
        base_name
    };

    ns_name_map.insert(specifier.to_string(), name.clone());
    name
}
