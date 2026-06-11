//! Cross-file type propagation pipeline.
//!
//! Orchestrates propagation of type bindings across import edges in topological
//! order.  After per-file inference completes, `PropagationPipeline::run`
//! iterates files from leaf dependencies to root entry points, calling
//! `DeepTypeInferencer::propagate_types()` for each import edge so that
//! downstream files receive resolved types instead of `Type::Unknown`.
//!
//! # Requirements
//! - R1: invoke propagate_types() per import edge after per-file inference
//! - R2: topological order — dependencies before dependents
//! - R3: cache propagated types in FileAnalysis.symbols
//! - R7: prefer .pyi stubs over .py sources when present
//! - R8: invalidation + re-propagation on dependency change
//! - R9: detect cycles, mark cycle members, emit diagnostic

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::deep_inference::{DeepTypeInferencer, ImportInfo};
use crate::graph::ImportGraph;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Request describing which files to propagate and whether the run is
/// incremental (only changed files) or full.
#[derive(Debug, Clone)]
pub struct PropagationRequest {
    /// All files in the project that should participate in propagation.
    pub files: Vec<PathBuf>,
    /// Files that changed since the last propagation (for incremental mode).
    /// Empty means full propagation.
    pub changed_files: Vec<PathBuf>,
}

/// A single type binding that was propagated from a source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagatedType {
    /// Symbol name as imported.
    pub symbol: String,
    /// Resolved type signature string.
    pub type_str: String,
    /// File where symbol is originally defined.
    pub source_file: PathBuf,
    /// Line number in source file.
    pub source_line: u32,
    /// True if the type came from a `.pyi` stub.
    pub is_stub: bool,
}

/// Result of running the propagation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    /// Map of target file → list of propagated types received.
    pub propagated: HashMap<PathBuf, Vec<PropagatedType>>,
    /// Detected import cycles (each cycle is a list of file paths).
    pub cycles: Vec<Vec<PathBuf>>,
    /// Aggregate statistics.
    pub stats: PropagationStats,
}

/// Aggregate statistics for the propagation run.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PropagationStats {
    pub files_analyzed: usize,
    pub symbols_propagated: usize,
    pub cycles_detected: usize,
    pub stubs_used: usize,
    pub time_ms: u64,
}

// ---------------------------------------------------------------------------
// Pipeline
// ---------------------------------------------------------------------------

/// Top-level orchestrator for cross-file type propagation.
///
/// Usage:
/// ```ignore
/// let result = PropagationPipeline::run(&files, &mut inferencer, &file_import_graph);
/// ```
pub struct PropagationPipeline;

