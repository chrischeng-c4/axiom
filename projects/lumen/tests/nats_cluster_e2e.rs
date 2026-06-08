//! The full-stack proof: two independent lumen serving nodes backed by
//! the SAME NATS write log. Write through node A's HTTP API; read it
//! back through node B's HTTP API. This exercises the entire data plane
//! end to end — write handler → coordinator → NATS publish → both nodes'
//! apply loops → B's local index → B's read handler.
//!
//! Run a JetStream server first: `nats-server -js` (or point
//! `LUMEN_TEST_NATS_URL` at one). Skips if unavailable.

use std::sync::Arc;
use std::time::Duration;

use axum_test::TestServer;
use serde_json::{json, Value};

use lumen::api::{router, AppState};
use lumen::auth::AuthConfig;
use lumen::coordinator::WriteCoordinator;
use lumen::storage::Engine;
use lumen::wal::SharedWal;
use lumen::wal_nats::NatsWal;

fn nats_url() -> String {
    std::env::var("LUMEN_TEST_NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".into())
}

/// Probe JetStream + reset the stream; `None` → skip.
async fn reset() -> Option<()> {
    let client = async_nats::connect(&nats_url()).await.ok()?;
    let js = async_nats::jetstream::new(client);
    let stream = js
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "lumen_wal".into(),
            subjects: vec!["lumen.wal".into()],
            ..Default::default()
        })
        .await
        .ok()?;
    let _ = stream.purge().await;
    Some(())
}

/// Build a serving node (its own Engine + apply loop) over a shared WAL.
async fn node(wal: SharedWal) -> TestServer {
    let engine = Arc::new(Engine::new());
    let writer = WriteCoordinator::start(wal, engine.clone());
    let state = AppState::with_components(engine, Arc::new(AuthConfig::open()), writer);
    TestServer::new(router(state)).expect("test server")
}

async fn search_total(s: &TestServer, field: &str, value: &str) -> u64 {
    let resp = s
        .post("/collections/users/search")
        .json(&json!({
            "query": { "term": { "field": field, "value": value } },
            "limit": 10
        }))
        .await;
    let body: Value = resp.json();
    body["total"].as_u64().unwrap_or(u64::MAX)
}

async fn wait_total(s: &TestServer, field: &str, value: &str, want: u64) -> bool {
    for _ in 0..100 {
        if search_total(s, field, value).await == want {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}

#[tokio::test]
async fn write_on_node_a_is_readable_on_node_b() {
    if reset().await.is_none() {
        eprintln!("skipping: no JetStream NATS at {}", nats_url());
        return;
    }

    // Two nodes, one shared NATS log.
    let wal_a: SharedWal = Arc::new(NatsWal::connect(&nats_url()).await.unwrap());
    let wal_b: SharedWal = Arc::new(NatsWal::connect(&nats_url()).await.unwrap());
    let node_a = node(wal_a).await;
    let node_b = node(wal_b).await;

    // Create schema + index via node A's HTTP API.
    node_a
        .put("/collections/users")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    node_a
        .post("/collections/users/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "email", "value": "a@x.com" },
            { "external_id": "u2", "field": "email", "value": "b@x.com" },
            { "external_id": "u3", "field": "email", "value": "a@x.com" }
        ]}))
        .await
        .assert_status_ok();

    // Node A sees its own write immediately (read-your-write via the
    // coordinator awaiting local apply).
    assert_eq!(search_total(&node_a, "email", "a@x.com").await, 2);

    // Node B — a DIFFERENT process-equivalent with its own engine —
    // converges by tailing the same NATS log. THIS is the payoff.
    assert!(
        wait_total(&node_b, "email", "a@x.com", 2).await,
        "node B did not converge from the shared NATS log"
    );
    assert_eq!(search_total(&node_b, "email", "b@x.com").await, 1);
}

#[tokio::test]
async fn delete_on_one_node_propagates_to_the_other() {
    if reset().await.is_none() {
        eprintln!("skipping: no JetStream NATS at {}", nats_url());
        return;
    }
    let node_a = node(Arc::new(NatsWal::connect(&nats_url()).await.unwrap())).await;
    let node_b = node(Arc::new(NatsWal::connect(&nats_url()).await.unwrap())).await;

    node_a
        .put("/collections/users")
        .json(&json!({ "fields": { "email": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    node_a
        .post("/collections/users/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "email", "value": "a@x.com" }
        ]}))
        .await
        .assert_status_ok();
    assert!(
        wait_total(&node_b, "email", "a@x.com", 1).await,
        "B never saw the index"
    );

    // Delete on B, observe on A.
    node_b
        .delete("/collections/users/index/u1")
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);
    assert!(
        wait_total(&node_a, "email", "a@x.com", 0).await,
        "delete on B did not propagate to A"
    );
}
