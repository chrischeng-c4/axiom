//! Focused restore and cold recovery contract.

use std::sync::Arc;
use axum::http::StatusCode;
use axum_test::TestServer;
use lumen::api::{router, AppState};
use lumen::storage::Engine;
use serde_json::json;

#[tokio::test]
async fn snapshot_restore_preserves_query_results() {
    let source = TestServer::new(router(AppState::open(Arc::new(Engine::new())))).expect("source");
    source.put("/collections/restore").json(&json!({"fields":{"kw":{"type":"keyword"}}})).await.assert_status_ok();
    source.post("/collections/restore/index").json(&json!({"items":[{"external_id":"r1","field":"kw","value":"kept"}]})).await.assert_status_ok();
    let backup = source.get("/admin/backup").await;
    backup.assert_status_ok();
    let snapshot: serde_json::Value = backup.json();
    let restored = TestServer::new(router(AppState::open(Arc::new(Engine::new())))).expect("restored");
    let response = restored.post("/admin/restore").json(&snapshot).await;
    response.assert_status(StatusCode::NO_CONTENT);
    let query = restored
        .post("/collections/restore/search")
        .json(&json!({"query":{"term":{"field":"kw","value":"kept"}}}))
        .await;
    query.assert_status_ok();
    let body: serde_json::Value = query.json();
    assert_eq!(body["total"], 1, "restored state must retain the indexed document");
    assert_eq!(body["hits"][0]["external_id"], "r1");
}
