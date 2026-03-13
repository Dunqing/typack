//! Scan stage: parses `.d.ts` and `.ts` entry files, discovers transitive imports,
//! and builds the module graph in topological order.

mod collectors;
mod graph;
mod resolution;
mod tsconfig;

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexVec;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::options::TypackOptions;
use crate::types::{AstTable, Module, ModuleIdx, ModuleTable};

use self::collectors::{
    ModuleDependencyHints, collect_export_import_info, collect_leading_reference_directives,
    collect_module_dependency_hints, has_top_level_augmentation,
};
use self::graph::{PendingInternalEdges, insert_pending_internal_edge, topological_sort};
use self::resolution::{
    ResolvedSpecifier, create_dts_resolver, maybe_push_resolution_warning,
    resolve_specifier_for_scan,
};
use self::tsconfig::IsolatedDeclarationsConfig;

/// Result of scanning: the module graph.
pub struct ScanStageOutput<'a> {
    /// All discovered modules, indexed by `ModuleIdx`.
    pub module_table: ModuleTable<'a>,
    /// Parsed ASTs for each module, indexed by `ModuleIdx`.
    pub ast_table: AstTable<'a>,
    /// All entry module indices.
    pub entry_points: Vec<ModuleIdx>,
    /// Non-fatal warnings collected during resolution/classification.
    pub warnings: Vec<OxcDiagnostic>,
}

/// The scan stage: parses `.d.ts` and `.ts` files, resolves imports, builds module graph.
pub struct ScanStage<'a, 'opt> {
    options: &'opt TypackOptions,
    allocator: &'a Allocator,
    id_config: Option<IsolatedDeclarationsConfig>,
}

impl<'a, 'opt> ScanStage<'a, 'opt> {
    /// # Errors
    ///
    /// Returns an error if an explicit `tsconfig` path is provided but cannot be read.
    pub fn new(
        options: &'opt TypackOptions,
        allocator: &'a Allocator,
    ) -> Result<Self, Vec<OxcDiagnostic>> {
        let id_config = if let Some(tsconfig_path) = &options.tsconfig {
            let resolved = if tsconfig_path.is_relative() {
                options.cwd.join(tsconfig_path)
            } else {
                tsconfig_path.clone()
            };
            Some(IsolatedDeclarationsConfig::load(&resolved).map_err(|e| {
                vec![OxcDiagnostic::error(format!(
                    "Cannot read tsconfig {}: {e}",
                    resolved.display()
                ))]
            })?)
        } else {
            IsolatedDeclarationsConfig::find(&options.cwd)
        };
        Ok(Self { options, allocator, id_config })
    }

    /// Run the scan stage: parse entry files, discover imports, build module table.
    ///
    /// # Errors
    ///
    /// Returns `Err` with diagnostics if entry files cannot be read or parsed.
    pub fn scan(&self) -> Result<ScanStageOutput<'a>, Vec<OxcDiagnostic>> {
        let mut modules: IndexVec<ModuleIdx, Module<'a>> = IndexVec::new();
        let mut ast_table = AstTable::new();
        // Map canonical path → module index for deduplication
        let mut path_to_idx: FxHashMap<PathBuf, ModuleIdx> = FxHashMap::default();
        // Per-module dependency hints collected once during parse.
        let mut module_hints: FxHashMap<ModuleIdx, ModuleDependencyHints> = FxHashMap::default();
        // Per-module internal edges (specifier -> canonical path), collected before final ModuleIdx remap.
        let mut pending_internal_edges: PendingInternalEdges = FxHashMap::default();
        let resolver = create_dts_resolver();
        let forced_external: FxHashSet<String> = self.options.external.iter().cloned().collect();
        let mut resolution_diagnostics: Vec<OxcDiagnostic> = Vec::new();
        let mut warnings: Vec<OxcDiagnostic> = Vec::new();
        let mut warning_dedup: FxHashSet<(String, String, String)> = FxHashSet::default();

        let mut explicit_internal_paths: FxHashSet<PathBuf> = FxHashSet::default();
        let mut entry_indices: Vec<ModuleIdx> = Vec::new();
        let mut queue: VecDeque<ModuleIdx> = VecDeque::new();
        let mut visited: FxHashSet<ModuleIdx> = FxHashSet::default();
        let mut warned_ts_without_id = false;

        if self.options.input.is_empty() {
            return Err(vec![OxcDiagnostic::error("No entry points specified")]);
        }

