---
id: libs-service-http-src-signal-rs
summary: Lossless rust-source-unit coverage for `libs/service-http/src/signal.rs`.
capability_refs:
  - id: shared-http-service-scaffold
    role: primary
    claim: shared-http-service-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Http library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/service-http/src/signal.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/service-http/src/signal.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `wait_shutdown_signal` | libs/service-http/src/signal.rs | re-export | pub | 6 | pub use server_lifecycle::{shutdown_with_drain, wait_shutdown_signal}; |
| `shutdown_with_drain` | libs/service-http/src/signal.rs | re-export | pub | 6 | pub use server_lifecycle::{shutdown_with_drain, wait_shutdown_signal}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Service-shell re-exports for protocol-neutral graceful shutdown.
//!
//! The drain dance every k8s-native service in the ecosystem repeats: on
//! SIGINT/SIGTERM, flip readiness to draining (so `/readyz` → 503 and k8s stops
//! routing), hold a grace window, then let the listener close. Factored out of
//! lumen's / keep's `shutdown_signal`. Ownership lives in `server-lifecycle`.

pub use server_lifecycle::{shutdown_with_drain, wait_shutdown_signal};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-http/src/signal.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/service-http/src/signal.rs` captured during libs codegen standardization.
```
