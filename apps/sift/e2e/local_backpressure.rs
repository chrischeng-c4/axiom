use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use sift::{ingest::IngestLimits, router, EventQuery, ServiceState};
use tower::ServiceExt;

#[test]
fn production_defaults_admit_the_mvp_target_with_headroom() {
    let limits = IngestLimits::default();
    assert_eq!(limits.max_items_per_project_window, 720_000);
    assert_eq!(limits.quota_window_secs, 60);
    assert_eq!(limits.max_concurrent_requests_per_project, 32);
    assert_eq!(limits.max_events_per_batch, 1_000);
}

#[test]
fn disk_levels_are_warning_at_70_backpressure_at_80_and_critical_at_90_percent() {
    use sift::storage::{CapacityLevel, LocalCapacity};

    for (bytes, expected) in [
        (69, CapacityLevel::Normal),
        (70, CapacityLevel::Warning),
        (80, CapacityLevel::Backpressure),
        (90, CapacityLevel::Critical),
    ] {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("data"), vec![0; bytes]).unwrap();
        let capacity = LocalCapacity::open(temp.path(), 100, 1).unwrap();
        assert_eq!(capacity.level(), expected);
    }
}

#[tokio::test]
async fn local_capacity_limit_returns_retryable_backpressure_without_wal_growth() {
    let temp = tempfile::tempdir().unwrap();
    let limits = IngestLimits {
        max_local_storage_bytes: 1,
        min_local_free_bytes: 1,
        ..IngestLimits::default()
    };
    let state = Arc::new(ServiceState::open_with_ingest_limits(temp.path(), limits).unwrap());
    let app = router(state.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("content-type", "application/json")
                .header("x-sift-project", "project-a")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "resourceLogs": [{
                            "resource": {"attributes": [{
                                "key": "service.name",
                                "value": {"stringValue": "checkout"}
                            }]},
                            "scopeLogs": [{"logRecords": [{
                                "timeUnixNano": "1788097659000000000",
                                "body": {"stringValue": "must stay out of WAL"}
                            }]}]
                        }]
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()["retry-after"], "5");
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(error["error"], "local_storage_backpressure");
    assert!(state
        .journal()
        .query(EventQuery::default())
        .unwrap()
        .is_empty());
    for signal in ["logs", "metrics", "traces"] {
        let frames = storage_durable::FramedLogReader::read_frames(
            temp.path().join("wal").join(signal).join("events.framed"),
            0,
        )
        .unwrap();
        assert!(frames.is_empty());
    }
}
