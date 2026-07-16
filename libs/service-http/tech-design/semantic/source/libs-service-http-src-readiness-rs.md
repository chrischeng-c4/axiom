---
id: libs-service-http-src-readiness-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/src/readiness.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Http library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/src/readiness.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-http/src/readiness.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReadinessHook` | libs/service-http/src/readiness.rs | re-export | pub | 12 | pub use server_lifecycle::Readiness as ReadinessHook; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Probe-facing name for the protocol-neutral lifecycle readiness contract.
//!
//! A service supplies a type that reports whether it is currently draining
//! (post-SIGTERM grace window). The shared probe router calls
//! [`server_lifecycle::Readiness::is_draining`] on every `/readyz` hit so k8s sees 503 the
//! moment a graceful shutdown begins, and stops routing before the listener
//! closes. In lumen/keep this is the engine's drain flag; any
//! `Arc`-shareable, `is_draining()`-reporting type works.

pub use server_lifecycle::Readiness as ReadinessHook;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/src/readiness.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-http/src/readiness.rs` captured during libs codegen standardization.
```
