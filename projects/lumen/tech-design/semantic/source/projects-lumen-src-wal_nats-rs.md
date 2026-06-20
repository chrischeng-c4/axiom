---
id: projects-lumen-src-wal_nats-rs
capability_refs:
  - id: "search"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/wal_nats.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/wal_nats.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `NatsWal` | projects/lumen/src/wal_nats.rs | struct | pub |
| `NatsWalConfig` | projects/lumen/src/wal_nats.rs | struct | pub |
| `new` | projects/lumen/src/wal_nats.rs | function | pub |
| `shard` | projects/lumen/src/wal_nats.rs | function | pub |
| `connect` | projects/lumen/src/wal_nats.rs | function | pub |
| `connect_with_config` | projects/lumen/src/wal_nats.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_nats-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! NATS JetStream backend for [`WalLog`].
//!
//! JetStream gives us the clustered log for free — durable, ordered,
//! replicated, with replay-from-sequence. The crucial property for
//! lumen is **fan-out**: every serving node creates its **own**
//! consumer with an independent cursor, so each reads the *full* stream
//! and builds a complete index. This is the opposite of a
//! competing-consumer queue (and the reason GCP Pub/Sub doesn't fit:
//! its subscriptions load-balance, forcing one-subscription-per-node and
//! hitting the per-project quota). JetStream consumers are lightweight;
//! thousands coexist with no such wall.
//!
//! Mapping:
//! - publish → `js.publish(subject, payload)`; the ack's
//!   `stream_sequence` *is* our global seq.
//! - subscribe(from) → an **ephemeral** pull consumer with
//!   `DeliverPolicy::ByStartSequence { start_sequence: from + 1 }`.
//!   Ephemeral is correct: a node's restart position comes from its RDB
//!   baseline + tracked applied-seq, not from a durable server-side
//!   cursor. When the node dies, the consumer is cleaned up — exactly
//!   right for Deployment + HPA cattle.

use std::sync::{Arc, Mutex};

use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_nats::jetstream::{self, consumer::DeliverPolicy};
use async_trait::async_trait;
use futures::StreamExt;
use rustc_hash::FxHashMap;

use crate::wal::{WalLog, WalRecord, WalStream};

const DEFAULT_STREAM_NAME: &str = "lumen_wal";
const DEFAULT_SUBJECT: &str = "lumen.wal";
const APPLY_PULL_BATCH: usize = 256;
const APPLY_PULL_EXPIRES: Duration = Duration::from_micros(500);
const LOCAL_PUBLISH_WINDOW: u64 = 16_384;
const LOCAL_PUBLISH_RETAIN_AFTER: usize = LOCAL_PUBLISH_WINDOW as usize * 2;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_nats-rs.md#source
pub struct NatsWal {
    js: jetstream::Context,
    config: NatsWalConfig,
    // Local writes still apply only when the NATS tail delivers their sequence.
    // This cache just lets that local tail use the original record instead of
    // deserializing the payload this process just serialized.
    local_publishes: Arc<Mutex<FxHashMap<u64, WalRecord>>>,
}

