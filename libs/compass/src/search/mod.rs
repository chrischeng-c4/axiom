//! Semantic search engine with persistent index.
//!
//! Provides 6 search modes over a project-wide inverted index:
//! 1. **ByTypeSignature** - find functions matching a type pattern
//! 2. **CallHierarchy** - BFS callers / callees
//! 3. **Implementations** - find implementors of a trait / protocol
//! 4. **Usages** - find all references to a symbol
//! 5. **SimilarCode** - find structurally similar functions
//! 6. **DocumentationSearch** - keyword search in docstrings

pub mod index;
pub mod query;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::semantic::SymbolTable;
use crate::type_inference::{SearchKind, SearchQuery, SearchResult};
use index::SearchIndex;
use query::CallGraphIndex;

/// Persistent index subdirectory under `cclab/.index/`.
const SEARCH_INDEX_DIR: &str = "search_index";
/// Main index file name.
const INDEX_FILE: &str = "index.bin";

// ============================================================================
// SearchEngine
// ============================================================================

/// Top-level search engine managing the index and query dispatch.
pub struct SearchEngine {
    /// The inverted symbol index.
    index: SearchIndex,
    /// Lightweight call-graph used by call hierarchy queries.
    call_graph: CallGraphIndex,
}

impl SearchEngine {
    /// Create a new empty engine.
    pub fn new() -> Self {
        Self {
            index: SearchIndex::new(),
            call_graph: CallGraphIndex::default(),
        }
    }

    // ---- Index building ----

    /// Build (or rebuild) the index from a full set of symbol tables.
    pub fn build_index(&mut self, symbols: &HashMap<PathBuf, SymbolTable>) {
        self.index = SearchIndex::new();
        self.call_graph = CallGraphIndex::default();

        for (path, table) in symbols {
            self.index_symbol_table(path, table);
        }
    }

    /// Incremental update: re-index a single file.
    pub fn update_file(&mut self, path: &Path, symbols: &SymbolTable) {
        self.index.remove_file(path);
        self.remove_file_from_call_graph(path);
        self.index_symbol_table(path, symbols);
    }

    /// Remove a file from the index entirely.
    pub fn remove_file(&mut self, path: &Path) {
        self.index.remove_file(path);
        self.remove_file_from_call_graph(path);
    }

    // ---- Persistence ----

    /// Persist the index to `{dir}/cclab/.index/search_index/`.
    pub fn save_index(&self, project_root: &Path) -> Result<()> {
        let dir = Self::resolve_dir(project_root);
        fs::create_dir_all(&dir)?;

        let bytes = self.index.to_bytes()?;
        fs::write(dir.join(INDEX_FILE), bytes)?;

        tracing::debug!("search index saved to {}", dir.display());
        Ok(())
    }

    /// Load a previously persisted index.
    pub fn load_index(project_root: &Path) -> Result<Self> {
        let dir = Self::resolve_dir(project_root);
        let index_path = dir.join(INDEX_FILE);

        let bytes = fs::read(&index_path)?;
        let index = SearchIndex::from_bytes(&bytes)?;

        tracing::debug!("search index loaded from {}", dir.display());
        Ok(Self {
            index,
            call_graph: CallGraphIndex::default(),
        })
    }

    // ---- Query dispatch ----

    /// Execute a search query, dispatching to the appropriate mode handler.
    pub fn search(&self, query: SearchQuery) -> SearchResult {
        match &query.kind {
            SearchKind::ByTypeSignature { .. } => {
                // Build the text representation to pass through
                let pattern = Self::type_sig_pattern(&query.kind);
                query::search_by_type_signature(&self.index, &query, &pattern)
            }
            SearchKind::CallHierarchy {
                symbol,
                file,
                direction,
            } => query::search_call_hierarchy(
                &self.index,
                &self.call_graph,
                &query,
                symbol,
                file,
                *direction,
                10, // default max depth
            ),
            SearchKind::Implementations { protocol } => {
                query::search_implementations(&self.index, &query, protocol)
            }
            SearchKind::Usages { symbol, file } => {
                query::search_usages(&self.index, &query, symbol, file)
            }
            SearchKind::SimilarPatterns { pattern } => {
                query::search_similar_code(&self.index, &query, pattern)
            }
            SearchKind::ByDocumentation { query: kw } => {
                query::search_documentation(&self.index, &query, kw)
            }
            SearchKind::TypeHierarchy { type_name, .. } => {
                // Delegate to implementations search as a reasonable fallback
                query::search_implementations(&self.index, &query, type_name)
            }
        }
    }

