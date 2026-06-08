---
id: projects-sdd-src-context-builder-traversal-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Change/context/git/spec-store logic supports TD/CB artifact lifecycle dispatch and review state."
---

# Standardized projects/agentic-workflow/src/context_builder/traversal.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/context_builder/traversal.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `backward_traverse` | projects/agentic-workflow/src/context_builder/traversal.rs | function | pub | 106 | backward_traverse(     call_graph: &CallGraphIndex,     start_symbol: &str,     max_depth: u32, ) -> Vec<ContextEntry> |
| `forward_traverse` | projects/agentic-workflow/src/context_builder/traversal.rs | function | pub | 29 | forward_traverse(     import_graph: &ImportGraph,     start_file: &Path,     max_depth: u32, ) -> Vec<ContextEntry> |
## Source
<!-- type: source lang: rust -->

```rust
//! Forward and backward BFS traversal for context building.
//!
//! - **Forward**: Follows import graph edges from a target file, collecting
//!   transitive dependencies up to a configurable depth (R3).
//! - **Backward**: Follows call graph `called_by` edges from a target symbol,
//!   collecting all callers up to a configurable depth (R4).

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::types::{ContextEntry, ContextReason};
use crate::graph::ImportGraph;
use crate::search::query::CallGraphIndex;

// ============================================================================
// Forward Traversal (Import Graph)
// ============================================================================

/// BFS forward traversal on the import graph starting from `start_file`.
///
/// Returns a list of `ContextEntry` for each dependency found, annotated with
/// depth and reason. The start file itself is NOT included in the result
/// (it's handled as a "target" entry by the orchestrator).
///
/// Ranking: score = 0.8 / 2^(depth-1) per the spec formula.
pub fn forward_traverse(
    import_graph: &ImportGraph,
    start_file: &Path,
    max_depth: u32,
) -> Vec<ContextEntry> {
    if max_depth == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();
    visited.insert(start_file.to_path_buf());

    // BFS queue: (file_path, current_depth)
    let mut queue: VecDeque<(PathBuf, u32)> = VecDeque::new();

    // Seed with direct dependencies of start_file
    for edge in import_graph.dependencies(start_file) {
        if let Some(ref resolved) = edge.resolved {
            if visited.insert(resolved.clone()) {
                queue.push_back((resolved.clone(), 1));
            }
        }
    }

    while let Some((file, depth)) = queue.pop_front() {
        let reason = if depth == 1 {
            ContextReason::ImportedByTarget
        } else {
            ContextReason::TransitiveDep
        };

        let score = depth_to_score(depth);

        // Collect symbol names from the import edges that led here
        // (we use the import_path as a proxy for the symbol name)
        let symbols = import_graph
            .dependencies(start_file)
            .iter()
            .filter(|e| e.resolved.as_deref() == Some(file.as_path()))
            .map(|e| e.import_path.clone())
            .collect::<Vec<_>>();

        result.push(ContextEntry {
            path: file.to_string_lossy().to_string(),
            reason,
            symbols,
            depth,
            score,
        });

        // Continue BFS if within depth limit
        if depth < max_depth {
            for edge in import_graph.dependencies(&file) {
                if let Some(ref resolved) = edge.resolved {
                    if visited.insert(resolved.clone()) {
                        queue.push_back((resolved.clone(), depth + 1));
                    }
                }
            }
        }
    }

    result
}

// ============================================================================
// Backward Traversal (Call Graph)
// ============================================================================

/// BFS backward traversal on the call graph starting from `start_symbol`.
///
/// Returns a list of `ContextEntry` for each caller found, annotated with
/// depth and reason.
///
/// Ranking: score = 0.8 / 2^(depth-1) per the spec formula.
pub fn backward_traverse(
    call_graph: &CallGraphIndex,
    start_symbol: &str,
    max_depth: u32,
) -> Vec<ContextEntry> {
    if max_depth == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(start_symbol.to_string());

    // BFS queue: (symbol_name, current_depth)
    let mut queue: VecDeque<(String, u32)> = VecDeque::new();

    // Seed with direct callers
    if let Some(callers) = call_graph.called_by.get(start_symbol) {
        for caller in callers {
            if visited.insert(caller.clone()) {
                queue.push_back((caller.clone(), 1));
            }
        }
    }

    while let Some((symbol, depth)) = queue.pop_front() {
        let reason = if depth == 1 {
            ContextReason::CallsTarget
        } else {
            ContextReason::TransitiveCaller
        };

        let score = depth_to_score(depth);

        result.push(ContextEntry {
            // For call graph entries, the "path" is the symbol name.
            // The orchestrator resolves symbol -> file via SearchIndex.
            path: symbol.clone(),
            reason,
            symbols: vec![symbol.clone()],
            depth,
            score,
        });

        // Continue BFS if within depth limit
        if depth < max_depth {
            if let Some(callers) = call_graph.called_by.get(&symbol) {
                for caller in callers {
                    if visited.insert(caller.clone()) {
                        queue.push_back((caller.clone(), depth + 1));
                    }
                }
            }
        }
    }

    result
}

// ============================================================================
// Scoring
// ============================================================================

/// Compute relevance score from depth.
///
/// Formula from spec: `0.8 / 2^(depth-1)`.
///   - depth 1 -> 0.8
///   - depth 2 -> 0.4
///   - depth 3 -> 0.2
///   - etc.
fn depth_to_score(depth: u32) -> f64 {
    if depth == 0 {
        1.0
    } else {
        0.8 / (2.0_f64.powi((depth as i32) - 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: write a Python file and return its absolute path.
    fn write_py(dir: &Path, name: &str, content: &str) -> PathBuf {
        let p = dir.join(name);
        if let Some(par) = p.parent() {
            fs::create_dir_all(par).unwrap();
        }
        fs::write(&p, content).unwrap();
        p
    }

    /// Build an ImportGraph from a temp directory with Python files.
    fn build_graph(dir: &Path, files: &[(PathBuf, String)]) -> ImportGraph {
        ImportGraph::build(files, dir)
    }

    // ====================================================================
    // Forward Traversal Tests
    // ====================================================================

    /// R3: Forward BFS stops at depth 1, returns direct imports only.
    #[test]
    fn test_forward_traversal_depth_1() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();

        // a.py -> b.py -> c.py (chain of 2 hops)
        let a = write_py(r, "a.py", "from .b import x");
        let b = write_py(r, "b.py", "from .c import y");
        let _c = write_py(r, "c.py", "z = 1");

        let files = vec![
            (a.clone(), fs::read_to_string(&a).unwrap()),
            (b.clone(), fs::read_to_string(&b).unwrap()),
            (_c.clone(), fs::read_to_string(&_c).unwrap()),
        ];
        let graph = build_graph(r, &files);

        // Depth 1: should only find b.py (direct import), not c.py
        let entries = forward_traverse(&graph, &a, 1);
        assert_eq!(entries.len(), 1, "depth 1 should return 1 direct import");
        assert_eq!(entries[0].path, b.to_string_lossy().to_string());
        assert_eq!(entries[0].reason, ContextReason::ImportedByTarget);
        assert_eq!(entries[0].depth, 1);
        assert!(
            (entries[0].score - 0.8).abs() < 1e-6,
            "depth 1 score should be 0.8"
        );
    }

    /// R3: Forward BFS returns transitive imports at depth 2.
    #[test]
    fn test_forward_traversal_depth_2() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();

        // a.py -> b.py -> c.py
        let a = write_py(r, "a.py", "from .b import x");
        let b = write_py(r, "b.py", "from .c import y");
        let c = write_py(r, "c.py", "z = 1");

        let files = vec![
            (a.clone(), fs::read_to_string(&a).unwrap()),
            (b.clone(), fs::read_to_string(&b).unwrap()),
            (c.clone(), fs::read_to_string(&c).unwrap()),
        ];
        let graph = build_graph(r, &files);

        // Depth 2: should find b.py (depth 1) and c.py (depth 2)
        let entries = forward_traverse(&graph, &a, 2);
        assert_eq!(
            entries.len(),
            2,
            "depth 2 should return 2 imports (direct + transitive)"
        );

        // b.py is at depth 1 (ImportedByTarget)
        let b_entry = entries
            .iter()
            .find(|e| e.path == b.to_string_lossy().to_string())
            .unwrap();
        assert_eq!(b_entry.depth, 1);
        assert_eq!(b_entry.reason, ContextReason::ImportedByTarget);
        assert!((b_entry.score - 0.8).abs() < 1e-6);

        // c.py is at depth 2 (TransitiveDep)
        let c_entry = entries
            .iter()
            .find(|e| e.path == c.to_string_lossy().to_string())
            .unwrap();
        assert_eq!(c_entry.depth, 2);
        assert_eq!(c_entry.reason, ContextReason::TransitiveDep);
        assert!((c_entry.score - 0.4).abs() < 1e-6);
    }

    /// Forward traversal returns nothing when depth is 0.
    #[test]
    fn test_forward_traversal_depth_0() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();

        let a = write_py(r, "a.py", "from .b import x");
        let _b = write_py(r, "b.py", "y = 1");

        let files = vec![
            (a.clone(), fs::read_to_string(&a).unwrap()),
            (_b.clone(), fs::read_to_string(&_b).unwrap()),
        ];
        let graph = build_graph(r, &files);

        let entries = forward_traverse(&graph, &a, 0);
        assert!(
            entries.is_empty(),
            "depth 0 should return no forward dependencies"
        );
    }

    /// Forward traversal does not include the start file itself.
    #[test]
    fn test_forward_traversal_excludes_start() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();

        let a = write_py(r, "a.py", "from .b import x");
        let b = write_py(r, "b.py", "y = 1");

        let files = vec![
            (a.clone(), fs::read_to_string(&a).unwrap()),
            (b.clone(), fs::read_to_string(&b).unwrap()),
        ];
        let graph = build_graph(r, &files);

        let entries = forward_traverse(&graph, &a, 2);
        let a_str = a.to_string_lossy().to_string();
        assert!(
            !entries.iter().any(|e| e.path == a_str),
            "start file should not be in forward traversal results"
        );
    }

    // ====================================================================
    // Backward Traversal Tests
    // ====================================================================

    /// R4: Backward BFS finds direct callers of a symbol.
    #[test]
    fn test_backward_traversal_callers() {
        let mut cg = CallGraphIndex::default();
        // handler calls get_user, process calls get_user
        cg.called_by.insert(
            "get_user".to_string(),
            vec!["handler".to_string(), "process".to_string()],
        );

        let entries = backward_traverse(&cg, "get_user", 1);
        assert_eq!(entries.len(), 2, "should find 2 direct callers");

        for entry in &entries {
            assert_eq!(entry.depth, 1);
            assert_eq!(entry.reason, ContextReason::CallsTarget);
            assert!((entry.score - 0.8).abs() < 1e-6);
        }

        let names: HashSet<_> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(names.contains("handler"));
        assert!(names.contains("process"));
    }

    /// R4, R2: Backward BFS respects depth limit.
    #[test]
    fn test_backward_traversal_depth_limit() {
        let mut cg = CallGraphIndex::default();
        // get_user <- handler <- controller <- router (3 levels)
        cg.called_by
            .insert("get_user".to_string(), vec!["handler".to_string()]);
        cg.called_by
            .insert("handler".to_string(), vec!["controller".to_string()]);
        cg.called_by
            .insert("controller".to_string(), vec!["router".to_string()]);

        // depth=1: only handler
        let entries_d1 = backward_traverse(&cg, "get_user", 1);
        assert_eq!(entries_d1.len(), 1);
        assert_eq!(entries_d1[0].path, "handler");
        assert_eq!(entries_d1[0].depth, 1);
        assert_eq!(entries_d1[0].reason, ContextReason::CallsTarget);

        // depth=2: handler + controller
        let entries_d2 = backward_traverse(&cg, "get_user", 2);
        assert_eq!(entries_d2.len(), 2);
        let d2_names: HashSet<_> = entries_d2.iter().map(|e| e.path.as_str()).collect();
        assert!(d2_names.contains("handler"));
        assert!(d2_names.contains("controller"));

        // controller should be depth 2 with TransitiveCaller reason
        let ctrl = entries_d2.iter().find(|e| e.path == "controller").unwrap();
        assert_eq!(ctrl.depth, 2);
        assert_eq!(ctrl.reason, ContextReason::TransitiveCaller);
        assert!((ctrl.score - 0.4).abs() < 1e-6);

        // depth=3: handler + controller + router
        let entries_d3 = backward_traverse(&cg, "get_user", 3);
        assert_eq!(entries_d3.len(), 3);
    }

    /// Backward traversal returns nothing when depth is 0.
    #[test]
    fn test_backward_traversal_depth_0() {
        let mut cg = CallGraphIndex::default();
        cg.called_by
            .insert("foo".to_string(), vec!["bar".to_string()]);

        let entries = backward_traverse(&cg, "foo", 0);
        assert!(entries.is_empty(), "depth 0 should return no callers");
    }

    /// Backward traversal with no callers returns empty.
    #[test]
    fn test_backward_traversal_no_callers() {
        let cg = CallGraphIndex::default();
        let entries = backward_traverse(&cg, "orphan_func", 5);
        assert!(
            entries.is_empty(),
            "symbol with no callers should return empty"
        );
    }

    // ====================================================================
    // Scoring Tests
    // ====================================================================

    #[test]
    fn test_depth_to_score_formula() {
        assert!((depth_to_score(0) - 1.0).abs() < 1e-6);
        assert!((depth_to_score(1) - 0.8).abs() < 1e-6);
        assert!((depth_to_score(2) - 0.4).abs() < 1e-6);
        assert!((depth_to_score(3) - 0.2).abs() < 1e-6);
        assert!((depth_to_score(4) - 0.1).abs() < 1e-6);
    }
}

```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/context_builder/traversal.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Regenerate graph traversal, call/import extraction, scoring, and tests from the source section.
```
