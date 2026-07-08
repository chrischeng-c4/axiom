---
id: libs-compass-src-diagnostic-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/diagnostic.rs`.
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

# Standardized libs/compass/src/diagnostic.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/diagnostic.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DiagnosticSeverity` | libs/compass/src/diagnostic.rs | enum | pub | 7 | pub enum DiagnosticSeverity { |
| `as_str` | libs/compass/src/diagnostic.rs | function | pub | 15 | pub fn as_str(&self) -> &'static str { |
| `DiagnosticCategory` | libs/compass/src/diagnostic.rs | enum | pub | 27 | pub enum DiagnosticCategory { |
| `Position` | libs/compass/src/diagnostic.rs | struct | pub | 54 | pub struct Position { |
| `new` | libs/compass/src/diagnostic.rs | function | pub | 60 | pub fn new(line: u32, character: u32) -> Self { |
| `Range` | libs/compass/src/diagnostic.rs | struct | pub | 67 | pub struct Range { |
| `from_node` | libs/compass/src/diagnostic.rs | function | pub | 77 | pub fn from_node(node: &tree_sitter::Node<'_>) -> Self { |
| `contains` | libs/compass/src/diagnostic.rs | function | pub | 87 | pub fn contains(&self, line: u32, character: u32) -> bool { |
| `QuickFix` | libs/compass/src/diagnostic.rs | struct | pub | 102 | pub struct QuickFix { |
| `TextEdit` | libs/compass/src/diagnostic.rs | struct | pub | 109 | pub struct TextEdit { |
| `Diagnostic` | libs/compass/src/diagnostic.rs | struct | pub | 116 | pub struct Diagnostic { |
| `error` | libs/compass/src/diagnostic.rs | function | pub | 144 | pub fn error( |
| `warning` | libs/compass/src/diagnostic.rs | function | pub | 153 | pub fn warning( |
| `with_fix` | libs/compass/src/diagnostic.rs | function | pub | 162 | pub fn with_fix(mut self, title: impl Into<String>, edits: Vec<TextEdit>) -> Self { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Diagnostic types (LSP-compatible)

use serde::{Deserialize, Serialize};

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Information => "info",
            DiagnosticSeverity::Hint => "hint",
        }
    }
}

/// Diagnostic category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticCategory {
    Syntax,
    Type,
    Names,
    Logic,
    Security,
    Style,
    /// Diagnostics produced by user-defined custom rules (CUSTOM_* prefix)
    Custom,
}

impl DiagnosticCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticCategory::Syntax => "syntax",
            DiagnosticCategory::Type => "type",
            DiagnosticCategory::Names => "names",
            DiagnosticCategory::Logic => "logic",
            DiagnosticCategory::Security => "security",
            DiagnosticCategory::Style => "style",
            DiagnosticCategory::Custom => "custom",
        }
    }
}

/// Position in a text document (0-indexed)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Range in a text document
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn from_node(node: &tree_sitter::Node<'_>) -> Self {
        let start = node.start_position();
        let end = node.end_position();
        Self {
            start: Position::new(start.row as u32, start.column as u32),
            end: Position::new(end.row as u32, end.column as u32),
        }
    }

    /// Check if a position is within this range
    pub fn contains(&self, line: u32, character: u32) -> bool {
        // Check if position is after start
        let after_start = line > self.start.line
            || (line == self.start.line && character >= self.start.character);

        // Check if position is before end
        let before_end =
            line < self.end.line || (line == self.end.line && character <= self.end.character);

        after_start && before_end
    }
}

/// Quick fix action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFix {
    pub title: String,
    pub edits: Vec<TextEdit>,
}

/// Text edit for quick fixes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// A code diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub category: DiagnosticCategory,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub quick_fixes: Vec<QuickFix>,
}

impl Diagnostic {
    pub fn new(
        range: Range,
        severity: DiagnosticSeverity,
        code: impl Into<String>,
        category: DiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            range,
            severity,
            code: code.into(),
            category,
            message: message.into(),
            quick_fixes: Vec::new(),
        }
    }

    pub fn error(
        range: Range,
        code: impl Into<String>,
        category: DiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self::new(range, DiagnosticSeverity::Error, code, category, message)
    }

    pub fn warning(
        range: Range,
        code: impl Into<String>,
        category: DiagnosticCategory,
        message: impl Into<String>,
    ) -> Self {
        Self::new(range, DiagnosticSeverity::Warning, code, category, message)
    }

    pub fn with_fix(mut self, title: impl Into<String>, edits: Vec<TextEdit>) -> Self {
        self.quick_fixes.push(QuickFix {
            title: title.into(),
            edits,
        });
        self
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/diagnostic.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/diagnostic.rs` captured during libs codegen standardization.
```
