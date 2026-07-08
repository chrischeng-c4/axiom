---
id: libs-compass-src-core-index-config-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/core/index_config.rs`.
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

# Standardized libs/compass/src/core/index_config.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/core/index_config.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `IndexConfig` | libs/compass/src/core/index_config.rs | struct | pub | 14 | pub struct IndexConfig { |
| `ScopeConfig` | libs/compass/src/core/index_config.rs | struct | pub | 26 | pub struct ScopeConfig { |
| `ScopeLang` | libs/compass/src/core/index_config.rs | enum | pub | 54 | pub enum ScopeLang { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Index server configuration — scoped toolchain binding (#1127)
//!
//! Supports auto-discovery of project roots from marker files
//! (Cargo.toml, pyproject.toml, tsconfig.json) and per-scope
//! configuration of search paths, interpreters, and cache directories.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level index server configuration.
///
/// Deserialized from `[index]` section in `.aw/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexConfig {
    /// Auto-discover scopes from marker files (default: true)
    #[serde(default = "default_true")]
    pub auto_discover: bool,

    /// Explicitly configured scopes (merged with auto-discovered)
    #[serde(default, rename = "scope")]
    pub scopes: Vec<ScopeConfig>,
}

/// Per-scope configuration for a project within the monorepo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    /// Unique scope identifier (e.g., "rust-workspace", "py-conductor")
    pub id: String,

    /// Programming language
    pub lang: ScopeLang,

    /// Root directory relative to project root
    pub root: PathBuf,

    /// Python interpreter path (relative to project root)
    /// Auto-detected from .venv if not specified
    #[serde(default)]
    pub interpreter: Option<PathBuf>,

    /// Additional search paths for import resolution
    /// Auto-populated from toolchain if not specified
    #[serde(default)]
    pub search_paths: Vec<PathBuf>,

    /// Marker file that triggered auto-discovery (not user-configurable)
    #[serde(skip)]
    pub marker: Option<PathBuf>,
}

/// Supported scope languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeLang {
    Rust,
    Python,
    #[serde(alias = "ts")]
    Typescript,
    #[serde(alias = "js")]
    Javascript,
    Go,
}

impl std::fmt::Display for ScopeLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeLang::Rust => write!(f, "rust"),
            ScopeLang::Python => write!(f, "python"),
            ScopeLang::Typescript => write!(f, "typescript"),
            ScopeLang::Javascript => write!(f, "javascript"),
            ScopeLang::Go => write!(f, "go"),
        }
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_index_config() {
        let toml_str = r#"
auto_discover = true

[[scope]]
id = "py-conductor"
lang = "python"
root = "projects/conductor"
interpreter = ".venv/bin/python"

[[scope]]
id = "rust-workspace"
lang = "rust"
root = "."
"#;
        let config: IndexConfig = toml::from_str(toml_str).unwrap();
        assert!(config.auto_discover);
        assert_eq!(config.scopes.len(), 2);
        assert_eq!(config.scopes[0].id, "py-conductor");
        assert_eq!(config.scopes[0].lang, ScopeLang::Python);
        assert_eq!(config.scopes[1].lang, ScopeLang::Rust);
    }

    #[test]
    fn test_default_config() {
        let config = IndexConfig::default();
        assert!(!config.auto_discover); // Default derives false, but default_true overrides in serde
        assert!(config.scopes.is_empty());
    }

    #[test]
    fn test_scope_lang_display() {
        assert_eq!(ScopeLang::Rust.to_string(), "rust");
        assert_eq!(ScopeLang::Python.to_string(), "python");
        assert_eq!(ScopeLang::Typescript.to_string(), "typescript");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/core/index_config.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/core/index_config.rs` captured during libs codegen standardization.
```
