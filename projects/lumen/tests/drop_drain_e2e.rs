// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-drop-drain-e2e-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Two-phase drop + drain behavior.

use std::sync::Arc;
use std::time::Duration;

use axum_test::TestServer;
use serde_json::json;

use lumen::api::{router, AppState};
use lumen::storage::Engine;

fn server_with_engine() -> (TestServer, Arc<Engine>) {
    let engine = Arc::new(Engine::new());
    let app = router(AppState::open(engine.clone()));
    (TestServer::new(app).expect("test server"), engine)
}

#[tokio::test]
async fn soft_delete_returns_202_and_reads_get_410() {
    let (s, _engine) = server_with_engine();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.post("/collections/u/index")
        .json(&json!({ "items": [
            { "external_id": "u1", "field": "e", "value": "a@x.com" }
        ]}))
        .await
        .assert_status_ok();

    // Soft delete.
    let del = s.delete("/collections/u").await;
    del.assert_status(axum::http::StatusCode::ACCEPTED);

    // Reads now 410 Gone.
    let r = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "e", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    r.assert_status(axum::http::StatusCode::GONE);

    // Second delete (already-marked) → 204.
    let again = s.delete("/collections/u").await;
    again.assert_status(axum::http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn force_delete_returns_204_and_404_after() {
    let (s, _engine) = server_with_engine();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    let del = s.delete("/collections/u?force=true").await;
    del.assert_status(axum::http::StatusCode::NO_CONTENT);

    let r = s
        .post("/collections/u/search")
        .json(&json!({
            "query": { "term": { "field": "e", "value": "a@x.com" } },
            "limit": 10
        }))
        .await;
    r.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sweep_removes_expired_soft_deleted() {
    let (s, engine) = server_with_engine();
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
    s.delete("/collections/u")
        .await
        .assert_status(axum::http::StatusCode::ACCEPTED);

    // Wait one ms then sweep with zero grace — should remove.
    tokio::time::sleep(Duration::from_millis(2)).await;
    let n = engine.sweep_deleted(Duration::from_millis(1)).unwrap();
    assert_eq!(n, 1);

    // Now namespace is gone — create succeeds again.
    s.put("/collections/u")
        .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn drain_flips_readyz_to_503() {
    let (s, engine) = server_with_engine();
    s.get("/readyz").await.assert_status_ok();
    engine.start_drain();
    let r = s.get("/readyz").await;
    r.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    // Healthz still 200 — process is still alive.
    s.get("/healthz").await.assert_status_ok();
}

#[tokio::test]
async fn list_collections_skips_soft_deleted() {
    let (s, _engine) = server_with_engine();
    for id in ["a", "b", "c"] {
        s.put(&format!("/collections/{id}"))
            .json(&json!({ "fields": { "e": { "type": "keyword" } } }))
            .await
            .assert_status_ok();
    }
    s.delete("/collections/b")
        .await
        .assert_status(axum::http::StatusCode::ACCEPTED);
    let resp = s.get("/collections").await;
    let body: serde_json::Value = resp.json();
    let arr = body.as_array().unwrap();
    let ids: Vec<&str> = arr.iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(ids, vec!["a", "c"]);
}

// </HANDWRITE>
