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
pub mod render;
pub mod service;

pub use controller::{run, Error};
pub use lease::Election;
pub use service::{ClusterSpec, ManagedService, ReadinessTarget, ReadyFacts, ResourceSpec};
