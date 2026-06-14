---
id: projects-meter-examples-profile-target-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: embedded-profiler-api
    claim: embedded-profiler-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/examples/profile_target.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/examples/profile_target.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! A tiny CPU-bound profiling target for the `meter profile` LIVE gate.
//!
//! It spends the overwhelming majority of its wall time inside a single
//! clearly-named function, [`hot_spin`], doing arithmetic in a tight loop, so a
//! platform stack sampler (`/usr/bin/sample`) captures `hot_spin` as the
//! dominant leaf. Runs for ~`METER_PROFILE_SECS` seconds (env, default 3) so the
//! sampling window has a stable hot leaf to rank. Dependency-free.

use std::time::{Duration, Instant};

/// The dominant hot function: pure arithmetic in a loop. `#[inline(never)]` so
/// it stays a distinct symbol the sampler can attribute self time to.
#[inline(never)]
fn hot_spin(seed: f64) -> f64 {
    let mut acc = seed;
    // A fixed chunk of work per call; the caller loops until the deadline.
    for i in 0..2_000_000u64 {
        acc = acc * 1.000_000_1 + (i as f64) * 0.5;
        // Keep the value bounded so it never goes to inf and the optimizer
        // cannot trivially fold the loop away.
        if acc > 1e9 {
            acc -= 1e9;
        }
    }
    acc
}

fn main() {
    let secs: u64 = std::env::var("METER_PROFILE_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let deadline = Instant::now() + Duration::from_secs(secs);

    let mut acc = 1.0_f64;
    while Instant::now() < deadline {
        acc = hot_spin(acc);
    }

    // Side-effect so the work is not optimized away; goes to stdout but the
    // sampler-spawned child has stdout discarded by meter, so this never pollutes
    // meter's JSON document.
    println!("{acc}");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/examples/profile_target.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/examples/profile_target.rs` captured during meter full-codegen standardization.
```
