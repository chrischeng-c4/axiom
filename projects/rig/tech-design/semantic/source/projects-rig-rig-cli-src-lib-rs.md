---
id: projects-rig-rig-cli-src-lib-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/rig-cli/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/rig-cli/src/lib.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Shared verb parse + dispatch for the `rig` agent-first CLI.
//!
//! Every verb produces a single `RigReport`; `print_report` emits it as
//! exactly one JSON document on stdout (diagnostics go to stderr).
//! JSON-on-stdout is the UNFLAGGED default; `--human` and `--compact` are
//! the only opt-ins.

pub mod dispatch;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/rig-cli/src/lib.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/rig-cli/src/lib.rs` captured during rig
      standardization onto the codegen ladder.
```
