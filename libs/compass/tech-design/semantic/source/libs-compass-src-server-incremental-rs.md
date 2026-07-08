---
id: libs-compass-src-server-incremental-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/server/incremental.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/server/incremental.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/server/incremental.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FileChangeKind` | libs/compass/src/server/incremental.rs | enum | pub | 47 | pub enum FileChangeKind { |
| `DirtyFileTracker` | libs/compass/src/server/incremental.rs | struct | pub | 65 | pub struct DirtyFileTracker { |
| `new` | libs/compass/src/server/incremental.rs | function | pub | 74 | pub fn new() -> Self { |
| `mark_dirty` | libs/compass/src/server/incremental.rs | function | pub | 83 | pub fn mark_dirty(&mut self, path: PathBuf, kind: FileChangeKind) { |
| `has_dirty` | libs/compass/src/server/incremental.rs | function | pub | 95 | pub fn has_dirty(&self) -> bool { |
| `drain` | libs/compass/src/server/incremental.rs | function | pub | 102 | pub fn drain(&mut self) -> HashMap<PathBuf, FileChangeKind> { |
| `dirty_paths` | libs/compass/src/server/incremental.rs | function | pub | 108 | pub fn dirty_paths(&self) -> impl Iterator<Item = &PathBuf> { |
| `dirty_count` | libs/compass/src/server/incremental.rs | function | pub | 113 | pub fn dirty_count(&self) -> usize { |
| `DependencyGraph` | libs/compass/src/server/incremental.rs | struct | pub | 130 | pub struct DependencyGraph { |
| `add_edge` | libs/compass/src/server/incremental.rs | function | pub | 147 | pub fn add_edge(&mut self, dependency: PathBuf, importer: PathBuf) { |
| `remove_file` | libs/compass/src/server/incremental.rs | function | pub | 159 | pub fn remove_file(&mut self, path: &Path) { |
| `direct_importers` | libs/compass/src/server/incremental.rs | function | pub | 171 | pub fn direct_importers(&self, dependency: &Path) -> Vec<PathBuf> { |
| `direct_dependencies` | libs/compass/src/server/incremental.rs | function | pub | 179 | pub fn direct_dependencies(&self, importer: &Path) -> Vec<PathBuf> { |
| `transitive_importers` | libs/compass/src/server/incremental.rs | function | pub | 190 | pub fn transitive_importers(&self, changed_files: &[PathBuf]) -> HashSet<PathBuf> { |
| `edge_count` | libs/compass/src/server/incremental.rs | function | pub | 214 | pub fn edge_count(&self) -> usize { |
| `IncrementalUpdateManager` | libs/compass/src/server/incremental.rs | struct | pub | 229 | pub struct IncrementalUpdateManager { |
| `file_changed` | libs/compass/src/server/incremental.rs | function | pub | 244 | pub fn file_changed(&mut self, path: PathBuf, kind: FileChangeKind) { |
| `add_import_edge` | libs/compass/src/server/incremental.rs | function | pub | 254 | pub fn add_import_edge(&mut self, dependency: PathBuf, importer: PathBuf) { |
| `clear_importer_edges` | libs/compass/src/server/incremental.rs | function | pub | 262 | pub fn clear_importer_edges(&mut self, importer: &Path) { |
| `has_pending_work` | libs/compass/src/server/incremental.rs | function | pub | 273 | pub fn has_pending_work(&self) -> bool { |
| `drain_dirty_files` | libs/compass/src/server/incremental.rs | function | pub | 281 | pub fn drain_dirty_files(&mut self) -> Vec<PathBuf> { |
| `dep_graph` | libs/compass/src/server/incremental.rs | function | pub | 298 | pub fn dep_graph(&self) -> &DependencyGraph { |
| `tracker` | libs/compass/src/server/incremental.rs | function | pub | 303 | pub fn tracker(&self) -> &DirtyFileTracker { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Incremental daemon update manager (R6)
//!
//! Replaces the former full-workspace re-indexing triggered on every file-
//! change event with a two-phase incremental pipeline:
//!
//! 1. **Dirty file tracking** — `DirtyFileTracker` records which files have
//!    changed (created / modified / deleted) since the last analysis pass.
//!
//! 2. **Dependency-aware invalidation** — `DependencyGraph` tracks the
//!    importer ↔ dependency relationship between files so that, when a module
//!    changes, every file that imports it is also marked dirty.
//!
//! 3. **Incremental update manager** — `IncrementalUpdateManager` wires the
//!    two components together and exposes a single `drain_dirty_files()` method
//!    that the daemon calls on each watch-bridge event instead of the previous
//!    full re-index.
//!
//! # Integration with the daemon
//!
//! ```no_run
//! use cclab_compass::server::incremental::{FileChangeKind, IncrementalUpdateManager};
//! use std::path::PathBuf;
//!
//! let mut manager = IncrementalUpdateManager::new();
//!
//! // Record watch events
//! manager.file_changed(PathBuf::from("src/lib.rs"), FileChangeKind::Modified);
//! manager.file_changed(PathBuf::from("src/foo.rs"), FileChangeKind::Created);
//!
//! // Register an import edge: src/main.rs imports src/lib.rs
//! manager.add_import_edge(PathBuf::from("src/lib.rs"), PathBuf::from("src/main.rs"));
//!
//! // Obtain the minimal set of files that need re-analysis (transitive)
//! let to_reanalyze = manager.drain_dirty_files();
//! ```

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Instant;

