---
id: libs-openapi-codegen-src-ir-mod-rs
summary: Lossless rust-source-unit coverage for `libs/openapi-codegen/src/ir/mod.rs`.
capability_refs:
  - id: multi-language-openapi-client-generation
    role: primary
    claim: multi-language-openapi-client-generation-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Openapi Codegen library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/openapi-codegen/src/ir/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/openapi-codegen/src/ir/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `names` | libs/openapi-codegen/src/ir/mod.rs | module | pub | 8 | pub mod names; |
| `openapi` | libs/openapi-codegen/src/ir/mod.rs | module | pub | 9 | pub mod openapi; |
| `operations` | libs/openapi-codegen/src/ir/mod.rs | module | pub | 10 | pub mod operations; |
| `typemap` | libs/openapi-codegen/src/ir/mod.rs | module | pub | 11 | pub mod typemap; |
| `build_type_map` | libs/openapi-codegen/src/ir/mod.rs | re-export | pub | 13 | pub use typemap::{build_type_map, TypeMap}; |
| `TypeMap` | libs/openapi-codegen/src/ir/mod.rs | re-export | pub | 13 | pub use typemap::{build_type_map, TypeMap}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Language-neutral OpenAPI intermediate representation shared by every emitter:
//! the document model ([`openapi`]), identifier naming ([`names`]), and the
//! schema-key → type-name map ([`typemap`]).
//!
//! The per-language *operation plan* and *type expressions* live under
//! `crate::emit::<lang>`, since they bake in language-specific type syntax.

pub mod names;
pub mod openapi;
pub mod operations;
pub mod typemap;

pub use typemap::{build_type_map, TypeMap};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/openapi-codegen/src/ir/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/openapi-codegen/src/ir/mod.rs` captured during libs codegen standardization.
```