        for entry in &self.options.input {
            let entry_path = Self::resolve_entry_path(entry)?;
            if !warned_ts_without_id && !is_declaration_file(&entry_path) {
                warned_ts_without_id =
                    Self::maybe_warn_ts_without_id(self.id_config.as_ref(), &mut warnings);
            }
            explicit_internal_paths.insert(entry_path.clone());

            let entry_idx = self.add_module(
                &entry_path,
                true,
                &mut modules,
                &mut ast_table,
                &mut path_to_idx,
                &mut module_hints,
            )?;
            if visited.insert(entry_idx) {
                queue.push_back(entry_idx);
            }
            entry_indices.push(entry_idx);
        }

        while let Some(idx) = queue.pop_front() {
            // Every module in the BFS queue was added via add_module, which populates module_hints.
            let hints = module_hints
                .get(&idx)
                .expect("module_hints populated for every module in BFS queue");
            let eager_specifiers = hints.eager.clone();
            let side_effect_specifiers = hints.side_effect.clone();
            let importer_path = modules[idx].path.clone();

            for specifier in &eager_specifiers {
                let resolved = resolve_specifier_for_scan(
                    &resolver,
                    &importer_path,
                    specifier,
                    &forced_external,
                    &explicit_internal_paths,
                );
                match resolved {
                    Ok(ResolvedSpecifier::External(reason)) => {
                        modules[idx].resolved_external_specifiers.insert(specifier.clone());
                        maybe_push_resolution_warning(
                            &mut warnings,
                            &mut warning_dedup,
                            &importer_path,
                            specifier,
                            reason,
                        );
                    }
                    Ok(ResolvedSpecifier::Internal(canonical)) => {
                        if !warned_ts_without_id && !is_declaration_file(&canonical) {
                            warned_ts_without_id = Self::maybe_warn_ts_without_id(
                                self.id_config.as_ref(),
                                &mut warnings,
                            );
                        }
                        let dep_idx = if let Some(&existing) = path_to_idx.get(&canonical) {
                            existing
                        } else {
                            self.add_module(
                                &canonical,
                                false,
                                &mut modules,
                                &mut ast_table,
                                &mut path_to_idx,
                                &mut module_hints,
                            )?
                        };
                        insert_pending_internal_edge(
                            &mut pending_internal_edges,
                            idx,
                            specifier,
                            canonical,
                        );
                        if visited.insert(dep_idx) {
                            queue.push_back(dep_idx);
                        }
                    }
                    Ok(ResolvedSpecifier::Skipped) => {}
                    Err(diagnostic) => resolution_diagnostics.push(diagnostic),
                }
            }

            // Side-effect imports are included only for modules with augmentations.
            for specifier in &side_effect_specifiers {
                let resolved = resolve_specifier_for_scan(
                    &resolver,
                    &importer_path,
                    specifier,
                    &forced_external,
                    &explicit_internal_paths,
                );
                match resolved {
                    Ok(ResolvedSpecifier::External(reason)) => {
                        modules[idx].resolved_external_specifiers.insert(specifier.clone());
                        maybe_push_resolution_warning(
                            &mut warnings,
                            &mut warning_dedup,
                            &importer_path,
                            specifier,
                            reason,
                        );
                    }
                    Ok(ResolvedSpecifier::Internal(canonical)) => {
                        if !warned_ts_without_id && !is_declaration_file(&canonical) {
                            warned_ts_without_id = Self::maybe_warn_ts_without_id(
                                self.id_config.as_ref(),
                                &mut warnings,
                            );
                        }
                        let dep_idx = if let Some(&existing) = path_to_idx.get(&canonical) {
                            existing
                        } else {
                            self.add_module(
                                &canonical,
                                false,
                                &mut modules,
                                &mut ast_table,
                                &mut path_to_idx,
                                &mut module_hints,
                            )?
                        };

                        if modules[dep_idx].has_augmentation {
                            insert_pending_internal_edge(
                                &mut pending_internal_edges,
                                idx,
                                specifier,
                                canonical,
                            );
                            if visited.insert(dep_idx) {
                                queue.push_back(dep_idx);
                            }
                        }
                    }
                    Ok(ResolvedSpecifier::Skipped) => {}
                    Err(diagnostic) => resolution_diagnostics.push(diagnostic),
                }
            }
        }

        if !resolution_diagnostics.is_empty() {
            return Err(resolution_diagnostics);
        }

        // Topological sort: dependencies before dependents
        let topo_result = topological_sort(&path_to_idx, &pending_internal_edges, &entry_indices);

