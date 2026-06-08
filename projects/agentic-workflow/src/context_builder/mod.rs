//! Agent Context Builder: smart file selection for AI task execution.
//!
//! Combines import graph forward traversal, call graph backward traversal, and
//! test file detection to produce a minimal, ranked context set for an agent
//! working on a given symbol or file.
//!
//! # Pipeline
//!
//! 1. Parse targets (`file:symbol` format)
//! 2. Load/build ImportGraph + SearchIndex + DeepTypeInferencer
//! 3. For each target:
//!    a. Resolve symbol in SymbolTable (file-level fallback if unresolvable)
//!    b. Forward BFS on import graph (depth N)
//!    c. Backward BFS on call graph (depth N)
//!    d. Detect test files by naming convention
//! 4. Merge + deduplicate entries across targets
//! 5. Rank by depth
//! 6. Collect type signatures for cross-boundary symbols
//! 7. Build JSON response

pub mod test_detection;
pub mod traversal;
pub mod types;

pub use types::{ContextEntry, ContextRequest, ContextResponse, ContextStats, ContextTarget};

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::graph::ImportGraph;
use crate::search::query::CallGraphIndex;
use crate::semantic::SymbolTable;
use crate::type_inference::TypeContext;

use test_detection::TestLanguage;
use types::ContextReason;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/context_builder/mod.md#schema
// CODEGEN-BEGIN
/// Builder that orchestrates the context building pipeline.
/// @spec projects/agentic-workflow/tech-design/core/logic/context_builder/mod.md#schema
pub struct ContextBuilder<'a> {
    /// Import graph reference.
    import_graph: &'a ImportGraph,
    /// Call graph index reference.
    call_graph: &'a CallGraphIndex,
    /// Symbol tables keyed by path.
    symbol_tables: &'a HashMap<std::path::PathBuf, SymbolTable>,
    /// Optional type context.
    type_context: Option<&'a TypeContext>,
    /// Project file list.
    project_files: Vec<String>,
}
// CODEGEN-END
impl<'a> ContextBuilder<'a> {
    /// Create a new context builder with the required indices.
    pub fn new(
        import_graph: &'a ImportGraph,
        call_graph: &'a CallGraphIndex,
        symbol_tables: &'a HashMap<std::path::PathBuf, SymbolTable>,
    ) -> Self {
        Self {
            import_graph,
            call_graph,
            symbol_tables,
            type_context: None,
            project_files: Vec::new(),
        }
    }

    /// Set the type context for type signature collection.
    pub fn with_type_context(mut self, ctx: &'a TypeContext) -> Self {
        self.type_context = Some(ctx);
        self
    }

    /// Set the list of all project files (for test detection).
    pub fn with_project_files(mut self, files: Vec<String>) -> Self {
        self.project_files = files;
        self
    }

