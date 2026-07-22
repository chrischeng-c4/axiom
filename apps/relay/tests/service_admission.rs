// HANDWRITE-BEGIN gap="missing-generator:unit-test:relay-service-admission" tracker="#1206" reason="Verify Relay-selected publish policy uses shared enforcement and default routing stays disabled."
use axum::body::Body;
use axum::http::{Request, StatusCode};
use relay::server::{router, router_with_admission, AppState};
use relay::server_config::RelayServerConfig;
use service_http::{AdmissionConfig, AdmissionController};
use tower::ServiceExt;

fn controller() -> AdmissionController {
    AdmissionConfig::from_lookup("RELAY", |key| match key {
        "RELAY_ADMISSION_WRITE_CAPACITY" => Some("1".into()),
        "RELAY_ADMISSION_REFILL_SECS" => Some("60".into()),
        "RELAY_ADMISSION_MAX_KEYS" => Some("16".into()),
        _ => None,
    })
    .unwrap()
    .controller("relay.read", "relay.write", "relay.admin")
    .unwrap()
}

fn publish() -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/jobs/publish")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"message_id":"m1","payload":{}}"#))
        .unwrap()
}

#[tokio::test]
async fn publish_uses_shared_write_admission() {
    let app = router_with_admission(
        AppState::new(RelayServerConfig::ephemeral()),
        Some(controller()),
    );
    assert_eq!(
        app.clone().oneshot(publish()).await.unwrap().status(),
        StatusCode::OK
    );
    let throttled = app.clone().oneshot(publish()).await.unwrap();
    assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(throttled.headers()["retry-after"], "60");
    assert_eq!(
        app.oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap()
        .status(),
        StatusCode::OK
    );
}

#[tokio::test]
async fn default_router_keeps_admission_disabled() {
    let app = router(AppState::new(RelayServerConfig::ephemeral()));
    assert_eq!(
        app.clone().oneshot(publish()).await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(
        app.oneshot(publish()).await.unwrap().status(),
        StatusCode::OK
    );
}
// HANDWRITE-END
