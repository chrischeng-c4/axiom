---
id: projects-sdd-src-state-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Workflow state interfaces define AW Core lifecycle state, locks, validations, and rollup invariants."
---

# Standardized projects/agentic-workflow/src/state/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/state/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/state/mod.rs -->
```rust
//! STATE.yaml Management Module
//!
//! Handles persistence and tracking of change state, including:
//! - Phase transitions
//! - File checksums for staleness detection
//! - Validation history
//! - LLM telemetry

mod manager;

pub(crate) use manager::run_blocking_io;
pub use manager::{AgentLock, StalenessReport, StateManager};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/state/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete state module facade.
```
