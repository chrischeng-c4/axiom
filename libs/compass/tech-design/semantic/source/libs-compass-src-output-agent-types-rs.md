---
id: libs-compass-src-output-agent-types-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/output/agent_types.rs`.
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

# Standardized libs/compass/src/output/agent_types.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/output/agent_types.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AgentOutput` | libs/compass/src/output/agent_types.rs | struct | pub | 14 | pub struct AgentOutput { |
| `SymbolDef` | libs/compass/src/output/agent_types.rs | struct | pub | 36 | pub struct SymbolDef { |
| `AgentIssue` | libs/compass/src/output/agent_types.rs | struct | pub | 53 | pub struct AgentIssue { |
| `AgentStats` | libs/compass/src/output/agent_types.rs | struct | pub | 75 | pub struct AgentStats { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Serde-serializable types for agent output format.
//!
//! Symbol-centric JSON output optimized for LLM agent consumption.
//! Uses `skip_serializing_if` to omit empty fields for compactness (R9).

use serde::Serialize;
use std::collections::BTreeMap;

/// Top-level agent output: symbol-centric analysis result.
///
/// Required fields: `symbols`, `stats`.
/// Optional (omitted when empty): `imports`, `issues`, `impact`.
#[derive(Debug, Clone, Serialize)]
pub struct AgentOutput {
    /// Map of symbol qualified name to definition info.
    pub symbols: BTreeMap<String, SymbolDef>,

    /// Map of file path to list of imported symbol qualified names.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub imports: BTreeMap<String, Vec<String>>,

    /// Diagnostics attributed to nearest enclosing symbol.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<AgentIssue>,

    /// Map of symbol qualified name to list of "file:line" reference locations.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub impact: BTreeMap<String, Vec<String>>,

    /// Summary statistics.
    pub stats: AgentStats,
}

/// Definition info for a single symbol.
#[derive(Debug, Clone, Serialize)]
pub struct SymbolDef {
    /// Type signature string (e.g. "(int) -> User"). Omitted if unknown.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_sig: Option<String>,

    /// Relative file path from project root.
    pub file: String,

    /// 1-based line number.
    pub line: u32,

    /// Symbol kind: function, class, method, variable, constant, interface, type_alias, module.
    pub kind: String,
}

/// A diagnostic issue attributed to the nearest enclosing symbol.
#[derive(Debug, Clone, Serialize)]
pub struct AgentIssue {
    /// Severity: error, warning, info, hint.
    pub severity: String,

    /// Nearest enclosing symbol name, or "<file-level>" if none.
    pub symbol: String,

    /// File path.
    pub file: String,

    /// 1-based line number.
    pub line: u32,

    /// Diagnostic rule code (e.g. "PY101").
    pub code: String,

    /// Diagnostic message.
    pub message: String,
}

/// Summary statistics for the agent output.
#[derive(Debug, Clone, Serialize)]
pub struct AgentStats {
    pub files_checked: usize,
    pub symbols_found: usize,
    pub issues_count: usize,
    pub impact_edges: usize,
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/output/agent_types.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/output/agent_types.rs` captured during libs codegen standardization.
```
