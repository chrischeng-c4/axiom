---
id: projects-sdd-src-generate-gen-rust-manifest-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Manifest Generator

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/manifest.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ManifestGenOutput` | projects/agentic-workflow/src/generate/gen/rust/manifest.rs | struct | pub | 43 |  |
| `generate_manifest` | projects/agentic-workflow/src/generate/gen/rust/manifest.rs | function | pub | 58 | generate_manifest(spec_content: &str) -> ManifestGenOutput |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/manifest.rs -->
````rust
//! Manifest generator — Cargo.toml `[dependencies]` fragment from a `manifest`
//! section.
//!
//! Section contract (YAML inside the spec's `## Manifest` section):
//!
//! ```yaml
//! dependencies:
//!   - { name: serde, spec: workspace, features: [derive] }
//!   - { name: thiserror, spec: workspace }
//!   - { name: once_cell, spec: version, version: "1.20" }
//!   - { name: cclab-mamba-registry, spec: path, path: "../../crates/cclab-mamba-registry" }
//! ```
//!
//! Output is a TOML fragment (one `key = value` per line) suitable for wrapping
//! inside a CODEGEN block under `[dependencies]` in the target `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! serde = { workspace = true, features = ["derive"] }
//! thiserror.workspace = true
//! once_cell = { version = "1.20" }
//! cclab-mamba-registry = { path = "../../crates/cclab-mamba-registry" }
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/manifest.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete manifest generator module.
```
