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

#[cfg(feature = "certificate")]
pub mod certificate;
#[cfg(feature = "controller")]
pub mod controller;
pub mod crd;
#[cfg(feature = "controller")]
pub mod lease;
pub mod lifecycle;
#[cfg(feature = "controller")]
pub mod llm;
#[cfg(feature = "controller")]
pub mod metrics;
pub mod render;
#[cfg(feature = "controller")]
pub mod resize;
pub mod service;
pub mod stateful;

#[cfg(feature = "certificate")]
pub use certificate::{
    CertificateFacts, CertificateProfile, InstanceScope, Issuer, IssuerId, Purpose, Reconciler,
};
#[cfg(feature = "controller")]
pub use controller::{run, Error};
#[cfg(feature = "controller")]
pub use lease::Election;
pub use lifecycle::{LifecyclePolicy, LifecyclePolicyError, ProbeTiming, TerminationBudget};
#[cfg(feature = "controller")]
pub use metrics::ControllerMetrics;
#[cfg(feature = "controller")]
pub use service::{
    ClusterSpec, Condition, ConditionFact, ConditionStatus, ManagedService, ReadinessTarget,
    ReadyFacts, ResourceSpec,
};
pub use stateful::{
    plan_replica_layer, plan_shard_split, ObservedShardUsage, ObservedUtilization,
    ReplicaLayerError, ReplicaLayerPlan, ReplicaLayerPolicy, ShardSplitError, ShardSplitPlan,
    ShardSplitPolicy, DEFAULT_CPU_REQUEST, DEFAULT_MEMORY_REQUEST,
    DEFAULT_SHARD_SPLIT_THRESHOLD_BYTES,
};
// CODEGEN-END
