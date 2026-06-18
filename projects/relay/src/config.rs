// SPEC-MANAGED: projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#config
// HANDWRITE-BEGIN gap="missing-generator:config:1758bf38" tracker="pending-tracker" reason="RelayCoreConfig per the Config contract."
//! In-process broker core engine settings.
//!
//! All durability/retention is local to this core; transport, sharding
//! fan-out, and HA live in the server issues (#115 / #109) and are out of
//! scope here.

use serde::{Deserialize, Serialize};

/// Durability flush policy for the on-disk log segments.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#config
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsyncPolicy {
    /// `sync_all` after every append (strongest, slowest).
    Always,
    /// Flush the writer on every append; defer the OS fsync to `Interval`.
    Interval,
    /// Leave flushing to the OS page cache (fastest, weakest).
    Os,
}

impl Default for FsyncPolicy {
    fn default() -> Self {
        FsyncPolicy::Interval
    }
}

/// Idempotent at-least-once append window: how long a `MessageId` is remembered.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#config
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DedupeConfig {
    /// `MessageId`s retained per shard for duplicate detection.
    pub window_entries: u64,
    /// Also evict dedupe keys older than this many seconds.
    pub ttl_secs: u64,
}

impl Default for DedupeConfig {
    fn default() -> Self {
        DedupeConfig {
            window_entries: 1_048_576,
            ttl_secs: 3_600,
        }
    }
}

/// Work-queue / competing-consumer delivery settings (standard at-least-once
/// lease / retry semantics).
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#config
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkQueueConfig {
    /// Lease duration before an unacked entry is redelivery-eligible.
    pub lease_ttl_ms: u64,
    /// Redelivery attempts before revocation / dead-letter.
    pub max_attempts: u32,
    /// Base backoff between delivery attempts.
    pub redeliver_backoff_ms: u64,
}

impl Default for WorkQueueConfig {
    fn default() -> Self {
        WorkQueueConfig {
            lease_ttl_ms: 30_000,
            max_attempts: 5,
            redeliver_backoff_ms: 1_000,
        }
    }
}

/// Broadcast / fan-out delivery settings (replayable from any retained seq).
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#config
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BroadcastConfig {
    /// `0` = replay from any retained seq; `>0` caps replay depth.
    pub max_replay_entries: u64,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        BroadcastConfig {
            max_replay_entries: 0,
        }
    }
}

/// Retention / pruning of the durable log.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#config
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// Prune fully-acked / aged segments after this many seconds.
    pub max_age_secs: u64,
    /// `0` = unbounded; else prune oldest segments past this size.
    pub max_bytes_per_shard: u64,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        RetentionConfig {
            max_age_secs: 604_800,
            max_bytes_per_shard: 0,
        }
    }
}

/// Engine configuration for the in-process relay core.
///
/// @spec projects/relay/tech-design/logic/core-durable-log-single-multi-broadcast-delivery-model.md#config
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RelayCoreConfig {
    /// Root directory for durable disk segments. Empty = RAM-only (no disk).
    pub data_dir: String,
    /// Roll to a new disk segment at this size in bytes.
    pub segment_bytes: u64,
    /// Hot in-memory entries retained per `(subject, shard)`.
    pub ram_ring_entries: u64,
    /// Durability policy.
    pub fsync: FsyncPolicy,
    /// Flush cadence when `fsync = Interval`.
    pub fsync_interval_ms: u64,
    /// Shards per subject unless the subject overrides it.
    pub default_shards: u32,
    pub dedupe: DedupeConfig,
    pub work_queue: WorkQueueConfig,
    pub broadcast: BroadcastConfig,
    pub retention: RetentionConfig,
}

impl Default for RelayCoreConfig {
    fn default() -> Self {
        RelayCoreConfig {
            data_dir: "./relay-data".to_string(),
            segment_bytes: 134_217_728,
            ram_ring_entries: 65_536,
            fsync: FsyncPolicy::Interval,
            fsync_interval_ms: 50,
            default_shards: 1,
            dedupe: DedupeConfig::default(),
            work_queue: WorkQueueConfig::default(),
            broadcast: BroadcastConfig::default(),
            retention: RetentionConfig::default(),
        }
    }
}

impl RelayCoreConfig {
    /// A RAM-only config (no disk persistence) — handy for tests and embedding.
    pub fn in_memory() -> Self {
        RelayCoreConfig {
            data_dir: String::new(),
            ..RelayCoreConfig::default()
        }
    }
}
// HANDWRITE-END
