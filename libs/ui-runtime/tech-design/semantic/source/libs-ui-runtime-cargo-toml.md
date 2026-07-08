---
id: libs-ui-runtime-cargo-toml
summary: Lossless text-source-unit coverage for `libs/ui-runtime/Cargo.toml`.
capability_refs:
  - id: renderer-neutral-component-runtime
    role: primary
    claim: renderer-neutral-component-runtime-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Ui Runtime library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/ui-runtime/Cargo.toml

## Overview
<!-- type: overview lang: markdown -->

Lossless text-source-unit coverage for `libs/ui-runtime/Cargo.toml` captured during libs codegen standardization.


## Source
<!-- type: text-source-unit lang: bash -->

````bash
[package]
name = "cclab-ui-runtime"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = "Renderer-neutral component runtime: hooks, fiber storage, mount, flush, and update scheduling over cclab-surface elements"

[features]
default = []
debug = ["serde_json"]

[dependencies]
cclab-surface = { path = "../surface" }
serde_json = { version = "1", optional = true }
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/ui-runtime/Cargo.toml"
    action: modify
    section: text-source-unit
    impl_mode: codegen
    description: |
      text-source-unit (td_ast) source for `libs/ui-runtime/Cargo.toml` captured during libs codegen standardization.
```
