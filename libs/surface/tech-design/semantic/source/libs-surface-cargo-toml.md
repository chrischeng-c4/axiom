---
id: libs-surface-cargo-toml
summary: Lossless text-source-unit coverage for `libs/surface/Cargo.toml`.
capability_refs:
  - id: renderer-neutral-ui-surface-model
    role: primary
    claim: renderer-neutral-ui-surface-model-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Surface library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/surface/Cargo.toml

## Overview
<!-- type: overview lang: markdown -->

Lossless text-source-unit coverage for `libs/surface/Cargo.toml` captured during libs codegen standardization.


## Source
<!-- type: text-source-unit lang: bash -->

````bash
[package]
name = "cclab-surface"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = "Renderer-neutral UI element model shared by Jet WASM, native desktop readers, renderers, and parity tools"

[lib]
name = "cclab_surface"
path = "src/lib.rs"

[dependencies]
serde = { workspace = true, features = ["derive"] }

[dev-dependencies]
serde_json.workspace = true
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/surface/Cargo.toml"
    action: modify
    section: text-source-unit
    impl_mode: codegen
    description: |
      text-source-unit (td_ast) source for `libs/surface/Cargo.toml` captured during libs codegen standardization.
```
