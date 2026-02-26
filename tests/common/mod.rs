#![allow(dead_code)]

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use cow_utils::CowUtils;
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, IndentChar};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rustc_hash::FxHashMap;

pub struct TempProject {
    pub root: PathBuf,
}

impl TempProject {
    pub fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time should be after unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("typack_{name}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&root).expect("temp project directory should be created");
        Self { root }
    }

    pub fn write_file(&self, relative_path: &str, content: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }
        fs::write(path, content).expect("fixture file should be written");
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Normalize output for lenient comparison.
///
/// Strips metadata (file header, region markers), normalizes whitespace,
/// and removes non-JSDoc comments. JSDoc comments (`/** ... */`) and
/// triple-slash reference directives are preserved.
pub fn normalize_for_comparison(input: &str) -> String {
    let input = input.cow_replace("\r\n", "\n").into_owned();
    let mut lines: Vec<String> = Vec::new();
    let mut in_jsdoc = false;

    for line in input.lines() {
        let trimmed = line.trim();

        // Track JSDoc block comments
        if trimmed.starts_with("/**") {
            in_jsdoc = true;
        }
        if in_jsdoc {
            lines.push(normalize_jsdoc_line(line));
            if trimmed.contains("*/") {
                in_jsdoc = false;
            }
            continue;
        }

        // Skip file header comment (// filename.d.ts)
        if lines.is_empty() && trimmed.starts_with("// ") && trimmed.ends_with(".d.ts") {
            continue;
        }

        // Skip region markers
        if trimmed.starts_with("//#region") || trimmed == "//#endregion" {
            continue;
        }

        // Keep triple-slash reference directives
        if trimmed.starts_with("///") {
            lines.push(line.trim_end().to_string());
            continue;
        }

        // Skip other single-line comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Keep the line (trimming trailing whitespace)
        lines.push(normalize_type_colon_spacing(line.trim_end()));
    }

    // Collapse consecutive blank lines into one
    let mut result: Vec<String> = Vec::new();
    let mut prev_blank = false;
    for line in lines {
        if line.trim().is_empty() {
            if !prev_blank {
                result.push(String::new());
            }
            prev_blank = true;
        } else {
            result.push(line);
            prev_blank = false;
        }
    }

    // Trim leading/trailing blank lines
    while result.first().is_some_and(String::is_empty) {
        result.remove(0);
    }
    while result.last().is_some_and(String::is_empty) {
        result.pop();
    }

    result.join("\n")
}

fn normalize_jsdoc_line(line: &str) -> String {
    let trimmed_end = line.trim_end();
    let trimmed_start = trimmed_end.trim_start();

    if trimmed_start.starts_with('*') || trimmed_start.starts_with("*/") {
        format!(" {trimmed_start}")
    } else {
        trimmed_start.to_string()
    }
}

fn normalize_type_colon_spacing(line: &str) -> String {
    line.cow_replace("] :", "]:").into_owned()
}

/// Parse export specifiers from a line like `export { A, type B, C as D };`
/// into (local, exported) pairs. Strips per-specifier `type` modifier.
fn parse_export_specifiers(line: &str) -> Vec<(String, String)> {
    let rest = line
        .strip_prefix("export")
        .unwrap_or(line)
        .trim()
        .strip_prefix("type")
        .unwrap_or(line.strip_prefix("export").unwrap_or(line).trim())
        .trim();

    let rest = rest.strip_prefix('{').unwrap_or(rest);
    let rest = rest.strip_suffix(';').unwrap_or(rest).trim();
    let rest = rest.strip_suffix('}').unwrap_or(rest);

    let mut mappings = Vec::new();
    for part in rest.split(',') {
        let mut part = part.trim();
        if part.is_empty() {
            continue;
        }
        // Strip per-specifier `type` modifier
        if let Some(stripped) = part.strip_prefix("type ") {
            part = stripped.trim();
        }
        if let Some((local, exported)) = part.split_once(" as ") {
            mappings.push((local.trim().to_string(), exported.trim().to_string()));
        } else {
            mappings.push((part.to_string(), part.to_string()));
        }
    }
    mappings
}

/// Check if a line is a top-level export statement (not indented, not inside a namespace).
fn is_top_level_export(line: &str) -> bool {
    let trimmed = line.trim();
    // Must start at column 0 (no leading whitespace) to be top-level
    line.starts_with("export")
        && (trimmed.starts_with("export {") || trimmed.starts_with("export type {"))
}

/// Extract exported public names from top-level export statements.
/// Returns a sorted set of exported names (the public API).
fn extract_export_names(input: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in input.lines() {
        if !is_top_level_export(line) {
            continue;
        }
        for (_, exported) in parse_export_specifiers(line) {
            names.insert(exported);
        }
    }
    names
}

/// Extract top-level export local->exported mappings.
fn extract_export_mappings(input: &str) -> Vec<(String, String)> {
    let mut mappings = Vec::new();
    for line in input.lines() {
        if !is_top_level_export(line) {
            continue;
        }
        mappings.extend(parse_export_specifiers(line));
    }
    mappings
}

/// Build an identifier mapping from expected->actual based on shared exported names.
///
/// If expected has `export { Stuff$1 as Stuff }` and actual has `export { Stuff$2 as Stuff }`,
/// this builds the mapping `Stuff$1 -> Stuff$2`.
fn build_identifier_mapping(expected: &str, actual: &str) -> FxHashMap<String, String> {
    let exp_mappings = extract_export_mappings(expected);
    let act_mappings = extract_export_mappings(actual);

    let mut mapping = FxHashMap::default();

    for (exp_local, exp_exported) in &exp_mappings {
        for (act_local, act_exported) in &act_mappings {
            if exp_exported == act_exported && exp_local != act_local {
                mapping.insert(exp_local.clone(), act_local.clone());
                break;
            }
        }
    }

    mapping
}

/// Apply identifier mapping to text using simultaneous replacement (placeholder-based).
///
/// This avoids interference between replacements (e.g., swapping A<->B).
fn apply_identifier_mapping(text: &str, mapping: &FxHashMap<String, String>) -> String {
    if mapping.is_empty() {
        return text.to_string();
    }

    // Sort by length (longest first) to avoid partial replacements
    let mut pairs: Vec<(&str, &str)> =
        mapping.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    // Phase 1: replace all source names with unique placeholders
    let mut result = text.to_string();
    let mut placeholder_to_target: Vec<(String, String)> = Vec::new();
    for (i, (old, new)) in pairs.iter().enumerate() {
        let placeholder = format!("\x00PLACEHOLDER_{i}\x00");
        result = replace_word(&result, old, &placeholder);
        placeholder_to_target.push((placeholder, new.to_string()));
    }

    // Phase 2: replace placeholders with target names
    for (placeholder, target) in &placeholder_to_target {
        result = result.cow_replace(placeholder.as_str(), target).into_owned();
    }

    result
}

/// Replace whole-word occurrences of `old` with `new` in text.
fn replace_word(text: &str, old: &str, new: &str) -> String {
    let bytes = text.as_bytes();
    let old_bytes = old.as_bytes();
    let old_len = old_bytes.len();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if i + old_len <= len && &bytes[i..i + old_len] == old_bytes {
            // Check word boundary before
            let before_ok =
                i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            // Check word boundary after
            let after_pos = i + old_len;
            let after_ok = after_pos >= len
                || !(bytes[after_pos].is_ascii_alphanumeric()
                    || bytes[after_pos] == b'_'
                    || bytes[after_pos] == b'$');
            if before_ok && after_ok {
                result.push_str(new);
                i += old_len;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Split normalized text into body lines and export lines.
/// Body = everything except top-level `export { ... }` statements.
/// Export = only top-level `export { ... }` statements.
fn split_body_and_exports(text: &str) -> (Vec<String>, Vec<String>) {
    let mut body = Vec::new();
    let mut exports = Vec::new();
    for line in text.lines() {
        if is_top_level_export(line) {
            exports.push(line.to_string());
        } else {
            body.push(line.to_string());
        }
    }
    (body, exports)
}

fn contains_comment_or_reference_lines(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("///")
            || trimmed.starts_with("/**")
            || trimmed.starts_with('*')
            || trimmed.starts_with("*/")
    })
}

fn canonicalize_with_parser_and_codegen(text: &str) -> Option<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, SourceType::d_ts()).parse();
    if !parsed.errors.is_empty() {
        return None;
    }

