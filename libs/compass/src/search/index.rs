//! Inverted search index for symbol-level code search.
//!
//! Maps symbol names, types, and documentation to file locations,
//! enabling fast lookup across the entire project. Serializable
//! with serde for disk persistence.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::semantic::{Symbol, SymbolKind};

// ============================================================================
// Serializable Symbol Entry
// ============================================================================

/// Serializable position within a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPosition {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// Kind of symbol stored in the index (mirrors `SymbolKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexSymbolKind {
    Variable,
    Function,
    Class,
    Parameter,
    Import,
    Module,
    TypeAlias,
    Decorator,
    Interface,
    TypeParameter,
    Enum,
    EnumMember,
    Struct,
    Trait,
    Impl,
    Macro,
    Const,
    Static,
    Resource,
    Stage,
    Job,
    Port,
    Label,
    Selector,
    Template,
}

impl From<SymbolKind> for IndexSymbolKind {
    fn from(kind: SymbolKind) -> Self {
        match kind {
            SymbolKind::Variable => Self::Variable,
            SymbolKind::Function => Self::Function,
            SymbolKind::Class => Self::Class,
            SymbolKind::Parameter => Self::Parameter,
            SymbolKind::Import => Self::Import,
            SymbolKind::Module => Self::Module,
            SymbolKind::TypeAlias => Self::TypeAlias,
            SymbolKind::Decorator => Self::Decorator,
            SymbolKind::Interface => Self::Interface,
            SymbolKind::TypeParameter => Self::TypeParameter,
            SymbolKind::Enum => Self::Enum,
            SymbolKind::EnumMember => Self::EnumMember,
            SymbolKind::Struct => Self::Struct,
            SymbolKind::Trait => Self::Trait,
            SymbolKind::Impl => Self::Impl,
            SymbolKind::Macro => Self::Macro,
            SymbolKind::Const => Self::Const,
            SymbolKind::Static => Self::Static,
            SymbolKind::Resource => Self::Resource,
            SymbolKind::Stage => Self::Stage,
            SymbolKind::Job => Self::Job,
            SymbolKind::Port => Self::Port,
            SymbolKind::Label => Self::Label,
            SymbolKind::Selector => Self::Selector,
            SymbolKind::Template => Self::Template,
        }
    }
}

/// A single symbol entry in the search index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    /// File containing the symbol.
    pub file: PathBuf,
    /// Position within the file.
    pub position: IndexPosition,
    /// Symbol kind.
    pub kind: IndexSymbolKind,
    /// Type signature string (e.g. `(str, int) -> bool`).
    pub type_signature: Option<String>,
    /// Documentation / docstring.
    pub documentation: Option<String>,
}

// ============================================================================
// Search Index
// ============================================================================

/// Current schema version for the on-disk index.
const INDEX_SCHEMA_VERSION: u32 = 1;

/// Serializable envelope for the persisted index.
#[derive(Serialize, Deserialize)]
struct PersistedIndex {
    version: u32,
    name_index: HashMap<String, Vec<SymbolEntry>>,
    type_index: HashMap<String, Vec<SymbolEntry>>,
    doc_index: HashMap<String, Vec<SymbolEntry>>,
}

/// Inverted index from symbol names / types to locations.
#[derive(Debug, Default)]
pub struct SearchIndex {
    /// name -> entries (case-sensitive)
    name_index: HashMap<String, Vec<SymbolEntry>>,
    /// normalised type-sig key -> entries
    type_index: HashMap<String, Vec<SymbolEntry>>,
    /// lowercase word -> entries (for doc search)
    doc_index: HashMap<String, Vec<SymbolEntry>>,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a single symbol from a `SymbolTable`.
    pub fn insert(&mut self, path: &Path, symbol: &Symbol) {
        let entry = SymbolEntry {
            file: path.to_path_buf(),
            position: IndexPosition {
                start_line: symbol.location.start.line,
                start_col: symbol.location.start.character,
                end_line: symbol.location.end.line,
                end_col: symbol.location.end.character,
            },
            kind: IndexSymbolKind::from(symbol.kind),
            type_signature: symbol.type_info.as_ref().map(|t| t.display()),
            documentation: symbol.doc.clone(),
        };

        // Name index
        self.name_index
            .entry(symbol.name.clone())
            .or_default()
            .push(entry.clone());

        // Type index (functions / callables only)
        if let Some(ref ti) = symbol.type_info {
            let key = Self::normalise_type_key(&ti.display());
            self.type_index.entry(key).or_default().push(entry.clone());
        }

        // Documentation index (one entry per unique word)
        if let Some(ref doc) = symbol.doc {
            for word in Self::extract_doc_words(doc) {
                self.doc_index.entry(word).or_default().push(entry.clone());
            }
        }
    }

