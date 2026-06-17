---
id: projects-rig-src-engine-sample-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/engine/sample.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/engine/sample.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SampleStats` | projects/rig/src/engine/sample.rs | struct | pub | 11 |  |
| `fold` | projects/rig/src/engine/sample.rs | function | pub | 26 | fold(observations: &[(f64, bool)]) -> Self |
| `get` | projects/rig/src/engine/sample.rs | function | pub | 50 | get(&self, key: &str) -> Option<f64> |
| `percentile` | projects/rig/src/engine/sample.rs | function | pub | 65 | percentile(sorted: &[f64], q: f64) -> f64 |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The `sample` step: repeat one request N times, fold latency statistics.
//!
//! Percentile = ceil-rank (`values[ceil(q/100 * n) - 1]` on the sorted
//! list), matching lumen's chaos.sh python so ported budgets keep their
//! meaning.

#[derive(Debug, Default, Clone)]
pub struct SampleStats {
    pub ok_count: u64,
    pub fail_count: u64,
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p99_ms: f64,
    pub mean_ms: f64,
}

impl SampleStats {
    /// Fold from per-request (latency_ms, ok) observations. Latencies of
    /// FAILED requests are excluded from percentiles (they often sit at the
    /// timeout ceiling and would drown the signal); the failure count is its
    /// own stat.
    pub fn fold(observations: &[(f64, bool)]) -> Self {
        let mut ok_latencies: Vec<f64> = observations
            .iter()
            .filter(|(_, ok)| *ok)
            .map(|(ms, _)| *ms)
            .collect();
        ok_latencies.sort_by(|a, b| a.partial_cmp(b).expect("latencies are finite"));
        let fail_count = observations.len() as u64 - ok_latencies.len() as u64;
        let mean = if ok_latencies.is_empty() {
            0.0
        } else {
            ok_latencies.iter().sum::<f64>() / ok_latencies.len() as f64
        };
        Self {
            ok_count: ok_latencies.len() as u64,
            fail_count,
            p50_ms: percentile(&ok_latencies, 50.0),
            p90_ms: percentile(&ok_latencies, 90.0),
            p99_ms: percentile(&ok_latencies, 99.0),
            mean_ms: mean,
        }
    }

    /// Look up a stat by its capture key.
    pub fn get(&self, key: &str) -> Option<f64> {
        match key {
            "p50_ms" => Some(self.p50_ms),
            "p90_ms" => Some(self.p90_ms),
            "p99_ms" => Some(self.p99_ms),
            "mean_ms" => Some(self.mean_ms),
            "ok_count" => Some(self.ok_count as f64),
            "fail_count" => Some(self.fail_count as f64),
            _ => None,
        }
    }
}

/// Ceil-rank percentile over a SORTED slice. Empty slice -> 0.0.
pub fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = ((q / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceil_rank_percentile_matches_chaos_sh() {
        // 100 samples 1..=100: p50 = 50th value = 50, p99 = 99th = 99.
        let v: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        assert_eq!(percentile(&v, 50.0), 50.0);
        assert_eq!(percentile(&v, 99.0), 99.0);
        assert_eq!(percentile(&v, 100.0), 100.0);
        // Small n: p99 of 10 samples = ceil(9.9)=10th value.
        let v: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        assert_eq!(percentile(&v, 99.0), 10.0);
    }

    #[test]
    fn fold_excludes_failures_from_percentiles() {
        let obs = vec![(10.0, true), (20.0, true), (5000.0, false)];
        let s = SampleStats::fold(&obs);
        assert_eq!(s.ok_count, 2);
        assert_eq!(s.fail_count, 1);
        assert_eq!(s.p99_ms, 20.0);
        assert_eq!(s.mean_ms, 15.0);
    }

    #[test]
    fn stat_lookup_by_key() {
        let s = SampleStats::fold(&[(10.0, true)]);
        assert_eq!(s.get("p99_ms"), Some(10.0));
        assert_eq!(s.get("fail_count"), Some(0.0));
        assert_eq!(s.get("nope"), None);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/engine/sample.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/engine/sample.rs` captured during rig
      standardization onto the codegen ladder.
```
