---
id: projects-arena-arena-cli-src-lib-rs
capability_refs:
  - id: n-target-comparison-runner
    role: primary
    claim: sequential-target-fanout-and-measurement
    coverage: partial
    rationale: "This source unit implements arena CLI, spec parsing, measurement, or runner orchestration for N-target comparisons."
fill_sections: [overview, source, changes]
---

# Standardized projects/arena/arena-cli/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/arena/arena-cli/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `dispatch` | projects/arena/arena-cli/src/lib.rs | module | pub | 5 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! arena CLI library — verb tree + dispatch, shared by the `arena` binary.

pub mod dispatch;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/arena/arena-cli/src/lib.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/arena/arena-cli/src/lib.rs`
      captured during arena standardization onto the codegen ladder.
```