        // Emit warnings for detected circular dependencies.
        for cycle in &topo_result.cycles {
            let cycle_paths: Vec<&str> = cycle
                .iter()
                .filter_map(|idx| modules.get(*idx))
                .map(|m| m.path.to_str().unwrap_or("<unknown>"))
                .collect();
            warnings.push(
                OxcDiagnostic::warn(format!(
                    "Circular dependency detected involving: {}",
                    cycle_paths.join(" → ")
                ))
                .with_help(
                    "Circular .d.ts dependencies may cause incomplete types in the bundled output",
                ),
            );
        }

        let mut old_to_new: FxHashMap<ModuleIdx, ModuleIdx> = FxHashMap::default();
        for (new_idx, old_idx) in topo_result.order.iter().copied().enumerate() {
            old_to_new.insert(old_idx, ModuleIdx::from_usize(new_idx));
        }

        // Rebuild modules and ast_table in topological order.
        // Convert IndexVec to Vec to allow taking ownership of individual elements.
        let mut module_vec: Vec<Option<Module<'a>>> = modules.into_iter().map(Some).collect();
        let mut ast_vec: Vec<Option<oxc_ast::ast::Program<'a>>> =
            ast_table.into_raw_vec().into_iter().map(Some).collect();
        let mut sorted_modules: IndexVec<ModuleIdx, Module<'a>> = IndexVec::new();
        let mut sorted_ast: IndexVec<ModuleIdx, oxc_ast::ast::Program<'a>> = IndexVec::new();
        let mut entry_points = Vec::new();
        for old_idx in topo_result.order {
            // Safe: topological_sort visits each index exactly once via its visited set.
            let mut module = module_vec[old_idx.index()].take().unwrap();
            let program = ast_vec[old_idx.index()].take().unwrap();
            let new_idx = ModuleIdx::from_usize(sorted_modules.len());

            let mut resolved_internal_specifiers: FxHashMap<String, ModuleIdx> =
                FxHashMap::default();
            if let Some(edges) = pending_internal_edges.remove(&old_idx) {
                for (specifier, canonical) in edges {
                    if let Some(dep_old_idx) = path_to_idx.get(&canonical)
                        && let Some(dep_new_idx) = old_to_new.get(dep_old_idx)
                    {
                        resolved_internal_specifiers.insert(specifier, *dep_new_idx);
                    }
                }
            }

            module.idx = new_idx;
            module.resolved_internal_specifiers = resolved_internal_specifiers;

            let pushed_idx = sorted_modules.push(module);
            debug_assert_eq!(pushed_idx, new_idx);
            sorted_ast.push(program);
        }

        let mut seen_entries = FxHashSet::default();
        for old_entry_idx in entry_indices {
            if let Some(&new_entry_idx) = old_to_new.get(&old_entry_idx)
                && seen_entries.insert(new_entry_idx)
            {
                entry_points.push(new_entry_idx);
            }
        }

        if entry_points.is_empty() {
            return Err(vec![OxcDiagnostic::error("No entry modules found after scan")]);
        }