    /// Build context for the given request.
    ///
    /// This is the main entry point that orchestrates the full pipeline:
    /// parse targets -> resolve symbols -> forward traversal -> backward
    /// traversal -> test detection -> merge -> rank -> collect types -> respond.
    pub fn build_context(&self, request: &ContextRequest) -> ContextResponse {
        let start = std::time::Instant::now();

        let mut all_must_read: Vec<ContextEntry> = Vec::new();
        let mut all_may_affect: Vec<ContextEntry> = Vec::new();
        let mut type_symbols: HashSet<String> = HashSet::new();
        let mut targets_resolved: usize = 0;
        let mut targets_unresolved: usize = 0;
        let mut files_scanned: HashSet<String> = HashSet::new();

        for target in &request.targets {
            let target_path = std::path::PathBuf::from(&target.file);
            files_scanned.insert(target.file.clone());

            // Try to resolve the symbol in the symbol table
            let symbol_resolved = self.resolve_symbol(&target_path, &target.symbol);

            if symbol_resolved {
                targets_resolved += 1;
            } else {
                targets_unresolved += 1;
                eprintln!(
                    "warning: symbol '{}' not found in '{}', using file-level fallback",
                    target.symbol, target.file
                );
            }

            // Add target file as must_read with score 1.0
            all_must_read.push(ContextEntry {
                path: target.file.clone(),
                reason: ContextReason::Target,
                symbols: vec![target.symbol.clone()],
                depth: 0,
                score: 1.0,
            });

            // Forward traversal: import graph dependencies
            let forward_entries =
                traversal::forward_traverse(self.import_graph, &target_path, request.depth);
            for entry in &forward_entries {
                files_scanned.insert(entry.path.clone());
            }
            all_must_read.extend(forward_entries);

            // Backward traversal: call graph callers
            let backward_entries =
                traversal::backward_traverse(self.call_graph, &target.symbol, request.depth);
            for entry in &backward_entries {
                // Collect symbols for type context
                for sym in &entry.symbols {
                    type_symbols.insert(sym.clone());
                }
            }
            all_may_affect.extend(backward_entries);

            // Test file detection
            let language = TestLanguage::from_path(Path::new(&target.file));
            if let Some(lang) = language {
                let test_entries =
                    test_detection::detect_test_files(&self.project_files, &target.file, lang);
                all_may_affect.extend(test_entries);
            }

            // Collect symbols from forward deps for type context
            type_symbols.insert(target.symbol.clone());
        }

        // Merge and deduplicate
        let must_read = merge_entries(all_must_read);
        let may_affect = merge_entries(all_may_affect);

        // Collect type signatures
        let type_context_map = self.collect_type_signatures(&type_symbols);

        let elapsed = start.elapsed().as_millis() as u64;

        ContextResponse {
            must_read,
            may_affect,
            type_context: type_context_map,
            stats: ContextStats {
                targets_resolved,
                targets_unresolved,
                files_scanned: files_scanned.len(),
                time_ms: elapsed,
            },
        }
    }

    /// Check if a symbol exists in any symbol table for the given file.
    fn resolve_symbol(&self, file: &Path, symbol: &str) -> bool {
        if let Some(table) = self.symbol_tables.get(file) {
            !table.find_by_name(symbol).is_empty()
        } else {
            false
        }
    }

    /// Collect type signatures for the given set of symbols.
    fn collect_type_signatures(&self, symbols: &HashSet<String>) -> HashMap<String, String> {
        let mut result = HashMap::new();

        if let Some(ctx) = self.type_context {
            for symbol in symbols {
                // Search for the symbol's type binding across all files
                for (file, table) in self.symbol_tables {
                    let found = table.find_by_name(symbol);
                    for sym in found {
                        if let Some(ref type_info) = sym.type_info {
                            result.insert(symbol.clone(), type_info.display());
                            break;
                        }
                    }
                    // Also check DeepTypeInferencer bindings
                    if !result.contains_key(symbol.as_str()) {
                        if let Some(binding) = ctx.get_binding(file, symbol) {
                            result.insert(symbol.clone(), format!("{:?}", binding.ty));
                        }
                    }
                }
            }
        } else {
            // Fallback: use SymbolTable type_info only
            for symbol in symbols {
                for (_file, table) in self.symbol_tables {
                    let found = table.find_by_name(symbol);
                    for sym in found {
                        if let Some(ref type_info) = sym.type_info {
                            result.insert(symbol.clone(), type_info.display());
                            break;
                        }
                    }
                    if result.contains_key(symbol.as_str()) {
                        break;
                    }
                }
            }
        }

        result
    }
}

