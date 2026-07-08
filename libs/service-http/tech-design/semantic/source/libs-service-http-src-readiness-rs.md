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
| `ReadinessHook` | libs/service-http/src/readiness.rs | trait | pub | 12 | pub trait ReadinessHook: Send + Sync { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Readiness seam for `/readyz`.
//!
//! A service supplies a type that reports whether it is currently draining
//! (post-SIGTERM grace window). The shared probe router calls
//! [`ReadinessHook::is_draining`] on every `/readyz` hit so k8s sees 503 the
//! moment a graceful shutdown begins, and stops routing before the listener
//! closes. In lumen/keep this is the engine's drain flag; any
//! `Arc`-shareable, `is_draining()`-reporting type works.

/// Reports whether the service is draining (shutting down). `/readyz` returns
/// 503 when this is `true`, 200 otherwise.
pub trait ReadinessHook: Send + Sync {
    /// `true` once graceful shutdown has begun, so `/readyz` should report 503.
    fn is_draining(&self) -> bool;
}
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