// ============================================================================
// FileChangeKind
// ============================================================================

/// The type of file-system event that made a file dirty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeKind {
    /// The file was newly created.
    Created,
    /// The file was modified (content change).
    Modified,
    /// The file was deleted.
    Deleted,
}

// ============================================================================
// DirtyFileTracker
// ============================================================================

/// Tracks which source files have changed since the last analysis flush.
///
/// Files are accumulated with `mark_dirty()` and consumed atomically with
/// `drain()`, which also resets the internal state for the next cycle.
#[derive(Debug, Default)]
pub struct DirtyFileTracker {
    /// Dirty files mapped to their most recent change kind.
    dirty: HashMap<PathBuf, FileChangeKind>,
    /// Timestamp of the last `drain()` call.
    last_drain: Option<Instant>,
}

impl DirtyFileTracker {
    /// Create a new, empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a file as dirty with the given change kind.
    ///
    /// If the file was already dirty, the change kind is updated — a `Deleted`
    /// event always wins over `Modified`, and `Modified` wins over `Created`
    /// when the same file appears multiple times in rapid succession.
    pub fn mark_dirty(&mut self, path: PathBuf, kind: FileChangeKind) {
        let entry = self.dirty.entry(path).or_insert(kind);
        // Merge: Deleted > Modified > Created
        *entry = match (*entry, kind) {
            (_, FileChangeKind::Deleted) => FileChangeKind::Deleted,
            (FileChangeKind::Deleted, _) => FileChangeKind::Deleted,
            (FileChangeKind::Created, FileChangeKind::Modified) => FileChangeKind::Modified,
            (existing, _) => existing,
        };
    }

    /// Return `true` when at least one file is currently dirty.
    pub fn has_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// Atomically consume all dirty entries and return them.
    ///
    /// The internal set is cleared after the call, ready for the next cycle.
    pub fn drain(&mut self) -> HashMap<PathBuf, FileChangeKind> {
        self.last_drain = Some(Instant::now());
        std::mem::take(&mut self.dirty)
    }

    /// Peek at the current dirty set without consuming it.
    pub fn dirty_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.dirty.keys()
    }

    /// Number of currently dirty files.
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }
}

// ============================================================================
// DependencyGraph
// ============================================================================

/// Directed import graph: `dependency → {set of importers}`.
///
/// When file A imports file B, there is an edge `B → A` in this graph, meaning
/// "if B changes, A may be affected".
///
/// The graph is intentionally kept as a simple adjacency list rather than a
/// full module-resolution system, which keeps it fast and workspace-agnostic.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// dependency → importers
    reverse_edges: HashMap<PathBuf, HashSet<PathBuf>>,
    /// importer → dependencies
    forward_edges: HashMap<PathBuf, HashSet<PathBuf>>,
}