    /// Remove all entries belonging to a file (for incremental update).
    pub fn remove_file(&mut self, path: &Path) {
        Self::retain_not_file(&mut self.name_index, path);
        Self::retain_not_file(&mut self.type_index, path);
        Self::retain_not_file(&mut self.doc_index, path);
    }

    /// Query by exact or prefix name match.
    pub fn query_by_name(&self, pattern: &str) -> Vec<&SymbolEntry> {
        let lower = pattern.to_lowercase();
        let mut results: Vec<&SymbolEntry> = Vec::new();
        for (name, entries) in &self.name_index {
            if name == pattern || name.to_lowercase().contains(&lower) {
                results.extend(entries.iter());
            }
        }
        results
    }

    /// Query by type signature substring.
    pub fn query_by_type(&self, type_sig: &str) -> Vec<&SymbolEntry> {
        let normalised = Self::normalise_type_key(type_sig);
        let mut results: Vec<&SymbolEntry> = Vec::new();
        for (key, entries) in &self.type_index {
            if key.contains(&normalised) || normalised.contains(key) {
                results.extend(entries.iter());
            }
        }
        results
    }

    /// Query documentation words.
    pub fn query_docs(&self, keyword: &str) -> Vec<&SymbolEntry> {
        let lower = keyword.to_lowercase();
        let mut results: Vec<&SymbolEntry> = Vec::new();
        for (word, entries) in &self.doc_index {
            if word.contains(&lower) {
                results.extend(entries.iter());
            }
        }
        results
    }

    /// Total number of name entries.
    pub fn len(&self) -> usize {
        self.name_index.values().map(|v| v.len()).sum()
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.name_index.is_empty()
    }

    // ---- Serialisation ----

    /// Serialize the index to bytes.
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let persisted = PersistedIndex {
            version: INDEX_SCHEMA_VERSION,
            name_index: self.name_index.clone(),
            type_index: self.type_index.clone(),
            doc_index: self.doc_index.clone(),
        };
        let bytes = bincode::serialize(&persisted)?;
        Ok(bytes)
    }

    /// Deserialize the index from bytes.
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let persisted: PersistedIndex = bincode::deserialize(bytes)?;
        if persisted.version != INDEX_SCHEMA_VERSION {
            anyhow::bail!(
                "index schema mismatch: expected {}, got {}",
                INDEX_SCHEMA_VERSION,
                persisted.version
            );
        }
        Ok(Self {
            name_index: persisted.name_index,
            type_index: persisted.type_index,
            doc_index: persisted.doc_index,
        })
    }

    // ---- Private helpers ----

    /// Normalise a type string for indexing (lowercase, whitespace-collapsed).
    fn normalise_type_key(sig: &str) -> String {
        sig.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Extract lowercase words from a docstring for the doc index.
    fn extract_doc_words(doc: &str) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        doc.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| w.len() >= 3)
            .map(|w| w.to_lowercase())
            .filter(|w| seen.insert(w.clone()))
            .collect()
    }

    /// Remove entries whose file matches `path`.
    fn retain_not_file(map: &mut HashMap<String, Vec<SymbolEntry>>, path: &Path) {
        for entries in map.values_mut() {
            entries.retain(|e| e.file != path);
        }
        map.retain(|_, v| !v.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Position, Range};

    fn make_symbol(name: &str, kind: SymbolKind, doc: Option<&str>) -> Symbol {
        Symbol {
            id: crate::semantic::SymbolId(0),
            name: name.to_string(),
            kind,
            location: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 10,
                },
            },
            type_info: None,
            doc: doc.map(|d| d.to_string()),
            scope_id: 0,
        }
    }

    #[test]
    fn test_insert_and_query_by_name() {
        let mut idx = SearchIndex::new();
        let sym = make_symbol("my_func", SymbolKind::Function, None);
        idx.insert(Path::new("/a.py"), &sym);

        let results = idx.query_by_name("my_func");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file, PathBuf::from("/a.py"));
    }

    #[test]
    fn test_remove_file() {
        let mut idx = SearchIndex::new();
        let sym = make_symbol("foo", SymbolKind::Function, Some("does stuff"));
        idx.insert(Path::new("/a.py"), &sym);
        assert!(!idx.is_empty());

        idx.remove_file(Path::new("/a.py"));
        assert!(idx.is_empty());
    }

    #[test]
    fn test_doc_search() {
        let mut idx = SearchIndex::new();
        let sym = make_symbol(
            "calc",
            SymbolKind::Function,
            Some("Calculate the total price"),
        );
        idx.insert(Path::new("/b.py"), &sym);

        let results = idx.query_docs("price");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut idx = SearchIndex::new();
        let sym = make_symbol("bar", SymbolKind::Variable, None);
        idx.insert(Path::new("/c.py"), &sym);

        let bytes = idx.to_bytes().unwrap();
        let loaded = SearchIndex::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.len(), 1);
    }
}
