---
id: projects-meter-src-fixtures-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/fixtures.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/fixtures.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FixtureMeta` | projects/meter/src/fixtures.rs | struct | pub | 85 |  |
| `FixtureRegistry` | projects/meter/src/fixtures.rs | struct | pub | 136 |  |
| `FixtureScope` | projects/meter/src/fixtures.rs | enum | pub | 22 |  |
| `detect_circular_deps` | projects/meter/src/fixtures.rs | function | pub | 226 | detect_circular_deps(&self) -> Result<(), Vec<String>> |
| `get_all_names` | projects/meter/src/fixtures.rs | function | pub | 161 | get_all_names(&self) -> Vec<String> |
| `get_autouse_fixtures` | projects/meter/src/fixtures.rs | function | pub | 166 | get_autouse_fixtures(&self, scope: FixtureScope) -> Vec<&FixtureMeta> |
| `get_dependencies` | projects/meter/src/fixtures.rs | function | pub | 174 | get_dependencies(&self, name: &str) -> Option<&[String]> |
| `get_meta` | projects/meter/src/fixtures.rs | function | pub | 156 | get_meta(&self, name: &str) -> Option<&FixtureMeta> |
| `has_fixture` | projects/meter/src/fixtures.rs | function | pub | 278 | has_fixture(&self, name: &str) -> bool |
| `is_empty` | projects/meter/src/fixtures.rs | function | pub | 288 | is_empty(&self) -> bool |
| `len` | projects/meter/src/fixtures.rs | function | pub | 283 | len(&self) -> usize |
| `new` | projects/meter/src/fixtures.rs | function | pub | 101 | new(name: impl Into<String>, scope: FixtureScope, autouse: bool) -> Self |
| `new` | projects/meter/src/fixtures.rs | function | pub | 144 | new() -> Self |
| `register` | projects/meter/src/fixtures.rs | function | pub | 151 | register(&mut self, meta: FixtureMeta) |
| `resolve_order` | projects/meter/src/fixtures.rs | function | pub | 181 | resolve_order(&self, fixture_names: &[String]) -> Result<Vec<String>, String> |
| `should_cleanup_before` | projects/meter/src/fixtures.rs | function | pub | 63 | should_cleanup_before(&self, other: &FixtureScope) -> bool |
| `with_dependencies` | projects/meter/src/fixtures.rs | function | pub | 124 | with_dependencies(mut self, deps: Vec<String>) -> Self |
| `with_dependency` | projects/meter/src/fixtures.rs | function | pub | 112 | with_dependency(mut self, dep: impl Into<String>) -> Self |
| `with_teardown` | projects/meter/src/fixtures.rs | function | pub | 118 | with_teardown(mut self, has_teardown: bool) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Fixture system for cclab-probe
//!
//! Provides pytest-compatible fixture functionality:
//! - Scope-based lifecycle (function, class, module, session)
//! - Dependency resolution (fixtures can depend on other fixtures)
//! - Setup/teardown support
//! - Autouse fixtures
//! - Cleanup guarantees (teardown runs even on failure)
//!
//! This is a pure Rust implementation. Python-compatible bindings are provided
//! by the Mamba layer.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// Fixture scope determines lifecycle and caching strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
pub enum FixtureScope {
    /// Function scope - executed once per test function (default)
    Function,
    /// Class scope - executed once per test class
    Class,
    /// Module scope - executed once per test module
    Module,
    /// Session scope - executed once per test session
    Session,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
impl std::fmt::Display for FixtureScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureScope::Function => write!(f, "function"),
            FixtureScope::Class => write!(f, "class"),
            FixtureScope::Module => write!(f, "module"),
            FixtureScope::Session => write!(f, "session"),
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
impl FromStr for FixtureScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "function" => Ok(FixtureScope::Function),
            "class" => Ok(FixtureScope::Class),
            "module" => Ok(FixtureScope::Module),
            "session" => Ok(FixtureScope::Session),
            _ => Err(format!("Invalid fixture scope: {}", s)),
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
impl FixtureScope {
    /// Check if this scope should be cleaned up before another scope
    pub fn should_cleanup_before(&self, other: &FixtureScope) -> bool {
        use FixtureScope::*;
        match (self, other) {
            // Function scope cleanup before any new scope
            (Function, _) => true,
            // Class scope cleanup before new class/module/session
            (Class, Class) | (Class, Module) | (Class, Session) => true,
            // Module scope cleanup before new module/session
            (Module, Module) | (Module, Session) => true,
            // Session scope cleanup only at end
            (Session, Session) => true,
            _ => false,
        }
    }
}

/// Metadata for a fixture
///
/// This is a pure Rust representation. The actual fixture function is stored
/// as a Python object in the  binding layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
pub struct FixtureMeta {
    /// Fixture name (function name)
    pub name: String,
    /// Fixture scope
    pub scope: FixtureScope,
    /// Whether fixture is automatically used (autouse)
    pub autouse: bool,
    /// List of fixture names this fixture depends on
    pub dependencies: Vec<String>,
    /// Whether the fixture uses yield (has teardown)
    pub has_teardown: bool,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
impl FixtureMeta {
    /// Create new fixture metadata
    pub fn new(name: impl Into<String>, scope: FixtureScope, autouse: bool) -> Self {
        Self {
            name: name.into(),
            scope,
            autouse,
            dependencies: Vec::new(),
            has_teardown: false,
        }
    }

    /// Add dependency on another fixture
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Set whether fixture has teardown
    pub fn with_teardown(mut self, has_teardown: bool) -> Self {
        self.has_teardown = has_teardown;
        self
    }

    /// Set dependencies
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }
}

/// Fixture registry - manages fixture registration and dependency tracking
///
/// This is a pure Rust implementation. The  binding layer wraps this
/// and handles Python-specific operations (calling functions, etc).
#[derive(Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
pub struct FixtureRegistry {
    /// Registered fixtures by name
    fixtures: HashMap<String, FixtureMeta>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-fixtures-rs.md#source
impl FixtureRegistry {
    /// Create a new fixture registry
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
        }
    }

    /// Register a fixture
    pub fn register(&mut self, meta: FixtureMeta) {
        self.fixtures.insert(meta.name.clone(), meta);
    }

    /// Get fixture metadata by name
    pub fn get_meta(&self, name: &str) -> Option<&FixtureMeta> {
        self.fixtures.get(name)
    }

    /// Get all fixture names
    pub fn get_all_names(&self) -> Vec<String> {
        self.fixtures.keys().cloned().collect()
    }

    /// Get all autouse fixtures for a given scope
    pub fn get_autouse_fixtures(&self, scope: FixtureScope) -> Vec<&FixtureMeta> {
        self.fixtures
            .values()
            .filter(|f| f.autouse && f.scope == scope)
            .collect()
    }

    /// Get dependencies for a fixture
    pub fn get_dependencies(&self, name: &str) -> Option<&[String]> {
        self.fixtures.get(name).map(|f| f.dependencies.as_slice())
    }

    /// Resolve fixture dependency order using topological sort
    ///
    /// Returns fixtures in order they should be executed (dependencies first)
    pub fn resolve_order(&self, fixture_names: &[String]) -> Result<Vec<String>, String> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for name in fixture_names {
            self.visit_fixture(name, &mut resolved, &mut visited, &mut visiting)?;
        }

        Ok(resolved)
    }

    /// DFS visit for topological sort
    fn visit_fixture(
        &self,
        name: &str,
        resolved: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
    ) -> Result<(), String> {
        if visited.contains(name) {
            return Ok(());
        }

        if visiting.contains(name) {
            return Err(format!("Circular dependency detected involving '{}'", name));
        }

        visiting.insert(name.to_string());

        // Visit dependencies first
        if let Some(meta) = self.fixtures.get(name) {
            for dep in &meta.dependencies {
                self.visit_fixture(dep, resolved, visited, visiting)?;
            }
        }

        visiting.remove(name);
        visited.insert(name.to_string());
        resolved.push(name.to_string());

        Ok(())
    }

    /// Detect circular dependencies in fixture graph
    pub fn detect_circular_deps(&self) -> Result<(), Vec<String>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for fixture_name in self.fixtures.keys() {
            if !visited.contains(fixture_name) {
                if let Some(cycle) = self.dfs_cycle_detect(
                    fixture_name,
                    &mut visited,
                    &mut recursion_stack,
                    &mut Vec::new(),
                ) {
                    return Err(cycle);
                }
            }
        }

        Ok(())
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detect(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(meta) = self.fixtures.get(node) {
            for dep in &meta.dependencies {
                if !visited.contains(dep) {
                    if let Some(cycle) = self.dfs_cycle_detect(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    // Found cycle
                    let cycle_start = path.iter().position(|n| n == dep).unwrap_or(0);
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        None
    }

    /// Check if a fixture exists
    pub fn has_fixture(&self, name: &str) -> bool {
        self.fixtures.contains_key(name)
    }

    /// Get number of registered fixtures
    pub fn len(&self) -> usize {
        self.fixtures.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_scope_parsing() {
        assert_eq!(
            FixtureScope::from_str("function").unwrap(),
            FixtureScope::Function
        );
        assert_eq!(
            FixtureScope::from_str("class").unwrap(),
            FixtureScope::Class
        );
        assert_eq!(
            FixtureScope::from_str("module").unwrap(),
            FixtureScope::Module
        );
        assert_eq!(
            FixtureScope::from_str("session").unwrap(),
            FixtureScope::Session
        );
        assert!(FixtureScope::from_str("invalid").is_err());
    }

    #[test]
    fn test_scope_cleanup_order() {
        assert!(FixtureScope::Function.should_cleanup_before(&FixtureScope::Class));
        assert!(FixtureScope::Class.should_cleanup_before(&FixtureScope::Module));
        assert!(!FixtureScope::Module.should_cleanup_before(&FixtureScope::Class));
    }

    #[test]
    fn test_fixture_registry_creation() {
        let registry = FixtureRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_fixture_registration() {
        let mut registry = FixtureRegistry::new();

        let meta = FixtureMeta::new("my_fixture", FixtureScope::Function, false);
        registry.register(meta);

        assert_eq!(registry.len(), 1);
        assert!(registry.has_fixture("my_fixture"));
        assert!(!registry.has_fixture("other_fixture"));
    }

    #[test]
    fn test_dependency_resolution() {
        let mut registry = FixtureRegistry::new();

        // Register fixtures with dependencies
        // fixture_a depends on nothing
        let meta_a = FixtureMeta::new("fixture_a", FixtureScope::Function, false);
        registry.register(meta_a);

        // fixture_b depends on fixture_a
        let meta_b = FixtureMeta::new("fixture_b", FixtureScope::Function, false)
            .with_dependency("fixture_a");
        registry.register(meta_b);

        // fixture_c depends on both fixture_a and fixture_b
        let meta_c = FixtureMeta::new("fixture_c", FixtureScope::Function, false)
            .with_dependency("fixture_a")
            .with_dependency("fixture_b");
        registry.register(meta_c);

        // Resolve order for fixture_c
        let order = registry.resolve_order(&["fixture_c".to_string()]).unwrap();

        // fixture_a should come before fixture_b and fixture_c
        let pos_a = order.iter().position(|x| x == "fixture_a").unwrap();
        let pos_b = order.iter().position(|x| x == "fixture_b").unwrap();
        let pos_c = order.iter().position(|x| x == "fixture_c").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_c);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut registry = FixtureRegistry::new();

        // Create circular dependency: a -> b -> c -> a
        let meta_a = FixtureMeta::new("fixture_a", FixtureScope::Function, false)
            .with_dependency("fixture_c");
        registry.register(meta_a);

        let meta_b = FixtureMeta::new("fixture_b", FixtureScope::Function, false)
            .with_dependency("fixture_a");
        registry.register(meta_b);

        let meta_c = FixtureMeta::new("fixture_c", FixtureScope::Function, false)
            .with_dependency("fixture_b");
        registry.register(meta_c);

        // Should detect circular dependency
        assert!(registry.detect_circular_deps().is_err());
    }

    #[test]
    fn test_autouse_fixtures() {
        let mut registry = FixtureRegistry::new();

        let meta1 = FixtureMeta::new("auto_fixture", FixtureScope::Class, true);
        registry.register(meta1);

        let meta2 = FixtureMeta::new("manual_fixture", FixtureScope::Class, false);
        registry.register(meta2);

        let autouse = registry.get_autouse_fixtures(FixtureScope::Class);
        assert_eq!(autouse.len(), 1);
        assert_eq!(autouse[0].name, "auto_fixture");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/fixtures.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/fixtures.rs` captured during meter full-codegen standardization.
```
