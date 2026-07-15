// HANDWRITE-BEGIN gap="missing-generator:unit-test:3d8c0bd7" tracker="#1642" reason="Verify Tape-selected append policy uses shared enforcement and default routing stays disabled."
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use service_http::{AdmissionController, AdmissionPolicy};
use tape::server::{router, router_with_admission, AppState};
use tape::TapeJournal;
use tower::ServiceExt;

fn write_controller() -> AdmissionController {
    AdmissionController::new([(
        "tape.write",
        AdmissionPolicy::new(1, Duration::from_secs(60), 16).unwrap(),
    )])
}

fn append_request() -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/topics/orders/append")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"payload":{"id":1}}"#))
        .unwrap()
}

#[tokio::test]
async fn tape_selects_append_as_write_admission() {
    let app = router_with_admission(
        AppState::new(TapeJournal::default(), None),
        Some(write_controller()),
    );
    assert_eq!(
        app.clone()
            .oneshot(append_request())
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.oneshot(append_request()).await.unwrap().status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn tape_default_router_keeps_admission_disabled() {
    let app = router(AppState::new(TapeJournal::default(), None));
    assert_eq!(
        app.clone()
            .oneshot(append_request())
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.oneshot(append_request()).await.unwrap().status(),
        StatusCode::OK
    );
}
// HANDWRITE-END
