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

use anyhow::{Context, Result};
use async_nats::jetstream::{self, consumer::DeliverPolicy};
use async_trait::async_trait;
use futures::StreamExt;

use crate::wal::{WalLog, WalRecord, WalStream};

const STREAM_NAME: &str = "lumen_wal";
const SUBJECT: &str = "lumen.wal";

pub struct NatsWal {
    js: jetstream::Context,
}

impl NatsWal {
    /// Connect to NATS at `url` (e.g. `nats://localhost:4222`) and
    /// ensure the WAL stream exists.
    pub async fn connect(url: &str) -> Result<Self> {
        let client = async_nats::connect(url)
            .await
            .with_context(|| format!("connect NATS at {url}"))?;
        let js = jetstream::new(client);
        js.get_or_create_stream(jetstream::stream::Config {
            name: STREAM_NAME.to_string(),
            subjects: vec![SUBJECT.to_string()],
            // File storage = durable across broker restart. Retention is
            // limits-based; operators size it to cover the gap between
            // RDB snapshots (a node behind retention re-bootstraps from
            // RDB).
            ..Default::default()
        })
        .await
        .context("get_or_create_stream")?;
        Ok(Self { js })
    }
}

#[async_trait]
impl WalLog for NatsWal {
    async fn publish(&self, record: &WalRecord) -> Result<u64> {
        let payload = record.encode()?;
        let ack = self
            .js
            .publish(SUBJECT, payload.into())
            .await
            .context("jetstream publish")?
            .await
            .context("jetstream publish ack")?;
        Ok(ack.sequence)
    }

    async fn subscribe(&self, from_seq: u64) -> Result<WalStream> {
        let stream = self
            .js
            .get_stream(STREAM_NAME)
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
        let messages = consumer.messages().await.context("consumer.messages")?;

        let out = messages.filter_map(|msg| async move {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => return Some(Err(anyhow::anyhow!("nats message error: {e}"))),
            };
            let seq = match msg.info() {
                Ok(info) => info.stream_sequence,
                Err(e) => return Some(Err(anyhow::anyhow!("nats message info: {e}"))),
            };
            // Best-effort ack; redelivery is handled idempotently by the
            // apply loop (it tracks applied-seq and skips <= applied).
            let _ = msg.ack().await;
            match WalRecord::decode(&msg.payload) {
                Ok(rec) => Some(Ok((seq, rec))),
                Err(e) => Some(Err(e)),
            }
        });

        Ok(Box::pin(out))
    }

    async fn latest_seq(&self) -> Result<u64> {
        let stream = self
            .js
            .get_stream(STREAM_NAME)
            .await
            .context("get_stream")?;
        let info = stream.cached_info();
        Ok(info.state.last_sequence)
    }
}
