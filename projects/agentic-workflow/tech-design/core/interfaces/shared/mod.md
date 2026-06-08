---
id: projects-sdd-src-shared-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Shared workflow utilities are part of the AW Core protocol support surface used across clients and lifecycle phases."
---

# Standardized projects/agentic-workflow/src/shared/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/shared/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `services` | projects/agentic-workflow/src/shared/mod.rs | module | pub | 10 |  |
| `tools` | projects/agentic-workflow/src/shared/mod.rs | module | pub | 11 |  |
| `workspace` | projects/agentic-workflow/src/shared/mod.rs | module | pub | 12 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/shared/mod.rs -->
```rust
//! Shared utilities and services used across workflows
//!
//! File and spec services, plus tool integration points used by workflow
//! phases. The legacy `cli` re-export submodule was deleted during the
//! Score unbundling — all user-facing CLI commands now live in
//! `projects/agentic-workflow/`.

pub mod services;
pub mod tools;
pub mod workspace;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/shared/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete shared module facade.
```