impl DependencyGraph {
    /// Create an empty dependency graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `importer` depends on `dependency`.
    ///
    /// This is the forward direction (`importer` imports `dependency`).
    /// The reverse edge is added automatically.
    pub fn add_edge(&mut self, dependency: PathBuf, importer: PathBuf) {
        self.reverse_edges
            .entry(dependency.clone())
            .or_default()
            .insert(importer.clone());
        self.forward_edges
            .entry(importer)
            .or_default()
            .insert(dependency);
    }

    /// Remove all edges for a deleted file (both as importer and dependency).
    pub fn remove_file(&mut self, path: &Path) {
        // Remove as dependency (clear all importer reverse-edges pointing here)
        self.reverse_edges.remove(path);
        // Remove as importer in reverse_edges
        for importers in self.reverse_edges.values_mut() {
            importers.remove(path);
        }
        // Remove forward edges
        self.forward_edges.remove(path);
    }

    /// Return the set of files that directly import `dependency`.
    pub fn direct_importers(&self, dependency: &Path) -> Vec<PathBuf> {
        self.reverse_edges
            .get(dependency)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Return the set of files that `importer` directly depends on.
    pub fn direct_dependencies(&self, importer: &Path) -> Vec<PathBuf> {
        self.forward_edges
            .get(importer)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Compute all files transitively affected by changes to `changed_files`.
    ///
    /// Uses BFS over the reverse-edge graph.  Returns the set of all files
    /// that need re-analysis, including the originally changed files.
    pub fn transitive_importers(&self, changed_files: &[PathBuf]) -> HashSet<PathBuf> {
        let mut visited: HashSet<PathBuf> = HashSet::new();
        let mut queue: VecDeque<PathBuf> = VecDeque::new();

        for f in changed_files {
            if visited.insert(f.clone()) {
                queue.push_back(f.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(importers) = self.reverse_edges.get(&current) {
                for importer in importers {
                    if visited.insert(importer.clone()) {
                        queue.push_back(importer.clone());
                    }
                }
            }
        }

        visited
    }

    /// Total number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.reverse_edges.values().map(|s| s.len()).sum()
    }
}

// ============================================================================
// IncrementalUpdateManager
// ============================================================================

/// Orchestrates dirty-file tracking and dependency-aware invalidation.
///
/// The daemon calls `file_changed()` for each event from the watch bridge,
/// then calls `drain_dirty_files()` to obtain the minimal set of files to
/// re-analyze.  The dependency graph is updated separately by the indexer
/// after it parses import statements.
pub struct IncrementalUpdateManager {
    tracker: DirtyFileTracker,
    dep_graph: DependencyGraph,
}

impl IncrementalUpdateManager {
    /// Create a new manager with empty tracker and dependency graph.
    pub fn new() -> Self {
        Self {
            tracker: DirtyFileTracker::new(),
            dep_graph: DependencyGraph::new(),
        }
    }

    /// Record a file-system change event.
    pub fn file_changed(&mut self, path: PathBuf, kind: FileChangeKind) {
        if kind == FileChangeKind::Deleted {
            self.dep_graph.remove_file(&path);
        }
        self.tracker.mark_dirty(path, kind);
    }

    /// Register an import relationship: `importer` imports `dependency`.
    ///
    /// Call this after (re-)analyzing `importer` to keep the graph current.
    pub fn add_import_edge(&mut self, dependency: PathBuf, importer: PathBuf) {
        self.dep_graph.add_edge(dependency, importer);
    }

    /// Remove all import edges originating from `importer`.
    ///
    /// Call this before re-analyzing `importer` so that stale edges don't
    /// linger after imports are removed or renamed.
    pub fn clear_importer_edges(&mut self, importer: &Path) {
        let deps = self.dep_graph.direct_dependencies(importer);
        for dep in &deps {
            if let Some(set) = self.dep_graph.reverse_edges.get_mut(dep) {
                set.remove(importer);
            }
        }
        self.dep_graph.forward_edges.remove(importer);
    }

    /// Return `true` when there are files pending re-analysis.
    pub fn has_pending_work(&self) -> bool {
        self.tracker.has_dirty()
    }

    /// Atomically consume all dirty files, expand via dependency-aware
    /// invalidation, and return the full set that must be re-analyzed.
    ///
    /// The return value is sorted for deterministic output.
    pub fn drain_dirty_files(&mut self) -> Vec<PathBuf> {
        let dirty_map = self.tracker.drain();
        if dirty_map.is_empty() {
            return vec![];
        }

        let dirty_list: Vec<PathBuf> = dirty_map.into_keys().collect();

        // BFS: expand dirty set to include all transitive importers.
        let expanded = self.dep_graph.transitive_importers(&dirty_list);

        let mut result: Vec<PathBuf> = expanded.into_iter().collect();
        result.sort();
        result
    }

    /// Access the dependency graph (read-only).
    pub fn dep_graph(&self) -> &DependencyGraph {
        &self.dep_graph
    }

    /// Access the dirty file tracker (read-only).
    pub fn tracker(&self) -> &DirtyFileTracker {
        &self.tracker
    }
}

impl Default for IncrementalUpdateManager {
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

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    // -----------------------------------------------------------------------
    // DirtyFileTracker
    // -----------------------------------------------------------------------

    #[test]
    fn test_mark_and_drain() {
        let mut tracker = DirtyFileTracker::new();
        tracker.mark_dirty(p("a.rs"), FileChangeKind::Modified);
        tracker.mark_dirty(p("b.rs"), FileChangeKind::Created);

        assert!(tracker.has_dirty());
        assert_eq!(tracker.dirty_count(), 2);

        let drained = tracker.drain();
        assert_eq!(drained.len(), 2);
        assert!(!tracker.has_dirty());
    }

    #[test]
    fn test_deleted_wins_over_modified() {
        let mut tracker = DirtyFileTracker::new();
        tracker.mark_dirty(p("a.rs"), FileChangeKind::Modified);
        tracker.mark_dirty(p("a.rs"), FileChangeKind::Deleted);

        let drained = tracker.drain();
        assert_eq!(*drained.get(&p("a.rs")).unwrap(), FileChangeKind::Deleted);
    }

    #[test]
    fn test_modified_wins_over_created() {
        let mut tracker = DirtyFileTracker::new();
        tracker.mark_dirty(p("a.rs"), FileChangeKind::Created);
        tracker.mark_dirty(p("a.rs"), FileChangeKind::Modified);

        let drained = tracker.drain();
        assert_eq!(*drained.get(&p("a.rs")).unwrap(), FileChangeKind::Modified);
    }

    #[test]
    fn test_drain_resets_state() {
        let mut tracker = DirtyFileTracker::new();
        tracker.mark_dirty(p("a.rs"), FileChangeKind::Modified);
        tracker.drain();
        assert!(!tracker.has_dirty());
    }

    // -----------------------------------------------------------------------
    // DependencyGraph
    // -----------------------------------------------------------------------

    #[test]
    fn test_add_and_query_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(p("lib.rs"), p("main.rs"));
        graph.add_edge(p("lib.rs"), p("tests.rs"));

        let importers = graph.direct_importers(&p("lib.rs"));
        assert_eq!(importers.len(), 2);
        assert!(importers.contains(&p("main.rs")));
        assert!(importers.contains(&p("tests.rs")));
    }

    #[test]
    fn test_transitive_importers() {
        let mut graph = DependencyGraph::new();
        // a.rs ← b.rs ← c.rs (chain)
        graph.add_edge(p("a.rs"), p("b.rs"));
        graph.add_edge(p("b.rs"), p("c.rs"));

        let affected = graph.transitive_importers(&[p("a.rs")]);
        assert!(affected.contains(&p("a.rs")));
        assert!(affected.contains(&p("b.rs")));
        assert!(affected.contains(&p("c.rs")));
    }

    #[test]
    fn test_remove_file_cleans_edges() {
        let mut graph = DependencyGraph::new();
        graph.add_edge(p("lib.rs"), p("main.rs"));
        graph.remove_file(&p("lib.rs"));

        assert!(graph.direct_importers(&p("lib.rs")).is_empty());
    }

    #[test]
    fn test_no_cycles_in_bfs() {
        let mut graph = DependencyGraph::new();
        // Create a diamond: base ← left/right ← top
        graph.add_edge(p("base.rs"), p("left.rs"));
        graph.add_edge(p("base.rs"), p("right.rs"));
        graph.add_edge(p("left.rs"), p("top.rs"));
        graph.add_edge(p("right.rs"), p("top.rs"));

        let affected = graph.transitive_importers(&[p("base.rs")]);
        // All four should appear exactly once (no duplicates, no infinite loop)
        assert_eq!(affected.len(), 4);
    }

    // -----------------------------------------------------------------------
    // IncrementalUpdateManager
    // -----------------------------------------------------------------------

    #[test]
    fn test_drain_empty_returns_empty() {
        let mut mgr = IncrementalUpdateManager::new();
        assert!(!mgr.has_pending_work());
        assert!(mgr.drain_dirty_files().is_empty());
    }

    #[test]
    fn test_direct_file_change() {
        let mut mgr = IncrementalUpdateManager::new();
        mgr.file_changed(p("src/lib.rs"), FileChangeKind::Modified);

        let files = mgr.drain_dirty_files();
        assert!(files.contains(&p("src/lib.rs")));
    }

    #[test]
    fn test_dependency_aware_expansion() {
        let mut mgr = IncrementalUpdateManager::new();
        // Register imports: main.rs imports lib.rs
        mgr.add_import_edge(p("src/lib.rs"), p("src/main.rs"));

        // lib.rs changes — main.rs should also be flagged
        mgr.file_changed(p("src/lib.rs"), FileChangeKind::Modified);

        let files = mgr.drain_dirty_files();
        assert!(
            files.contains(&p("src/lib.rs")),
            "lib.rs must be in dirty set"
        );
        assert!(
            files.contains(&p("src/main.rs")),
            "main.rs (importer) must be in dirty set"
        );
    }

    #[test]
    fn test_deleted_file_removes_from_graph() {
        let mut mgr = IncrementalUpdateManager::new();
        mgr.add_import_edge(p("lib.rs"), p("main.rs"));
        mgr.file_changed(p("lib.rs"), FileChangeKind::Deleted);

        // After deletion, lib.rs is no longer in the graph
        assert!(mgr.dep_graph().direct_importers(&p("lib.rs")).is_empty());
    }

    #[test]
    fn test_transitive_expansion() {
        let mut mgr = IncrementalUpdateManager::new();
        // chain: a ← b ← c
        mgr.add_import_edge(p("a.rs"), p("b.rs"));
        mgr.add_import_edge(p("b.rs"), p("c.rs"));

        mgr.file_changed(p("a.rs"), FileChangeKind::Modified);
        let files = mgr.drain_dirty_files();

        assert!(files.contains(&p("a.rs")));
        assert!(files.contains(&p("b.rs")));
        assert!(files.contains(&p("c.rs")));
    }

    #[test]
    fn test_drain_clears_tracker() {
        let mut mgr = IncrementalUpdateManager::new();
        mgr.file_changed(p("x.rs"), FileChangeKind::Modified);
        mgr.drain_dirty_files();
        assert!(!mgr.has_pending_work());
    }

    #[test]
    fn test_result_is_sorted() {
        let mut mgr = IncrementalUpdateManager::new();
        mgr.file_changed(p("z.rs"), FileChangeKind::Modified);
        mgr.file_changed(p("a.rs"), FileChangeKind::Modified);
        mgr.file_changed(p("m.rs"), FileChangeKind::Modified);

        let files = mgr.drain_dirty_files();
        let sorted = {
            let mut s = files.clone();
            s.sort();
            s
        };
        assert_eq!(files, sorted);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/server/incremental.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/server/incremental.rs` captured during libs codegen standardization.
```
