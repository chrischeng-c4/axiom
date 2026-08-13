use axum::{body::Body, http::Request};
use server_lifecycle::{LifecycleController, LifecyclePhase};
use service_observability::{LifecycleMetrics, MetricsProvider};
use std::sync::Arc;
use tokio::time::Duration;
use tower::ServiceExt;

async fn wait_snapshot(
    metrics: &LifecycleMetrics,
    generation: u64,
) -> service_observability::LifecycleMetricsSnapshot {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if let Some(snapshot) = metrics.lifecycle_snapshot() {
                if snapshot.generation >= generation {
                    return snapshot;
                }
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("lifecycle observation should arrive")
}

#[tokio::test]
async fn lifecycle_observer_records_bounded_series() {
    let c = LifecycleController::new();
    let metrics = Arc::new(LifecycleMetrics::new());
    let task = tokio::spawn(metrics.clone().observe_lifecycle(c.subscribe_transitions()));
    let _ = wait_snapshot(&metrics, 0).await;
    c.transition(LifecyclePhase::Serving, "reason-secret", "detail-secret")
        .unwrap();
    let snapshot = wait_snapshot(&metrics, 1).await;
    assert_eq!(snapshot.phase, LifecyclePhase::Serving);
    assert_eq!(snapshot.generation, 1);
    assert_eq!(snapshot.transition_count, 2);
    assert_eq!(snapshot.reason_code, "reason-secret");
    let router = service_http::lifecycle_probe_routes(c.clone(), Some(metrics.clone()), || {
        utoipa::openapi::OpenApi::new(
            utoipa::openapi::Info::new("test", "1"),
            utoipa::openapi::Paths::new(),
        )
    });
    let response = router
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.headers().get("x-lifecycle-phase").unwrap(),
        "serving"
    );
    assert_eq!(
        response.headers().get("x-lifecycle-generation").unwrap(),
        "1"
    );
    assert_eq!(
        response.headers().get("x-lifecycle-reason-code").unwrap(),
        "reason-secret"
    );
    let rendered = metrics.render_metrics();
    assert!(rendered.contains("service_lifecycle_phase 2\n"));
    assert!(rendered.contains("service_lifecycle_generation 1\n"));
    assert!(rendered.contains("service_lifecycle_transitions_total 2\n"));
    assert!(!rendered.contains("reason-secret") && !rendered.contains("detail-secret"));
    metrics.record_lifecycle(&c.observation());
    tokio::time::sleep(Duration::from_secs(1)).await;
    let duplicate = metrics.lifecycle_snapshot().unwrap();
    assert_eq!(duplicate.transition_count, snapshot.transition_count);
    assert!(duplicate.age_seconds >= 1);
    c.transition(LifecyclePhase::Draining, "drain", "detail")
        .unwrap();
    c.transition(LifecyclePhase::Stopping, "stop", "detail")
        .unwrap();
    c.transition(LifecyclePhase::Stopped, "done", "detail")
        .unwrap();
    tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .expect("observer timeout")
        .expect("observer join")
        .expect("observer result");
}
