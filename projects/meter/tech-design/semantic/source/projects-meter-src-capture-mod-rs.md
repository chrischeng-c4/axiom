---
id: projects-meter-src-capture-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/capture/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `audit` | projects/meter/src/capture/mod.rs | module | pub | 20 |  |
| `bench` | projects/meter/src/capture/mod.rs | module | pub | 21 |  |
| `delegate` | projects/meter/src/capture/mod.rs | module | pub | 22 |  |
| `fold` | projects/meter/src/capture/mod.rs | module | pub | 23 |  |
| `fuzz` | projects/meter/src/capture/mod.rs | module | pub | 24 |  |
| `run` | projects/meter/src/capture/mod.rs | module | pub | 25 |  |
| `sampler` | projects/meter/src/capture/mod.rs | module | pub | 26 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/capture/mod.rs -->
````rust
//! Capture-mode populators (擷取) — observe a workload from the outside.
//!
//! Capture is the half of `meter` that runs/observes external processes (in
//! contrast to the in-process `embed`/埋点 probes in `performance`). It is gated
//! behind the `capture` feature so the engine rlib stays free of process-spawn
//! machinery for pure-library consumers.
//!
//! This wave ships [`delegate`] (the `meter test` delegate+forward path),
//! [`audit`] (the `meter audit` cargo-audit caller), [`bench`] (the `meter bench`
//! cargo-bench delegate + regression-baseline loader), the C1 profiling pair
//! [`sampler`] (spawn + platform stack sampler -> folded stacks) + [`fold`]
//! (folded stacks -> ranked `Hotspot` findings, the default stdout),
//! [`vitals`] (the meter.toml measurement contract + L1 vitals capture window),
//! and [`run`] (the composite `meter run` sweep that folds every sub-verb into
//! ONE worst-wins report).

pub mod audit;
pub mod bench;
pub mod delegate;
pub mod fold;
pub mod run;
pub mod sampler;
pub mod vitals;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/capture/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/capture/mod.rs` captured during meter full-codegen standardization.
```