/// JetStream stream/subject binding for one WAL partition.
///
/// The default is the historical single stream (`lumen_wal` / `lumen.wal`).
/// Sharded write apply uses one config per shard so each shard has its own
/// ordered log and apply loop instead of every write contending on one stream.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_nats-rs.md#source
pub struct NatsWalConfig {
    pub stream_name: String,
    pub subject: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_nats-rs.md#source
impl Default for NatsWalConfig {
    fn default() -> Self {
        Self {
            stream_name: DEFAULT_STREAM_NAME.to_string(),
            subject: DEFAULT_SUBJECT.to_string(),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_nats-rs.md#source
impl NatsWalConfig {
    pub fn new(stream_name: impl Into<String>, subject: impl Into<String>) -> Result<Self> {
        let stream_name = stream_name.into();
        let subject = subject.into();
        if stream_name.trim().is_empty() {
            bail!("NATS WAL stream name cannot be empty");
        }
        if subject.trim().is_empty() {
            bail!("NATS WAL subject cannot be empty");
        }
        Ok(Self {
            stream_name,
            subject,
        })
    }

    pub fn shard(shard: usize) -> Self {
        Self {
            stream_name: format!("{DEFAULT_STREAM_NAME}_{shard}"),
            subject: format!("{DEFAULT_SUBJECT}.{shard}"),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_nats-rs.md#source
impl NatsWal {
    /// Connect to NATS at `url` (e.g. `nats://localhost:4222`) and
    /// ensure the WAL stream exists.
    pub async fn connect(url: &str) -> Result<Self> {
        Self::connect_with_config(url, NatsWalConfig::default()).await
    }

    /// Connect to NATS using an explicit WAL stream/subject binding.
    pub async fn connect_with_config(url: &str, config: NatsWalConfig) -> Result<Self> {
        let client = async_nats::connect(url)
            .await
            .with_context(|| format!("connect NATS at {url}"))?;
        let js = jetstream::new(client);
        js.get_or_create_stream(jetstream::stream::Config {
            name: config.stream_name.clone(),
            subjects: vec![config.subject.clone()],
            // File storage = durable across broker restart. Retention is
            // limits-based; operators size it to cover the gap between
            // RDB snapshots (a node behind retention re-bootstraps from
            // RDB).
            ..Default::default()
        })
        .await
        .context("get_or_create_stream")?;
        Ok(Self {
            js,
            config,
            local_publishes: Arc::new(Mutex::new(FxHashMap::default())),
        })
    }
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_nats-rs.md#source
impl WalLog for NatsWal {
    async fn publish(&self, record: WalRecord) -> Result<u64> {
        let payload = record.encode()?;
        let ack = self
            .js
            .publish(self.config.subject.clone(), payload.into())
            .await
            .context("jetstream publish")?
            .await
            .context("jetstream publish ack")?;
        {
            let mut local = self
                .local_publishes
                .lock()
                .expect("local publish cache poisoned");
            local.insert(ack.sequence, record);
            if local.len() > LOCAL_PUBLISH_RETAIN_AFTER {
                let cutoff = ack.sequence.saturating_sub(LOCAL_PUBLISH_WINDOW);
                local.retain(|seq, _| *seq >= cutoff);
            }
        }
        Ok(ack.sequence)
    }

    async fn subscribe(&self, from_seq: u64) -> Result<WalStream> {
        let stream = self
            .js
            .get_stream(self.config.stream_name.clone())
            .await
            .context("get_stream")?;
        // ByStartSequence is inclusive, so start at from_seq + 1 to get
        // strictly-greater semantics (matching MemWal).
        let start = from_seq.saturating_add(1).max(1);
        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                deliver_policy: DeliverPolicy::ByStartSequence {
                    start_sequence: start,
                },
                ..Default::default()
            })
            .await
            .context("create ephemeral consumer")?;
        // Continuous pull stream: it prefetches the next batch as the local
        // buffer drains and yields each message as soon as it arrives. That keeps
        // the high-load batch behavior without adding the low-concurrency expiry
        // floor caused by draining a whole `batch()` before yielding the first
        // record to the apply loop.
        let messages = consumer
            .stream()
            .max_messages_per_batch(APPLY_PULL_BATCH)
            .expires(APPLY_PULL_EXPIRES)
            .messages()
            .await
            .context("create continuous pull stream")?;
        let local_publishes = self.local_publishes.clone();
        let out = messages.then(move |msg| {
            let local_publishes = local_publishes.clone();
            async move {
                let msg = msg.map_err(|e| anyhow::anyhow!("nats message error: {e}"))?;
                let seq = msg
                    .info()
                    .map_err(|e| anyhow::anyhow!("nats message info: {e}"))?
                    .stream_sequence;
                // Best-effort ack; redelivery is handled idempotently by the apply
                // loop (it tracks applied-seq and skips <= applied).
                let _ = msg.ack().await;
                let local_record = {
                    let mut local = local_publishes
                        .lock()
                        .expect("local publish cache poisoned");
                    local.remove(&seq)
                };
                match local_record {
                    Some(rec) => Ok((seq, rec)),
                    None => WalRecord::decode(&msg.payload).map(|rec| (seq, rec)),
                }
            }
        });

        Ok(Box::pin(out))
    }

    async fn latest_seq(&self) -> Result<u64> {
        let stream = self
            .js
            .get_stream(self.config.stream_name.clone())
            .await
            .context("get_stream")?;
        let info = stream.cached_info();
        Ok(info.state.last_sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_config_uses_distinct_stream_and_subject() {
        assert_eq!(NatsWalConfig::default().stream_name, "lumen_wal");
        assert_eq!(NatsWalConfig::default().subject, "lumen.wal");
        assert_eq!(NatsWalConfig::shard(3).stream_name, "lumen_wal_3");
        assert_eq!(NatsWalConfig::shard(3).subject, "lumen.wal.3");
    }

    #[test]
    fn config_rejects_empty_stream_or_subject() {
        assert!(NatsWalConfig::new("", "lumen.wal").is_err());
        assert!(NatsWalConfig::new("lumen_wal", "").is_err());
    }
}
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/wal_nats.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/wal_nats.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
