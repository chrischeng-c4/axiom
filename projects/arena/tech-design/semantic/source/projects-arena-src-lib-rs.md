---
id: projects-arena-src-lib-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/arena/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/arena/src/lib.rs`, captured as a rust-source-unit (td_ast) item-tree
during arena standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! arena — an N-target competitive comparison runner above rig/meter.
//!
//! arena runs the SAME logical "cell" against N targets, reduces each to one
//! comparable scalar, computes `ratio = peer/base`, classifies the cell as
//! WIN / EXEMPT / TARGET, gates WIN cells against a ratcheted per-host
//! baseline, and emits ONE comparison report.
//!
//! Measurement is delegated wholesale: service targets reuse
//! [`rig::engine::loadgen`]; the runtime flavor (deferred) shells out to
//! `meter profile`. Per-target workload TRANSLATION (lumen-JSON vs pg-SQL vs
//! OS-DSL) stays glue in the spec — arena never reads request bodies.
//!
//! Ecosystem layering: vat = outer container (provisions each target's env),
//! arena = middle compare layer, rig/meter = per-target measurement units.

pub mod compare;
pub mod engine;
pub mod measure;
pub mod report;
pub mod spec;

pub use engine::{run, RunOpts};
pub use report::ArenaReport;
pub use spec::Spec;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/arena/src/lib.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/arena/src/lib.rs` captured during arena
      standardization onto the codegen ladder.
```
