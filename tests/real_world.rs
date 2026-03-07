use std::{
    fs,
    path::{Path, PathBuf},
};

use cow_utils::CowUtils;
use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, ExportDefaultDeclarationKind, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_sourcemap::SourceMap;
use oxc_span::SourceType;
use rustc_hash::FxHashSet;
use typack::{TypackBundler, TypackOptions};

/// Real-world snapshot tests.
///
/// These are full-size bundles from real projects (e.g. rolldown) that serve as
/// regression guards. Any change to tree-shaking, cross-module propagation,
/// alias resolution, or import rewriting will show up as a snapshot diff here.
///
/// Each subdirectory under `tests/real-world/` is a project fixture that can
/// contain one or more entry files:
///
/// - **Single entry**: `index.d.ts` + `snapshot.d.ts` (or `.d.mts` variants)
/// - **Multiple entries**: `<name>.d.ts` + `<name>.snapshot.d.ts` (or `.d.mts`)
///
/// Shared dependency files live in a `shared/` subdirectory and are excluded
/// from entry discovery.
#[test]
fn real_world() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let real_world_dir = crate_dir.join("tests").join("real-world");

    if !real_world_dir.exists() {
        return;
    }

    let mut dirs: Vec<PathBuf> = fs::read_dir(&real_world_dir)
        .unwrap()
        .filter_map(|e| {
            let path = e.unwrap().path();
            if path.is_dir() { Some(path) } else { None }
        })
        .collect();
    dirs.sort();

    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    for dir in &dirs {
        let project_name = dir.file_name().unwrap().to_string_lossy().to_string();

        for (entry, snapshot, fixture_name) in collect_entries(dir, &project_name) {
            let result = TypackBundler::bundle(&TypackOptions {
                input: vec![entry.to_string_lossy().to_string()],
                cwd: crate_dir.clone(),
                sourcemap: true,
                ..Default::default()
            });

            let bundle = match result {
                Ok(bundle) => bundle,
                Err(diagnostics) => {
                    failed += 1;
                    let msgs: Vec<String> =
                        diagnostics.iter().map(std::string::ToString::to_string).collect();
                    failures.push(format!("{fixture_name}: error: {}", msgs.join(", ")));
                    continue;
                }
            };
            let output = bundle.outputs.into_iter().next()
                .unwrap_or_else(|| panic!("{fixture_name}: should have at least one output"));
            let actual = output.code;

            // Validate source map
            let map =
                output.map.unwrap_or_else(|| panic!("{fixture_name}: sourcemap should be present"));
            if let Err(msg) = validate_sourcemap(&fixture_name, &actual, &map) {
                failed += 1;
                failures.push(msg);
                continue;
            }

            if let Err(msg) = validate_generated_declaration(&fixture_name, &entry, &actual) {
                failed += 1;
                failures.push(msg);
                continue;
            }

            // Snapshot comparison (optional — skip if no snapshot file exists)
            if !snapshot.exists() {
                passed += 1;
                continue;
            }

            let expected = fs::read_to_string(&snapshot)
                .unwrap_or_else(|e| panic!("Failed to read snapshot for {fixture_name}: {e}"));
            let expected_norm = expected.cow_replace("\r\n", "\n");
            let actual_norm = actual.cow_replace("\r\n", "\n");
            if actual_norm == expected_norm {
                passed += 1;
            } else {
                failed += 1;
                // Find first diff line for debugging
                let exp_lines: Vec<&str> = expected_norm.lines().collect();
                let act_lines: Vec<&str> = actual_norm.lines().collect();
                let first_diff = exp_lines
                    .iter()
                    .zip(act_lines.iter())
                    .enumerate()
                    .find(|(_, (e, a))| e != a)
                    .map_or_else(
                        || {
                            format!(
                                "length mismatch: expected {} lines, got {} lines",
                                exp_lines.len(),
                                act_lines.len()
                            )
                        },
                        |(i, (e, a))| format!("line {}: expected {:?}, got {:?}", i + 1, e, a),
                    );
                failures.push(format!("{fixture_name}:\n  {first_diff}"));
            }
        }
    }

    let total = passed + failed;
    assert!(
        failed == 0,
        "Real-world snapshot tests failed: {failed}/{total} fixtures failed\n{}",
        failures.join("\n")
    );
}

