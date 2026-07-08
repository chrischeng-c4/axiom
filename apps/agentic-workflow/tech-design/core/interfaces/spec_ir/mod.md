---
id: projects-sdd-src-spec-ir-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Spec IR interfaces drive code artifact generation from TD/spec manifests in the TD/CB lifecycle."
---

# Standardized apps/agentic-workflow/src/spec_ir/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/spec_ir/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `codegen` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 20 |  |
| `generator` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 21 |  |
| `migration` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 22 |  |
| `orchestrator` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 23 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/src/spec_ir/mod.rs -->
````rust
//! SpecIR YAML Manifest types (k8s/Kustomize style)
//!
//! Language-agnostic intermediate representation for the spec-to-code pipeline.
//! SDD writes these YAML files, Lens reads them for codegen.
//!
//! ## Manifest format
//!
//! ```yaml
//! apiVersion: cclab.dev/v1
//! kind: Api
//! metadata:
//!   name: user-service
//!   change_id: genesis-372
//! spec:
//!   # kind-specific payload
//! ```

pub mod codegen;
pub mod generator;
pub mod migration;
pub mod orchestrator;
mod types;

pub use types::*;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/spec_ir/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete SpecIR module facade.
```
