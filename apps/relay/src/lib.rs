// SPEC-MANAGED: apps/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:cb6414bd" tracker="pending-tracker" reason="Crate root: module wiring and public re-exports for the in-process core."
//! # relay core
//!
//! The broker server core: a single-cast **work-queue** over a durable ordered
//! log per `(subject, shard)`. Each message is leased to exactly one competing
//! consumer, acked, then reclaimed (delete-on-ack), with lease-expiry
//! redelivery, a dead-letter path, priority, and a committed offset (see
//! [`WorkQueue`]). Pure pull broker — no broadcast/replay (that is `tape`) and no
//! push/ETA dispatch (that is `defer`).
//!
//! The core is payload-agnostic (epic #120: the broker "knows nothing about
//! workflows"): a message body is an opaque [`Payload`] (JSON) plus a
//! caller-supplied [`MessageId`] for sequencing and dedupe. A producer
//! serializes whatever message type it uses into the payload. relay owns the
//! log, sequencing, dedupe, and leases, and depends on no other axiom project.
//!
//! ```
//! use std::collections::BTreeMap;
//! use chrono::Utc;
//! use relay::{Relay, RelayCoreConfig};
//!
//! let relay = Relay::new(RelayCoreConfig::in_memory());
//! let now = Utc::now();
//! let body = serde_json::json!({ "task": "greet", "args": ["hi"] });
//!
//! // Publish a task; a worker leases it exactly once and acks it.
//! let out = relay.publish("tasks", "m-1", body, BTreeMap::new(), now).unwrap();
//! assert_eq!(out.seq, 0);
//!
//! let lease = relay.lease("tasks", "worker-a", now).unwrap().unwrap();
//! assert_eq!(lease.seq, 0);
//! assert!(relay.ack("tasks", &lease.lease_id, Some(lease.epoch)).unwrap());
//! ```

pub mod auth;
#[cfg(feature = "backup")]
pub mod backup;
pub mod config;
pub mod consume;
pub mod engine;
pub mod log;
pub mod metrics;
pub mod openapi;
#[cfg(feature = "operator")]
pub mod operator;
pub mod peer_tls;
pub mod perf_gate;
pub mod raft;
pub mod reconciler;
pub mod server;
pub mod server_config;
pub mod shard;
pub mod tls;
pub mod types;
pub mod wire;
pub mod workqueue;

pub use config::{DedupeConfig, FsyncPolicy, RelayCoreConfig, RetentionConfig, WorkQueueConfig};
pub use engine::{Relay, SubjectLive};
pub use log::Log;
pub use raft::{
    load_snapshot_bytes, snapshot_bytes, EngineSnapshot, PubCommand, RelayRaft, RelayStateMachine,
};
pub use reconciler::{spawn_reconciler, ReconcilerHandle};
pub use server::{router, AppState};
pub use server_config::RelayServerConfig;
pub use shard::shard_for;
pub use types::{
    AppendOutcome, CommittedOffset, Lease, LogEntry, MessageId, Payload, Seq, ShardId, Subject,
    DEFAULT_PRIORITY,
};
pub use workqueue::WorkQueue;
// HANDWRITE-END