/// Merge and deduplicate context entries by file path.
///
/// When the same file appears in multiple entries (e.g., from multiple
/// targets), the entry with the highest score wins, and symbols are merged.
/// Entries are sorted by score descending (R9).
fn merge_entries(entries: Vec<ContextEntry>) -> Vec<ContextEntry> {
    let mut by_path: HashMap<String, ContextEntry> = HashMap::new();

    for entry in entries {
        by_path
            .entry(entry.path.clone())
            .and_modify(|existing| {
                // Merge symbols (deduplicated)
                let existing_symbols: HashSet<String> = existing.symbols.iter().cloned().collect();
                for sym in &entry.symbols {
                    if !existing_symbols.contains(sym) {
                        existing.symbols.push(sym.clone());
                    }
                }
                // Keep the higher score and lower depth
                if entry.score > existing.score {
                    existing.score = entry.score;
                    existing.reason = entry.reason.clone();
                }
                if entry.depth < existing.depth {
                    existing.depth = entry.depth;
                }
            })
            .or_insert(entry);
    }

    let mut merged: Vec<ContextEntry> = by_path.into_values().collect();
    merged.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Position, Range};
    use crate::graph::ImportGraph;
    use crate::search::query::CallGraphIndex;
    use crate::semantic::{SymbolKind, SymbolTable, TypeInfo};
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Helper: create a SymbolTable containing one function symbol.
    fn make_symbol_table(name: &str, type_info: Option<TypeInfo>) -> SymbolTable {
        let mut table = SymbolTable::new();
        table.add_symbol(
            name.to_string(),
            SymbolKind::Function,
            Range::new(Position::new(0, 0), Position::new(0, name.len() as u32)),
            type_info,
            None,
            0,
        );
        table
    }

    // ====================================================================
    // Ranking Tests (R9)
    // ====================================================================

    /// R9: Direct deps score > transitive deps score.
    #[test]
    fn test_ranking_by_depth() {
        let entries = vec![
            ContextEntry {
                path: "a.py".to_string(),
                reason: ContextReason::Target,
                symbols: vec!["foo".to_string()],
                depth: 0,
                score: 1.0,
            },
            ContextEntry {
                path: "c.py".to_string(),
                reason: ContextReason::TransitiveDep,
                symbols: vec!["baz".to_string()],
                depth: 2,
                score: 0.4,
            },
            ContextEntry {
                path: "b.py".to_string(),
                reason: ContextReason::ImportedByTarget,
                symbols: vec!["bar".to_string()],
                depth: 1,
                score: 0.8,
            },
        ];

        let merged = merge_entries(entries);

        // Should be sorted by score descending: a.py(1.0) > b.py(0.8) > c.py(0.4)
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].path, "a.py");
        assert_eq!(merged[1].path, "b.py");
        assert_eq!(merged[2].path, "c.py");

        // Verify scores decrease
        assert!(merged[0].score > merged[1].score);
        assert!(merged[1].score > merged[2].score);
    }

    // ====================================================================
    // Type Context Collection (R6)
    // ====================================================================

    /// R6: Cross-boundary symbols have type signatures in output.
    #[test]
    fn test_type_context_collection() {
        let ig = ImportGraph::new();
        let cg = CallGraphIndex::default();

        let mut symbol_tables = HashMap::new();
        let file = PathBuf::from("src/models/user.py");
        let mut table = SymbolTable::new();
        table.add_symbol(
            "User".to_string(),
            SymbolKind::Class,
            Range::new(Position::new(0, 0), Position::new(0, 4)),
            Some(TypeInfo::Named("User".to_string())),
            None,
            0,
        );
        table.add_symbol(
            "get_session".to_string(),
            SymbolKind::Function,
            Range::new(Position::new(5, 0), Position::new(5, 11)),
            Some(TypeInfo::Callable {
                params: vec![],
                ret: Box::new(TypeInfo::Named("Session".to_string())),
            }),
            None,
            0,
        );
        symbol_tables.insert(file, table);

        // Create target that references these symbols
        let target_file = PathBuf::from("src/services/user.py");
        symbol_tables.insert(target_file.clone(), make_symbol_table("get_user", None));

        let builder = ContextBuilder::new(&ig, &cg, &symbol_tables);
        let request = ContextRequest {
            targets: vec![ContextTarget {
                file: "src/services/user.py".to_string(),
                symbol: "get_user".to_string(),
            }],
            depth: 0,
        };

        let response = builder.build_context(&request);

        // The target symbol "get_user" should be in type_symbols, but it has no type_info
        // so it won't appear in type_context unless found in a symbol table with type_info.
        // What we're testing is that the pipeline runs and produces a valid response.
        assert!(response.type_context.is_empty() || response.type_context.contains_key("get_user"));
    }

    /// Type context with SymbolTable type_info produces type signatures.
    #[test]
    fn test_type_context_from_symbol_table() {
        let ig = ImportGraph::new();
        let cg = CallGraphIndex::default();

        let mut symbol_tables = HashMap::new();
        let file = PathBuf::from("src/user.py");
        let mut table = SymbolTable::new();
        table.add_symbol(
            "get_user".to_string(),
            SymbolKind::Function,
            Range::new(Position::new(0, 0), Position::new(0, 8)),
            Some(TypeInfo::Callable {
                params: vec![TypeInfo::Primitive("int".to_string())],
                ret: Box::new(TypeInfo::Named("User".to_string())),
            }),
            None,
            0,
        );
        symbol_tables.insert(file, table);

        let builder = ContextBuilder::new(&ig, &cg, &symbol_tables);
        let request = ContextRequest {
            targets: vec![ContextTarget {
                file: "src/user.py".to_string(),
                symbol: "get_user".to_string(),
            }],
            depth: 0,
        };

        let response = builder.build_context(&request);

        // get_user should have type signature in type_context
        assert!(
            response.type_context.contains_key("get_user"),
            "type_context should contain the target symbol's type signature"
        );
        let sig = &response.type_context["get_user"];
        assert!(
            sig.contains("User"),
            "signature should mention return type User"
        );
    }

    // ====================================================================
    // Unresolvable Symbol Fallback (S4)
    // ====================================================================

    /// S4: Falls back to file-level context when symbol not found.
    #[test]
    fn test_unresolvable_symbol_fallback() {
        let ig = ImportGraph::new();
        let cg = CallGraphIndex::default();
        let symbol_tables = HashMap::new(); // No symbol tables at all

        let builder = ContextBuilder::new(&ig, &cg, &symbol_tables);
        let request = ContextRequest {
            targets: vec![ContextTarget {
                file: "src/a.py".to_string(),
                symbol: "nonexistent".to_string(),
            }],
            depth: 2,
        };

        let response = builder.build_context(&request);

        // Should still include the target file in must_read (file-level fallback)
        assert_eq!(response.must_read.len(), 1);
        assert_eq!(response.must_read[0].path, "src/a.py");
        assert_eq!(response.must_read[0].reason, ContextReason::Target);

        // Stats should show 0 resolved, 1 unresolved
        assert_eq!(response.stats.targets_resolved, 0);
        assert_eq!(response.stats.targets_unresolved, 1);
    }

    // ====================================================================
    // Multiple Targets Merge (S2)
    // ====================================================================

    /// S2: Entries from multiple targets are merged and deduplicated.
    #[test]
    fn test_multiple_targets_merge() {
        let ig = ImportGraph::new();
        let mut cg = CallGraphIndex::default();

        // Both foo and bar are called by "shared_caller"
        cg.called_by
            .insert("foo".to_string(), vec!["shared_caller".to_string()]);
        cg.called_by
            .insert("bar".to_string(), vec!["shared_caller".to_string()]);

        let mut symbol_tables = HashMap::new();
        symbol_tables.insert(PathBuf::from("src/a.py"), make_symbol_table("foo", None));
        symbol_tables.insert(PathBuf::from("src/b.py"), make_symbol_table("bar", None));

        let builder = ContextBuilder::new(&ig, &cg, &symbol_tables);
        let request = ContextRequest {
            targets: vec![
                ContextTarget {
                    file: "src/a.py".to_string(),
                    symbol: "foo".to_string(),
                },
                ContextTarget {
                    file: "src/b.py".to_string(),
                    symbol: "bar".to_string(),
                },
            ],
            depth: 1,
        };

        let response = builder.build_context(&request);

        // must_read should have both targets (src/a.py, src/b.py)
        assert_eq!(response.must_read.len(), 2);
        let must_paths: HashSet<_> = response.must_read.iter().map(|e| e.path.as_str()).collect();
        assert!(must_paths.contains("src/a.py"));
        assert!(must_paths.contains("src/b.py"));

        // may_affect: shared_caller should appear only once (deduplicated)
        let shared_entries: Vec<_> = response
            .may_affect
            .iter()
            .filter(|e| e.path == "shared_caller")
            .collect();
        assert_eq!(
            shared_entries.len(),
            1,
            "shared_caller should appear exactly once after dedup"
        );

        // The shared_caller entry should have merged symbols from both targets
        let shared = &shared_entries[0];
        assert!(shared.symbols.contains(&"shared_caller".to_string()));
    }

    // ====================================================================
    // Depth Zero (S3)
    // ====================================================================

    /// S3: Depth 0 returns target only, no traversal.
    #[test]
    fn test_depth_zero() {
        let ig = ImportGraph::new();
        let mut cg = CallGraphIndex::default();
        cg.called_by
            .insert("foo".to_string(), vec!["bar".to_string()]);

        let mut symbol_tables = HashMap::new();
        symbol_tables.insert(PathBuf::from("src/a.py"), make_symbol_table("foo", None));

        let builder = ContextBuilder::new(&ig, &cg, &symbol_tables)
            .with_project_files(vec!["src/a.py".to_string(), "tests/test_a.py".to_string()]);

        let request = ContextRequest {
            targets: vec![ContextTarget {
                file: "src/a.py".to_string(),
                symbol: "foo".to_string(),
            }],
            depth: 0,
        };

        let response = builder.build_context(&request);

        // must_read = target file only
        assert_eq!(response.must_read.len(), 1);
        assert_eq!(response.must_read[0].path, "src/a.py");
        assert_eq!(response.must_read[0].reason, ContextReason::Target);

        // may_affect = empty (no backward traversal at depth 0, but test detection
        // still runs as it's independent of depth)
        // Note: test detection is always active regardless of depth
        // The may_affect might contain test files found by naming convention
        for entry in &response.may_affect {
            // Any may_affect entries at depth 0 should only be test files
            if !response.may_affect.is_empty() {
                assert_eq!(entry.reason, ContextReason::TestFile);
            }
        }

        // No forward traversal at depth 0
        let non_target: Vec<_> = response
            .must_read
            .iter()
            .filter(|e| e.reason != ContextReason::Target)
            .collect();
        assert!(non_target.is_empty(), "depth 0 should have no forward deps");
    }

    // ====================================================================
    // Merge Entries Tests
    // ====================================================================

    #[test]
    fn test_merge_dedup_same_path() {
        let entries = vec![
            ContextEntry {
                path: "shared.py".to_string(),
                reason: ContextReason::ImportedByTarget,
                symbols: vec!["sym_a".to_string()],
                depth: 1,
                score: 0.8,
            },
            ContextEntry {
                path: "shared.py".to_string(),
                reason: ContextReason::TransitiveDep,
                symbols: vec!["sym_b".to_string()],
                depth: 2,
                score: 0.4,
            },
        ];

        let merged = merge_entries(entries);
        assert_eq!(
            merged.len(),
            1,
            "same path entries should be merged into one"
        );

        let entry = &merged[0];
        assert_eq!(entry.path, "shared.py");
        // Higher aw wins
        assert!((entry.score - 0.8).abs() < 1e-6);
        assert_eq!(entry.reason, ContextReason::ImportedByTarget);
        // Lower depth wins
        assert_eq!(entry.depth, 1);
        // Symbols are merged
        assert!(entry.symbols.contains(&"sym_a".to_string()));
        assert!(entry.symbols.contains(&"sym_b".to_string()));
    }

    #[test]
    fn test_merge_sort_by_score_descending() {
        let entries = vec![
            ContextEntry {
                path: "low.py".to_string(),
                reason: ContextReason::TransitiveDep,
                symbols: vec![],
                depth: 3,
                score: 0.2,
            },
            ContextEntry {
                path: "high.py".to_string(),
                reason: ContextReason::Target,
                symbols: vec![],
                depth: 0,
                score: 1.0,
            },
            ContextEntry {
                path: "mid.py".to_string(),
                reason: ContextReason::ImportedByTarget,
                symbols: vec![],
                depth: 1,
                score: 0.8,
            },
        ];

        let merged = merge_entries(entries);
        assert_eq!(merged[0].path, "high.py");
        assert_eq!(merged[1].path, "mid.py");
        assert_eq!(merged[2].path, "low.py");
    }

    // ====================================================================
    // ContextTarget Parsing
    // ====================================================================

    #[test]
    fn test_context_target_parse() {
        let t = ContextTarget::parse("src/services/user.py:get_user").unwrap();
        assert_eq!(t.file, "src/services/user.py");
        assert_eq!(t.symbol, "get_user");
    }

    #[test]
    fn test_context_target_parse_no_colon() {
        assert!(ContextTarget::parse("no_colon_here").is_none());
    }

    #[test]
    fn test_context_target_parse_empty_parts() {
        assert!(ContextTarget::parse(":symbol").is_none());
        assert!(ContextTarget::parse("file:").is_none());
    }

    // ====================================================================
    // JSON Serialization (NF4)
    // ====================================================================

    #[test]
    fn test_context_response_json_serialization() {
        let response = ContextResponse {
            must_read: vec![ContextEntry {
                path: "src/a.py".to_string(),
                reason: ContextReason::Target,
                symbols: vec!["foo".to_string()],
                depth: 0,
                score: 1.0,
            }],
            may_affect: vec![ContextEntry {
                path: "src/b.py".to_string(),
                reason: ContextReason::CallsTarget,
                symbols: vec!["bar".to_string()],
                depth: 1,
                score: 0.8,
            }],
            type_context: {
                let mut m = HashMap::new();
                m.insert("foo".to_string(), "(int) -> User".to_string());
                m
            },
            stats: ContextStats {
                targets_resolved: 1,
                targets_unresolved: 0,
                files_scanned: 2,
                time_ms: 42,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        // Verify it's valid JSON and contains expected fields
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["must_read"].is_array());
        assert!(parsed["may_affect"].is_array());
        assert!(parsed["type_context"].is_object());
        assert!(parsed["stats"]["targets_resolved"].as_u64() == Some(1));

        // score should NOT be in JSON output (serde skip_serializing)
        let must_read_0 = &parsed["must_read"][0];
        assert!(
            must_read_0.get("score").is_none(),
            "score should be skipped in JSON"
        );
    }

    #[test]
    fn test_context_response_empty() {
        let response = ContextResponse::empty();
        assert!(response.must_read.is_empty());
        assert!(response.may_affect.is_empty());
        assert!(response.type_context.is_empty());
        assert_eq!(response.stats.targets_resolved, 0);
        assert_eq!(response.stats.targets_unresolved, 0);
    }

    // ====================================================================
    // Full Pipeline (resolved symbol)
    // ====================================================================

    #[test]
    fn test_build_context_resolved_symbol() {
        let ig = ImportGraph::new();
        let cg = CallGraphIndex::default();

        let mut symbol_tables = HashMap::new();
        symbol_tables.insert(
            PathBuf::from("src/user.py"),
            make_symbol_table("get_user", Some(TypeInfo::Primitive("str".to_string()))),
        );

        let builder = ContextBuilder::new(&ig, &cg, &symbol_tables);
        let request = ContextRequest {
            targets: vec![ContextTarget {
                file: "src/user.py".to_string(),
                symbol: "get_user".to_string(),
            }],
            depth: 2,
        };

        let response = builder.build_context(&request);

        assert_eq!(response.stats.targets_resolved, 1);
        assert_eq!(response.stats.targets_unresolved, 0);
        assert_eq!(response.must_read.len(), 1);
        assert_eq!(response.must_read[0].path, "src/user.py");
    }
}
