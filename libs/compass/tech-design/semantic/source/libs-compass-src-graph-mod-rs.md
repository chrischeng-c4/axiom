---
id: libs-compass-src-graph-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/graph/mod.rs`.
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

# Standardized libs/compass/src/graph/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/graph/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `resolve` | libs/compass/src/graph/mod.rs | module | pub | 7 | pub mod resolve; |
| `GraphNode` | libs/compass/src/graph/mod.rs | struct | pub | 16 | pub struct GraphNode { |
| `ImportEdge` | libs/compass/src/graph/mod.rs | struct | pub | 23 | pub struct ImportEdge { |
| `ImportGraph` | libs/compass/src/graph/mod.rs | struct | pub | 33 | pub struct ImportGraph { |
| `new` | libs/compass/src/graph/mod.rs | function | pub | 45 | pub fn new() -> Self { |
| `build` | libs/compass/src/graph/mod.rs | function | pub | 53 | pub fn build(files: &[(PathBuf, String)], project_root: &Path) -> Self { |
| `add_file` | libs/compass/src/graph/mod.rs | function | pub | 63 | pub fn add_file(&mut self, path: PathBuf, source: &str, project_root: &Path) { |
| `remove_file` | libs/compass/src/graph/mod.rs | function | pub | 83 | pub fn remove_file(&mut self, path: &Path) { |
| `detect_entry_points` | libs/compass/src/graph/mod.rs | function | pub | 89 | pub fn detect_entry_points(&mut self) { |
| `find_circular_dependencies` | libs/compass/src/graph/mod.rs | function | pub | 120 | pub fn find_circular_dependencies(&self) -> Vec<Vec<PathBuf>> { |
| `find_unused_files` | libs/compass/src/graph/mod.rs | function | pub | 162 | pub fn find_unused_files(&self) -> Vec<PathBuf> { |
| `dependencies` | libs/compass/src/graph/mod.rs | function | pub | 201 | pub fn dependencies(&self, path: &Path) -> Vec<&ImportEdge> { |
| `dependents` | libs/compass/src/graph/mod.rs | function | pub | 209 | pub fn dependents(&self, path: &Path) -> Vec<PathBuf> { |
| `file_count` | libs/compass/src/graph/mod.rs | function | pub | 221 | pub fn file_count(&self) -> usize { |
| `edge_count` | libs/compass/src/graph/mod.rs | function | pub | 225 | pub fn edge_count(&self) -> usize { |
| `entry_points` | libs/compass/src/graph/mod.rs | function | pub | 233 | pub fn entry_points(&self) -> &[PathBuf] { |
| `set_entry_points` | libs/compass/src/graph/mod.rs | function | pub | 237 | pub fn set_entry_points(&mut self, entries: Vec<PathBuf>) { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Cross-file import graph for project-wide dependency analysis
//!
//! Builds a directed graph where nodes are source files and edges are import
//! relationships. Supports circular dependency detection, unused file detection,
//! and incremental updates.

pub mod resolve;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use resolve::{extract_imports, resolve_import};

/// A node in the import graph (represents a file)
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub path: PathBuf,
    pub imports: Vec<ImportEdge>,
}

/// An edge in the import graph (an import statement)
#[derive(Debug, Clone)]
pub struct ImportEdge {
    /// The raw import path as written in source
    pub import_path: String,
    /// Resolved absolute file path (if resolvable)
    pub resolved: Option<PathBuf>,
    /// Line number where the import appears
    pub line: u32,
}

/// Project-wide import dependency graph
pub struct ImportGraph {
    nodes: HashMap<PathBuf, GraphNode>,
    entry_points: Vec<PathBuf>,
}

impl Default for ImportGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            entry_points: Vec::new(),
        }
    }

    /// Build the graph from a list of files and their source contents
    pub fn build(files: &[(PathBuf, String)], project_root: &Path) -> Self {
        let mut g = Self::new();
        for (path, source) in files {
            g.add_file(path.clone(), source, project_root);
        }
        g.detect_entry_points();
        g
    }

    /// Add a single file to the graph (for incremental updates)
    pub fn add_file(&mut self, path: PathBuf, source: &str, project_root: &Path) {
        let extracted = extract_imports(source, &path);
        let edges = extracted
            .iter()
            .map(|ext| ImportEdge {
                import_path: ext.path.clone(),
                resolved: resolve_import(&ext.path, &path, project_root, ext.language),
                line: ext.line,
            })
            .collect();
        self.nodes.insert(
            path.clone(),
            GraphNode {
                path,
                imports: edges,
            },
        );
    }

    /// Remove a file from the graph
    pub fn remove_file(&mut self, path: &Path) {
        self.nodes.remove(path);
        self.entry_points.retain(|p| p != path);
    }

    /// Auto-detect entry points
    pub fn detect_entry_points(&mut self) {
        let names: HashSet<&str> = [
            "main.py",
            "app.py",
            "__main__.py",
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "main.ts",
            "main.js",
            "main.rs",
            "lib.rs",
            "main.go",
        ]
        .into_iter()
        .collect();
        self.entry_points = self
            .nodes
            .keys()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| names.contains(n))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
    }

    /// Find all circular dependencies via DFS with recursion stack
    pub fn find_circular_dependencies(&self) -> Vec<Vec<PathBuf>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        let mut on_stack = HashSet::new();
        for start in self.nodes.keys() {
            if !visited.contains(start) {
                self.dfs_cycles(start, &mut visited, &mut stack, &mut on_stack, &mut cycles);
            }
        }
        dedup_cycles(&mut cycles);
        cycles
    }

    fn dfs_cycles(
        &self,
        node: &Path,
        visited: &mut HashSet<PathBuf>,
        stack: &mut Vec<PathBuf>,
        on_stack: &mut HashSet<PathBuf>,
        cycles: &mut Vec<Vec<PathBuf>>,
    ) {
        visited.insert(node.to_path_buf());
        stack.push(node.to_path_buf());
        on_stack.insert(node.to_path_buf());
        if let Some(gn) = self.nodes.get(node) {
            for edge in &gn.imports {
                if let Some(ref target) = edge.resolved {
                    if on_stack.contains(target) {
                        let pos = stack.iter().position(|p| p == target).unwrap();
                        cycles.push(stack[pos..].to_vec());
                    } else if !visited.contains(target) {
                        self.dfs_cycles(target, visited, stack, on_stack, cycles);
                    }
                }
            }
        }
        stack.pop();
        on_stack.remove(node);
    }

    /// Find files unreachable from any entry point
    pub fn find_unused_files(&self) -> Vec<PathBuf> {
        if self.entry_points.is_empty() {
            return Vec::new();
        }
        let reachable = self.bfs_reachable(&self.entry_points);
        let mut unused: Vec<PathBuf> = self
            .nodes
            .keys()
            .filter(|p| !reachable.contains(*p))
            .cloned()
            .collect();
        unused.sort();
        unused
    }

    fn bfs_reachable(&self, starts: &[PathBuf]) -> HashSet<PathBuf> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        for s in starts {
            if self.nodes.contains_key(s) {
                visited.insert(s.clone());
                queue.push_back(s.clone());
            }
        }
        while let Some(cur) = queue.pop_front() {
            if let Some(n) = self.nodes.get(&cur) {
                for e in &n.imports {
                    if let Some(ref t) = e.resolved {
                        if visited.insert(t.clone()) {
                            queue.push_back(t.clone());
                        }
                    }
                }
            }
        }
        visited
    }

    /// Get all direct dependencies of a file
    pub fn dependencies(&self, path: &Path) -> Vec<&ImportEdge> {
        self.nodes
            .get(path)
            .map(|n| n.imports.iter().collect())
            .unwrap_or_default()
    }

    /// Get all files that import this file (reverse dependencies)
    pub fn dependents(&self, path: &Path) -> Vec<PathBuf> {
        self.nodes
            .iter()
            .filter(|(_, n)| {
                n.imports
                    .iter()
                    .any(|e| e.resolved.as_deref() == Some(path))
            })
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn file_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.nodes
            .values()
            .flat_map(|n| &n.imports)
            .filter(|e| e.resolved.is_some())
            .count()
    }

    pub fn entry_points(&self) -> &[PathBuf] {
        &self.entry_points
    }

    pub fn set_entry_points(&mut self, entries: Vec<PathBuf>) {
        self.entry_points = entries;
    }
}

// -- Helpers -----------------------------------------------------------------

fn dedup_cycles(cycles: &mut Vec<Vec<PathBuf>>) {
    let mut seen: HashSet<Vec<PathBuf>> = HashSet::new();
    cycles.retain(|c| {
        if c.is_empty() {
            return false;
        }
        let min_idx = c
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.cmp(b))
            .unwrap()
            .0;
        let mut norm = Vec::with_capacity(c.len());
        norm.extend_from_slice(&c[min_idx..]);
        norm.extend_from_slice(&c[..min_idx]);
        seen.insert(norm)
    });
}

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_py(dir: &Path, name: &str, content: &str) -> PathBuf {
        let p = dir.join(name);
        if let Some(par) = p.parent() {
            fs::create_dir_all(par).unwrap();
        }
        fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn test_build_chain_a_b_c() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();
        let a = write_py(r, "a.py", "from .b import x");
        let b = write_py(r, "b.py", "from .c import y");
        let c = write_py(r, "c.py", "x = 1");
        let files = vec![
            (a.clone(), fs::read_to_string(&a).unwrap()),
            (b.clone(), fs::read_to_string(&b).unwrap()),
            (c.clone(), fs::read_to_string(&c).unwrap()),
        ];
        let g = ImportGraph::build(&files, r);
        assert_eq!(g.file_count(), 3);
        assert_eq!(g.edge_count(), 2);
        assert_eq!(g.dependencies(&a)[0].resolved, Some(b.clone()));
        assert_eq!(g.dependencies(&b)[0].resolved, Some(c.clone()));
        assert!(g.dependencies(&c).is_empty());
    }

    #[test]
    fn test_circular_dependency() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();
        let a = write_py(r, "a.py", "from .b import x");
        let b = write_py(r, "b.py", "from .c import y");
        let c = write_py(r, "c.py", "from .a import z");
        let files = vec![
            (a.clone(), fs::read_to_string(&a).unwrap()),
            (b.clone(), fs::read_to_string(&b).unwrap()),
            (c.clone(), fs::read_to_string(&c).unwrap()),
        ];
        let g = ImportGraph::build(&files, r);
        let cycles = g.find_circular_dependencies();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
        let set: HashSet<_> = cycles[0].iter().collect();
        assert!(set.contains(&a) && set.contains(&b) && set.contains(&c));
    }

    #[test]
    fn test_unused_file_detection() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();
        let entry = write_py(r, "main.py", "from .a import x");
        let a = write_py(r, "a.py", "from .b import y");
        let _b = write_py(r, "b.py", "z = 1");
        let orphan = write_py(r, "orphan.py", "lonely = True");
        let files = vec![
            (entry.clone(), fs::read_to_string(&entry).unwrap()),
            (a.clone(), fs::read_to_string(&a).unwrap()),
            (_b.clone(), fs::read_to_string(&_b).unwrap()),
            (orphan.clone(), fs::read_to_string(&orphan).unwrap()),
        ];
        let g = ImportGraph::build(&files, r);
        assert!(g.entry_points().contains(&entry));
        let unused = g.find_unused_files();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0], orphan);
    }

    #[test]
    fn test_entry_point_detection() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();
        let mp = write_py(r, "main.py", "");
        let it = write_py(r, "index.ts", "");
        let lr = write_py(r, "lib.rs", "");
        let ut = write_py(r, "utils.py", "");
        let files = vec![
            (mp.clone(), String::new()),
            (it.clone(), String::new()),
            (lr.clone(), String::new()),
            (ut.clone(), String::new()),
        ];
        let g = ImportGraph::build(&files, r);
        let ep: HashSet<_> = g.entry_points().iter().collect();
        assert!(ep.contains(&mp) && ep.contains(&it) && ep.contains(&lr));
        assert!(!ep.contains(&ut));
    }

    #[test]
    fn test_incremental_add_remove() {
        let tmp = TempDir::new().unwrap();
        let r = tmp.path();
        let a = write_py(r, "a.py", "x = 1");
        let b = write_py(r, "b.py", "from .a import x");
        let mut g = ImportGraph::new();
        g.add_file(a.clone(), &fs::read_to_string(&a).unwrap(), r);
        assert_eq!(g.file_count(), 1);
        g.add_file(b.clone(), &fs::read_to_string(&b).unwrap(), r);
        assert_eq!(g.file_count(), 2);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.dependencies(&b)[0].resolved, Some(a.clone()));
        assert_eq!(g.dependents(&a), vec![b.clone()]);
        g.remove_file(&a);
        assert_eq!(g.file_count(), 1);
        assert!(g.dependencies(&a).is_empty());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/graph/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/graph/mod.rs` captured during libs codegen standardization.
```
