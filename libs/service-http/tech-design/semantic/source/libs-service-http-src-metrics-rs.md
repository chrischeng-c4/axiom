---
id: libs-service-http-src-metrics-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/src/metrics.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Http library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/src/metrics.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-http/src/metrics.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MetricsProvider` | libs/service-http/src/metrics.rs | trait | pub | 10 | pub trait MetricsProvider: Send + Sync { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Metrics seam for `/metrics`.
//!
//! A service supplies a type that renders its Prometheus text-format body; the
//! shared probe router serves it at `GET /metrics` as
//! `text/plain; version=0.0.4`. When a service has no metrics it can omit the
//! provider entirely (the probe router serves an empty body), so the default
//! method returns `String::new()`.

/// Renders the Prometheus text-format `/metrics` body.
pub trait MetricsProvider: Send + Sync {
    /// The full Prometheus text-format exposition. Defaults to empty.
    fn render_metrics(&self) -> String {
        String::new()
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/src/metrics.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-http/src/metrics.rs` captured during libs codegen standardization.
```
