// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-streaming-subscrib.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:7c3c08fd" tracker="pending-tracker" reason="In-process h2c integration tests for the unit-test plan."
//! HTTP/2 (h2c) transport integration tests over a real ephemeral server.
//!
//! Each test starts the axum app on a loopback port and drives it with a
//! reqwest client forced to HTTP/2 prior-knowledge (h2c), covering the #115
//! acceptance: a worker leases/acks over h2c and a subscriber tails a broadcast
//! stream from a seq.

use std::net::SocketAddr;
use std::time::Duration;

use futures::StreamExt;
use serde_json::json;

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;
use relay::wire::{
    from_cbor, to_cbor, AckResponse, LeaseRequest, LeaseResponse, PublishRequest, CBOR,
};
use relay::{AppendOutcome, LogEntry};

async fn start_server() -> SocketAddr {
    let state = AppState::new(RelayServerConfig::ephemeral());
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

fn url(addr: SocketAddr, path: &str) -> String {
    format!("http://{addr}{path}")
}

async fn publish(
    client: &reqwest::Client,
    addr: SocketAddr,
    subject: &str,
    id: &str,
) -> serde_json::Value {
    client
        .post(url(addr, &format!("/v1/{subject}/publish")))
        .json(&json!({ "message_id": id, "payload": { "n": id } }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn publish_is_idempotent_over_h2c() {
    let addr = start_server().await;
    let client = h2c_client();
    let first = publish(&client, addr, "s", "m0").await;
    let second = publish(&client, addr, "s", "m0").await;
    assert_eq!(first["seq"], 0);
    assert_eq!(first["deduped"], false);
    assert_eq!(second["seq"], 0);
    assert_eq!(second["deduped"], true);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn publish_cbor_fast_path_over_h2c() {
    let addr = start_server().await;
    let client = h2c_client();
    let body = to_cbor(&PublishRequest {
        message_id: "m0".into(),
        payload: json!({ "n": 1 }),
        headers: Default::default(),
    });
    let bytes = client
        .post(url(addr, "/v1/s/publish"))
        .header("content-type", CBOR)
        .header("accept", CBOR)
        .body(body)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let outcome: AppendOutcome = from_cbor(bytes.as_ref()).unwrap();
    assert_eq!(outcome.seq, 0);
    assert!(!outcome.deduped);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn worker_leases_and_acks_over_h2c() {
    // #115 acceptance: a worker leases then acks over h2c.
    let addr = start_server().await;
    let client = h2c_client();
    publish(&client, addr, "q", "m0").await;

    let lease: LeaseResponse = client
        .post(url(addr, "/v1/q/lease"))
        .json(&LeaseRequest {
            consumer_id: "c1".into(),
        })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let lease = lease.lease.expect("a lease was granted");
    assert_eq!(lease.seq, 0);
    assert_eq!(lease.attempt, 1);

    let ack: AckResponse = client
        .post(url(addr, "/v1/q/ack"))
        .json(&json!({ "lease_id": lease.lease_id }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(ack.acked);
    assert_eq!(ack.committed_seq, Some(0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lease_is_null_when_empty() {
    let addr = start_server().await;
    let client = h2c_client();
    let resp: LeaseResponse = client
        .post(url(addr, "/v1/empty/lease"))
        .json(&LeaseRequest {
            consumer_id: "c1".into(),
        })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(resp.lease.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn lease_cbor_fast_path() {
    let addr = start_server().await;
    let client = h2c_client();
    publish(&client, addr, "q", "m0").await;

    let body = to_cbor(&LeaseRequest {
        consumer_id: "c1".into(),
    });
    let bytes = client
        .post(url(addr, "/v1/q/lease"))
        .header("content-type", "application/cbor")
        .body(body)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let resp: LeaseResponse = ciborium::from_reader(bytes.as_ref()).unwrap();
    assert_eq!(resp.lease.expect("lease").seq, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscriber_tails_broadcast_from_seq() {
    // #115 acceptance: a subscriber tails a broadcast stream from a seq.
    let addr = start_server().await;
    let client = h2c_client();
    for id in ["e0", "e1", "e2"] {
        publish(&client, addr, "events", id).await;
    }

    let resp = client
        .get(url(addr, "/v1/events/subscribe?from_seq=1"))
        .send()
        .await
        .unwrap();
    let mut stream = resp.bytes_stream();

    let mut buf: Vec<u8> = Vec::new();
    let mut frames: Vec<LogEntry> = Vec::new();
    while frames.len() < 2 {
        let chunk = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("stream did not stall")
            .expect("a chunk")
            .expect("chunk ok");
        buf.extend_from_slice(&chunk);
        let (decoded, consumed) = relay::wire::decode_frames::<LogEntry>(&buf);
        if !decoded.is_empty() {
            frames.extend(decoded);
            buf.drain(0..consumed);
        }
    }
    drop(stream); // disconnect the tail

    let seqs: Vec<u64> = frames.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, vec![1, 2], "tail replays from from_seq in order");
}
// HANDWRITE-END
