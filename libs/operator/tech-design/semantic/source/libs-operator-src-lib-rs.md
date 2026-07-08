---
id: libs-operator-src-lib-rs
summary: Lossless rust-source-unit coverage for `libs/operator/src/lib.rs`.
capability_refs:
  - id: shared-kubernetes-operator-scaffold
    role: primary
    claim: shared-kubernetes-operator-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Operator library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/operator/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/operator/src/lib.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `controller` | libs/operator/src/lib.rs | module | pub | 15 | pub mod controller; |
| `lease` | libs/operator/src/lib.rs | module | pub | 16 | pub mod lease; |
| `llm` | libs/operator/src/lib.rs | module | pub | 17 | pub mod llm; |
| `render` | libs/operator/src/lib.rs | module | pub | 18 | pub mod render; |
| `resize` | libs/operator/src/lib.rs | module | pub | 19 | pub mod resize; |
| `service` | libs/operator/src/lib.rs | module | pub | 20 | pub mod service; |
| `run` | libs/operator/src/lib.rs | re-export | pub | 22 | pub use controller::{run, Error}; |
| `Error` | libs/operator/src/lib.rs | re-export | pub | 22 | pub use controller::{run, Error}; |
| `Election` | libs/operator/src/lib.rs | re-export | pub | 23 | pub use lease::Election; |
| `ClusterSpec` | libs/operator/src/lib.rs | re-export | pub | 24 | pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec}; |
| `ManagedService` | libs/operator/src/lib.rs | re-export | pub | 24 | pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec}; |
| `ReadinessTarget` | libs/operator/src/lib.rs | re-export | pub | 24 | pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec}; |
| `ReadyFacts` | libs/operator/src/lib.rs | re-export | pub | 24 | pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec}; |
| `ResourceSpec` | libs/operator/src/lib.rs | re-export | pub | 24 | pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec}; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `operator` — the ecosystem's shared k8s operator scaffold.
//!
//! Every axiom service that ships a CRD reconciles the same way: a controller
//! that watches the CR cluster-wide, server-side-applies the rendered child
//! objects, and writes back a status — gated by a leader-election Lease so
//! `replicas > 1` is safe. This crate centralizes that loop + the lease + a
//! render toolkit for the common sharded-HA objects and maintenance CronJobs,
//! so a service supplies only a [`ManagedService`] (its CRD type +
//! `render`/`status_patch`/readiness) and its service-specific rendering.
//!
//! See `CONTRIBUTING.md` "Service archetype" — this is the deploy-layer member of
//! the shared service kit (`raft-core` + `raft-host` + `h2c` + `service-http` +
//! `service-backup` + `cli-std` + this).

pub mod controller;
pub mod lease;
pub mod llm;
pub mod render;
pub mod resize;
pub mod service;

pub use controller::{run, Error};
pub use lease::Election;
pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec};
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/operator/src/lib.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/operator/src/lib.rs` captured during libs codegen standardization.
```
