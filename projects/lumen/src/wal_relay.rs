// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#rust-source-unit
// HANDWRITE-BEGIN gap="missing-generator:logic:54088576" tracker="pending-tracker" reason="RelayWal: a WalLog backed by relay's broadcast. publish POSTs to relay /v1/{subject}/publish (payload=json(WalRecord)); subscribe GETs /v1/{subject}/subscribe and decodes relay's length-prefixed CBOR LogEntry frames (relay::wire::decode_frames), mapping each to (seq+1, WalRecord). Plaintext h2c, no TLS."
//! #124 — tail **relay**'s broadcast as the WAL backend (behind the `relay-wal`
//! feature), replacing NATS.
//!
//! lumen is a log-tailing derived index (see HA.md): a write is published to the
//! broker and every pod folds the ordered log into its own index. This backend
//! makes that broker `relay` (itself HA via `libs/raftcore`): `publish` POSTs to
//! relay, `subscribe` tails relay's broadcast — the loop `relay::spawn_follower`
//! proved, mapped onto [`WalRecord`].
//!
//! relay is **h2c (cleartext)**, so the client links no TLS/openssl; the backend
//! is off by default, keeping the serving binary's clean cross-compile.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use relay::wire::decode_frames;
use relay::LogEntry;

use crate::wal::{WalLog, WalRecord, WalStream};

/// A `WalLog` backed by a relay broker's broadcast log.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
pub struct RelayWal {
    client: reqwest::Client,
    base: String,
    subject: String,
    counter: AtomicU64,
}

impl RelayWal {
    /// Connect to `base_url` (e.g. `http://relay:8080`) for `subject`.
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    pub fn new(base_url: impl Into<String>, subject: impl Into<String>) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().build()?,
            base: base_url.into().trim_end_matches('/').to_string(),
            subject: subject.into(),
            counter: AtomicU64::new(0),
        })
    }
}

#[async_trait]
impl WalLog for RelayWal {
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn publish(&self, record: WalRecord) -> Result<u64> {
        let payload = serde_json::to_value(&record)?;
        let message_id = format!("lumen-{}", self.counter.fetch_add(1, Ordering::Relaxed));
        let body = serde_json::json!({ "message_id": message_id, "payload": payload });
        let resp = self
            .client
            .post(format!("{}/v1/{}/publish", self.base, self.subject))
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        let out: serde_json::Value = resp.json().await?;
        let seq = out
            .get("seq")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("relay publish: no seq in response"))?;
        // relay seq is 0-based; the lumen WAL seq is 1-based.
        Ok(seq + 1)
    }

    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn subscribe(&self, from_seq: u64) -> Result<WalStream> {
        let url = format!(
            "{}/v1/{}/subscribe?from_seq={}&subscriber_id=lumen",
            self.base, self.subject, from_seq
        );
        let resp = self.client.get(url).send().await?.error_for_status()?;
        let bytes = resp.bytes_stream();
        // Decode relay's length-prefixed CBOR LogEntry frames, mapping each to a
        // (seq, WalRecord). Buffer partial frames across chunks.
        let stream = futures::stream::unfold(
            (bytes, Vec::<u8>::new(), VecDeque::<(u64, WalRecord)>::new()),
            |(mut bytes, mut buf, mut queue)| async move {
                loop {
                    if let Some(item) = queue.pop_front() {
                        return Some((Ok(item), (bytes, buf, queue)));
                    }
                    match bytes.next().await {
                        Some(Ok(chunk)) => {
                            buf.extend_from_slice(&chunk);
                            let (frames, used): (Vec<LogEntry>, usize) = decode_frames(&buf);
                            if used > 0 {
                                buf.drain(0..used);
                            }
                            for e in frames {
                                if let Ok(rec) = serde_json::from_value::<WalRecord>(e.payload) {
                                    // relay seq is 0-based; WAL seq is 1-based.
                                    queue.push_back((e.seq + 1, rec));
                                }
                            }
                        }
                        // Stream ended or errored: end the WAL stream (the caller
                        // reconnects from its last offset).
                        _ => return None,
                    }
                }
            },
        );
        Ok(Box::pin(stream))
    }

    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn latest_seq(&self) -> Result<u64> {
        // relay exposes no length endpoint; a derived index is rebuildable, so a
        // consumer replays from its requested offset (0 = "from the start").
        Ok(0)
    }
}
// HANDWRITE-END
