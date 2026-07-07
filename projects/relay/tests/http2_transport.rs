// SPEC-MANAGED: projects/relay/tech-design/interfaces/rest/http-2-openapi-transport-client-side-sharding-work-queue-consume.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:7c3c08fd" tracker="pending-tracker" reason="In-process h2c integration tests for the unit-test plan."
//! HTTP/2 (h2c) transport integration tests over a real ephemeral server.
//!
//! Each test starts the app on a loopback port through the shared service
//! shell's serve loop (`service_http::serve` — HTTP/1.1 + h2c on one port,
//! #1205) and drives it with a reqwest client forced to HTTP/2
//! prior-knowledge (h2c), covering the #115 acceptance (a worker leases/acks
//! over h2c) plus the #1205 shell surface: standard probes, drain-aware
//! readiness, per-op metrics, and the shared error envelope.

use std::net::SocketAddr;

use serde_json::json;

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;
use relay::wire::{
    from_cbor, to_cbor, AckResponse, LeaseRequest, LeaseResponse, PublishRequest, CBOR,
};
use relay::{AppendOutcome, DEFAULT_PRIORITY};

async fn start_server() -> (SocketAddr, AppState) {
    let state = AppState::new(RelayServerConfig::ephemeral());
    let app = router(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    // The shared shell's serve loop (HTTP/1.1 + h2c on one port); tests never
    // signal shutdown, so the loop lives for the test process.
    tokio::spawn(service_http::serve(
        listener,
        app,
        std::future::pending::<()>(),
    ));
    (addr, state)
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
    let (addr, _state) = start_server().await;
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
    let (addr, _state) = start_server().await;
    let client = h2c_client();
    let body = to_cbor(&PublishRequest {
        message_id: "m0".into(),
        payload: json!({ "n": 1 }),
        headers: Default::default(),
        not_before: None,
        delay_ms: None,
        priority: DEFAULT_PRIORITY,
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
    let (addr, _state) = start_server().await;
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
    let (addr, _state) = start_server().await;
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
    let (addr, _state) = start_server().await;
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

/// Length-prefix one consume up-frame (the stream's wire framing).
fn consume_frame(v: &serde_json::Value) -> Vec<u8> {
    let body = serde_json::to_vec(v).unwrap();
    let mut buf = (body.len() as u32).to_be_bytes().to_vec();
    buf.extend_from_slice(&body);
    buf
}

/// #1205 AC1: the five standard probe endpoints (shared service shell) all
/// answer on the one serve port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn probe_surface_answers_on_serve_port() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();
    for (path, needle) in [
        ("/healthz", "ok"),
        ("/readyz", "ok"),
        ("/metrics", "relay_publish_requests_total"),
        ("/openapi.json", "/v1/{subject}/publish"),
        ("/docs", "swagger-ui"),
    ] {
        let resp = client.get(url(addr, path)).send().await.unwrap();
        assert_eq!(resp.status(), 200, "GET {path}");
        let body = resp.text().await.unwrap();
        assert!(
            body.contains(needle),
            "GET {path} body must contain {needle:?}"
        );
    }
}

/// #1205 AC1: `/readyz` flips to 503 once drain begins (SIGTERM reaches
/// `start_drain` through `service_http::shutdown_with_drain`).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn readyz_flips_to_503_on_drain() {
    let (addr, state) = start_server().await;
    let client = reqwest::Client::new();
    let ready = client.get(url(addr, "/readyz")).send().await.unwrap();
    assert_eq!(ready.status(), 200);
    state.start_drain();
    let draining = client.get(url(addr, "/readyz")).send().await.unwrap();
    assert_eq!(draining.status(), 503);
    assert_eq!(draining.text().await.unwrap(), "draining");
}

/// #1205 AC2: an HTTP/2 prior-knowledge (h2c, via libs/h2c `h2c_client`)
/// request AND an HTTP/1.1 request both succeed on the same serve port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h2c_and_http11_share_the_serve_port() {
    let (addr, _state) = start_server().await;

    let h2 = h2c::h2c_client().unwrap();
    let resp = h2.get(url(addr, "/healthz")).send().await.unwrap();
    assert_eq!(resp.version(), reqwest::Version::HTTP_2);
    assert_eq!(resp.status(), 200);
    let out = publish(&h2, addr, "p", "m-h2").await;
    assert_eq!(out["seq"], 0);

    let h1 = reqwest::Client::new();
    let resp = h1.get(url(addr, "/healthz")).send().await.unwrap();
    assert_eq!(resp.version(), reqwest::Version::HTTP_11);
    assert_eq!(resp.status(), 200);
    let resp = h1
        .post(url(addr, "/v1/p/publish"))
        .json(&json!({ "message_id": "m-h1", "payload": { "n": 1 } }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.version(), reqwest::Version::HTTP_11);
}

/// #1205 AC3: `/metrics` reports relay's per-op request counters (Prometheus
/// text) after traffic.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn metrics_report_relay_request_counters_after_traffic() {
    let (addr, _state) = start_server().await;
    let client = h2c_client();
    publish(&client, addr, "m", "m0").await;
    publish(&client, addr, "m", "m1").await;
    let body = client
        .get(url(addr, "/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(body.contains("# TYPE relay_publish_requests_total counter"));
    assert!(body.contains("relay_publish_requests_total 2"));
    assert!(body.contains("relay_publish_latency_ms_count 2"));
    assert!(body.contains("relay_lease_requests_total 0"));
}

/// #1205 AC4: error paths render the shared `{error, message}` envelope —
/// an undecodable publish body and a consume stream that opens with a
/// non-Subscribe frame.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn errors_render_the_shared_envelope() {
    let (addr, _state) = start_server().await;
    let client = h2c_client();

    let resp = client
        .post(url(addr, "/v1/s/publish"))
        .header("content-type", "application/json")
        .body("{not json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert!(!body["message"].as_str().unwrap().is_empty());

    let resp = client
        .post(url(addr, "/v1/s/consume"))
        .body(consume_frame(
            &json!({ "type": "ack", "lease_id": "L", "epoch": 1 }),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert!(!body["message"].as_str().unwrap().is_empty());
}

// HANDWRITE-END
