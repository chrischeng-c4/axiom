---
id: projects-rig-src-lib-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/lib.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! rig — declarative test-scenario harness engine.
//!
//! Runs declarative SCENARIOS (e2e behavior steps) and LOAD profiles
//! (open-loop QPS) against a real serving process, judged by assertions and
//! declarative pins (floors/ratchets), emitting ONE agent-readable JSON
//! report per verb.
//!
//! Division of labor in the ecosystem: vat owns the environment (services,
//! workspace, readiness), rig owns case orchestration + assertions + gates,
//! meter owns resource attribution.

pub mod discovery;
pub mod engine;
pub mod pins;
pub mod report;
pub mod scenario;
pub mod vat;
pub mod verdict;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/lib.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/lib.rs` captured during rig
      standardization onto the codegen ladder.
```
