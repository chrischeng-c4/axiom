//! Streaming work-queue consume (#447/#448) — ONE bidi h2 stream replacing the
//! polling lease/ack/heartbeat dance.
//!
//! The consumer opens the stream and sends `Subscribe{prefetch}`; relay pushes
//! leased entries down (up to the credit window); the consumer sends `Ack`/`Nack`
//! up; each ack frees a credit. Connection drop → the consumer's in-flight leases
//! expire and redeliver via the engine's existing TTL — so exclusivity and
//! redelivery are unchanged; this is purely a streaming wrapper over the
//! work-queue engine. Frames are length-prefixed JSON.

use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::{Body, BodyDataStream, Bytes};
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::engine::Relay;
use crate::server::AppState;

static CONSUMER_SEQ: AtomicU64 = AtomicU64::new(0);

/// Up-frames the consumer sends.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ConsumeUp {
    Subscribe { prefetch: u32 },
    Ack { lease_id: String, epoch: u64 },
    Nack { lease_id: String },
}

/// Down-frame: one leased entry.
#[derive(Debug, Serialize)]
struct LeasedEntry {
    lease_id: String,
    epoch: u64,
    message_id: String,
    payload: serde_json::Value,
}

fn encode_frame<T: Serialize>(v: &T) -> Vec<u8> {
    let body = serde_json::to_vec(v).unwrap_or_default();
    let mut buf = Vec::with_capacity(4 + body.len());
    buf.extend_from_slice(&(body.len() as u32).to_be_bytes());
    buf.extend_from_slice(&body);
    buf
}

#[derive(Default)]
struct Frames {
    buf: Vec<u8>,
}
impl Frames {
    fn push(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }
    fn next(&mut self) -> Option<Vec<u8>> {
        if self.buf.len() < 4 {
            return None;
        }
        let n = u32::from_be_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]]) as usize;
        if self.buf.len() < 4 + n {
            return None;
        }
        let f = self.buf[4..4 + n].to_vec();
        self.buf.drain(..4 + n);
        Some(f)
    }
}

async fn read_up(stream: &mut BodyDataStream, dec: &mut Frames) -> Option<ConsumeUp> {
    loop {
        if let Some(raw) = dec.next() {
            return serde_json::from_slice(&raw).ok();
        }
        match stream.next().await {
            Some(Ok(chunk)) => dec.push(&chunk),
            _ => return None,
        }
    }
}

/// `POST /v1/{subject}/consume` — bidi streaming work-queue consume: the consumer
/// streams `Subscribe`/`Ack`/`Nack` frames up while relay streams leased entries
/// down (length-prefixed JSON) within a `prefetch` credit window. The primary
/// work-queue consume path (#447/#448), superseding the polling lease/ack/
/// heartbeat routes.
#[utoipa::path(
    post,
    path = "/v1/{subject}/consume",
    params(("subject" = String, Path, description = "Target subject")),
    responses((status = 200, description = "A length-prefixed JSON frame stream of leased entries; the request body streams Subscribe/Ack/Nack frames"))
)]
pub async fn consume(State(st): State<AppState>, Path(subject): Path<String>, req: Body) -> Response {
    let consumer_id = format!("consume-{}", CONSUMER_SEQ.fetch_add(1, Ordering::Relaxed));
    let (tx, rx) = mpsc::channel::<Vec<u8>>(64);
    tokio::spawn(drive(st.relay_handle(), subject, consumer_id, req.into_data_stream(), tx));
    let stream = futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|f| (Ok::<Bytes, Infallible>(Bytes::from(f)), rx))
    });
    (StatusCode::OK, [(header::CONTENT_TYPE, "application/octet-stream")], Body::from_stream(stream))
        .into_response()
}

/// Drive one consume session: Subscribe, then keep ≤ prefetch entries in flight
/// (lease + push), freeing a credit per Ack/Nack; poll the queue while idle.
async fn drive(
    relay: std::sync::Arc<Relay>,
    subject: String,
    consumer_id: String,
    mut up: BodyDataStream,
    tx: mpsc::Sender<Vec<u8>>,
) {
    let mut dec = Frames::default();
    let prefetch = match read_up(&mut up, &mut dec).await {
        Some(ConsumeUp::Subscribe { prefetch }) => prefetch.max(1),
        _ => return, // first frame must be Subscribe
    };
    let mut inflight: u32 = 0;
    // Wake-based push (#465): re-lease the instant a publish or release touches
    // this subject, instead of polling the queue every 50ms while idle. The
    // watch channel tracks the latest revision, so a wake signaled between our
    // drain and the `.changed()` await is not lost.
    let mut wake = relay.subscribe_wake(&subject);
    loop {
        while inflight < prefetch {
            match relay.lease(&subject, &consumer_id, Utc::now()).unwrap_or(None) {
                Some(l) => {
                    let (message_id, payload) = relay
                        .entry(&l.subject, l.shard, l.seq)
                        .ok()
                        .flatten()
                        .map(|e| (e.message_id, e.payload))
                        .unwrap_or_default();
                    let frame = LeasedEntry { lease_id: l.lease_id, epoch: l.epoch, message_id, payload };
                    if tx.send(encode_frame(&frame)).await.is_err() {
                        return;
                    }
                    inflight += 1;
                }
                None => break, // queue empty
            }
        }
        tokio::select! {
            up_frame = read_up(&mut up, &mut dec) => match up_frame {
                Some(ConsumeUp::Ack { lease_id, epoch }) => {
                    let _ = relay.ack(&subject, &lease_id, Some(epoch));
                    inflight = inflight.saturating_sub(1);
                }
                Some(ConsumeUp::Nack { lease_id }) => {
                    // Fast release (#465): reset this lease for immediate
                    // redelivery rather than waiting out the lease TTL.
                    let _ = relay.release(&subject, &lease_id);
                    inflight = inflight.saturating_sub(1);
                }
                Some(ConsumeUp::Subscribe { .. }) => {}
                None => return, // disconnect → in-flight leases expire (TTL redelivery)
            },
            // A publish/release on this subject (or a reconciler reclaim) woke us;
            // loop back and lease whatever is now ready.
            _ = wake.changed() => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_codec_and_up_parse() {
        // length-prefixed framing tolerates split reads
        let a = encode_frame(&serde_json::json!({"type": "ack", "lease_id": "L", "epoch": 3}));
        let mut d = Frames::default();
        d.push(&a[..2]);
        assert!(d.next().is_none());
        d.push(&a[2..]);
        match serde_json::from_slice::<ConsumeUp>(&d.next().unwrap()).unwrap() {
            ConsumeUp::Ack { lease_id, epoch } => assert_eq!((lease_id.as_str(), epoch), ("L", 3)),
            _ => panic!("expected Ack"),
        }
        assert!(matches!(
            serde_json::from_slice::<ConsumeUp>(br#"{"type":"subscribe","prefetch":4}"#).unwrap(),
            ConsumeUp::Subscribe { prefetch: 4 }
        ));
        assert!(matches!(
            serde_json::from_slice::<ConsumeUp>(br#"{"type":"nack","lease_id":"x"}"#).unwrap(),
            ConsumeUp::Nack { .. }
        ));
        let e = LeasedEntry {
            lease_id: "L".into(),
            epoch: 1,
            message_id: "m".into(),
            payload: serde_json::json!({"k": 1}),
        };
        assert!(encode_frame(&e).len() > 4);
    }
}
