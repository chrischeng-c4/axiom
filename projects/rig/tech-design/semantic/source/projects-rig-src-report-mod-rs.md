---
id: projects-rig-src-report-mod-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/report/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/report/mod.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! rig's report surface: one JSON document per verb on stdout.

pub mod builder;
pub mod envelope;
pub mod finding;

pub use builder::{persist, ReportBuilder};
pub use envelope::{OverallStatus, RigReport, SCHEMA_VERSION};
pub use finding::{finding_id, Finding, Invoke, Kind, Severity};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/report/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/report/mod.rs` captured during rig
      standardization onto the codegen ladder.
```
