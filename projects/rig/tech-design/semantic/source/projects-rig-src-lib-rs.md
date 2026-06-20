---
id: projects-rig-src-lib-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `discovery` | projects/rig/src/lib.rs | module | pub | 14 |  |
| `engine` | projects/rig/src/lib.rs | module | pub | 15 |  |
| `pins` | projects/rig/src/lib.rs | module | pub | 16 |  |
| `report` | projects/rig/src/lib.rs | module | pub | 17 |  |
| `scenario` | projects/rig/src/lib.rs | module | pub | 18 |  |
| `vat` | projects/rig/src/lib.rs | module | pub | 19 |  |
| `verdict` | projects/rig/src/lib.rs | module | pub | 20 |  |
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
