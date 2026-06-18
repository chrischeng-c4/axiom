// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#schema
// HANDWRITE-BEGIN gap="missing-generator:schema:a9efe379" tracker="pending-tracker" reason="Transport DTOs and length-prefixed CBOR framing."
//! HTTP/2 transport wire types and framing.
//!
//! JSON shapes are the OpenAPI contract; the hot lease/ack path and the
//! broadcast stream use the same shapes as length-prefixed CBOR. A frame is a
//! big-endian `u32` byte length followed by that many CBOR bytes.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::types::{Lease, Payload, Seq};

/// Publish one message to the path's subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    /// Caller-supplied idempotency key; dedupe is on this id.
    pub message_id: String,
    /// Opaque message body, stored verbatim.
    pub payload: Payload,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

/// Lease the next eligible entry to a competing consumer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseRequest {
    pub consumer_id: String,
}

/// A granted lease, or `null` when nothing is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseResponse {
    pub lease: Option<Lease>,
}

/// Acknowledge a lease. The optional `epoch` fences a stale worker: when given
/// it must match the live lease or the ack is a no-op.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckRequest {
    pub lease_id: String,
    #[serde(default)]
    pub epoch: Option<u64>,
}

/// Extend a held lease; proves the worker is alive (work-queue API #113).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub lease_id: String,
    pub epoch: u64,
}

/// Heartbeat result: whether the lease was extended, and the new expiry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub extended: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Ack result plus the resulting committed offset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckResponse {
    pub acked: bool,
    pub committed_seq: Option<Seq>,
}

/// Broadcast tail query; delivery starts at `from_seq`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeQuery {
    pub from_seq: Seq,
    #[serde(default)]
    pub subscriber_id: Option<String>,
}

/// Content type for the CBOR fast path / stream.
pub const CBOR: &str = "application/cbor";
/// Content type for the broadcast frame stream.
pub const CBOR_SEQ: &str = "application/cbor-seq";

/// Serialize `value` to CBOR bytes.
pub fn to_cbor<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    ciborium::into_writer(value, &mut buf).expect("CBOR serialization of an owned value");
    buf
}

/// Deserialize CBOR `bytes` into `T`.
pub fn from_cbor<T: DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, ciborium::de::Error<std::io::Error>> {
    ciborium::from_reader(bytes)
}

/// Encode one value as a length-prefixed CBOR frame: `u32` BE length + CBOR.
pub fn encode_frame<T: Serialize>(value: &T) -> Vec<u8> {
    let body = to_cbor(value);
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
    frame.extend_from_slice(&body);
    frame
}

/// Decode as many whole length-prefixed CBOR frames as `buf` contains.
///
/// Returns the decoded values and the number of bytes consumed; a trailing
/// partial frame is left unconsumed for the caller to complete.
pub fn decode_frames<T: DeserializeOwned>(buf: &[u8]) -> (Vec<T>, usize) {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos + 4 <= buf.len() {
        let len = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]) as usize;
        if pos + 4 + len > buf.len() {
            break;
        }
        if let Ok(v) = from_cbor::<T>(&buf[pos + 4..pos + 4 + len]) {
            out.push(v);
        }
        pos += 4 + len;
    }
    (out, pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cbor_round_trips() {
        let req = LeaseRequest {
            consumer_id: "c1".into(),
        };
        let bytes = to_cbor(&req);
        let back: LeaseRequest = from_cbor(&bytes).unwrap();
        assert_eq!(back.consumer_id, "c1");
    }

    #[test]
    fn frames_round_trip() {
        let a = AckResponse {
            acked: true,
            committed_seq: Some(0),
        };
        let b = AckResponse {
            acked: false,
            committed_seq: None,
        };
        let mut buf = encode_frame(&a);
        buf.extend(encode_frame(&b));
        let (vals, consumed): (Vec<AckResponse>, usize) = decode_frames(&buf);
        assert_eq!(consumed, buf.len());
        assert_eq!(vals.len(), 2);
        assert!(vals[0].acked && !vals[1].acked);
    }
}
// HANDWRITE-END
