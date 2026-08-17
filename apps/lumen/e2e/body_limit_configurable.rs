//! End-to-end test suite for configurable request body limit (#2584).

use std::sync::Arc;
use std::sync::Mutex;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use lumen::api::{router, AppState};
use lumen::operator::crd::{
    AuthMode, LogFormat, LumenSpec, PlacementSpec, ReshardPolicy, ServingSpec, ShardMapSpec,
    MAX_BODY_LIMIT_BYTES, MIN_BODY_LIMIT_BYTES,
};
use lumen::storage::Engine;
use serde_json::Value;
use tower::ServiceExt;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn app_router() -> axum::Router {
    let engine = Arc::new(Engine::new());
    let state = AppState::open(engine);
    router(state)
}

fn test_spec() -> LumenSpec {
    LumenSpec {
        image: "lumen:latest".into(),
        image_pull_policy: None,
        placement: PlacementSpec::default(),
        shard_count: 1,
        shard_map: ShardMapSpec::default(),
        replicas_per_shard: 1,
        voter_count: 1,
        log_format: LogFormat::Pretty,
        log_level: None,
        auth: AuthMode::Off,
        serving: ServingSpec::default(),
        reshard_policy: ReshardPolicy::default(),
        observability: false,
        network_policy: false,
        admission: None,
        service_account_name: None,
        service_account_annotations: std::collections::BTreeMap::new(),
        peer_tls_secret: None,
        serving_tls_secret: None,
        body_limit_bytes: None,
    }
}

#[tokio::test]
async fn omitted_body_limit_preserves_default_8mib() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var("LUMEN_BODY_LIMIT_BYTES");

    let app = app_router();

    // 8 MiB + 1 KiB exceeds default 8 MiB cap
    let oversized_len = 8 * 1024 * 1024 + 1024;
    let req = Request::builder()
        .method("POST")
        .uri("/admin/reshard:apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, oversized_len.to_string())
        .body(Body::from(vec![b'a'; oversized_len]))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).expect("structured 413 JSON");
    assert_eq!(json["error"], "payload_too_large");
    assert_eq!(
        json["message"],
        "request body exceeds the configured size limit"
    );
}

#[tokio::test]
async fn configured_body_limit_enforces_at_custom_threshold() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let limit_1mib = 1024 * 1024;
    std::env::set_var("LUMEN_BODY_LIMIT_BYTES", limit_1mib.to_string());

    let app = app_router();

    // 1 MiB + 512 bytes exceeds 1 MiB cap
    let oversized_len = limit_1mib + 512;
    let req = Request::builder()
        .method("POST")
        .uri("/admin/reshard:apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, oversized_len.to_string())
        .body(Body::from(vec![b'a'; oversized_len]))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).expect("structured 413 JSON");
    assert_eq!(json["error"], "payload_too_large");
    assert_eq!(
        json["message"],
        "request body exceeds the configured size limit"
    );

    std::env::remove_var("LUMEN_BODY_LIMIT_BYTES");
}

#[tokio::test]
async fn streamed_body_exceeding_limit_fails_mid_read_with_413() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let limit_1mib = 1024 * 1024;
    std::env::set_var("LUMEN_BODY_LIMIT_BYTES", limit_1mib.to_string());

    let app = app_router();

    // Streamed body with chunks without Content-Length
    let stream = futures::stream::iter(vec![
        Ok::<_, std::io::Error>(axum::body::Bytes::from(vec![b'x'; limit_1mib])),
        Ok::<_, std::io::Error>(axum::body::Bytes::from(vec![b'y'; 1024])),
    ]);
    let req = Request::builder()
        .method("POST")
        .uri("/admin/reshard:apply")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from_stream(stream))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).expect("structured 413 JSON");
    assert_eq!(json["error"], "payload_too_large");
    assert_eq!(
        json["message"],
        "request body exceeds the configured size limit"
    );

    std::env::remove_var("LUMEN_BODY_LIMIT_BYTES");
}

#[test]
fn crd_validation_rejects_out_of_range_body_limit() {
    let mut spec = test_spec();
    assert!(spec.validate().is_ok());

    spec.body_limit_bytes = Some(MIN_BODY_LIMIT_BYTES);
    assert!(spec.validate().is_ok());

    spec.body_limit_bytes = Some(MAX_BODY_LIMIT_BYTES);
    assert!(spec.validate().is_ok());

    for invalid in [0, 512, MIN_BODY_LIMIT_BYTES - 1, MAX_BODY_LIMIT_BYTES + 1] {
        spec.body_limit_bytes = Some(invalid);
        let err = spec.validate().expect_err("should reject invalid body limit");
        assert!(err.contains("bodyLimitBytes"));
        assert!(err.contains(&MIN_BODY_LIMIT_BYTES.to_string()));
        assert!(err.contains(&MAX_BODY_LIMIT_BYTES.to_string()));
    }
}

#[tokio::test]
async fn under_limit_body_is_not_refused_by_body_limit_layer() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let limit_1mib = 1024 * 1024;
    std::env::set_var("LUMEN_BODY_LIMIT_BYTES", limit_1mib.to_string());

    let app = app_router();

    let under_limit_len = 4096;
    let req = Request::builder()
        .method("POST")
        .uri("/admin/reshard:apply")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, under_limit_len.to_string())
        .body(Body::from(vec![b'a'; under_limit_len]))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_ne!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);

    std::env::remove_var("LUMEN_BODY_LIMIT_BYTES");
}
