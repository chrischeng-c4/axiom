// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#schema
// HANDWRITE-BEGIN gap="missing-generator:schema:1e4054bb" tracker="pending-tracker" reason="Core data model per the Schema contract."
//! Core data model for the relay broker.
//!
//! A durable ordered log per `(subject, shard)` plus the per-model delivery
//! state that reads from it. The message payload is an opaque body stored
//! verbatim; relay owns only the log, sequencing, dedupe, subscriber cursors,
//! and work-queue leases.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Opaque message body carried by the broker.
///
/// The relay core is payload-agnostic (epic #120: the broker "knows nothing
/// about workflows"): it stores the body verbatim and never reinterprets it. A
/// producer serializes whatever message type it uses into this JSON value; the
/// broker only needs the caller-supplied [`MessageId`] for sequencing and
/// dedupe.
pub type Payload = serde_json::Value;

/// Logical channel a producer publishes to and consumers subscribe on.
pub type Subject = String;

/// Partition of a subject's log; ordering and sequencing are per `(subject, shard)`.
pub type ShardId = u32;

/// Monotonic, gap-free position assigned on append within one `(subject, shard)`.
///
/// The broadcast replay cursor and the work-queue ack cursor are both expressed
/// in this space.
pub type Seq = u64;

/// Deterministic id derived from producer key + content, used as the
/// idempotency / dedupe key so an at-least-once retry maps to the same entry.
pub type MessageId = String;

/// How a subject's appended messages are delivered.
///
/// `Broadcast`/`Multicast` fan out every message to every (group) subscriber,
/// replayable from a seq. `WorkQueue` leases each message to exactly one
/// competing consumer. `Singlecast` is the degenerate one-consumer case of
/// `WorkQueue`. The relay core supports broadcast and work-queue delivery over
/// the *same* log simultaneously; this enum is descriptive routing metadata.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryModel {
    Singlecast,
    Multicast,
    Broadcast,
    WorkQueue,
}

/// One durable record in the ordered log; the unit of both broadcast replay
/// and work-queue lease.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    /// Monotonic position within `(subject, shard)`.
    pub seq: Seq,
    pub message_id: MessageId,
    pub subject: Subject,
    pub shard: ShardId,
    /// Opaque message body, stored verbatim.
    pub payload: Payload,
    /// Opaque routing/trace headers carried with the entry.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// Server time the entry was durably appended.
    pub appended_at: DateTime<Utc>,
}

/// Result of a publish/append; idempotent on `MessageId`.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendOutcome {
    /// Seq of the (new or pre-existing) entry.
    pub seq: Seq,
    /// True when the id was already present and no new entry was written.
    pub deduped: bool,
}

/// Broadcast/fan-out read position; each subscriber advances independently and
/// may replay from any seq.
///
/// `position` is the next seq this subscriber will be delivered (exclusive of
/// what it has already received).
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriberCursor {
    pub subscriber_id: String,
    pub subject: Subject,
    pub shard: ShardId,
    /// Seq the subscription (re)started replay from.
    pub from_seq: Seq,
    /// Next seq to deliver to this subscriber.
    pub position: Seq,
}

/// Work-queue grant of one entry to exactly one consumer until it acks or the
/// lease expires.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lease {
    /// Unique id for this grant; required to ack.
    pub lease_id: String,
    /// Leased entry position.
    pub seq: Seq,
    pub subject: Subject,
    pub shard: ShardId,
    /// Consumer the entry is currently leased to.
    pub consumer_id: String,
    pub granted_at: DateTime<Utc>,
    /// On expiry the entry becomes eligible for redelivery to another consumer.
    pub expires_at: DateTime<Utc>,
    /// 1-based delivery attempt; drives retry / revocation policy.
    pub attempt: u32,
}

/// Work-queue durable progress: every entry at or below `committed_seq` has
/// been acked.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommittedOffset {
    pub subject: Subject,
    pub shard: ShardId,
    /// Highest seq such that every entry `0..=committed_seq` has been acked.
    pub committed_seq: Seq,
}
// HANDWRITE-END
