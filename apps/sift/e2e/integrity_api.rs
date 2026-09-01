use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use sha2::{Digest, Sha256};
use sift::{router, EventEnvelope, MetricPoint, MetricTemporality, ServiceState, SignalKind};
use tower::ServiceExt;

fn event(project: &str, id: &str, signal: SignalKind) -> EventEnvelope {
    let mut event =
        EventEnvelope::for_project(project, "prod", id, signal, serde_json::json!({"id": id}));
    event
        .resource
        .insert("service.name".into(), "integrity-test".into());
    if signal == SignalKind::Span {
        event.trace_id = Some(format!("trace-{id}"));
        event.span_id = Some(format!("span-{id}"));
    } else if signal == SignalKind::Metric {
        event.metric = Some(MetricPoint {
            name: "integrity.value".into(),
            value: 1.0,
            stale: false,
            unit: None,
            temporality: MetricTemporality::Gauge,
            exemplars: Vec::new(),
        });
    }
    event
}

fn xor_digest(ids: &[&str]) -> String {
    let mut output = [0_u8; 32];
    for id in ids {
        let digest: [u8; 32] = Sha256::digest(id.as_bytes()).into();
        for (slot, byte) in output.iter_mut().zip(digest) {
            *slot ^= byte;
        }
    }
    hex::encode(output)
}

#[tokio::test]
async fn admin_integrity_reports_project_counts_digest_and_signal_watermarks() {
    let data = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(data.path()).unwrap());
    state
        .journal()
        .append(event("project-a", "log-1", SignalKind::Log))
        .unwrap();
    state
        .journal()
        .append(event("project-b", "other", SignalKind::Log))
        .unwrap();
    state
        .journal()
        .append(event("project-a", "metric-1", SignalKind::Metric))
        .unwrap();
    state
        .journal()
        .append(event("project-a", "span-1", SignalKind::Span))
        .unwrap();

    let response = router(state)
        .oneshot(
            Request::builder()
                .uri("/admin/integrity?project=project-a")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["project"], "project-a");
    assert!(body["cluster_id"].as_str().unwrap().starts_with("cluster-"));
    assert!(body["restored_from"].is_null());
    assert_eq!(body["event_count"], 3);
    assert_eq!(body["event_id_digest_algorithm"], "xor-sha256-v1");
    assert_eq!(
        body["event_id_sha256"],
        xor_digest(&["log-1", "metric-1", "span-1"])
    );
    assert_eq!(body["signals"]["logs"]["count"], 1);
    assert_eq!(body["signals"]["logs"]["watermark"], 1);
    assert_eq!(body["signals"]["metrics"]["count"], 1);
    assert_eq!(body["signals"]["metrics"]["watermark"], 3);
    assert_eq!(body["signals"]["traces"]["count"], 1);
    assert_eq!(body["signals"]["traces"]["watermark"], 4);
    assert_eq!(body["watermark"], 4);
    assert!(body["storage"]["wal_bytes"]["logs"].as_u64().unwrap() > 0);
    assert!(body["storage"]["wal_bytes"]["metrics"].as_u64().unwrap() > 0);
    assert!(body["storage"]["wal_bytes"]["traces"].as_u64().unwrap() > 0);
    assert_eq!(body["storage"]["archive"]["watermarks"]["logs"], 0);
    assert!(body["storage"]["archive"]["manifest_uri"].is_null());
}