    let code = Codegen::new()
        .with_options(CodegenOptions {
            indent_char: IndentChar::Space,
            indent_width: 2,
            ..CodegenOptions::default()
        })
        .build(&parsed.program)
        .code;
    Some(code.cow_replace("\r\n", "\n").into_owned())
}

/// Compare two outputs leniently.
///
/// Returns `Ok(())` if they match, or `Err(diff_message)` if they don't.
///
/// Comparison strategy:
/// 1. Normalize both outputs (strip metadata, whitespace, non-JSDoc comments)
/// 2. Check exported names match (the public API)
/// 3. Separate body from export statements
/// 4. Build identifier mapping from exports, apply only to body
/// 5. Compare body texts
pub fn lenient_compare(expected_raw: &str, actual_raw: &str) -> Result<(), String> {
    let expected = normalize_for_comparison(expected_raw);
    let actual = normalize_for_comparison(actual_raw);

    // Quick check: if they already match, no mapping needed
    if expected == actual {
        return Ok(());
    }

    // Check that the public API (exported names) matches
    let exp_exports = extract_export_names(&expected);
    let act_exports = extract_export_names(&actual);
    if exp_exports != act_exports {
        return Err(format!(
            "export mismatch:\n  expected: {exp_exports:?}\n  actual:   {act_exports:?}"
        ));
    }

    // Separate body from export statements
    let (exp_body, _) = split_body_and_exports(&expected);
    let (act_body, _) = split_body_and_exports(&actual);

    let exp_body_text = exp_body.join("\n");
    let act_body_text = act_body.join("\n");

    // If bodies already match, pass
    if exp_body_text == act_body_text {
        return Ok(());
    }

    // Build identifier mapping from export statements and apply to body only
    let mapping = build_identifier_mapping(&expected, &actual);
    let exp_body_mapped = apply_identifier_mapping(&exp_body_text, &mapping);

    if exp_body_mapped == act_body_text {
        return Ok(());
    }

    // Formatting-only fallback: compare canonicalized parser/codegen output.
    // Keep comment/reference-sensitive bodies strict.
    if !contains_comment_or_reference_lines(&exp_body_mapped)
        && !contains_comment_or_reference_lines(&act_body_text)
        && let (Some(exp_canonical), Some(act_canonical)) = (
            canonicalize_with_parser_and_codegen(&exp_body_mapped),
            canonicalize_with_parser_and_codegen(&act_body_text),
        )
        && exp_canonical == act_canonical
    {
        return Ok(());
    }

    // Find first difference for debugging (show mapped body comparison)
    let expected_lines: Vec<&str> = exp_body_mapped.lines().collect();
    let actual_lines: Vec<&str> = act_body_text.lines().collect();

    let first_diff = if let Some((i, (e, a))) =
        expected_lines.iter().zip(actual_lines.iter()).enumerate().find(|(_, (e, a))| e != a)
    {
        format!("line {}: expected {:?}, got {:?}", i + 1, e, a)
    } else {
        format!(
            "length mismatch: expected {} lines, got {} lines",
            expected_lines.len(),
            actual_lines.len()
        )
    };

    let mapping_str = if mapping.is_empty() {
        String::new()
    } else {
        format!("\n  (identifier mapping applied: {mapping:?})")
    };

    Err(format!("{first_diff}{mapping_str}"))
}