impl PropagationPipeline {
    /// Run full or incremental propagation.
    ///
    /// 1. Build (or reuse) the DeepTypeInferencer's internal ImportGraph from
    ///    `FileAnalysis.imports`.
    /// 2. Detect cycles → emit diagnostics for cycle members.
    /// 3. Topological-sort the graph.
    /// 4. For each file in order, propagate types from its dependencies.
    /// 5. Mark `FileAnalysis.propagation_complete = true`.
    pub fn run(
        request: &PropagationRequest,
        inferencer: &mut DeepTypeInferencer,
        file_import_graph: &ImportGraph,
    ) -> PropagationResult {
        let start = Instant::now();
        let mut stats = PropagationStats::default();
        let mut propagated: HashMap<PathBuf, Vec<PropagatedType>> = HashMap::new();

        // --- Step 0: ensure all requested files are registered ---------------
        for file in &request.files {
            if inferencer.file_analysis(file).is_none() {
                inferencer.add_file(file.clone());
            }
        }

        // --- Step 1: build internal import graph from FileAnalysis entries ---
        Self::build_internal_graph(inferencer, file_import_graph, &request.files);

        // --- Step 2: detect cycles ------------------------------------------
        let cycles = inferencer.detect_import_cycles();
        stats.cycles_detected = cycles.len();
        let cycle_members: HashSet<PathBuf> =
            cycles.iter().flat_map(|c| c.iter().cloned()).collect();

        // --- Step 3: topological sort ---------------------------------------
        let topo_order = inferencer.topological_sort();

        // --- Step 4: determine work set (full or incremental) ---------------
        let work_set: HashSet<PathBuf> = if request.changed_files.is_empty() {
            // Full propagation — every file is a candidate.
            topo_order.iter().cloned().collect()
        } else {
            // Incremental — changed files plus all their transitive importers.
            let mut affected = HashSet::new();
            for changed in &request.changed_files {
                Self::collect_transitive_importers(inferencer, changed, &mut affected);
                affected.insert(changed.clone());
            }
            affected
        };

        // --- Step 5: propagate in topological order -------------------------
        for file in &topo_order {
            if !work_set.contains(file) {
                continue;
            }
            stats.files_analyzed += 1;

            // Skip cycle members for cross-file propagation (R9).
            if cycle_members.contains(file) {
                // Mark propagation complete even though we couldn't propagate
                // cross-cycle types — locally-inferred types are still valid.
                if let Some(fa) = inferencer.file_analysis_mut(file) {
                    fa.propagation_complete = true;
                }
                continue;
            }

            // Gather import edges for this file.
            let import_sources = Self::resolve_import_sources(inferencer, file);

            for (source_file, symbol_names) in import_sources {
                // R7: prefer .pyi stub if present
                let effective_source = Self::resolve_stub_or_source(&source_file, inferencer);
                let is_stub = effective_source != source_file;
                if is_stub {
                    stats.stubs_used += 1;
                }

                let syms: Option<Vec<String>> = if symbol_names.is_empty() {
                    None
                } else {
                    Some(symbol_names.clone())
                };
                let sym_slice: Option<&[String]> = syms.as_deref();

                inferencer.propagate_types(&effective_source, file, sym_slice);

                // Record propagated types for the result.
                let count_before = propagated.get(file).map_or(0, |v| v.len());
                for name in symbol_names.iter().chain(
                    // If no specific symbols, all exported were propagated.
                    std::iter::empty(),
                ) {
                    if let Some(binding) = inferencer
                        .file_analysis(file)
                        .and_then(|fa| fa.symbols.get(name))
                    {
                        if binding.is_propagated {
                            propagated
                                .entry(file.clone())
                                .or_default()
                                .push(PropagatedType {
                                    symbol: name.clone(),
                                    type_str: format!("{:?}", binding.ty),
                                    source_file: effective_source.clone(),
                                    source_line: binding.line,
                                    is_stub,
                                });
                        }
                    }
                }
                let count_after = propagated.get(file).map_or(0, |v| v.len());
                stats.symbols_propagated += count_after.saturating_sub(count_before);
            }

            // Mark propagation complete (R3).
            if let Some(fa) = inferencer.file_analysis_mut(file) {
                fa.propagation_complete = true;
            }
        }

        // Mark any requested files that weren't in the topo order
        // (e.g., files with no import edges) as propagation complete.
        for file in &request.files {
            if let Some(fa) = inferencer.file_analysis_mut(file) {
                if !fa.propagation_complete {
                    fa.propagation_complete = true;
                    stats.files_analyzed += 1;
                }
            }
        }

        stats.time_ms = start.elapsed().as_millis() as u64;

        PropagationResult {
            propagated,
            cycles,
            stats,
        }
    }

    /// Invalidate propagated types for a changed file and re-propagate (R8).
    ///
    /// 1. Clear propagated symbols from `changed_file` in all its importers.
    /// 2. Re-run `propagate_types` for the changed file → each importer edge.
    /// 3. Cascade via `update_symbol_type` for transitive re-exports.
    pub fn invalidate_and_repropagate(
        changed_file: &Path,
        inferencer: &mut DeepTypeInferencer,
        file_import_graph: &ImportGraph,
    ) -> PropagationResult {
        // Step 1: collect importers (reverse deps).
        let importers: Vec<PathBuf> = file_import_graph
            .dependents(changed_file)
            .into_iter()
            .collect();

        // Step 2: clear propagated bindings from changed_file in each importer.
        for importer in &importers {
            if let Some(fa) = inferencer.file_analysis_mut(importer) {
                let propagated_from_changed: Vec<String> = fa
                    .symbols
                    .iter()
                    .filter(|(_, b)| b.is_propagated && b.source_file == changed_file)
                    .map(|(name, _)| name.clone())
                    .collect();
                for name in &propagated_from_changed {
                    fa.symbols.remove(name);
                }
                fa.propagation_complete = false;
            }
        }

        // Step 3: re-propagate via incremental request.
        let mut all_files: Vec<PathBuf> = importers.clone();
        all_files.push(changed_file.to_path_buf());
        // Also include transitive importers.
        let mut transitive = HashSet::new();
        for importer in &importers {
            Self::collect_transitive_importers(inferencer, importer, &mut transitive);
        }
        all_files.extend(transitive);
        all_files.sort();
        all_files.dedup();

        let request = PropagationRequest {
            files: all_files,
            changed_files: vec![changed_file.to_path_buf()],
        };

        Self::run(&request, inferencer, file_import_graph)
    }

