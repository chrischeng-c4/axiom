---
id: libs-openapi-codegen-src-ir-typemap-rs
summary: Lossless rust-source-unit coverage for `libs/openapi-codegen/src/ir/typemap.rs`.
capability_refs:
  - id: multi-language-openapi-client-generation
    role: primary
    claim: multi-language-openapi-client-generation-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Openapi Codegen library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/openapi-codegen/src/ir/typemap.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/openapi-codegen/src/ir/typemap.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TypeMap` | libs/openapi-codegen/src/ir/typemap.rs | struct | pub | 14 | pub struct TypeMap { |
| `resolve_ref` | libs/openapi-codegen/src/ir/typemap.rs | function | pub | 21 | pub fn resolve_ref(&self, reference: &str) -> Option<String> { |
| `build_type_map` | libs/openapi-codegen/src/ir/typemap.rs | function | pub | 33 | pub fn build_type_map(spec: &Spec) -> TypeMap { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Component-schema key → generated type-name map.
//!
//! Language-neutral: a deterministic, collision-free PascalCase name per
//! component schema works as a TypeScript interface, a Python (pydantic) class,
//! and a Rust (serde) struct alike. Each emitter renders the *type expression*
//! in its own language; the *name* assignment is shared and lives here.

use crate::ir::names::{self, to_pascal};
use crate::ir::openapi::Spec;
use std::collections::BTreeMap;

/// Maps an OpenAPI component-schema key to its final, collision-free type name.
#[derive(Debug, Default)]
pub struct TypeMap {
    pub names: BTreeMap<String, String>,
}

impl TypeMap {
    /// Resolve a `#/components/schemas/<key>` reference to its type name.
    /// Non-schema or external references yield `None`.
    pub fn resolve_ref(&self, reference: &str) -> Option<String> {
        let key = reference.strip_prefix("#/components/schemas/")?;
        Some(
            self.names
                .get(key)
                .cloned()
                .unwrap_or_else(|| to_pascal(key)),
        )
    }
}

/// Assign a deterministic, collision-free type name to each component schema key.
pub fn build_type_map(spec: &Spec) -> TypeMap {
    let mut reg = names::NameRegistry::new();
    let mut map = BTreeMap::new();
    for key in spec.components.schemas.keys() {
        let name = reg.unique(&to_pascal(key));
        map.insert(key.clone(), name);
    }
    TypeMap { names: map }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/openapi-codegen/src/ir/typemap.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/openapi-codegen/src/ir/typemap.rs` captured during libs codegen standardization.
```
