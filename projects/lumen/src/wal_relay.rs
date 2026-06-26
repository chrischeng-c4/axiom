// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! #124 — tail **relay**'s broadcast as the WAL backend (behind the `relay-wal`
//! feature), replacing the legacy NATS deployment backend.
//!
//! lumen is a log-tailing derived index (see HA.md): a write is published to the
//! broker and every pod folds the ordered log into its own index. This backend
//! makes that broker `relay` (itself HA via `libs/raft-core`): `publish` POSTs to
//! relay, `subscribe` tails relay's broadcast — the loop `relay::spawn_follower`
//! proved, mapped onto [`WalRecord`].
//!
//! relay is **h2c (cleartext)**, so the client links no TLS/openssl; the backend
//! is off by default, keeping the serving binary's clean cross-compile.

use std::collections::VecDeque;
use std::env;
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use base64::Engine as _;
use futures::StreamExt;
use relay::wire::{decode_frames, from_cbor, to_cbor, PublishRequest, CBOR};
use relay::{AppendOutcome, LogEntry};
use serde_json::Value;

use crate::wal::{WalLog, WalRecord, WalStream};

/// A `WalLog` backed by a relay broker's broadcast log.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
pub struct RelayWal {
    client: reqwest::Client,
    base: String,
    subject: String,
    subscriber_id: String,
    publisher_id: String,
    counter: AtomicU64,
}

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);
const WAL_BINARY_FORMAT: &str = "lumen-wal-bytes-v1";
const WAL_BINARY_PREFIX: &str = "lumen-wal-bytes-v1:";
const WAL_BINARY_FORMAT_KEY: &str = "lumen_wal_format";
const WAL_BINARY_BYTES_KEY: &str = "lumen_wal_bytes_b64";

fn next_client_id() -> u64 {
    NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}

fn default_subscriber_id() -> String {
    env::var("POD_NAME")
        .or_else(|_| env::var("HOSTNAME"))
        .ok()
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("lumen-{}", next_client_id()))
}

fn default_publisher_id(subscriber_id: &str) -> String {
    format!(
        "{subscriber_id}-pid{}-client{}",
        process::id(),
        next_client_id()
    )
}

fn wal_payload(record: &WalRecord) -> Result<Value> {
    let bytes = record.encode()?;
    Ok(Value::String(format!(
        "{WAL_BINARY_PREFIX}{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )))
}

fn decode_wal_payload(seq: u64, payload: Value) -> Result<WalRecord> {
    if let Some(encoded) = payload
        .as_str()
        .and_then(|raw| raw.strip_prefix(WAL_BINARY_PREFIX))
    {
        return decode_binary_wal_payload(seq, encoded);
    }
    if let Value::Object(map) = &payload {
        let is_binary = map
            .get(WAL_BINARY_FORMAT_KEY)
            .and_then(Value::as_str)
            .is_some_and(|format| format == WAL_BINARY_FORMAT);
        if is_binary {
            let encoded = map
                .get(WAL_BINARY_BYTES_KEY)
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("relay WAL payload at seq {seq} is missing binary bytes"))?;
            return decode_binary_wal_payload(seq, encoded);
        }
    }
    serde_json::from_value::<WalRecord>(payload)
        .map_err(|err| anyhow!("relay WAL payload at seq {seq} is not a WalRecord: {err}"))
}

fn decode_binary_wal_payload(seq: u64, encoded: &str) -> Result<WalRecord> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|err| anyhow!("relay WAL payload at seq {seq} has invalid base64: {err}"))?;
    WalRecord::decode(&bytes)
        .map_err(|err| anyhow!("relay WAL binary payload at seq {seq} is invalid: {err}"))
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
impl RelayWal {
    /// Connect to `base_url` (e.g. `http://relay:8080`) for `subject`.
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    pub fn new(base_url: impl Into<String>, subject: impl Into<String>) -> Result<Self> {
        let subscriber_id = default_subscriber_id();
        Self::new_with_subscriber_id(base_url, subject, subscriber_id)
    }

    /// Connect with an explicit broadcast subscriber id.
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    pub fn new_with_subscriber_id(
        base_url: impl Into<String>,
        subject: impl Into<String>,
        subscriber_id: impl Into<String>,
    ) -> Result<Self> {
        let subscriber_id = subscriber_id.into();
        let publisher_id = default_publisher_id(&subscriber_id);
        Self::new_with_ids(base_url, subject, subscriber_id, publisher_id)
    }

    /// Connect with explicit subscriber and publisher identities. Tests use this
    /// to prove subscriber fan-out and publisher restart idempotency.
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    pub fn new_with_ids(
        base_url: impl Into<String>,
        subject: impl Into<String>,
        subscriber_id: impl Into<String>,
        publisher_id: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            client: h2c::h2c_client()?,
            base: base_url.into().trim_end_matches('/').to_string(),
            subject: subject.into(),
            subscriber_id: subscriber_id.into(),
            publisher_id: publisher_id.into(),
            counter: AtomicU64::new(0),
        })
    }
}

