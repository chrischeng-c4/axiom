// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-service-admission" tracker="#766" reason="Verify Defer queue writes use shared admission while probes remain exempt."
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use defer::{AuthConfig, DeferRaft, DeferScheduler, HttpDispatcher};
use raft_runtime::Membership;
use service_http::{AdmissionConfig, AdmissionController};
use tower::ServiceExt;

fn controller() -> AdmissionController {
    AdmissionConfig::from_lookup("DEFER", |key| match key {
        "DEFER_ADMISSION_WRITE_CAPACITY" => Some("1".into()),
        "DEFER_ADMISSION_REFILL_SECS" => Some("60".into()),
        "DEFER_ADMISSION_MAX_KEYS" => Some("16".into()),
        _ => None,
    })
    .unwrap()
    .controller("defer.read", "defer.write", "defer.admin")
    .unwrap()
}

fn queue_put() -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri("/v1/queues/jobs")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"max_in_flight":100,"max_dispatch_per_tick":100,"max_dispatches_per_second":100,"max_burst_size":100,"lease_ttl_ms":30000,"retry_backoff_ms":1000}"#,
        ))
        .unwrap()
}

async fn app(admission: Option<AdmissionController>) -> (tempfile::TempDir, axum::Router) {
    let dir = tempfile::tempdir().unwrap();
    let raft = Arc::new(
        DeferRaft::spawn(
            Arc::new(Mutex::new(DeferScheduler::new())),
            &dir.path().join("raft"),
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            DeferRaft::host_config(8),
        )
        .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(4);
    while !raft.is_leader().await {
        assert!(Instant::now() < deadline, "single node did not elect");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let state = defer::server::AppState::new(
        raft,
        HttpDispatcher::new(Duration::from_secs(1), None).unwrap(),
        AuthConfig::open(),
    );
    (dir, defer::server::router_with_admission(state, admission))
}

#[tokio::test]
async fn queue_mutation_uses_shared_write_admission() {
    let (_dir, app) = app(Some(controller())).await;
    assert_eq!(
        app.clone().oneshot(queue_put()).await.unwrap().status(),
        StatusCode::OK
    );
    let throttled = app.clone().oneshot(queue_put()).await.unwrap();
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
async fn default_configuration_does_not_limit_requests() {
    let (_dir, app) = app(None).await;
    assert_eq!(
        app.clone().oneshot(queue_put()).await.unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(
        app.oneshot(queue_put()).await.unwrap().status(),
        StatusCode::OK
    );
}
// HANDWRITE-END
