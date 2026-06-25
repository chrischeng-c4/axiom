// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:cb6414bd" tracker="pending-tracker" reason="Crate root: module wiring and public re-exports for the in-process core."
//! # relay core
//!
//! The broker server core: a durable ordered log per `(subject, shard)` that
//! serves **both** delivery models over the same log —
//!
//! - **broadcast / fan-out** — every subscriber receives every message in seq
//!   order, replayable from any seq (see [`BroadcastDelivery`]);
//! - **work-queue / competing** — each message is leased to exactly one
//!   consumer, with ack / lease-expiry redelivery and a committed offset (see
//!   [`WorkQueue`]).
//!
//! The core is payload-agnostic (epic #120: the broker "knows nothing about
//! workflows"): a message body is an opaque [`Payload`] (JSON) plus a
//! caller-supplied [`MessageId`] for sequencing and dedupe. A producer
//! serializes whatever message type it uses into the payload. relay owns the
//! log, sequencing, dedupe, cursors, and leases, and depends on no other axiom
//! project. This is the new server core (relay epic #120, issue #122).
//!
//! ```
//! use std::collections::BTreeMap;
//! use chrono::Utc;
//! use relay::{Relay, RelayCoreConfig};
//!
//! let mut relay = Relay::new(RelayCoreConfig::in_memory());
//! let now = Utc::now();
//! let body = serde_json::json!({ "task": "greet", "args": ["hi"] });
//!
//! // One log, two delivery models.
//! relay.subscribe("events", "subscriber-a", 0).unwrap();
//! let out = relay.publish("events", "m-1", body, BTreeMap::new(), now).unwrap();
//! assert_eq!(out.seq, 0);
//!
//! let delivered = relay.poll("events", "subscriber-a").unwrap();
//! assert_eq!(delivered.len(), 1);
//! ```

pub mod broadcast;
pub mod config;
pub mod consume;
pub mod engine;
pub mod log;
pub mod openapi;
pub mod perf_gate;
pub mod raft;
pub mod raft_config;
pub mod raft_driver;
pub mod raft_store;
pub mod reconciler;
pub mod replication;
pub mod server;
pub mod server_config;
pub mod shard;
pub mod types;
pub mod wire;
pub mod workqueue;

pub use broadcast::BroadcastDelivery;
pub use config::{
    BroadcastConfig, DedupeConfig, FsyncPolicy, RelayCoreConfig, RetentionConfig, WorkQueueConfig,
};
pub use engine::Relay;
pub use log::Log;
pub use raft::{
    auto_membership, AppendReq, AppendResp, Membership, NodeId, Outgoing, PersistedState,
    RaftEntry, RaftMsg, RaftNode, RaftTransport, Role, VoteReq, VoteResp,
};
pub use raft_config::{ordinal_from_hostname, peer_urls, RaftClusterConfig};
pub use raft_driver::RaftDriver;
pub use raft_store::RaftStore;
pub use reconciler::{spawn_reconciler, ReconcilerHandle};
pub use replication::{spawn_follower, FollowerHandle};
pub use server::{router, AppState};
pub use server_config::RelayServerConfig;
pub use shard::shard_for;
pub use types::{
    AppendOutcome, CommittedOffset, DeliveryModel, Lease, LogEntry, MessageId, Payload, Seq,
    ShardId, Subject, SubscriberCursor,
};
pub use workqueue::WorkQueue;
// HANDWRITE-END