/// Collect entry/snapshot pairs from a project directory.
///
/// Finds all `<name>.d.ts` / `<name>.d.mts` files (excluding `*.snapshot.*`)
/// and pairs each with its snapshot `<name>.snapshot.d.ts` / `<name>.snapshot.d.mts`.
///
/// Returns `(entry_path, snapshot_path, display_name)` triples, sorted by name.
fn collect_entries(dir: &PathBuf, project_name: &str) -> Vec<(PathBuf, PathBuf, String)> {
    let mut entries: Vec<(PathBuf, PathBuf, String)> = Vec::new();

    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let path = e.unwrap().path();
            if path.is_file() { Some(path) } else { None }
        })
        .collect();
    files.sort();

    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy();

        // Skip snapshot files and non-declaration files
        if name.contains(".snapshot.") {
            continue;
        }

        let (stem, ext) = if let Some(s) = name.strip_suffix(".d.mts") {
            (s, "d.mts")
        } else if let Some(s) = name.strip_suffix(".d.ts") {
            (s, "d.ts")
        } else {
            continue;
        };

        let snapshot = dir.join(format!("{stem}.snapshot.{ext}"));
        let display = format!("{project_name}/{stem}");
        entries.push((file.clone(), snapshot, display));
    }

    entries
}

/// Validate that the generated declaration parses, passes semantic checks, and
/// does not retain dead top-level bindings.
fn validate_generated_declaration(
    fixture_name: &str,
    entry: &Path,
    code: &str,
) -> Result<(), String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(entry).map_err(|err| {
        format!("{fixture_name}: failed to infer source type for {}: {err}", entry.display())
    })?;

    let parsed = Parser::new(&allocator, code, source_type).parse();
    if !parsed.errors.is_empty() {
        let messages: Vec<String> =
            parsed.errors.iter().map(std::string::ToString::to_string).collect();
        return Err(format!(
            "{fixture_name}: generated declaration has parse errors: {}",
            messages.join(", ")
        ));
    }

    let semantic = SemanticBuilder::new().with_check_syntax_error(true).build(&parsed.program);
    if !semantic.errors.is_empty() {
        let messages: Vec<String> =
            semantic.errors.iter().map(std::string::ToString::to_string).collect();
        return Err(format!(
            "{fixture_name}: generated declaration has semantic errors: {}",
            messages.join(", ")
        ));
    }

    let exported_locals = collect_exported_local_names(&parsed.program.body);
    let scoping = semantic.semantic.scoping();
    let root_bindings: FxHashSet<&str> = scoping
        .get_bindings(scoping.root_scope_id())
        .into_iter()
        .map(|(name, _)| name.as_str())
        .collect();

    // Check 1: exported names must exist as root-scope bindings.
    let mut dangling_exports: Vec<String> = exported_locals
        .iter()
        .filter(|name| !root_bindings.contains(name.as_str()))
        .cloned()
        .collect();
    dangling_exports.sort();

    if !dangling_exports.is_empty() {
        return Err(format!(
            "{fixture_name}: export list references names not in scope: {}",
            dangling_exports.join(", ")
        ));
    }

    // Check 2: root-scope bindings must be exported (no dead declarations).
    let mut unused_root_bindings = Vec::new();
    for (name, &symbol_id) in scoping.get_bindings(scoping.root_scope_id()) {
        if exported_locals.contains(name.as_str()) {
            continue;
        }
        if scoping.symbol_is_unused(symbol_id) {
            unused_root_bindings.push(name.to_string());
        }
    }
    unused_root_bindings.sort();

    if !unused_root_bindings.is_empty() {
        return Err(format!(
            "{fixture_name}: generated declaration retains unused root bindings: {}",
            unused_root_bindings.join(", ")
        ));
    }

    // Check 3: type references must resolve (no references to undeclared names).
    let mut unresolved_names: Vec<String> = scoping
        .root_unresolved_references()
        .keys()
        .filter(|name| !is_known_global(name.as_str()))
        .map(|name| name.as_str().to_string())
        .collect();
    unresolved_names.sort();

    if !unresolved_names.is_empty() {
        return Err(format!(
            "{fixture_name}: generated declaration has unresolved references: {}",
            unresolved_names.join(", ")
        ));
    }

    Ok(())
}

fn collect_exported_local_names(body: &[Statement<'_>]) -> FxHashSet<String> {
    let mut exported = FxHashSet::default();

    for stmt in body {
        match stmt {
            Statement::ExportNamedDeclaration(decl) => {
                if let Some(declaration) = &decl.declaration {
                    collect_decl_names(declaration, &mut exported);
                } else if decl.source.is_none() {
                    for specifier in &decl.specifiers {
                        exported.insert(specifier.local.name().to_string());
                    }
                }
            }
            Statement::ExportDefaultDeclaration(decl) => match &decl.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                    if let Some(id) = &func.id {
                        exported.insert(id.name.to_string());
                    }
                }
                ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                    if let Some(id) = &class.id {
                        exported.insert(id.name.to_string());
                    }
                }
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => {
                    exported.insert(iface.id.name.to_string());
                }
                ExportDefaultDeclarationKind::Identifier(id) => {
                    exported.insert(id.name.to_string());
                }
                _ => {}
            },
            _ => {}
        }
    }

    exported
}

