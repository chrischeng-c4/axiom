---
id: libs-openapi-codegen-src-emit-mod-rs
summary: Lossless rust-source-unit coverage for `libs/openapi-codegen/src/emit/mod.rs`.
capability_refs:
  - id: multi-language-openapi-client-generation
    role: primary
    claim: multi-language-openapi-client-generation-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Openapi Codegen library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/openapi-codegen/src/emit/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/openapi-codegen/src/emit/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `py` | libs/openapi-codegen/src/emit/mod.rs | module | pub | 8 | pub mod py; |
| `rust` | libs/openapi-codegen/src/emit/mod.rs | module | pub | 9 | pub mod rust; |
| `ts` | libs/openapi-codegen/src/emit/mod.rs | module | pub | 10 | pub mod ts; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Per-language emitters. Each reads the shared [`crate::ir`] and renders a typed
//! client in its target language.
//!
//! - [`ts`]: TypeScript — types + fetch/axios client + TanStack Query hooks.
//! - [`py`]: Python — pydantic models + generated sync/async HTTP/2 runtime.
//! - [`rust`]: Rust — serde models + reqwest client.

pub mod py;
pub mod rust;
pub mod ts;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/openapi-codegen/src/emit/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/openapi-codegen/src/emit/mod.rs` captured during libs codegen standardization.
```
