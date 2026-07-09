// SPEC-MANAGED: apps/tape/tech-design/semantic/source/apps-tape-src-lib-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:tape-bootstrap" tracker="#768" reason="Initial local replay journal and checkpoint core before Tape has a generated TD source unit.">
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use utoipa::ToSchema;

pub mod auth;
pub mod bench;
pub mod metrics;
pub mod openapi;
pub mod server;
pub mod spec;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TapeError {
    #[error("checkpoint offset {new_offset} is behind existing offset {current_offset}")]
    StaleCheckpoint {
        current_offset: u64,
        new_offset: u64,
    },
    #[error("checkpoint offset {offset} is beyond topic end offset {end_offset}")]
    CheckpointBeyondEnd { offset: u64, end_offset: u64 },
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TapeJournal {
    topics: BTreeMap<String, Vec<TapeEvent>>,
    checkpoints: BTreeMap<String, ConsumerCheckpoint>,
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
        let entries = self.topics.entry(topic.clone()).or_default();
        let event = TapeEvent {
            topic,
            offset: entries.len() as u64,
            timestamp_ms: timestamp_ms.unwrap_or_else(now_ms),
            key,
            payload,
        };
        entries.push(event.clone());
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
            updated_at_ms: now_ms(),
        };
        self.checkpoints.insert(key, checkpoint.clone());
        Ok(checkpoint)
    }

    pub fn checkpoint(&self, topic: &str, consumer: &str) -> Option<&ConsumerCheckpoint> {
        self.checkpoints.get(&checkpoint_key(topic, consumer))
    }

    pub fn end_offset(&self, topic: &str) -> u64 {
        self.topics.get(topic).map(Vec::len).unwrap_or_default() as u64
    }
}
// </HANDWRITE>

fn checkpoint_key(topic: &str, consumer: &str) -> String {
    format!("{topic}\u{1f}{consumer}")
}

fn now_ms() -> u64 {
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
}
