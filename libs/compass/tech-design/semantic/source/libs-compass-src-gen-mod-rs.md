---
id: libs-compass-src-gen-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/mod.rs`.
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

# Standardized libs/compass/src/gen/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `python` | libs/compass/src/gen/mod.rs | module | pub | 7 | pub mod python; |
| `registry` | libs/compass/src/gen/mod.rs | module | pub | 8 | pub mod registry; |
| `rust` | libs/compass/src/gen/mod.rs | module | pub | 9 | pub mod rust; |
| `traits` | libs/compass/src/gen/mod.rs | module | pub | 10 | pub mod traits; |
| `GeneratorRegistry` | libs/compass/src/gen/mod.rs | re-export | pub | 14 | pub use registry::GeneratorRegistry; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Code generation module for spec-to-code
//!
//! Contains generators for various technology stacks:
//! - Python: cclab.shield, cclab.titan, cclab.nebula, cclab.photon, cclab.quasar
//! - Rust: serde, axum, sqlx, reqwest

pub mod python;
pub mod registry;
pub mod rust;
pub mod traits;
// framework/ generators (fastapi, express, axum) were in sdd::gen_framework;
// now deleted — use sdd::generate::generators for SpecIR-based codegen.

pub use registry::GeneratorRegistry;
pub use traits::*;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/mod.rs` captured during libs codegen standardization.
```