fn collect_decl_names(decl: &Declaration<'_>, names: &mut FxHashSet<String>) {
    match decl {
        Declaration::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                    names.insert(id.name.to_string());
                }
            }
        }
        Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                names.insert(id.name.to_string());
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                names.insert(id.name.to_string());
            }
        }
        Declaration::TSTypeAliasDeclaration(alias) => {
            names.insert(alias.id.name.to_string());
        }
        Declaration::TSInterfaceDeclaration(iface) => {
            names.insert(iface.id.name.to_string());
        }
        Declaration::TSEnumDeclaration(enum_decl) => {
            names.insert(enum_decl.id.name.to_string());
        }
        Declaration::TSModuleDeclaration(module_decl) => {
            if let oxc_ast::ast::TSModuleDeclarationName::Identifier(id) = &module_decl.id {
                names.insert(id.name.to_string());
            }
        }
        Declaration::TSGlobalDeclaration(_) | Declaration::TSImportEqualsDeclaration(_) => {}
    }
}

/// Validate structural properties and semantic correctness of a source map.
///
/// Returns `Err(message)` on the first violation found.
fn validate_sourcemap(fixture_name: &str, code: &str, map: &SourceMap) -> Result<(), String> {
    // Must have at least one source file
    let sources: Vec<&str> = map.get_sources().map(AsRef::as_ref).collect();
    if sources.is_empty() {
        return Err(format!("{fixture_name}: sourcemap has no sources"));
    }

    // All source paths must be relative
    for source in &sources {
        if source.starts_with('/') {
            return Err(format!(
                "{fixture_name}: sourcemap source should be relative, got: {source}"
            ));
        }
    }

    // Every source must have non-empty sourcesContent
    let source_contents: Vec<Option<&str>> =
        map.get_source_contents().map(|c| c.map(AsRef::as_ref)).collect();
    for (i, content) in source_contents.iter().enumerate() {
        match content {
            None => {
                return Err(format!(
                    "{fixture_name}: sourcesContent[{i}] is missing for source {:?}",
                    sources.get(i)
                ));
            }
            Some("") => {
                return Err(format!(
                    "{fixture_name}: sourcesContent[{i}] is empty for source {:?}",
                    sources.get(i)
                ));
            }
            _ => {}
        }
    }

    // Pre-compute source line info for bounds checking
    let source_lines: Vec<Vec<&str>> =
        source_contents.iter().map(|c| c.map_or_else(Vec::new, |s| s.lines().collect())).collect();

    // Must have mapped tokens
    let token_count = map.get_tokens().count();
    if token_count == 0 {
        return Err(format!("{fixture_name}: sourcemap has no tokens"));
    }

    // Validate every token
    let gen_lines: Vec<&str> = code.lines().collect();
    let gen_line_count = u32::try_from(gen_lines.len()).unwrap_or(u32::MAX);

    for token in map.get_tokens() {
        let dst_line = token.get_dst_line();
        let dst_col = token.get_dst_col();

        // dst_line must be within the generated output
        if dst_line >= gen_line_count {
            return Err(format!(
                "{fixture_name}: token dst_line {dst_line} >= generated line count {gen_line_count}"
            ));
        }

        // dst_col must not exceed the generated line length
        let gen_line_len = u32::try_from(gen_lines[dst_line as usize].len()).unwrap_or(u32::MAX);
        if dst_col > gen_line_len {
            return Err(format!(
                "{fixture_name}: token dst_col {dst_col} > line {dst_line} length {gen_line_len}"
            ));
        }

        // If the token has a source, validate src_line and src_col
        if let Some(source_id) = token.get_source_id() {
            let src_line = token.get_src_line();
            let src_col = token.get_src_col();
            let lines = &source_lines[source_id as usize];

            if src_line as usize >= lines.len() {
                return Err(format!(
                    "{fixture_name}: token src_line {src_line} >= source {:?} line count {}",
                    sources.get(source_id as usize),
                    lines.len()
                ));
            }

            let src_line_len = u32::try_from(lines[src_line as usize].len()).unwrap_or(u32::MAX);
            if src_col > src_line_len {
                return Err(format!(
                    "{fixture_name}: token src_col {src_col} > source {:?} line {src_line} length {src_line_len}",
                    sources.get(source_id as usize)
                ));
            }
        }
    }

    // Synthetic lines (region markers, export statements, import statements)
    // must NOT have any sourcemap mappings.
    let mapped_lines: FxHashSet<u32> = map
        .get_tokens()
        .filter(|t| t.get_source_id().is_some())
        .map(|t| t.get_dst_line())
        .collect();

    for (i, line) in gen_lines.iter().enumerate() {
        let Ok(line_index) = u32::try_from(i) else {
            return Err(format!("{fixture_name}: generated line index {i} exceeds u32 range"));
        };
        let trimmed = line.trim();
        let is_synthetic = trimmed.starts_with("//#region")
            || trimmed.starts_with("//#endregion")
            || trimmed.starts_with("export {")
            || trimmed.starts_with("export =");
        if is_synthetic && mapped_lines.contains(&line_index) {
            return Err(format!(
                "{fixture_name}: synthetic line {i} has sourcemap mapping: {trimmed:?}"
            ));
        }
    }

    // JSON round-trip: serialise -> re-parse -> compare token count
    let json = map.to_json_string();
    let reparsed = SourceMap::from_json_string(&json)
        .map_err(|e| format!("{fixture_name}: sourcemap JSON round-trip parse failed: {e}"))?;
    let reparsed_token_count = reparsed.get_tokens().count();
    if reparsed_token_count != token_count {
        return Err(format!(
            "{fixture_name}: JSON round-trip token count mismatch: {token_count} -> {reparsed_token_count}"
        ));
    }

    Ok(())
}

