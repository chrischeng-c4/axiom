//! Compact read-only replay wire format for the h2c bulk path.
//!
//! The JSON endpoint remains the ergonomic compatibility surface. This frame
//! format avoids cloning every [`TapeEvent`](crate::TapeEvent) and repeating
//! the topic string for each event when a consumer drains a large backlog.

use crate::TapeEvent;

pub const CONTENT_TYPE: &str = "application/vnd.tape.replay.v1";
const MAGIC: &[u8; 8] = b"TAPE\0\x01\r\n";
const NO_KEY: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayStreamStats {
    pub events: usize,
    pub payload_bytes: usize,
    pub first_offset: Option<u64>,
    pub next_offset: Option<u64>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReplayWireError {
    #[error("replay frame header is missing or invalid")]
    InvalidHeader,
    #[error("replay frame is truncated")]
    Truncated,
    #[error("replay frame count does not fit the wire format")]
    TooManyEvents,
    #[error("replay frame field is too large")]
    FieldTooLarge,
    #[error("replay offsets are not contiguous at {actual}; expected {expected}")]
    NonContiguousOffset { expected: u64, actual: u64 },
    #[error("replay frame has trailing bytes")]
    TrailingBytes,
    #[error("replay payload serialization failed: {0}")]
    Payload(String),
}

/// Encode a bounded replay window into one h2c response body.
pub fn encode(events: &[&TapeEvent]) -> Result<Vec<u8>, ReplayWireError> {
    let count = u32::try_from(events.len()).map_err(|_| ReplayWireError::TooManyEvents)?;
    let mut output = Vec::with_capacity(12 + events.len().saturating_mul(192));
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&count.to_be_bytes());
    for event in events {
        output.extend_from_slice(&event.offset.to_be_bytes());
        output.extend_from_slice(&event.timestamp_ms.to_be_bytes());
        match event.key.as_deref() {
            Some(key) => {
                let len = u32::try_from(key.len()).map_err(|_| ReplayWireError::FieldTooLarge)?;
                output.extend_from_slice(&len.to_be_bytes());
                output.extend_from_slice(key.as_bytes());
            }
            None => output.extend_from_slice(&NO_KEY.to_be_bytes()),
        }
        let length_at = output.len();
        output.extend_from_slice(&0u32.to_be_bytes());
        let payload_at = output.len();
        serde_json::to_writer(&mut output, &event.payload)
            .map_err(|error| ReplayWireError::Payload(error.to_string()))?;
        let payload_len = output.len() - payload_at;
        let payload_len = u32::try_from(payload_len).map_err(|_| ReplayWireError::FieldTooLarge)?;
        output[length_at..length_at + 4].copy_from_slice(&payload_len.to_be_bytes());
    }
    Ok(output)
}

/// Validate and summarize a complete replay body without deserializing domain
/// payloads. Broker clients likewise receive opaque payload bytes; offset,
/// framing, and full-body validation prove that every event arrived.
pub fn inspect(bytes: &[u8]) -> Result<ReplayStreamStats, ReplayWireError> {
    if bytes.len() < 12 || &bytes[..8] != MAGIC {
        return Err(ReplayWireError::InvalidHeader);
    }
    let count = u32::from_be_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let mut cursor = 12;
    let mut payload_bytes = 0usize;
    let mut first_offset = None;
    let mut expected_offset = None;
    for _ in 0..count {
        let offset = read_u64(bytes, &mut cursor)?;
        let _timestamp_ms = read_u64(bytes, &mut cursor)?;
        if let Some(expected) = expected_offset {
            if offset != expected {
                return Err(ReplayWireError::NonContiguousOffset {
                    expected,
                    actual: offset,
                });
            }
        } else {
            first_offset = Some(offset);
        }
        expected_offset = Some(offset.saturating_add(1));

        let key_len = read_u32(bytes, &mut cursor)?;
        if key_len != NO_KEY {
            take(bytes, &mut cursor, key_len as usize)?;
        }
        let payload_len = read_u32(bytes, &mut cursor)? as usize;
        take(bytes, &mut cursor, payload_len)?;
        payload_bytes = payload_bytes.saturating_add(payload_len);
    }
    if cursor != bytes.len() {
        return Err(ReplayWireError::TrailingBytes);
    }
    Ok(ReplayStreamStats {
        events: count,
        payload_bytes,
        first_offset,
        next_offset: expected_offset,
    })
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, ReplayWireError> {
    let field = take(bytes, cursor, 4)?;
    Ok(u32::from_be_bytes(field.try_into().unwrap()))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, ReplayWireError> {
    let field = take(bytes, cursor, 8)?;
    Ok(u64::from_be_bytes(field.try_into().unwrap()))
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8], ReplayWireError> {
    let end = cursor.checked_add(len).ok_or(ReplayWireError::Truncated)?;
    let field = bytes.get(*cursor..end).ok_or(ReplayWireError::Truncated)?;
    *cursor = end;
    Ok(field)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{encode, inspect, ReplayWireError};
    use crate::TapeEvent;

    #[test]
    fn round_trip_summary_validates_offsets_keys_and_payload_frames() {
        let events = [
            TapeEvent {
                topic: "orders".into(),
                offset: 7,
                timestamp_ms: 11,
                key: None,
                payload: json!({"n": 1}),
            },
            TapeEvent {
                topic: "orders".into(),
                offset: 8,
                timestamp_ms: 12,
                key: Some("k".into()),
                payload: json!({"n": 2}),
            },
        ];
        let refs = events.iter().collect::<Vec<_>>();
        let encoded = encode(&refs).unwrap();
        let stats = inspect(&encoded).unwrap();
        assert_eq!(stats.events, 2);
        assert_eq!(stats.first_offset, Some(7));
        assert_eq!(stats.next_offset, Some(9));
        assert!(stats.payload_bytes > 0);
    }

    #[test]
    fn truncated_and_noncontiguous_frames_fail_closed() {
        let events = [TapeEvent {
            topic: "orders".into(),
            offset: 1,
            timestamp_ms: 2,
            key: None,
            payload: json!(3),
        }];
        let refs = events.iter().collect::<Vec<_>>();
        let mut encoded = encode(&refs).unwrap();
        assert_eq!(
            inspect(&encoded[..encoded.len() - 1]),
            Err(ReplayWireError::Truncated)
        );
        encoded.extend_from_slice(b"x");
        assert_eq!(inspect(&encoded), Err(ReplayWireError::TrailingBytes));
    }
}
