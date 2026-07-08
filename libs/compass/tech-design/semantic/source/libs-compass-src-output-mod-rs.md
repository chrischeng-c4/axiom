---
id: libs-compass-src-output-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/output/mod.rs`.
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

# Standardized libs/compass/src/output/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/output/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `agent` | libs/compass/src/output/mod.rs | module | pub | 7 | pub mod agent; |
| `agent_types` | libs/compass/src/output/mod.rs | module | pub | 8 | pub mod agent_types; |
| `reporter` | libs/compass/src/output/mod.rs | module | pub | 9 | pub mod reporter; |
| `AgentOutputBuilder` | libs/compass/src/output/mod.rs | re-export | pub | 11 | pub use agent::AgentOutputBuilder; |
| `AgentIssue` | libs/compass/src/output/mod.rs | re-export | pub | 12 | pub use agent_types::{AgentIssue, AgentOutput, AgentStats, SymbolDef}; |
| `AgentOutput` | libs/compass/src/output/mod.rs | re-export | pub | 12 | pub use agent_types::{AgentIssue, AgentOutput, AgentStats, SymbolDef}; |
| `AgentStats` | libs/compass/src/output/mod.rs | re-export | pub | 12 | pub use agent_types::{AgentIssue, AgentOutput, AgentStats, SymbolDef}; |
| `SymbolDef` | libs/compass/src/output/mod.rs | re-export | pub | 12 | pub use agent_types::{AgentIssue, AgentOutput, AgentStats, SymbolDef}; |
| `OutputFormat` | libs/compass/src/output/mod.rs | re-export | pub | 13 | pub use reporter::{OutputFormat, Reporter}; |
| `Reporter` | libs/compass/src/output/mod.rs | re-export | pub | 13 | pub use reporter::{OutputFormat, Reporter}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Output formatters for analysis results.
//!
//! Post lens-dissolution location for output modules.
//! The `agent` submodule provides symbol-centric JSON output
//! optimized for LLM agent consumption.

pub mod agent;
pub mod agent_types;
pub mod reporter;

pub use agent::AgentOutputBuilder;
pub use agent_types::{AgentIssue, AgentOutput, AgentStats, SymbolDef};
pub use reporter::{OutputFormat, Reporter};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/output/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/output/mod.rs` captured during libs codegen standardization.
```
