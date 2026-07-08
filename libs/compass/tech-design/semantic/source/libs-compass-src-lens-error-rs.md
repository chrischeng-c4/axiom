---
id: libs-compass-src-lens-error-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lens_error.rs`.
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

# Standardized libs/compass/src/lens_error.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lens_error.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Result` | libs/compass/src/lens_error.rs | type | pub | 10 | pub type Result<T> = std::result::Result<T, ArgusError>; |
| `ArgusError` | libs/compass/src/lens_error.rs | enum | pub | 14 | pub enum ArgusError { |
| `parser` | libs/compass/src/lens_error.rs | function | pub | 54 | pub fn parser(msg: impl Into<String>) -> Self { |
| `ast_cache_not_found` | libs/compass/src/lens_error.rs | function | pub | 59 | pub fn ast_cache_not_found(path: PathBuf) -> Self { |
| `invalid_identifier` | libs/compass/src/lens_error.rs | function | pub | 64 | pub fn invalid_identifier(id: impl Into<String>) -> Self { |
| `definition_not_found` | libs/compass/src/lens_error.rs | function | pub | 69 | pub fn definition_not_found(name: impl Into<String>) -> Self { |
| `type_error` | libs/compass/src/lens_error.rs | function | pub | 74 | pub fn type_error(msg: impl Into<String>) -> Self { |
| `other` | libs/compass/src/lens_error.rs | function | pub | 79 | pub fn other(msg: impl Into<String>) -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Unified error handling for Argus
//!
//! This module provides a comprehensive error type that covers all error cases
//! in the Argus static analysis tool.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for Argus operations
pub type Result<T> = std::result::Result<T, ArgusError>;

/// Unified error type for all Argus operations
#[derive(Error, Debug)]
pub enum ArgusError {
    /// Parser initialization or operation failed
    #[error("Parser error: {0}")]
    Parser(String),

    /// AST cache operation failed
    #[error("AST cache error: file not found in cache: {0}")]
    AstCacheNotFound(PathBuf),

    /// File I/O error
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid identifier or name
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    /// Definition not found during refactoring
    #[error("Definition not found: {0}")]
    DefinitionNotFound(String),

    /// Type system error
    #[error("Type error: {0}")]
    Type(String),

    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// Tree-sitter language error
    #[error("Tree-sitter language error: {0}")]
    TreeSitterLanguage(#[from] tree_sitter::LanguageError),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

impl ArgusError {
    /// Create a parser error
    pub fn parser(msg: impl Into<String>) -> Self {
        Self::Parser(msg.into())
    }

    /// Create an AST cache not found error
    pub fn ast_cache_not_found(path: PathBuf) -> Self {
        Self::AstCacheNotFound(path)
    }

    /// Create an invalid identifier error
    pub fn invalid_identifier(id: impl Into<String>) -> Self {
        Self::InvalidIdentifier(id.into())
    }

    /// Create a definition not found error
    pub fn definition_not_found(name: impl Into<String>) -> Self {
        Self::DefinitionNotFound(name.into())
    }

    /// Create a type error
    pub fn type_error(msg: impl Into<String>) -> Self {
        Self::Type(msg.into())
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ArgusError::parser("Failed to initialize");
        assert_eq!(err.to_string(), "Parser error: Failed to initialize");

        let err = ArgusError::ast_cache_not_found(PathBuf::from("test.py"));
        assert!(err.to_string().contains("test.py"));

        let err = ArgusError::invalid_identifier("123invalid");
        assert_eq!(err.to_string(), "Invalid identifier: 123invalid");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ArgusError = io_err.into();
        assert!(matches!(err, ArgusError::Io(_)));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lens_error.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lens_error.rs` captured during libs codegen standardization.
```
