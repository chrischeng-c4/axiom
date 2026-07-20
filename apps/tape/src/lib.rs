// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-src-lib-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:tape-bootstrap" tracker="#768" reason="Initial local replay journal and checkpoint core before Tape has a generated TD source unit.">
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use utoipa::ToSchema;

pub const DEFAULT_PULL_BATCH: usize = 100;
pub const MAX_PULL_BATCH: usize = 1_000;

pub mod auth;
#[cfg(feature = "backup")]
pub mod backup;
pub mod bench;
pub mod metrics;
pub mod openapi;
#[cfg(feature = "operator")]
pub mod operator;
pub mod peer_tls;
pub mod raft;
pub mod replay_wire;
pub mod server;
pub mod spec;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum TapeError {
    #[error("checkpoint offset {new_offset} is behind existing offset {current_offset}")]
    StaleCheckpoint {
        current_offset: u64,
        new_offset: u64,
    },
    #[error("checkpoint offset {offset} is beyond topic end offset {end_offset}")]
    CheckpointBeyondEnd { offset: u64, end_offset: u64 },
}

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionError {
    #[error("subscription {name} already exists for topic {topic}")]
    AlreadyExists { topic: String, name: String },
    #[error("subscription {name} does not exist for topic {topic}")]
    NotFound { topic: String, name: String },
    #[error("pull batch limit {limit} exceeds maximum {max}")]
    PullBatchTooLarge { limit: usize, max: usize },
}

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionAckError {
    #[error(transparent)]
    Subscription(#[from] SubscriptionError),
    #[error(transparent)]
    Checkpoint(#[from] TapeError),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct TapeEvent {
    pub topic: String,
    pub offset: u64,
    pub timestamp_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub payload: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ConsumerCheckpoint {
    pub topic: String,
    pub consumer: String,
    pub offset: u64,
    pub updated_at_ms: u64,
}

// @spec apps/tape/tech-design/logic/expose-subscriptions-as-topic-delivery-resources.md#changes
/// A named caller-driven pull cursor owned by a topic. Tape has no delivery
/// mode switch: push, leases, and consumer-group ownership belong elsewhere.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Subscription {
    pub topic: String,
    pub name: String,
}

/// One caller-driven pull window. `cursor` is the checkpoint used to read;
/// `next_offset` is advisory until an explicit [`TapeJournal::ack_subscription`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PullSubscriptionBatch {
    pub topic: String,
    pub subscription: String,
    pub cursor: u64,
    pub limit: usize,
    pub next_offset: u64,
    pub events: Vec<TapeEvent>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RetentionPolicy {
    /// Explicit lower bound: events below this offset are eligible for removal.
    #[serde(default)]
    pub min_offset: Option<u64>,
    /// Events older than this wall-clock window are eligible for removal.
    #[serde(default)]
    pub max_age_seconds: Option<u64>,
    /// Never prune beyond the oldest named consumer checkpoint.
    #[serde(default)]
    pub protected_consumers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RetentionOutcome {
    pub topic: String,
    pub policy: RetentionPolicy,
    pub earliest_offset: u64,
    pub end_offset: u64,
    pub removed: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TapeJournal {
    topics: BTreeMap<String, Vec<TapeEvent>>,
    #[serde(default)]
    next_offsets: BTreeMap<String, u64>,
    checkpoints: BTreeMap<String, ConsumerCheckpoint>,
    #[serde(default)]
    subscriptions: BTreeMap<String, Subscription>,
    #[serde(default)]
    retention: BTreeMap<String, RetentionPolicy>,
}

impl TapeJournal {
    pub fn append(
        &mut self,
        topic: impl Into<String>,
        key: Option<String>,
        payload: Value,
        timestamp_ms: Option<u64>,
    ) -> TapeEvent {
        let topic = topic.into();
        let timestamp_ms = timestamp_ms.unwrap_or_else(now_ms);
        self.append_at(topic, key, payload, timestamp_ms, now_ms())
    }

    /// Deterministic append + retention transition for Raft apply. Event time
    /// and policy-evaluation time are separate so historical backfill does not
    /// rewind the retention clock.
    pub fn append_at(
        &mut self,
        topic: impl Into<String>,
        key: Option<String>,
        payload: Value,
        timestamp_ms: u64,
        applied_at_ms: u64,
    ) -> TapeEvent {
        let topic = topic.into();
        let recovered_next = self
            .topics
            .get(&topic)
            .and_then(|events| events.last())
            .map(|event| event.offset + 1)
            .unwrap_or(0);
        let next = self
            .next_offsets
            .entry(topic.clone())
            .or_insert(recovered_next);
        let event = TapeEvent {
            topic: topic.clone(),
            offset: *next,
            timestamp_ms,
            key,
            payload,
        };
        *next = next.saturating_add(1);
        self.topics
            .entry(topic.clone())
            .or_default()
            .push(event.clone());
        self.enforce_retention(&topic, applied_at_ms);
        event
    }

    pub fn replay(
        &self,
        topic: &str,
        from_offset: Option<u64>,
        from_timestamp_ms: Option<u64>,
        limit: Option<usize>,
    ) -> Vec<TapeEvent> {
        self.replay_refs(topic, from_offset, from_timestamp_ms, limit)
            .into_iter()
            .cloned()
            .collect()
    }

    pub fn replay_refs(
        &self,
        topic: &str,
        from_offset: Option<u64>,
        from_timestamp_ms: Option<u64>,
        limit: Option<usize>,
    ) -> Vec<&TapeEvent> {
        let from_offset = from_offset.unwrap_or(0);
        let events = self
            .topics
            .get(topic)
            .into_iter()
            .flatten()
            .filter(|event| {
                event.offset >= from_offset
                    && from_timestamp_ms
                        .map(|timestamp| event.timestamp_ms >= timestamp)
                        .unwrap_or(true)
            });
        match limit {
            Some(limit) => events.take(limit).collect(),
            None => events.collect(),
        }
    }

    pub fn put_checkpoint(
        &mut self,
        topic: impl Into<String>,
        consumer: impl Into<String>,
        offset: u64,
    ) -> Result<ConsumerCheckpoint, TapeError> {
        self.put_checkpoint_at(topic, consumer, offset, now_ms())
    }

    /// Same validation/ordering as [`Self::put_checkpoint`], parameterized on
    /// the timestamp so raft replicas apply an identical `updated_at_ms`
    /// instead of each computing `now_ms()` independently (#1327).
    pub fn put_checkpoint_at(
        &mut self,
        topic: impl Into<String>,
        consumer: impl Into<String>,
        offset: u64,
        updated_at_ms: u64,
    ) -> Result<ConsumerCheckpoint, TapeError> {
        let topic = topic.into();
        let consumer = consumer.into();
        let end_offset = self.end_offset(&topic);
        if offset > end_offset {
            return Err(TapeError::CheckpointBeyondEnd { offset, end_offset });
        }
        let key = checkpoint_key(&topic, &consumer);
        if let Some(existing) = self.checkpoints.get(&key) {
            if offset < existing.offset {
                return Err(TapeError::StaleCheckpoint {
                    current_offset: existing.offset,
                    new_offset: offset,
                });
            }
        }
        let checkpoint = ConsumerCheckpoint {
            topic,
            consumer,
            offset,
            updated_at_ms,
        };
        self.checkpoints.insert(key, checkpoint.clone());
        Ok(checkpoint)
    }

    pub fn checkpoint(&self, topic: &str, consumer: &str) -> Option<&ConsumerCheckpoint> {
        self.checkpoints.get(&checkpoint_key(topic, consumer))
    }

    // @spec apps/tape/tech-design/logic/expose-subscriptions-as-topic-delivery-resources.md#changes
    /// Create a topic-scoped subscription without moving a pull checkpoint.
    pub fn create_subscription(
        &mut self,
        topic: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Subscription, SubscriptionError> {
        let topic = topic.into();
        let name = name.into();
        let key = subscription_key(&topic, &name);
        if self.subscriptions.contains_key(&key) {
            return Err(SubscriptionError::AlreadyExists { topic, name });
        }
        let subscription = Subscription { topic, name };
        self.subscriptions.insert(key, subscription.clone());
        Ok(subscription)
    }

    pub fn subscriptions(&self, topic: &str) -> Vec<Subscription> {
        self.subscriptions
            .values()
            .filter(|subscription| subscription.topic == topic)
            .cloned()
            .collect()
    }

    pub fn subscription(&self, topic: &str, name: &str) -> Option<&Subscription> {
        self.subscriptions.get(&subscription_key(topic, name))
    }

    /// Delete subscription metadata only; a matching pull checkpoint remains
    /// available through the existing checkpoint interface.
    pub fn delete_subscription(
        &mut self,
        topic: &str,
        name: &str,
    ) -> Result<Subscription, SubscriptionError> {
        self.subscriptions
            .remove(&subscription_key(topic, name))
            .ok_or_else(|| SubscriptionError::NotFound {
                topic: topic.to_string(),
                name: name.to_string(),
            })
    }

    // @spec apps/tape/tech-design/logic/normalize-replay-checkpoints-into-pull-subscriptions.md#changes
    /// Read a bounded, caller-driven window from a pull subscription cursor.
    /// Pulling is deliberately side-effect free: a caller must explicitly ack
    /// after processing to advance the durable checkpoint.
    pub fn pull_subscription(
        &self,
        topic: &str,
        name: &str,
        limit: Option<usize>,
    ) -> Result<PullSubscriptionBatch, SubscriptionError> {
        self.require_pull_subscription(topic, name)?;
        let limit = limit.unwrap_or(DEFAULT_PULL_BATCH);
        if limit > MAX_PULL_BATCH {
            return Err(SubscriptionError::PullBatchTooLarge {
                limit,
                max: MAX_PULL_BATCH,
            });
        }
        let cursor = self
            .checkpoint(topic, name)
            .map(|checkpoint| checkpoint.offset)
            .unwrap_or(0);
        let events = self.replay(topic, Some(cursor), None, Some(limit));
        let next_offset = events
            .last()
            .map(|event| event.offset + 1)
            .unwrap_or(cursor);
        Ok(PullSubscriptionBatch {
            topic: topic.to_string(),
            subscription: name.to_string(),
            cursor,
            limit,
            next_offset,
            events,
        })
    }

    /// Acknowledge a completed pull window by advancing its existing durable
    /// topic/name checkpoint. The checkpoint's stale and beyond-end guards are
    /// intentionally reused without introducing leases or in-flight state.
    pub fn ack_subscription(
        &mut self,
        topic: &str,
        name: &str,
        offset: u64,
    ) -> Result<ConsumerCheckpoint, SubscriptionAckError> {
        self.require_pull_subscription(topic, name)?;
        Ok(self.put_checkpoint(topic, name, offset)?)
    }

    pub(crate) fn require_pull_subscription(
        &self,
        topic: &str,
        name: &str,
    ) -> Result<&Subscription, SubscriptionError> {
        let subscription =
            self.subscription(topic, name)
                .ok_or_else(|| SubscriptionError::NotFound {
                    topic: topic.to_string(),
                    name: name.to_string(),
                })?;
        Ok(subscription)
    }

    pub fn end_offset(&self, topic: &str) -> u64 {
        self.next_offsets.get(topic).copied().unwrap_or_else(|| {
            self.topics
                .get(topic)
                .and_then(|events| events.last())
                .map(|event| event.offset + 1)
                .unwrap_or(0)
        })
    }

    pub fn retention(&self, topic: &str) -> Option<&RetentionPolicy> {
        self.retention.get(topic)
    }

    pub fn put_retention(
        &mut self,
        topic: impl Into<String>,
        policy: RetentionPolicy,
        now_ms: u64,
    ) -> RetentionOutcome {
        let topic = topic.into();
        self.retention.insert(topic.clone(), policy.clone());
        let removed = self.enforce_retention(&topic, now_ms);
        RetentionOutcome {
            earliest_offset: self
                .topics
                .get(&topic)
                .and_then(|events| events.first())
                .map(|event| event.offset)
                .unwrap_or_else(|| self.end_offset(&topic)),
            end_offset: self.end_offset(&topic),
            topic,
            policy,
            removed,
        }
    }

    fn enforce_retention(&mut self, topic: &str, now_ms: u64) -> usize {
        let Some(policy) = self.retention.get(topic).cloned() else {
            return 0;
        };
        let end = self.end_offset(topic);
        let events = self.topics.entry(topic.to_string()).or_default();
        let age_boundary = policy.max_age_seconds.map(|seconds| {
            let cutoff = now_ms.saturating_sub(seconds.saturating_mul(1_000));
            events
                .iter()
                .find(|event| event.timestamp_ms >= cutoff)
                .map(|event| event.offset)
                .unwrap_or(end)
        });
        let mut boundary = policy
            .min_offset
            .into_iter()
            .chain(age_boundary)
            .max()
            .unwrap_or(0)
            .min(end);
        if let Some(protected) = policy
            .protected_consumers
            .iter()
            .filter_map(|consumer| self.checkpoints.get(&checkpoint_key(topic, consumer)))
            .map(|checkpoint| checkpoint.offset)
            .min()
        {
            boundary = boundary.min(protected);
        }
        let before = events.len();
        events.retain(|event| event.offset >= boundary);
        before - events.len()
    }
}
// </HANDWRITE>

fn checkpoint_key(topic: &str, consumer: &str) -> String {
    format!("{topic}\u{1f}{consumer}")
}

fn subscription_key(topic: &str, name: &str) -> String {
    format!("{topic}\u{1f}{name}")
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_replay_by_offset_and_time() {
        let mut journal = TapeJournal::default();
        journal.append(
            "orders",
            Some("a".into()),
            serde_json::json!({"n": 1}),
            Some(100),
        );
        journal.append(
            "orders",
            Some("b".into()),
            serde_json::json!({"n": 2}),
            Some(200),
        );

        let by_offset = journal.replay("orders", Some(1), None, None);
        assert_eq!(by_offset.len(), 1);
        assert_eq!(by_offset[0].payload, serde_json::json!({"n": 2}));

        let by_time = journal.replay("orders", None, Some(150), Some(1));
        assert_eq!(by_time.len(), 1);
        assert_eq!(by_time[0].offset, 1);
    }

    #[test]
    fn checkpoints_advance_and_reject_stale_offsets() {
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        journal.append("orders", None, serde_json::json!({"n": 2}), Some(200));

        let checkpoint = journal.put_checkpoint("orders", "worker-a", 1).unwrap();
        assert_eq!(checkpoint.offset, 1);
        assert_eq!(journal.checkpoint("orders", "worker-a").unwrap().offset, 1);
        assert!(matches!(
            journal.put_checkpoint("orders", "worker-a", 0),
            Err(TapeError::StaleCheckpoint { .. })
        ));
        assert!(matches!(
            journal.put_checkpoint("orders", "worker-a", 3),
            Err(TapeError::CheckpointBeyondEnd { .. })
        ));
    }

    #[test]
    fn pull_subscription_preserves_checkpoint_compatibility() {
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        let checkpoint = journal.put_checkpoint("orders", "worker-a", 1).unwrap();

        let subscription = journal.create_subscription("orders", "worker-a").unwrap();
        assert_eq!(subscription.name, "worker-a");
        assert_eq!(journal.checkpoint("orders", "worker-a"), Some(&checkpoint));

        let deleted = journal.delete_subscription("orders", "worker-a").unwrap();
        assert_eq!(deleted.name, "worker-a");
        assert_eq!(journal.checkpoint("orders", "worker-a"), Some(&checkpoint));
    }

    #[test]
    fn pull_subscription_uses_checkpoint_cursor_and_never_implicitly_acks() {
        let mut journal = TapeJournal::default();
        for offset in 0..3 {
            journal.append(
                "orders",
                None,
                serde_json::json!({"offset": offset}),
                Some(100),
            );
        }
        journal.create_subscription("orders", "worker-a").unwrap();
        journal.put_checkpoint("orders", "worker-a", 1).unwrap();

        let batch = journal
            .pull_subscription("orders", "worker-a", Some(2))
            .unwrap();
        assert_eq!(batch.cursor, 1);
        assert_eq!(
            batch
                .events
                .iter()
                .map(|event| event.offset)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(batch.next_offset, 3);
        assert_eq!(journal.checkpoint("orders", "worker-a").unwrap().offset, 1);
    }

    #[test]
    fn pull_subscription_ack_reuses_checkpoint_guards() {
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        journal.create_subscription("orders", "worker-a").unwrap();
        assert!(matches!(
            journal.ack_subscription("orders", "worker-a", 2),
            Err(SubscriptionAckError::Checkpoint(
                TapeError::CheckpointBeyondEnd { .. }
            ))
        ));
        journal.ack_subscription("orders", "worker-a", 1).unwrap();
        assert!(matches!(
            journal.ack_subscription("orders", "worker-a", 0),
            Err(SubscriptionAckError::Checkpoint(
                TapeError::StaleCheckpoint { .. }
            ))
        ));
    }

    #[test]
    fn pull_subscription_rejects_oversized_window() {
        let mut journal = TapeJournal::default();
        journal.append("orders", None, serde_json::json!({"n": 1}), Some(100));
        journal.create_subscription("orders", "worker-a").unwrap();

        assert!(matches!(
            journal.pull_subscription("orders", "worker-a", Some(MAX_PULL_BATCH + 1)),
            Err(SubscriptionError::PullBatchTooLarge { .. })
        ));
        assert!(journal.checkpoint("orders", "worker-a").is_none());
    }

    #[test]
    fn retention_prunes_history_without_rewinding_offsets_and_protects_consumers() {
        let mut journal = TapeJournal::default();
        for offset in 0..5 {
            journal.append_at(
                "orders",
                None,
                serde_json::json!({"offset": offset}),
                1_000 + offset * 1_000,
                5_000,
            );
        }
        journal
            .put_checkpoint_at("orders", "audit", 2, 5_000)
            .unwrap();
        let outcome = journal.put_retention(
            "orders",
            RetentionPolicy {
                min_offset: Some(4),
                max_age_seconds: None,
                protected_consumers: vec!["audit".into()],
            },
            5_000,
        );
        assert_eq!(outcome.earliest_offset, 2);
        assert_eq!(outcome.removed, 2);
        assert_eq!(
            journal
                .replay("orders", None, None, None)
                .into_iter()
                .map(|event| event.offset)
                .collect::<Vec<_>>(),
            vec![2, 3, 4]
        );

        let appended = journal.append_at(
            "orders",
            None,
            serde_json::json!({"offset": 5}),
            6_000,
            6_000,
        );
        assert_eq!(appended.offset, 5, "retention must not reuse offsets");
    }
}