    // ---- Accessors ----

    /// Number of entries in the index.
    pub fn index_size(&self) -> usize {
        self.index.len()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Access the raw index (for advanced queries).
    pub fn raw_index(&self) -> &SearchIndex {
        &self.index
    }

    /// Mutable access to the call graph (to let callers populate it).
    pub fn call_graph_mut(&mut self) -> &mut CallGraphIndex {
        &mut self.call_graph
    }

    // ---- Private helpers ----

    /// Resolve the on-disk directory for the search index.
    fn resolve_dir(project_root: &Path) -> PathBuf {
        project_root
            .join("cclab")
            .join(".index")
            .join(SEARCH_INDEX_DIR)
    }

    /// Index all symbols from a single file's `SymbolTable`.
    fn index_symbol_table(&mut self, path: &Path, table: &SymbolTable) {
        for symbol in table.all_symbols() {
            self.index.insert(path, symbol);
        }
    }

    /// Remove a file's entries from the call graph.
    fn remove_file_from_call_graph(&mut self, _path: &Path) {
        // Call graph entries are keyed by symbol name, not file.
        // A full rebuild would be needed for precise removal.
        // For now this is a no-op; call graph is rebuilt on full index.
    }

    /// Build a text pattern from a `SearchKind::ByTypeSignature`.
    fn type_sig_pattern(kind: &SearchKind) -> String {
        match kind {
            SearchKind::ByTypeSignature {
                params,
                return_type,
            } => {
                let params_str = params
                    .iter()
                    .map(|t| format!("{:?}", t))
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(ret) = return_type {
                    format!("({}) -> {:?}", params_str, ret)
                } else {
                    format!("({})", params_str)
                }
            }
            _ => String::new(),
        }
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Position, Range};
    use crate::semantic::{SymbolKind, SymbolTable};
    use crate::type_inference::SearchScope;
    use tempfile::TempDir;

    fn build_table(symbols: Vec<(&str, SymbolKind)>) -> SymbolTable {
        let mut table = SymbolTable::new();
        for (name, kind) in symbols {
            table.add_symbol(
                name.to_string(),
                kind,
                Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 5,
                    },
                },
                None,
                None,
                0,
            );
        }
        table
    }

    #[test]
    fn test_build_and_search() {
        let mut engine = SearchEngine::new();
        let mut tables = HashMap::new();
        tables.insert(
            PathBuf::from("/src/main.py"),
            build_table(vec![("process", SymbolKind::Function)]),
        );
        engine.build_index(&tables);

        assert_eq!(engine.index_size(), 1);

        let query = SearchQuery {
            kind: SearchKind::Usages {
                symbol: "process".into(),
                file: PathBuf::from("/src/main.py"),
            },
            scope: SearchScope::Project,
            max_results: 10,
        };
        let result = engine.search(query);
        assert_eq!(result.matches.len(), 1);
    }

    #[test]
    fn test_incremental_update() {
        let mut engine = SearchEngine::new();
        let mut tables = HashMap::new();
        tables.insert(
            PathBuf::from("/a.py"),
            build_table(vec![("alpha", SymbolKind::Function)]),
        );
        engine.build_index(&tables);
        assert_eq!(engine.index_size(), 1);

        // Update with new symbol
        let new_table = build_table(vec![
            ("alpha", SymbolKind::Function),
            ("beta", SymbolKind::Function),
        ]);
        engine.update_file(Path::new("/a.py"), &new_table);
        assert_eq!(engine.index_size(), 2);
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let mut engine = SearchEngine::new();
        let mut tables = HashMap::new();
        tables.insert(
            PathBuf::from("/x.py"),
            build_table(vec![("gamma", SymbolKind::Variable)]),
        );
        engine.build_index(&tables);

        engine.save_index(tmp.path()).unwrap();

        let loaded = SearchEngine::load_index(tmp.path()).unwrap();
        assert_eq!(loaded.index_size(), 1);
    }
}
