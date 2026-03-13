//! IsolatedDeclarations-related config extracted from tsconfig.json.

use std::path::{Path, PathBuf};

/// IsolatedDeclarations-related config extracted from tsconfig.json.
pub struct IsolatedDeclarationsConfig {
    pub isolated_declarations: bool,
    pub strip_internal: bool,
}

impl IsolatedDeclarationsConfig {
    /// Load config from an explicit tsconfig.json path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::parse(&content))
    }

    /// Find tsconfig.json by walking up from `start_dir` and extract relevant fields.
    /// Returns `None` if no tsconfig.json is found.
    pub fn find(start_dir: &Path) -> Option<Self> {
        let tsconfig_path = find_tsconfig(start_dir)?;
        let content = std::fs::read_to_string(&tsconfig_path).ok()?;
        Some(Self::parse(&content))
    }

    /// Parse tsconfig.json content (with comments stripped) to extract relevant fields.
    fn parse(content: &str) -> Self {
        let stripped = strip_json_comments(content);
        // Scope the search to the "compilerOptions" block to avoid false positives
        // from other top-level keys.
        let compiler_options = extract_compiler_options(&stripped);
        let isolated_declarations =
            find_bool_field(compiler_options, "isolatedDeclarations").unwrap_or(false);
        let strip_internal = find_bool_field(compiler_options, "stripInternal").unwrap_or(false);
        Self { isolated_declarations, strip_internal }
    }
}

/// Walk up from `start_dir` looking for tsconfig.json.
fn find_tsconfig(start_dir: &Path) -> Option<PathBuf> {
    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join("tsconfig.json");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Strip single-line (`//`) and multi-line (`/* */`) comments from JSON-like text.
/// Also handles trailing commas before `}` or `]`.
fn strip_json_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;
    let mut in_string = false;

    while i < len {
        if in_string {
            result.push(bytes[i] as char);
            if bytes[i] == b'\\' && i + 1 < len {
                i += 1;
                result.push(bytes[i] as char);
            } else if bytes[i] == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if bytes[i] == b'"' {
            in_string = true;
            result.push('"');
            i += 1;
            continue;
        }

        // Single-line comment
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // Skip to end of line
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Multi-line comment
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < len {
                i += 2; // skip */
            }
            continue;
        }

        result.push(bytes[i] as char);
        i += 1;
    }

    // Remove trailing commas before } or ]
    let mut cleaned = String::with_capacity(result.len());
    let result_bytes = result.as_bytes();
    let rlen = result_bytes.len();
    let mut j = 0;
    while j < rlen {
        if result_bytes[j] == b',' {
            // Look ahead for only whitespace followed by } or ]
            let mut k = j + 1;
            while k < rlen
                && (result_bytes[k] == b' '
                    || result_bytes[k] == b'\n'
                    || result_bytes[k] == b'\r'
                    || result_bytes[k] == b'\t')
            {
                k += 1;
            }
            if k < rlen && (result_bytes[k] == b'}' || result_bytes[k] == b']') {
                // Skip this trailing comma
                j += 1;
                continue;
            }
        }
        cleaned.push(result_bytes[j] as char);
        j += 1;
    }

    cleaned
}

/// Extract the `"compilerOptions": { ... }` block from JSON text.
/// Returns the substring containing the block's contents, or the full
/// input if `compilerOptions` is not found (graceful fallback).
fn extract_compiler_options(json: &str) -> &str {
    let key = "\"compilerOptions\"";
    let Some(pos) = json.find(key) else {
        return json;
    };
    let after_key = &json[pos + key.len()..];
    let after_key = after_key.trim_start();
    let Some(after_colon) = after_key.strip_prefix(':') else {
        return json;
    };
    let after_colon = after_colon.trim_start();
    let Some(block_start) = after_colon.strip_prefix('{') else {
        return json;
    };
    // Find the matching closing brace, respecting nesting
    let mut depth = 1u32;
    let mut in_string = false;
    for (i, b) in block_start.bytes().enumerate() {
        if in_string {
            if b == b'\\' {
                // skip next char handled by the fact that \ is not " or {/}
                continue;
            }
            if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &block_start[..i];
                }
            }
            _ => {}
        }
    }
    block_start
}

/// Find a boolean field value in JSON text by field name.
fn find_bool_field(json: &str, field_name: &str) -> Option<bool> {
    let pattern = format!("\"{field_name}\"");
    let pos = json.find(&pattern)?;
    let after = &json[pos + pattern.len()..];
    let after = after.trim_start();
    let after = after.strip_prefix(':')?;
    let after = after.trim_start();
    if after.starts_with("true") {
        Some(true)
    } else if after.starts_with("false") {
        Some(false)
    } else {
        None
    }
}
