---
id: libs-compass-src-refactoring-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/refactoring/mod.rs`.
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

# Standardized libs/compass/src/refactoring/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/refactoring/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ExtractEngine` | libs/compass/src/refactoring/mod.rs | re-export | pub | 14 | pub use extract::ExtractEngine; |
| `InlineEngine` | libs/compass/src/refactoring/mod.rs | re-export | pub | 15 | pub use inline::InlineEngine; |
| `MoveDefEngine` | libs/compass/src/refactoring/mod.rs | re-export | pub | 16 | pub use move_def::MoveDefEngine; |
| `RenameEngine` | libs/compass/src/refactoring/mod.rs | re-export | pub | 17 | pub use rename::RenameEngine; |
| `SignatureEngine` | libs/compass/src/refactoring/mod.rs | re-export | pub | 18 | pub use signature::SignatureEngine; |
| `FileContext` | libs/compass/src/refactoring/mod.rs | struct | pub | 33 | pub struct FileContext<'a> { |
| `ProjectContext` | libs/compass/src/refactoring/mod.rs | struct | pub | 47 | pub struct ProjectContext<'a> { |
| `RefactoringOp` | libs/compass/src/refactoring/mod.rs | trait | pub | 57 | pub trait RefactoringOp { |
| `RefactoringRegistry` | libs/compass/src/refactoring/mod.rs | struct | pub | 75 | pub struct RefactoringRegistry { |
| `new` | libs/compass/src/refactoring/mod.rs | function | pub | 84 | pub fn new() -> Self { |
| `apply` | libs/compass/src/refactoring/mod.rs | function | pub | 95 | pub fn apply( |
| `validate_identifier` | libs/compass/src/refactoring/mod.rs | function | pub | 125 | pub(crate) fn validate_identifier(name: &str, language: Language) -> Result<()> { |
| `is_keyword` | libs/compass/src/refactoring/mod.rs | function | pub | 152 | pub(crate) fn is_keyword(name: &str, language: Language) -> bool { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Refactoring engine with pluggable operation strategies
//!
//! Dispatches refactoring requests to the appropriate engine based on
//! the `RefactorKind`. Each engine implements the `RefactoringOp` trait.

mod extract;
mod extract_helpers;
mod inline;
mod move_def;
mod rename;
mod signature;
mod signature_helpers;

pub use extract::ExtractEngine;
pub use inline::InlineEngine;
pub use move_def::MoveDefEngine;
pub use rename::RenameEngine;
pub use signature::SignatureEngine;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::lens_error::{ArgusError, Result};
use crate::semantic::symbols::SymbolTable;
use crate::syntax::{Language, ParsedFile};
use crate::type_inference::{RefactorKind, RefactorRequest, RefactorResult};

// ============================================================================
// File context passed to every operation
// ============================================================================

/// Parsed context for a single file, shared across operations.
pub struct FileContext<'a> {
    /// Absolute path
    pub path: &'a PathBuf,
    /// Raw source text
    pub source: &'a str,
    /// tree-sitter parse result
    pub parsed: &'a ParsedFile,
    /// Symbol table built from the parsed file
    pub symbols: &'a SymbolTable,
    /// Detected language
    pub language: Language,
}

/// Multi-file project context for cross-file operations.
pub struct ProjectContext<'a> {
    /// All parsed files keyed by path
    pub files: &'a HashMap<PathBuf, (String, ParsedFile, SymbolTable)>,
}

// ============================================================================
// Trait every refactoring operation implements
// ============================================================================

/// A single refactoring operation.
pub trait RefactoringOp {
    /// Apply the operation and return edits.
    ///
    /// `file` is the primary file context.
    /// `project` provides cross-file data when available.
    fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult>;
}

// ============================================================================
// Registry that dispatches to the right engine
// ============================================================================

/// Central dispatcher that maps `RefactorKind` to the correct engine.
pub struct RefactoringRegistry {
    rename: RenameEngine,
    extract: ExtractEngine,
    inline: InlineEngine,
    move_def: MoveDefEngine,
    signature: SignatureEngine,
}

impl RefactoringRegistry {
    pub fn new() -> Self {
        Self {
            rename: RenameEngine,
            extract: ExtractEngine,
            inline: InlineEngine,
            move_def: MoveDefEngine,
            signature: SignatureEngine,
        }
    }

    /// Dispatch a request to the appropriate engine.
    pub fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult> {
        let engine: &dyn RefactoringOp = match &request.kind {
            RefactorKind::Rename { .. } => &self.rename,
            RefactorKind::ExtractFunction { .. }
            | RefactorKind::ExtractMethod { .. }
            | RefactorKind::ExtractVariable { .. } => &self.extract,
            RefactorKind::Inline => &self.inline,
            RefactorKind::MoveDefinition { .. } => &self.move_def,
            RefactorKind::ChangeSignature { .. } => &self.signature,
        };
        engine.apply(request, file, project)
    }
}

impl Default for RefactoringRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helpers shared across engines
// ============================================================================

/// Validate that `name` is a legal identifier for the given language.
pub(crate) fn validate_identifier(name: &str, language: Language) -> Result<()> {
    if name.is_empty() {
        return Err(ArgusError::invalid_identifier("name cannot be empty"));
    }
    let first = name.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return Err(ArgusError::invalid_identifier(format!(
            "'{}' must start with a letter or underscore",
            name
        )));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ArgusError::invalid_identifier(format!(
            "'{}' contains invalid characters",
            name
        )));
    }
    if is_keyword(name, language) {
        return Err(ArgusError::invalid_identifier(format!(
            "'{}' is a reserved keyword",
            name
        )));
    }
    Ok(())
}

/// Check whether `name` is a reserved keyword in `language`.
pub(crate) fn is_keyword(name: &str, language: Language) -> bool {
    match language {
        Language::Python => matches!(
            name,
            "False"
                | "None"
                | "True"
                | "and"
                | "as"
                | "assert"
                | "async"
                | "await"
                | "break"
                | "class"
                | "continue"
                | "def"
                | "del"
                | "elif"
                | "else"
                | "except"
                | "finally"
                | "for"
                | "from"
                | "global"
                | "if"
                | "import"
                | "in"
                | "is"
                | "lambda"
                | "nonlocal"
                | "not"
                | "or"
                | "pass"
                | "raise"
                | "return"
                | "try"
                | "while"
                | "with"
                | "yield"
        ),
        Language::TypeScript | Language::JavaScript => matches!(
            name,
            "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "enum"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
        ),
        Language::Rust => matches!(
            name,
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
        ),
        _ => false,
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/refactoring/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/refactoring/mod.rs` captured during libs codegen standardization.
```
