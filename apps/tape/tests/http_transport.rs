//! HTTP transport integration tests over a real ephemeral server.
//!
//! Each test starts the app on a loopback port through the shared service
//! shell's serve loop (`service_http::serve` — HTTP/1.1 + h2c on one port)
//! and drives it with a reqwest client, covering the #1325 shell surface:
//! standard probes, drain-aware readiness, per-op metrics, the shared error
//! envelope, and the unchanged append/replay/checkpoint domain round trip.

use std::net::SocketAddr;

use serde_json::json;
use tape::server::{router, AppState};
use tape::TapeJournal;

async fn start_server() -> (SocketAddr, AppState) {
    let state = AppState::new(TapeJournal::default(), None);
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

fn url(addr: SocketAddr, path: &str) -> String {
    format!("http://{addr}{path}")
}

async fn append(
    client: &reqwest::Client,
    addr: SocketAddr,
    topic: &str,
    n: i64,
) -> serde_json::Value {
    client
        .post(url(addr, &format!("/topics/{topic}/append")))
        .json(&json!({ "payload": { "n": n } }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

/// R4: the five standard probe endpoints (shared service shell) all answer
/// on the one serve port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn probe_surface_answers_on_serve_port() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();
    for (path, needle) in [
        ("/healthz", "ok"),
        ("/readyz", "ok"),
        ("/metrics", "tape_append_requests_total"),
        ("/openapi.json", "/topics/{topic}/append"),
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

/// R3: `/readyz` flips to 503 once drain begins (SIGTERM reaches
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

/// R2: an HTTP/2 prior-knowledge (h2c, via libs/h2c `h2c_client`) request AND
/// an HTTP/1.1 request both succeed on the same serve port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h2c_and_http11_share_the_serve_port() {
    let (addr, _state) = start_server().await;

    let h2 = h2c::h2c_client().unwrap();
    let resp = h2.get(url(addr, "/healthz")).send().await.unwrap();
    assert_eq!(resp.version(), reqwest::Version::HTTP_2);
    assert_eq!(resp.status(), 200);
    let out = append(&h2, addr, "p", 1).await;
    assert_eq!(out["offset"], 0);

    let h1 = reqwest::Client::new();
    let resp = h1.get(url(addr, "/healthz")).send().await.unwrap();
    assert_eq!(resp.version(), reqwest::Version::HTTP_11);
    assert_eq!(resp.status(), 200);
    let resp = h1
        .post(url(addr, "/topics/p/append"))
        .json(&json!({ "payload": { "n": 2 } }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.version(), reqwest::Version::HTTP_11);
}

/// R5: `/metrics` reports tape's per-op request counters (Prometheus text)
/// after traffic.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn metrics_report_tape_request_counters_after_traffic() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();
    append(&client, addr, "m", 1).await;
    append(&client, addr, "m", 2).await;
    let body = client
        .get(url(addr, "/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(body.contains("# TYPE tape_append_requests_total counter"));
    assert!(body.contains("tape_append_requests_total 2"));
    assert!(body.contains("tape_append_latency_ms_count 2"));
    assert!(body.contains("tape_replay_requests_total 0"));
}

/// R6: error paths render the shared `{error, message}` envelope — an
/// undecodable append body and a checkpoint write beyond the topic's end
/// offset.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn errors_render_the_shared_envelope() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(url(addr, "/topics/s/append"))
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
        .put(url(addr, "/topics/s/consumers/c1/checkpoint"))
        .json(&json!({ "offset": 5 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "conflict");
    assert!(!body["message"].as_str().unwrap().is_empty());
}

/// R7: append/replay/checkpoint wrap the unchanged `TapeJournal` API end to
/// end over HTTP, with no new domain behavior.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn append_replay_checkpoint_round_trip_over_http() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();

    let first = append(&client, addr, "orders", 1).await;
    let second = append(&client, addr, "orders", 2).await;
    assert_eq!(first["offset"], 0);
    assert_eq!(second["offset"], 1);

    let replayed: serde_json::Value = client
        .get(url(addr, "/topics/orders/replay?from_offset=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let events = replayed["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["payload"]["n"], 2);

    let checkpoint: serde_json::Value = client
        .put(url(addr, "/topics/orders/consumers/worker-a/checkpoint"))
        .json(&json!({ "offset": 1 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(checkpoint["offset"], 1);

    let fetched: serde_json::Value = client
        .get(url(addr, "/topics/orders/consumers/worker-a/checkpoint"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched["checkpoint"]["offset"], 1);
}