        Ok(ScanStageOutput {
            module_table: ModuleTable::from_raw_parts(sorted_modules),
            ast_table: AstTable::from_raw_parts(sorted_ast),
            entry_points,
            warnings,
        })
    }

    /// Add a module to the table, returning its index.
    fn add_module(
        &self,
        path: &Path,
        is_entry: bool,
        modules: &mut IndexVec<ModuleIdx, Module<'a>>,
        ast_table: &mut AstTable<'a>,
        path_to_idx: &mut FxHashMap<PathBuf, ModuleIdx>,
        module_hints: &mut FxHashMap<ModuleIdx, ModuleDependencyHints>,
    ) -> Result<ModuleIdx, Vec<OxcDiagnostic>> {
        if let Some(&idx) = path_to_idx.get(path) {
            if is_entry {
                modules[idx].is_entry = true;
            }
            return Ok(idx);
        }

        let file_contents = fs::read_to_string(path).map_err(|e| {
            vec![OxcDiagnostic::error(format!("Cannot read {}: {e}", path.display()))]
        })?;

        let (source, program) = self.parse_module(path, &file_contents)?;

        // Run semantic analysis and extract scoping information.
        let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

        let mut hints = collect_module_dependency_hints(&program);
        let resolved_external_specifiers = std::mem::take(&mut hints.external);

        let reference_directives = collect_leading_reference_directives(source, &program);
        let has_augmentation = has_top_level_augmentation(&program);
        let export_import_info = collect_export_import_info(&program);
        let relative_path = self.relative_path(path);

        // Try to load an adjacent .d.ts.map for sourcemap composition
        let input_sourcemap = if self.options.sourcemap {
            let map_path = PathBuf::from(format!("{}.map", path.display()));
            fs::read_to_string(&map_path)
                .ok()
                .and_then(|s| oxc_sourcemap::SourceMap::from_json_string(&s).ok())
        } else {
            None
        };

        let idx = ModuleIdx::from_usize(modules.len());
        let module_idx = modules.push(Module {
            idx,
            path: path.to_path_buf(),
            relative_path,
            source,
            scoping,
            reference_directives,
            has_augmentation,
            resolved_internal_specifiers: FxHashMap::default(),
            resolved_external_specifiers,
            is_entry,
            input_sourcemap,
            export_import_info,
        });
        let ast_idx = ast_table.push(program);
        debug_assert_eq!(ast_idx, module_idx);
        path_to_idx.insert(path.to_path_buf(), module_idx);
        module_hints.insert(module_idx, hints);
        Ok(module_idx)
    }

    /// Parse a module file, applying IsolatedDeclarations transform for `.ts` files.
    /// Returns `(source_text, program)` where `source_text` is always `.d.ts` content.
    fn parse_module(
        &self,
        path: &Path,
        file_contents: &str,
    ) -> Result<(&'a str, oxc_ast::ast::Program<'a>), Vec<OxcDiagnostic>> {
        if is_declaration_file(path) {
            let source: &'a str = self.allocator.alloc_str(file_contents);
            let parsed = Parser::new(self.allocator, source, SourceType::d_ts()).parse();
            if !parsed.errors.is_empty() {
                return Err(parsed.errors);
            }
            Ok((source, parsed.program))
        } else {
            // .ts file: parse as TS, transform via IsolatedDeclarations, re-parse as .d.ts
            let source: &'a str = self.allocator.alloc_str(file_contents);
            let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::d_ts());
            let parsed = Parser::new(self.allocator, source, source_type).parse();
            if !parsed.errors.is_empty() {
                return Err(parsed.errors);
            }

            let strip_internal = self.id_config.as_ref().is_some_and(|c| c.strip_internal);
            let id_ret = IsolatedDeclarations::new(
                self.allocator,
                IsolatedDeclarationsOptions { strip_internal },
            )
            .build(&parsed.program);
            if !id_ret.errors.is_empty() {
                return Err(id_ret.errors);
            }

            // Codegen to .d.ts text, then re-parse for correct spans
            let dts_code = Codegen::new().build(&id_ret.program).code;
            let dts_source: &'a str = self.allocator.alloc_str(&dts_code);
            let dts_parsed = Parser::new(self.allocator, dts_source, SourceType::d_ts()).parse();
            if !dts_parsed.errors.is_empty() {
                return Err(dts_parsed.errors);
            }
            Ok((dts_source, dts_parsed.program))
        }
    }

    /// Emit a warning if `isolatedDeclarations` is not enabled in tsconfig.
    /// Returns `true` so the caller can set the dedup flag.
    fn maybe_warn_ts_without_id(
        id_config: Option<&IsolatedDeclarationsConfig>,
        warnings: &mut Vec<OxcDiagnostic>,
    ) -> bool {
        let id_enabled = id_config.is_some_and(|c| c.isolated_declarations);
        if !id_enabled {
            warnings.push(OxcDiagnostic::warn(
                "Using .ts file without isolatedDeclarations enabled in tsconfig.json \
                 — transform may produce errors",
            ));
        }
        true
    }

    fn resolve_entry_path(entry: &str) -> Result<PathBuf, Vec<OxcDiagnostic>> {
        let entry_path = PathBuf::from(entry);
        // Try canonicalize first (resolves symlinks), fall back to checking
        // existence directly for platforms that don't support it (e.g. WASI).
        match entry_path.canonicalize() {
            Ok(p) => Ok(p),
            Err(_) if entry_path.exists() => {
                // Normalize to an absolute path so deduplication works correctly.
                if entry_path.is_absolute() {
                    Ok(entry_path)
                } else {
                    match std::env::current_dir() {
                        Ok(cwd) => Ok(cwd.join(&entry_path)),
                        Err(_) => Ok(entry_path),
                    }
                }
            }
            Err(e) => Err(vec![OxcDiagnostic::error(format!(
                "Cannot resolve entry point {}: {e}",
                entry_path.display()
            ))]),
        }
    }

    /// Compute a path relative to the CWD for region markers.
    fn relative_path(&self, path: &Path) -> String {
        match path.strip_prefix(&self.options.cwd) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        }
    }
}

/// Check whether a path refers to a declaration file (`.d.ts`, `.d.mts`, `.d.cts`).
fn is_declaration_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}