#[async_trait]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
impl WalLog for RelayWal {
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn publish(&self, record: WalRecord) -> Result<u64> {
        let payload = wal_payload(&record)?;
        let message_id = format!(
            "{}-{}",
            self.publisher_id,
            self.counter.fetch_add(1, Ordering::Relaxed)
        );
        let body = PublishRequest {
            message_id,
            payload,
            headers: Default::default(),
        };
        let resp = self
            .client
            .post(format!("{}/v1/{}/publish", self.base, self.subject))
            .header(reqwest::header::CONTENT_TYPE, CBOR)
            .header(reqwest::header::ACCEPT, CBOR)
            .body(to_cbor(&body))
            .send()
            .await?
            .error_for_status()?;
        let bytes = resp.bytes().await?;
        let out: AppendOutcome =
            from_cbor(bytes.as_ref()).map_err(|err| anyhow!("relay publish response: {err}"))?;
        let seq = out.seq;
        // relay seq is 0-based; the lumen WAL seq is 1-based.
        Ok(seq + 1)
    }

    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn subscribe(&self, from_seq: u64) -> Result<WalStream> {
        let url = format!("{}/v1/{}/subscribe", self.base, self.subject);
        let resp = self
            .client
            .get(url)
            .query(&[
                ("from_seq", from_seq.to_string()),
                ("subscriber_id", self.subscriber_id.clone()),
            ])
            .send()
            .await?
            .error_for_status()?;
        let bytes = resp.bytes_stream();
        // Decode relay's length-prefixed CBOR LogEntry frames, mapping each to a
        // (seq, WalRecord). Buffer partial frames across chunks.
        let stream = futures::stream::unfold(
            (
                bytes,
                Vec::<u8>::new(),
                VecDeque::<Result<(u64, WalRecord)>>::new(),
            ),
            |(mut bytes, mut buf, mut queue)| async move {
                loop {
                    if let Some(item) = queue.pop_front() {
                        return Some((item, (bytes, buf, queue)));
                    }
                    match bytes.next().await {
                        Some(Ok(chunk)) => {
                            buf.extend_from_slice(&chunk);
                            let (frames, used): (Vec<LogEntry>, usize) = decode_frames(&buf);
                            if used > 0 {
                                buf.drain(0..used);
                            }
                            for e in frames {
                                match decode_wal_payload(e.seq, e.payload) {
                                    Ok(rec) => {
                                        // relay seq is 0-based; WAL seq is 1-based.
                                        queue.push_back(Ok((e.seq + 1, rec)));
                                    }
                                    Err(err) => queue.push_back(Err(err)),
                                }
                            }
                        }
                        Some(Err(err)) => {
                            return Some((
                                Err(anyhow!("relay subscribe stream failed: {err}")),
                                (bytes, buf, queue),
                            ));
                        }
                        // Stream ended: the caller reconnects from its last offset.
                        None => return None,
                    }
                }
            },
        );
        Ok(Box::pin(stream))
    }

    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-wal_relay-rs.md#source
    async fn latest_seq(&self) -> Result<u64> {
        let resp = self
            .client
            .get(format!("{}/v1/{}/len", self.base, self.subject))
            .send()
            .await?
            .error_for_status()?;
        let out: serde_json::Value = resp.json().await?;
        out.get("latest_seq")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("relay len: no latest_seq in response"))
    }
}
// CODEGEN-END