/// Known TypeScript, DOM, and Node.js globals that are legitimate unresolved references
/// in generated declaration files. Extend as needed when new fixtures surface globals.
const KNOWN_GLOBALS: &[&str] = &[
    // TS utility types
    "Array",
    "ArrayLike",
    "AsyncGenerator",
    "AsyncIterable",
    "AsyncIterableIterator",
    "AsyncIterator",
    "Awaited",
    "Capitalize",
    "ConstructorParameters",
    "Disposable",
    "Exclude",
    "Extract",
    "Generator",
    "InstanceType",
    "Iterable",
    "IterableIterator",
    "Iterator",
    "Lowercase",
    "Map",
    "NoInfer",
    "NonNullable",
    "Omit",
    "Parameters",
    "Partial",
    "Pick",
    "Promise",
    "PromiseLike",
    "PropertyKey",
    "Readonly",
    "ReadonlyArray",
    "ReadonlyMap",
    "ReadonlySet",
    "Record",
    "Required",
    "ReturnType",
    "Set",
    "TemplateStringsArray",
    "ThisParameterType",
    "ThisType",
    "Uncapitalize",
    "Uppercase",
    "WeakMap",
    "WeakRef",
    "WeakSet",
    // TS primitive / special types
    "ArrayConstructor",
    "BigInt",
    "Boolean",
    "BooleanConstructor",
    "Date",
    "DateConstructor",
    "Error",
    "Function",
    "FunctionConstructor",
    "JSON",
    "MapConstructor",
    "Math",
    "Number",
    "Object",
    "ObjectConstructor",
    "Proxy",
    "ProxyHandler",
    "Reflect",
    "RegExp",
    "RegExpConstructor",
    "RangeError",
    "String",
    "Symbol",
    "SyntaxError",
    "TypeError",
    // Typed arrays
    "Uint8Array",
    // DOM types
    "AbortController",
    "AbortSignal",
    "AnimationEvent",
    "Attr",
    "Blob",
    "CSSStyleDeclaration",
    "ClipboardEvent",
    "Comment",
    "CompositionEvent",
    "CustomEvent",
    "DOMTokenList",
    "Document",
    "DocumentFragment",
    "DragEvent",
    "Element",
    "Event",
    "EventTarget",
    "File",
    "FocusEvent",
    "FormData",
    "FormDataEvent",
    "HTMLCollection",
    "HTMLElement",
    "HTMLElementEventMap",
    "HTMLElementTagNameMap",
    "HTMLInputElement",
    "HTMLSelectElement",
    "HTMLTextAreaElement",
    "Headers",
    "History",
    "InputEvent",
    "IntersectionObserver",
    "IntersectionObserverInit",
    "KeyboardEvent",
    "Location",
    "MediaEncryptedEvent",
    "MouseEvent",
    "MutationObserver",
    "NamedNodeMap",
    "Navigator",
    "Node",
    "NodeList",
    "Performance",
    "PointerEvent",
    "ProgressEvent",
    "ReadableStream",
    "Request",
    "ResizeObserver",
    "Response",
    "SVGElement",
    "SecurityPolicyViolationEvent",
    "ShadowRoot",
    "ShadowRootInit",
    "Storage",
    "SubmitEvent",
    "Text",
    "ToggleEvent",
    "TouchEvent",
    "TransitionEvent",
    "UIEvent",
    "URL",
    "URLSearchParams",
    "WheelEvent",
    "Window",
    "WritableStream",
    // Node.js / runtime globals
    "Buffer",
    "NodeJS",
    "clearInterval",
    "clearTimeout",
    "console",
    "globalThis",
    "process",
    "queueMicrotask",
    "setInterval",
    "setTimeout",
    // Third-party / test framework globals
    "Chai",
    "jest",
    // Special
    "Infinity",
    "NaN",
    "never",
    "undefined",
    "void",
];

fn is_known_global(name: &str) -> bool {
    KNOWN_GLOBALS.contains(&name)
}
