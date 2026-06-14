---
id: projects-rig-src-scenario-load-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/scenario/load.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/scenario/load.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The `[load]` block — open-loop load profile for `kind = "load"`
//! scenarios.
//!
//! Open-loop means the request schedule is FIXED at `target_qps`
//! regardless of response latency, so queueing delay shows up in the
//! measured percentiles instead of silently throttling the offered load
//! (coordinated-omission honesty). `achieved_qps` is always reported
//! alongside latency; a shortfall below [`ACHIEVED_QPS_HONESTY_RATIO`]
//! emits a `load_honesty` finding.

use serde::{Deserialize, Serialize};

use super::step::HttpRequest;

/// achieved/target ratio below which the latency percentiles are no longer
/// trustworthy and the report must say so.
pub const ACHIEVED_QPS_HONESTY_RATIO: f64 = 0.95;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadProfile {
    /// Offered load: requests per second on a fixed-interval schedule.
    pub target_qps: u32,
    /// Concurrent sender threads draining the tick schedule.
    pub workers: u32,
    pub duration_secs: u64,
    /// Leading seconds excluded from the statistics.
    #[serde(default)]
    pub warmup_secs: u64,
    pub request: HttpRequest,
}

/// Metric names a load run publishes into the var store / report — the
/// names pins reference.
pub const LOAD_METRICS: &[&str] = &["p50_ms", "p99_ms", "error_rate", "achieved_qps"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_profile_parses() {
        let p: LoadProfile = toml::from_str(
            r#"
target_qps = 200
workers = 8
duration_secs = 30
warmup_secs = 5
[request]
method = "POST"
url = "http://{{upstream}}/search"
body = '{"q":1}'
"#,
        )
        .unwrap();
        assert_eq!(p.target_qps, 200);
        assert_eq!(p.warmup_secs, 5);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/scenario/load.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/scenario/load.rs` captured during rig
      standardization onto the codegen ladder.
```
