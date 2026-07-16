// SPEC-MANAGED: libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `service-k8s` — the ecosystem's shared Kubernetes operator scaffold.
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
//! the shared service kit (`raft-core` + `raft-runtime` + `transport-h2c` + `service-http` +
//! `service-backup` + `cli-std` + this).

pub mod controller;
pub mod crd;
pub mod lease;
pub mod llm;
pub mod render;
pub mod resize;
pub mod service;
pub mod stateful;

pub use controller::{run, Error};
pub use lease::Election;
pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec};
pub use stateful::{
    plan_replica_layer, ObservedUtilization, ReplicaLayerError, ReplicaLayerPlan,
    ReplicaLayerPolicy, DEFAULT_CPU_REQUEST, DEFAULT_MEMORY_REQUEST,
};
// CODEGEN-END