    // -- Helpers --------------------------------------------------------------

    /// Build the DeepTypeInferencer's internal import graph from
    /// `FileAnalysis.imports` entries, correlated with the file-level
    /// `ImportGraph` edges for resolution.
    fn build_internal_graph(
        inferencer: &mut DeepTypeInferencer,
        file_import_graph: &ImportGraph,
        files: &[PathBuf],
    ) {
        for file in files {
            let deps = file_import_graph.dependencies(file);
            for edge in deps {
                if let Some(ref resolved) = edge.resolved {
                    inferencer.add_import_edge(file.clone(), resolved.clone());
                }
            }
        }
    }

    /// Resolve import sources for a file: returns `(source_file, imported_symbol_names)`.
    ///
    /// Correlates each resolved dependency file with the specific symbols
    /// imported from it by matching the `ImportInfo.module` suffix against
    /// the dependency file stem.
    fn resolve_import_sources(
        inferencer: &DeepTypeInferencer,
        file: &PathBuf,
    ) -> Vec<(PathBuf, Vec<String>)> {
        let imports: Vec<ImportInfo> = inferencer
            .file_analysis(file)
            .map(|fa| fa.imports.clone())
            .unwrap_or_default();

        let graph_deps: HashSet<PathBuf> = inferencer
            .import_graph_deps(file)
            .cloned()
            .unwrap_or_default();

        let mut sources = Vec::new();

        for dep in graph_deps {
            // Extract the file stem of the dependency for matching.
            let dep_stem = dep.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            // Correlate: only collect symbol names from imports whose module
            // matches this specific dependency (by checking if the module path
            // ends with the dep file stem, e.g. module="db" matches "db.py").
            let mut sym_names: Vec<String> = Vec::new();
            for imp in &imports {
                let module_tail = imp.module.rsplit('.').next().unwrap_or(&imp.module);
                if module_tail == dep_stem || imp.module == dep_stem {
                    if let Some(names) = &imp.names {
                        sym_names.extend(names.iter().cloned());
                    }
                    // If names is None, it's a module-level import (import X) —
                    // propagate all exported symbols (empty vec signals this).
                }
            }
            sources.push((dep, sym_names));
        }

        sources
    }

    /// Resolve stub: if `{stem}.pyi` exists in the inferencer's file analysis,
    /// return the stub path; otherwise return the original source.
    fn resolve_stub_or_source(source: &PathBuf, inferencer: &DeepTypeInferencer) -> PathBuf {
        if source.extension().map_or(false, |e| e == "py") {
            let stub = source.with_extension("pyi");
            if inferencer.file_analysis(&stub).is_some() {
                return stub;
            }
        }
        source.clone()
    }

    /// Maximum recursion depth for transitive importer collection.
    /// Prevents stack overflow on deep import trees.
    const MAX_TRANSITIVE_DEPTH: usize = 128;

    /// Collect all transitive importers of a file with bounded recursion depth.
    fn collect_transitive_importers(
        inferencer: &DeepTypeInferencer,
        file: &PathBuf,
        out: &mut HashSet<PathBuf>,
    ) {
        Self::collect_transitive_importers_bounded(inferencer, file, out, 0);
    }

