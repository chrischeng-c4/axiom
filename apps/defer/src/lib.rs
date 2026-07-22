// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:defer-core-scheduler" tracker="#766" reason="In-memory delayed push-queue scheduler core: ETA gate, u8 priority ordering, Defer-owned dispatch budget/concurrency, retry, DLQ, cancel, and terminal ack."
//! Defer core scheduler.
//!
//! This crate owns the main delayed push-queue logic: a task must be due before
//! priority matters, queue policy owns dispatch rate/concurrency, and leased
//! attempts end in ack, retry, cancellation, or dead-letter.

pub mod auth;
pub mod dispatch;
pub mod metrics;
pub mod openapi;
#[cfg(feature = "operator")]
pub mod operator;
pub mod peer_tls;
pub mod raft;
pub mod scheduler;
pub mod server;
pub mod types;

pub use auth::AuthConfig;
pub use dispatch::{DispatchDisposition, DispatchReport, HttpDispatcher, TargetSigningKey};
pub use raft::{DeferCommand, DeferOutcome, DeferRaft, DeferStateMachine};
pub use scheduler::DeferScheduler;
pub use types::{
    AttemptSettlement, CreateTask, DispatchLease, NackOutcome, QueueControlState, QueuePolicy,
    QueueSnapshot, SchedulerError, SchedulerResult, SettlementOutcome, Target, TaskStatus,
    DEFAULT_PRIORITY,
};
// HANDWRITE-END
