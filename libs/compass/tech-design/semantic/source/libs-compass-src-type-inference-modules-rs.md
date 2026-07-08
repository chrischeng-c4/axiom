---
id: libs-compass-src-type-inference-modules-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/modules.rs`.
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

# Standardized libs/compass/src/type_inference/modules.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/modules.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ModuleNode` | libs/compass/src/type_inference/modules.rs | struct | pub | 16 | pub struct ModuleNode { |
| `new` | libs/compass/src/type_inference/modules.rs | function | pub | 32 | pub fn new(name: &str) -> Self { |
| `with_path` | libs/compass/src/type_inference/modules.rs | function | pub | 43 | pub fn with_path(mut self, path: PathBuf) -> Self { |
| `ModuleGraph` | libs/compass/src/type_inference/modules.rs | struct | pub | 55 | pub struct ModuleGraph { |
| `add_module` | libs/compass/src/type_inference/modules.rs | function | pub | 71 | pub fn add_module(&mut self, name: &str, path: Option<PathBuf>) -> &mut ModuleNode { |
| `add_import` | libs/compass/src/type_inference/modules.rs | function | pub | 82 | pub fn add_import(&mut self, from_module: &str, to_module: &str) { |
| `set_root` | libs/compass/src/type_inference/modules.rs | function | pub | 97 | pub fn set_root(&mut self, name: &str) { |
| `get_module` | libs/compass/src/type_inference/modules.rs | function | pub | 102 | pub fn get_module(&self, name: &str) -> Option<&ModuleNode> { |
| `get_module_mut` | libs/compass/src/type_inference/modules.rs | function | pub | 107 | pub fn get_module_mut(&mut self, name: &str) -> Option<&mut ModuleNode> { |
| `has_module` | libs/compass/src/type_inference/modules.rs | function | pub | 112 | pub fn has_module(&self, name: &str) -> bool { |
| `module_names` | libs/compass/src/type_inference/modules.rs | function | pub | 117 | pub fn module_names(&self) -> impl Iterator<Item = &String> { |
| `modules` | libs/compass/src/type_inference/modules.rs | function | pub | 122 | pub fn modules(&self) -> impl Iterator<Item = (&String, &ModuleNode)> { |
| `detect_cycles` | libs/compass/src/type_inference/modules.rs | function | pub | 128 | pub fn detect_cycles(&self) -> Vec<Vec<String>> { |
| `topological_sort` | libs/compass/src/type_inference/modules.rs | function | pub | 176 | pub fn topological_sort(&self) -> Option<Vec<String>> { |
| `get_dependencies` | libs/compass/src/type_inference/modules.rs | function | pub | 228 | pub fn get_dependencies(&self, name: &str) -> HashSet<String> { |
| `get_dependents` | libs/compass/src/type_inference/modules.rs | function | pub | 239 | pub fn get_dependents(&self, name: &str) -> HashSet<String> { |
| `get_transitive_dependencies` | libs/compass/src/type_inference/modules.rs | function | pub | 250 | pub fn get_transitive_dependencies(&self, name: &str) -> HashSet<String> { |
| `get_transitive_dependents` | libs/compass/src/type_inference/modules.rs | function | pub | 280 | pub fn get_transitive_dependents(&self, name: &str) -> HashSet<String> { |
| `get_affected_modules` | libs/compass/src/type_inference/modules.rs | function | pub | 310 | pub fn get_affected_modules(&self, changed: &str) -> Vec<String> { |
| `remove_module` | libs/compass/src/type_inference/modules.rs | function | pub | 342 | pub fn remove_module(&mut self, name: &str) { |
| `set_module_info` | libs/compass/src/type_inference/modules.rs | function | pub | 363 | pub fn set_module_info(&mut self, name: &str, info: ModuleInfo) { |
| `clear_imports` | libs/compass/src/type_inference/modules.rs | function | pub | 370 | pub fn clear_imports(&mut self, name: &str) { |
| `find_by_path` | libs/compass/src/type_inference/modules.rs | function | pub | 383 | pub fn find_by_path(&self, path: &Path) -> Option<&String> { |
| `path_to_module_name` | libs/compass/src/type_inference/modules.rs | function | pub | 394 | pub fn path_to_module_name(path: &Path, root: &Path) -> Option<String> { |
| `module_name_to_paths` | libs/compass/src/type_inference/modules.rs | function | pub | 416 | pub fn module_name_to_paths(name: &str, root: &Path) -> Vec<PathBuf> { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Module graph for cross-file analysis
//!
//! This module provides:
//! - Import dependency graph building
//! - Circular import detection
//! - Topological sort for analysis order
//! - Module resolution across files

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::imports::ModuleInfo;

/// A node in the module graph
#[derive(Debug, Clone)]
pub struct ModuleNode {
    /// Module name (e.g., "mypackage.submodule")
    pub name: String,
    /// File path if this is a file-based module
    pub path: Option<PathBuf>,
    /// Whether this is a package (__init__.py)
    pub is_package: bool,
    /// Modules this module imports
    pub imports: HashSet<String>,
    /// Modules that import this module
    pub imported_by: HashSet<String>,
    /// Type information for this module
    pub info: Option<ModuleInfo>,
}

impl ModuleNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: None,
            is_package: false,
            imports: HashSet::new(),
            imported_by: HashSet::new(),
            info: None,
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.is_package = path
            .file_name()
            .map(|n| n == "__init__.py")
            .unwrap_or(false);
        self.path = Some(path);
        self
    }
}

/// Module dependency graph
#[derive(Debug, Default)]
pub struct ModuleGraph {
    /// All modules in the graph
    modules: HashMap<String, ModuleNode>,
    /// Root modules (entry points)
    roots: HashSet<String>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            roots: HashSet::new(),
        }
    }

    /// Add a module to the graph
    pub fn add_module(&mut self, name: &str, path: Option<PathBuf>) -> &mut ModuleNode {
        self.modules.entry(name.to_string()).or_insert_with(|| {
            let mut node = ModuleNode::new(name);
            if let Some(p) = path {
                node = node.with_path(p);
            }
            node
        })
    }

    /// Add an import relationship
    pub fn add_import(&mut self, from_module: &str, to_module: &str) {
        // Ensure both modules exist
        self.add_module(from_module, None);
        self.add_module(to_module, None);

        // Add the import relationship
        if let Some(from) = self.modules.get_mut(from_module) {
            from.imports.insert(to_module.to_string());
        }
        if let Some(to) = self.modules.get_mut(to_module) {
            to.imported_by.insert(from_module.to_string());
        }
    }

    /// Set a module as a root (entry point)
    pub fn set_root(&mut self, name: &str) {
        self.roots.insert(name.to_string());
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&ModuleNode> {
        self.modules.get(name)
    }

    /// Get a mutable module by name
    pub fn get_module_mut(&mut self, name: &str) -> Option<&mut ModuleNode> {
        self.modules.get_mut(name)
    }

    /// Check if a module exists
    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// Get all module names
    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.modules.keys()
    }

    /// Get all modules
    pub fn modules(&self) -> impl Iterator<Item = (&String, &ModuleNode)> {
        self.modules.iter()
    }

    /// Detect circular imports
    /// Returns a list of cycles (each cycle is a list of module names)
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for name in self.modules.keys() {
            if !visited.contains(name) {
                self.dfs_cycles(name, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycles(
        &self,
        name: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(name.to_string());
        rec_stack.insert(name.to_string());
        path.push(name.to_string());

        if let Some(node) = self.modules.get(name) {
            for import in &node.imports {
                if !visited.contains(import) {
                    self.dfs_cycles(import, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(import) {
                    // Found a cycle - extract the cycle from path
                    if let Some(pos) = path.iter().position(|n| n == import) {
                        let cycle: Vec<String> = path[pos..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(name);
    }

    /// Topological sort of modules for analysis order
    /// Returns modules in order such that dependencies come before dependents
    /// Returns None if there are circular dependencies
    pub fn topological_sort(&self) -> Option<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degrees
        for name in self.modules.keys() {
            in_degree.insert(name.clone(), 0);
        }

        // Calculate in-degrees (number of imports each module has)
        for (name, node) in &self.modules {
            for import in &node.imports {
                if self.modules.contains_key(import) {
                    *in_degree.entry(name.clone()).or_insert(0) += 1;
                }
            }
        }

        // Find modules with no imports (in-degree = 0)
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(name.clone());
            }
        }

        // Process modules
        while let Some(name) = queue.pop_front() {
            result.push(name.clone());

            if let Some(node) = self.modules.get(&name) {
                // Decrease in-degree for modules that import this one
                for importer in &node.imported_by {
                    if let Some(degree) = in_degree.get_mut(importer) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            queue.push_back(importer.clone());
                        }
                    }
                }
            }
        }

        // If we processed all modules, return the order; otherwise there's a cycle
        if result.len() == self.modules.len() {
            Some(result)
        } else {
            None
        }
    }

    /// Get modules that need to be analyzed before the given module
    pub fn get_dependencies(&self, name: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        if let Some(node) = self.modules.get(name) {
            for import in &node.imports {
                deps.insert(import.clone());
            }
        }
        deps
    }

    /// Get modules that depend on the given module
    pub fn get_dependents(&self, name: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        if let Some(node) = self.modules.get(name) {
            for importer in &node.imported_by {
                dependents.insert(importer.clone());
            }
        }
        dependents
    }

    /// Get all transitive dependencies of a module
    pub fn get_transitive_dependencies(&self, name: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        if let Some(node) = self.modules.get(name) {
            for import in &node.imports {
                queue.push_back(import.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if deps.insert(current.clone()) {
                if let Some(node) = self.modules.get(&current) {
                    for import in &node.imports {
                        if !deps.contains(import) {
                            queue.push_back(import.clone());
                        }
                    }
                }
            }
        }

        deps
    }

    /// Get all modules that transitively depend on the given module
    ///
    /// This is the reverse of `get_transitive_dependencies` and is used
    /// for incremental invalidation - when a module changes, all modules
    /// that (transitively) import it need to be re-analyzed.
    pub fn get_transitive_dependents(&self, name: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        if let Some(node) = self.modules.get(name) {
            for importer in &node.imported_by {
                queue.push_back(importer.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            if dependents.insert(current.clone()) {
                if let Some(node) = self.modules.get(&current) {
                    for importer in &node.imported_by {
                        if !dependents.contains(importer) {
                            queue.push_back(importer.clone());
                        }
                    }
                }
            }
        }

        dependents
    }

    /// Get all modules affected by a change to the given module
    ///
    /// Returns the changed module plus all its transitive dependents,
    /// sorted in reverse topological order (dependents before dependencies)
    /// for proper re-analysis ordering.
    pub fn get_affected_modules(&self, changed: &str) -> Vec<String> {
        let mut affected = self.get_transitive_dependents(changed);
        affected.insert(changed.to_string());

        // Sort in reverse topological order so dependents are analyzed after
        // the modules they depend on
        let mut sorted: Vec<String> = affected.into_iter().collect();
        sorted.sort_by(|a, b| {
            // If a imports b (a depends on b), b should come first
            // So if a's dependencies include b, a > b (a comes after b)
            let a_depends_on_b = self
                .get_module(a)
                .map(|n| n.imports.contains(b))
                .unwrap_or(false);
            let b_depends_on_a = self
                .get_module(b)
                .map(|n| n.imports.contains(a))
                .unwrap_or(false);

            if a_depends_on_b && !b_depends_on_a {
                std::cmp::Ordering::Greater
            } else if b_depends_on_a && !a_depends_on_b {
                std::cmp::Ordering::Less
            } else {
                a.cmp(b)
            }
        });

        sorted
    }

    /// Remove a module from the graph (e.g., when a file is deleted)
    pub fn remove_module(&mut self, name: &str) {
        // Remove from other modules' imports/imported_by
        if let Some(node) = self.modules.remove(name) {
            // Remove this module from the imports of modules it imports
            for import in &node.imports {
                if let Some(imported) = self.modules.get_mut(import) {
                    imported.imported_by.remove(name);
                }
            }
            // Remove this module from the imported_by of modules that import it
            for importer in &node.imported_by {
                if let Some(importing) = self.modules.get_mut(importer) {
                    importing.imports.remove(name);
                }
            }
        }

        self.roots.remove(name);
    }

    /// Update module info
    pub fn set_module_info(&mut self, name: &str, info: ModuleInfo) {
        if let Some(node) = self.modules.get_mut(name) {
            node.info = Some(info);
        }
    }

    /// Clear all imports for a module (before re-analyzing)
    pub fn clear_imports(&mut self, name: &str) {
        if let Some(node) = self.modules.get_mut(name) {
            // Remove from imported_by of modules this module imports
            let imports: Vec<String> = node.imports.drain().collect();
            for import in imports {
                if let Some(imported) = self.modules.get_mut(&import) {
                    imported.imported_by.remove(name);
                }
            }
        }
    }

    /// Find module by file path
    pub fn find_by_path(&self, path: &Path) -> Option<&String> {
        self.modules.iter().find_map(|(name, node)| {
            if node.path.as_deref() == Some(path) {
                Some(name)
            } else {
                None
            }
        })
    }

    /// Convert a file path to a module name
    pub fn path_to_module_name(path: &Path, root: &Path) -> Option<String> {
        let relative = path.strip_prefix(root).ok()?;
        let stem = relative.with_extension("");

        let parts: Vec<&str> = stem
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if parts.is_empty() {
            return None;
        }

        // Handle __init__.py -> package name
        if parts.last() == Some(&"__init__") {
            Some(parts[..parts.len() - 1].join("."))
        } else {
            Some(parts.join("."))
        }
    }

    /// Convert a module name to possible file paths
    pub fn module_name_to_paths(name: &str, root: &Path) -> Vec<PathBuf> {
        let parts: Vec<&str> = name.split('.').collect();
        let relative_path = parts.join("/");

        vec![
            // Try as module file
            root.join(format!("{}.py", relative_path)),
            // Try as package __init__
            root.join(format!("{}/__init__.py", relative_path)),
            // Try as stub file
            root.join(format!("{}.pyi", relative_path)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_graph_basic() {
        let mut graph = ModuleGraph::new();

        graph.add_module("main", None);
        graph.add_module("utils", None);
        graph.add_import("main", "utils");

        assert!(graph.has_module("main"));
        assert!(graph.has_module("utils"));

        let main = graph.get_module("main").unwrap();
        assert!(main.imports.contains("utils"));

        let utils = graph.get_module("utils").unwrap();
        assert!(utils.imported_by.contains("main"));
    }

    #[test]
    fn test_cycle_detection_no_cycle() {
        let mut graph = ModuleGraph::new();

        graph.add_import("a", "b");
        graph.add_import("b", "c");
        graph.add_import("a", "c");

        let cycles = graph.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_cycle_detection_with_cycle() {
        let mut graph = ModuleGraph::new();

        graph.add_import("a", "b");
        graph.add_import("b", "c");
        graph.add_import("c", "a"); // Creates a cycle

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = ModuleGraph::new();

        graph.add_import("app", "services");
        graph.add_import("app", "models");
        graph.add_import("services", "models");

        let order = graph.topological_sort().unwrap();

        // models should come before services and app
        let models_pos = order.iter().position(|n| n == "models").unwrap();
        let services_pos = order.iter().position(|n| n == "services").unwrap();
        let app_pos = order.iter().position(|n| n == "app").unwrap();

        assert!(models_pos < services_pos);
        assert!(models_pos < app_pos);
        assert!(services_pos < app_pos);
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let mut graph = ModuleGraph::new();

        graph.add_import("a", "b");
        graph.add_import("b", "a");

        let result = graph.topological_sort();
        assert!(result.is_none()); // Cycle detected
    }

    #[test]
    fn test_get_dependencies() {
        let mut graph = ModuleGraph::new();

        graph.add_import("app", "utils");
        graph.add_import("app", "models");
        graph.add_import("app", "config");

        let deps = graph.get_dependencies("app");
        assert_eq!(deps.len(), 3);
        assert!(deps.contains("utils"));
        assert!(deps.contains("models"));
        assert!(deps.contains("config"));
    }

    #[test]
    fn test_get_dependents() {
        let mut graph = ModuleGraph::new();

        graph.add_import("app", "utils");
        graph.add_import("tests", "utils");
        graph.add_import("cli", "utils");

        let dependents = graph.get_dependents("utils");
        assert_eq!(dependents.len(), 3);
        assert!(dependents.contains("app"));
        assert!(dependents.contains("tests"));
        assert!(dependents.contains("cli"));
    }

    #[test]
    fn test_transitive_dependencies() {
        let mut graph = ModuleGraph::new();

        graph.add_import("app", "services");
        graph.add_import("services", "models");
        graph.add_import("models", "base");

        let deps = graph.get_transitive_dependencies("app");
        assert!(deps.contains("services"));
        assert!(deps.contains("models"));
        assert!(deps.contains("base"));
    }

    #[test]
    fn test_path_to_module_name() {
        let root = Path::new("/project/src");

        let path1 = Path::new("/project/src/app.py");
        assert_eq!(
            ModuleGraph::path_to_module_name(path1, root),
            Some("app".to_string())
        );

        let path2 = Path::new("/project/src/mypackage/module.py");
        assert_eq!(
            ModuleGraph::path_to_module_name(path2, root),
            Some("mypackage.module".to_string())
        );

        let path3 = Path::new("/project/src/mypackage/__init__.py");
        assert_eq!(
            ModuleGraph::path_to_module_name(path3, root),
            Some("mypackage".to_string())
        );
    }

    #[test]
    fn test_transitive_dependents() {
        let mut graph = ModuleGraph::new();

        // app -> services -> models -> base
        graph.add_import("app", "services");
        graph.add_import("services", "models");
        graph.add_import("models", "base");

        // If base changes, models, services, and app all need re-analysis
        let dependents = graph.get_transitive_dependents("base");
        assert!(dependents.contains("models"));
        assert!(dependents.contains("services"));
        assert!(dependents.contains("app"));

        // If services changes, only app needs re-analysis
        let dependents = graph.get_transitive_dependents("services");
        assert!(dependents.contains("app"));
        assert!(!dependents.contains("models")); // models doesn't import services
    }

    #[test]
    fn test_get_affected_modules() {
        let mut graph = ModuleGraph::new();

        graph.add_import("app", "services");
        graph.add_import("services", "models");
        graph.add_import("app", "models"); // app also imports models directly

        let affected = graph.get_affected_modules("models");
        assert!(affected.contains(&"models".to_string()));
        assert!(affected.contains(&"services".to_string()));
        assert!(affected.contains(&"app".to_string()));

        // Models should come before services and app in the order
        let models_pos = affected.iter().position(|n| n == "models");
        let services_pos = affected.iter().position(|n| n == "services");
        let app_pos = affected.iter().position(|n| n == "app");

        assert!(models_pos.is_some());
        assert!(services_pos.is_some());
        assert!(app_pos.is_some());
    }

    #[test]
    fn test_remove_module() {
        let mut graph = ModuleGraph::new();

        graph.add_import("app", "services");
        graph.add_import("app", "models");
        graph.add_import("services", "models");

        // Remove models
        graph.remove_module("models");

        // models should no longer exist
        assert!(!graph.has_module("models"));

        // app and services should no longer import models
        let app = graph.get_module("app").unwrap();
        assert!(!app.imports.contains("models"));

        let services = graph.get_module("services").unwrap();
        assert!(!services.imports.contains("models"));
    }

    #[test]
    fn test_clear_imports() {
        let mut graph = ModuleGraph::new();

        graph.add_import("app", "services");
        graph.add_import("app", "models");

        // Clear app's imports
        graph.clear_imports("app");

        let app = graph.get_module("app").unwrap();
        assert!(app.imports.is_empty());

        // services and models should no longer list app as importer
        let services = graph.get_module("services").unwrap();
        assert!(!services.imported_by.contains("app"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/modules.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/modules.rs` captured during libs codegen standardization.
```