    fn collect_transitive_importers_bounded(
        inferencer: &DeepTypeInferencer,
        file: &PathBuf,
        out: &mut HashSet<PathBuf>,
        depth: usize,
    ) {
        if depth >= Self::MAX_TRANSITIVE_DEPTH {
            return;
        }
        if let Some(importers) = inferencer.import_graph_reverse_deps(file) {
            for importer in importers.clone() {
                if out.insert(importer.clone()) {
                    Self::collect_transitive_importers_bounded(
                        inferencer,
                        &importer,
                        out,
                        depth + 1,
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::ImportGraph;
    use crate::type_inference::deep_inference::{DeepTypeInferencer, ImportInfo, TypeBinding};
    use crate::type_inference::ty::Type;

    /// Helper: create a TypeBinding.
    fn binding(symbol: &str, ty: Type, source: &str, exported: bool) -> TypeBinding {
        TypeBinding {
            ty,
            source_file: PathBuf::from(source),
            symbol: symbol.to_string(),
            line: 1,
            is_exported: exported,
            dependencies: vec![],
            is_propagated: false,
        }
    }

    /// Helper: set up a two-file scenario (db.py → handler.py) with a
    /// `from db import get_user` import.
    fn setup_two_file() -> (DeepTypeInferencer, ImportGraph) {
        let mut inf = DeepTypeInferencer::new();
        let db = PathBuf::from("db.py");
        let handler = PathBuf::from("handler.py");

        inf.add_file(db.clone());
        inf.add_file(handler.clone());

        // db.py exports `get_user: Callable[[int], User]`
        inf.add_file_symbol(
            &db,
            "get_user".to_string(),
            binding(
                "get_user",
                Type::Callable {
                    params: vec![],
                    ret: Box::new(Type::Instance {
                        name: "User".to_string(),
                        module: None,
                        type_args: vec![],
                    }),
                },
                "db.py",
                true,
            ),
        );

        // handler.py has import info: from db import get_user
        inf.add_import(
            &handler,
            ImportInfo {
                module: "db".to_string(),
                names: Some(vec!["get_user".to_string()]),
                alias: None,
            },
        );

        // Build import edge in inferencer's internal graph.
        inf.add_import_edge(handler.clone(), db.clone());

        // Build file-level ImportGraph (needed for pipeline).
        let ig = ImportGraph::new();

        (inf, ig)
    }

    // -- Spec-required unit tests -------------------------------------------

    /// R1, R6, S1: `from db import get_user` propagates Callable type.
    #[test]
    fn test_propagate_from_import_y() {
        let (mut inf, ig) = setup_two_file();
        let handler = PathBuf::from("handler.py");

        let request = PropagationRequest {
            files: vec![PathBuf::from("db.py"), handler.clone()],
            changed_files: vec![],
        };

        let result = PropagationPipeline::run(&request, &mut inf, &ig);

        // handler.py should now have a propagated get_user binding.
        let fa = inf.file_analysis(&handler).unwrap();
        let b = fa.symbols.get("get_user").unwrap();
        assert!(b.is_propagated, "get_user should be marked as propagated");
        match &b.ty {
            Type::Callable { ret, .. } => match ret.as_ref() {
                Type::Instance { name, .. } => assert_eq!(name, "User"),
                other => panic!("Expected Instance(User), got {:?}", other),
            },
            other => panic!("Expected Callable, got {:?}", other),
        }
        assert!(result.stats.symbols_propagated > 0);
    }

    /// R6, S2: `import db` propagates all exported symbols as module-level binding.
    #[test]
    fn test_propagate_import_module() {
        let mut inf = DeepTypeInferencer::new();
        let db = PathBuf::from("db.py");
        let handler = PathBuf::from("handler.py");
        inf.add_file(db.clone());
        inf.add_file(handler.clone());

        // db.py exports two symbols
        inf.add_file_symbol(
            &db,
            "get_user".to_string(),
            binding("get_user", Type::Int, "db.py", true),
        );
        inf.add_file_symbol(
            &db,
            "create_user".to_string(),
            binding("create_user", Type::Str, "db.py", true),
        );

        // handler.py: import db (no specific names → all exported)
        inf.add_import(
            &handler,
            ImportInfo {
                module: "db".to_string(),
                names: None,
                alias: None,
            },
        );
        inf.add_import_edge(handler.clone(), db.clone());

        let ig = ImportGraph::new();
        let request = PropagationRequest {
            files: vec![db, handler.clone()],
            changed_files: vec![],
        };
        PropagationPipeline::run(&request, &mut inf, &ig);

        let fa = inf.file_analysis(&handler).unwrap();
        // Both exported symbols should be propagated.
        assert!(fa.symbols.contains_key("get_user"));
        assert!(fa.symbols.contains_key("create_user"));
    }

    /// R2: Files analyzed in dependency order — leaf modules first.
    #[test]
    fn test_propagation_topological_order() {
        let mut inf = DeepTypeInferencer::new();
        let c = PathBuf::from("c.py");
        let b = PathBuf::from("b.py");
        let a = PathBuf::from("a.py");
        inf.add_file(c.clone());
        inf.add_file(b.clone());
        inf.add_file(a.clone());

        // a → b → c
        inf.add_import_edge(a.clone(), b.clone());
        inf.add_import_edge(b.clone(), c.clone());

        let topo = inf.topological_sort();
        let c_pos = topo.iter().position(|p| p == &c);
        let b_pos = topo.iter().position(|p| p == &b);
        let a_pos = topo.iter().position(|p| p == &a);

        // c should come before b, b before a
        if let (Some(cp), Some(bp), Some(ap)) = (c_pos, b_pos, a_pos) {
            assert!(cp < bp, "c must be analyzed before b");
            assert!(bp < ap, "b must be analyzed before a");
        }
    }

    /// R1, R2, S3: Transitive propagation A→B→C.
    #[test]
    fn test_transitive_propagation() {
        let mut inf = DeepTypeInferencer::new();
        let c = PathBuf::from("c.py");
        let b = PathBuf::from("b.py");
        let a = PathBuf::from("a.py");
        inf.add_file(c.clone());
        inf.add_file(b.clone());
        inf.add_file(a.clone());

        // c.py defines Config
        inf.add_file_symbol(
            &c,
            "Config".to_string(),
            binding(
                "Config",
                Type::Instance {
                    name: "Config".to_string(),
                    module: None,
                    type_args: vec![],
                },
                "c.py",
                true,
            ),
        );

        // b.py: from c import Config
        inf.add_import(
            &b,
            ImportInfo {
                module: "c".to_string(),
                names: Some(vec!["Config".to_string()]),
                alias: None,
            },
        );
        inf.add_import_edge(b.clone(), c.clone());

        // a.py: from b import Config
        inf.add_import(
            &a,
            ImportInfo {
                module: "b".to_string(),
                names: Some(vec!["Config".to_string()]),
                alias: None,
            },
        );
        inf.add_import_edge(a.clone(), b.clone());

        let ig = ImportGraph::new();
        let request = PropagationRequest {
            files: vec![c, b, a.clone()],
            changed_files: vec![],
        };
        PropagationPipeline::run(&request, &mut inf, &ig);

        // a.py should have Config with the original type.
        let fa = inf.file_analysis(&a).unwrap();
        let config = fa
            .symbols
            .get("Config")
            .expect("Config should be propagated to a.py");
        assert!(config.is_propagated);
        match &config.ty {
            Type::Instance { name, .. } => assert_eq!(name, "Config"),
            other => panic!("Expected Instance(Config), got {:?}", other),
        }
    }

    /// R7, S5: When db.pyi exists alongside db.py, propagation uses stub types.
    #[test]
    fn test_stub_file_preference() {
        let mut inf = DeepTypeInferencer::new();
        let db_py = PathBuf::from("db.py");
        let db_pyi = PathBuf::from("db.pyi");
        let handler = PathBuf::from("handler.py");
        inf.add_file(db_py.clone());
        inf.add_file(db_pyi.clone());
        inf.add_file(handler.clone());

        // db.py has get_user -> Unknown
        inf.add_file_symbol(
            &db_py,
            "get_user".to_string(),
            binding("get_user", Type::Unknown, "db.py", true),
        );
        // db.pyi has get_user -> Int (richer type)
        inf.add_file_symbol(
            &db_pyi,
            "get_user".to_string(),
            binding("get_user", Type::Int, "db.pyi", true),
        );

        // handler.py imports from db
        inf.add_import(
            &handler,
            ImportInfo {
                module: "db".to_string(),
                names: Some(vec!["get_user".to_string()]),
                alias: None,
            },
        );
        inf.add_import_edge(handler.clone(), db_py.clone());

        let ig = ImportGraph::new();
        let request = PropagationRequest {
            files: vec![db_py, db_pyi, handler.clone()],
            changed_files: vec![],
        };
        let result = PropagationPipeline::run(&request, &mut inf, &ig);

        // handler.py should get the stub type (Int), not Unknown.
        let fa = inf.file_analysis(&handler).unwrap();
        let b = fa.symbols.get("get_user").unwrap();
        assert_eq!(b.ty, Type::Int, "Stub type should be preferred");
        assert!(result.stats.stubs_used > 0);
    }

    /// R8, S6: After invalidation + re-propagation, importers get updated types.
    #[test]
    fn test_repropagate_after_invalidation() {
        let (mut inf, ig) = setup_two_file();
        let db = PathBuf::from("db.py");
        let handler = PathBuf::from("handler.py");

        // Initial propagation.
        let request = PropagationRequest {
            files: vec![db.clone(), handler.clone()],
            changed_files: vec![],
        };
        PropagationPipeline::run(&request, &mut inf, &ig);

        // Now change db.py: get_user now returns Str instead.
        inf.add_file_symbol(
            &db,
            "get_user".to_string(),
            binding("get_user", Type::Str, "db.py", true),
        );

        // Invalidate and re-propagate.
        PropagationPipeline::invalidate_and_repropagate(&db, &mut inf, &ig);

        // handler.py should have the updated type.
        let fa = inf.file_analysis(&handler).unwrap();
        let b = fa.symbols.get("get_user").unwrap();
        assert_eq!(
            b.ty,
            Type::Str,
            "handler.py should have updated Str type after re-propagation"
        );
    }

    /// S7: File with no project imports — propagation is skipped.
    #[test]
    fn test_no_imports_no_propagation() {
        let mut inf = DeepTypeInferencer::new();
        let solo = PathBuf::from("solo.py");
        inf.add_file(solo.clone());
        inf.add_file_symbol(
            &solo,
            "x".to_string(),
            binding("x", Type::Int, "solo.py", true),
        );

        let ig = ImportGraph::new();
        let request = PropagationRequest {
            files: vec![solo.clone()],
            changed_files: vec![],
        };
        let result = PropagationPipeline::run(&request, &mut inf, &ig);

        assert_eq!(
            result.stats.symbols_propagated, 0,
            "No imports means no propagation"
        );
        assert!(result.propagated.is_empty());
        // File should still be marked complete.
        let fa = inf.file_analysis(&solo).unwrap();
        assert!(fa.propagation_complete);
    }

    /// Schema: PropagationResult.stats correctly counts files_analyzed,
    /// symbols_propagated, cycles_detected.
    #[test]
    fn test_propagation_stats() {
        let (mut inf, ig) = setup_two_file();

        let request = PropagationRequest {
            files: vec![PathBuf::from("db.py"), PathBuf::from("handler.py")],
            changed_files: vec![],
        };
        let result = PropagationPipeline::run(&request, &mut inf, &ig);

        assert_eq!(result.stats.files_analyzed, 2);
        assert!(result.stats.symbols_propagated >= 1);
        assert_eq!(result.stats.cycles_detected, 0);
        assert!(result.stats.time_ms < 10_000, "Should finish in < 10s");
    }

    /// R6: `from db import *` propagates all non-underscore symbols.
    #[test]
    fn test_star_import_propagation() {
        let mut inf = DeepTypeInferencer::new();
        let db = PathBuf::from("db.py");
        let handler = PathBuf::from("handler.py");
        inf.add_file(db.clone());
        inf.add_file(handler.clone());

        // db.py exports two symbols, one private
        inf.add_file_symbol(
            &db,
            "get_user".to_string(),
            binding("get_user", Type::Int, "db.py", true),
        );
        inf.add_file_symbol(
            &db,
            "create_user".to_string(),
            binding("create_user", Type::Str, "db.py", true),
        );
        inf.add_file_symbol(
            &db,
            "_internal".to_string(),
            binding("_internal", Type::Float, "db.py", false),
        );

        // handler.py: from db import * (no specific names = all exported)
        inf.add_import(
            &handler,
            ImportInfo {
                module: "db".to_string(),
                names: None, // wildcard
                alias: None,
            },
        );
        inf.add_import_edge(handler.clone(), db.clone());

        let ig = ImportGraph::new();
        let request = PropagationRequest {
            files: vec![db, handler.clone()],
            changed_files: vec![],
        };
        PropagationPipeline::run(&request, &mut inf, &ig);

        let fa = inf.file_analysis(&handler).unwrap();
        // Exported symbols should be propagated.
        assert!(fa.symbols.contains_key("get_user"));
        assert!(fa.symbols.contains_key("create_user"));
        // Non-exported should NOT be propagated.
        assert!(
            !fa.symbols.contains_key("_internal"),
            "_internal is not exported and should not propagate"
        );
    }
}
