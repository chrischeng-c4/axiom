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
    let state = AppState::new(TapeJournal::default(), None, 8 * 1024 * 1024);
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

/// #2490: every response carries a `Server-Timing: app;dur=<ms>` baseline
/// from the shared `service_http::server_timing_middleware` layer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn responses_carry_server_timing_header() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();
    let resp = client.get(url(addr, "/healthz")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let header = resp
        .headers()
        .get("server-timing")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(
        header.starts_with("app;dur="),
        "Server-Timing header must start with app;dur=, got {header:?}"
    );
    let digit = header
        .strip_prefix("app;dur=")
        .and_then(|rest| rest.chars().next());
    assert!(
        digit.is_some_and(|c| c.is_ascii_digit()),
        "app;dur= must be followed by a digit, got {header:?}"
    );
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

/// R2: an HTTP/2 prior-knowledge (h2c, via libs/transport-h2c `h2c_client`) request AND
/// an HTTP/1.1 request both succeed on the same serve port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn h2c_and_http11_share_the_serve_port() {
    let (addr, _state) = start_server().await;

    let h2 = transport_h2c::h2c_client().unwrap();
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

/// The bulk replay surface stays a read-only replay operation while carrying
/// the same offsets and payloads over compact frames on the real h2c server.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replay_stream_is_validated_h2c_and_counted_as_replay() {
    let (addr, _state) = start_server().await;
    let client = transport_h2c::h2c_client().unwrap();
    for n in 0..3 {
        append(&client, addr, "bulk", n).await;
    }

    let response = client
        .get(url(
            addr,
            "/topics/bulk/replay/stream?from_offset=1&limit=2",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.version(), reqwest::Version::HTTP_2);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some(tape::replay_wire::CONTENT_TYPE)
    );
    let stats = tape::replay_wire::inspect(&response.bytes().await.unwrap()).unwrap();
    assert_eq!(stats.events, 2);
    assert_eq!(stats.first_offset, Some(1));
    assert_eq!(stats.next_offset, Some(3));
    assert!(stats.payload_bytes > 0);

    let metrics = client
        .get(url(addr, "/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(metrics.contains("tape_replay_requests_total 1"));
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

/// #2484: an append body over the shared data-plane body cap is rejected
/// with 413 rather than buffered/accepted, guarding against unbounded
/// request bodies on the data plane (probes stay exempt/unbounded).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn oversized_append_body_is_rejected_with_413() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();

    // One byte over the 8 MiB data-plane cap (`DEFAULT_BODY_LIMIT_BYTES` in
    // `src/server.rs`).
    let oversized_payload = "a".repeat(8 * 1024 * 1024 + 1);
    let result = client
        .post(url(addr, "/topics/orders/append"))
        .header("content-type", "application/json")
        .body(format!(r#"{{"payload":"{oversized_payload}"}}"#))
        .send()
        .await;

    // The Content-Length short-circuit is the point of the layer: the server
    // answers and closes without reading 8 MiB it has already decided to
    // refuse. The client is still uploading when that happens, so it races
    // between reading the 413 and having its own write fail with
    // ECONNRESET — both outcomes are the same refusal, and asserting only on
    // the status code makes this test flaky (~1 run in 5).
    match result {
        Ok(resp) => assert_eq!(resp.status(), 413, "oversized append must be refused"),
        Err(err) => assert!(
            err.is_request(),
            "expected a transport-level refusal from the early close, got: {err}"
        ),
    }

    // Race-free invariant, and the one that actually matters: whichever way
    // the client observed the refusal, the oversized body never reached the
    // journal. A regression that buffered and accepted it would show up here
    // even if the status assertion above were satisfied.
    let replay: serde_json::Value = client
        .get(url(addr, "/topics/orders/replay"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        replay["events"].as_array().map(Vec::len),
        Some(0),
        "the refused body must not be appended: {replay}"
    );
}

/// #2485: `/metrics` exposes `tape_topic_latest_offset` and
/// `tape_subscription_lag` gauges computed at scrape time, with topic and
/// subscription label escaping. Lag reflects the gap between the topic's end
/// offset and the subscription's checkpoint.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn metrics_expose_topic_and_subscription_lag_gauges() {
    let (addr, _state) = start_server().await;
    let client = reqwest::Client::new();

    // Append 5 events to the topic.
    for n in 0..5 {
        append(&client, addr, "test_topic", n).await;
    }

    // Create a subscription and pull 2 events (cursor at 2).
    client
        .post(url(addr, "/topics/test_topic/subscriptions"))
        .json(&json!({ "name": "test_sub" }))
        .send()
        .await
        .unwrap();

    let pull: serde_json::Value = client
        .post(url(addr, "/topics/test_topic/subscriptions/test_sub/pull"))
        .json(&json!({ "limit": 2 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(pull["events"].as_array().unwrap().len(), 2);

    // Ack to offset 2 (exclusive), so checkpoint is at 2.
    let ack: serde_json::Value = client
        .post(url(addr, "/topics/test_topic/subscriptions/test_sub/ack"))
        .json(&json!({ "offset": 2 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ack["offset"], 2);

    // Scrape metrics: lag should be 5 - 2 = 3.
    let metrics = client
        .get(url(addr, "/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        metrics.contains("# TYPE tape_topic_latest_offset gauge"),
        "metrics must include tape_topic_latest_offset TYPE line"
    );
    assert!(
        metrics.contains("tape_topic_latest_offset{topic=\"test_topic\"} 5"),
        "metrics must expose the latest offset for test_topic as 5"
    );
    assert!(
        metrics.contains("# TYPE tape_subscription_lag gauge"),
        "metrics must include tape_subscription_lag TYPE line"
    );
    assert!(
        metrics.contains("tape_subscription_lag{subscription=\"test_sub\",topic=\"test_topic\"} 3"),
        "metrics must expose lag of 3 for test_sub on test_topic"
    );

    // Ack more events: pull 3 more (offsets 2-4, next_offset 5), ack to 5.
    let pull2: serde_json::Value = client
        .post(url(addr, "/topics/test_topic/subscriptions/test_sub/pull"))
        .json(&json!({ "limit": 10 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(pull2["events"].as_array().unwrap().len(), 3);

    client
        .post(url(addr, "/topics/test_topic/subscriptions/test_sub/ack"))
        .json(&json!({ "offset": 5 }))
        .send()
        .await
        .unwrap();

    // Scrape again: lag should be 0.
    let metrics2 = client
        .get(url(addr, "/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        metrics2
            .contains("tape_subscription_lag{subscription=\"test_sub\",topic=\"test_topic\"} 0"),
        "metrics must expose lag of 0 after full ack"
    );
}
