// HANDWRITE-BEGIN gap="missing-generator:unit-test:76ab3ef3" tracker="#1642" reason="Verify Lumen-selected collection-read policy rejects excess requests while the default router remains unchanged."
use std::sync::Arc;
use std::time::Duration;

use axum::http::StatusCode;
use axum_test::TestServer;
use lumen::api::{router, router_with_admission, AppState};
use lumen::storage::Engine;
use service_http::{AdmissionController, AdmissionPolicy};

fn read_controller() -> AdmissionController {
    AdmissionController::new([(
        "lumen.read",
        AdmissionPolicy::new(1, Duration::from_secs(60), 16).unwrap(),
    )])
}

#[tokio::test]
async fn lumen_selects_collection_reads_while_probes_stay_outside_admission() {
    let state = AppState::open(Arc::new(Engine::new()));
    let server = TestServer::new(router_with_admission(state, Some(read_controller()))).unwrap();

    server.get("/healthz").await.assert_status_ok();
    server.get("/healthz").await.assert_status_ok();
    server.get("/collections").await.assert_status_ok();
    let denied = server.get("/collections").await;
    denied.assert_status(StatusCode::TOO_MANY_REQUESTS);
    let body: serde_json::Value = denied.json();
    assert_eq!(body["error"], "rate_limited");
}

#[tokio::test]
async fn lumen_default_router_keeps_admission_disabled() {
    let server = TestServer::new(router(AppState::open(Arc::new(Engine::new())))).unwrap();
    server.get("/collections").await.assert_status_ok();
    server.get("/collections").await.assert_status_ok();
}
// HANDWRITE-END
