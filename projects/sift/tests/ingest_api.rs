// HANDWRITE-BEGIN gap="sift-ingest-contract-tests" tracker="1576" reason="Verify ingest acknowledgement, duplicate idempotency, direct metric preservation, query, and replay."
use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sift::{
    openapi, router, DurableJournal, EventEnvelope, EventQuery, MetricExemplar, MetricPoint,
    MetricTemporality, ServiceState, SignalKind,
};
use tower::ServiceExt;

fn event(id: &str, signal: SignalKind) -> EventEnvelope {
    let mut event = EventEnvelope::new(id, signal, serde_json::json!({ "message": "accepted" }));
    event
        .resource
        .insert("service.name".to_string(), "checkout".to_string());
    event.trace_id = Some("0af7651916cd43dd8448eb211c80319c".to_string());
    event.span_id = Some("b7ad6b7169203331".to_string());
    if signal == SignalKind::Metric {
        event.metric = Some(MetricPoint {
            name: "checkout.duration".to_string(),
            value: 42.5,
            unit: Some("ms".to_string()),
            temporality: MetricTemporality::Delta,
            exemplars: vec![MetricExemplar {
                value: 42.5,
                trace_id: event.trace_id.clone().unwrap(),
                span_id: event.span_id.clone().unwrap(),
            }],
        });
    }
    event
}

#[test]
fn all_six_signal_envelopes_validate_and_metric_context_is_required() {
    for signal in SignalKind::ALL {
        event(&format!("event-{signal}"), signal)
            .validate()
            .expect("fixture must satisfy the versioned envelope contract");
    }

    let mut invalid = event("missing-metric", SignalKind::Metric);
    invalid.metric = None;
    assert!(invalid.validate().is_err());
}

#[test]
fn journal_fsync_boundary_restart_and_replay_are_idempotent() {
    let temp = tempfile::tempdir().unwrap();
    let journal = DurableJournal::open(temp.path()).unwrap();
    let metric = event("metric-1", SignalKind::Metric);

    let accepted = journal.append(metric.clone()).unwrap();
    assert!(!accepted.duplicate);
    assert_eq!(accepted.cursor, 1);

    let duplicate = journal.append(metric).unwrap();
    assert!(duplicate.duplicate);
    assert_eq!(duplicate.cursor, accepted.cursor);

    let logs = journal.append(event("log-1", SignalKind::Log)).unwrap();
    assert_eq!(logs.cursor, 2);
    drop(journal);

    let recovered = DurableJournal::open(temp.path()).unwrap();
    let all = recovered.query(EventQuery::default()).unwrap();
    assert_eq!(
        all.len(),
        2,
        "acknowledged events survive restart exactly once"
    );
    assert_eq!(all[0].event.event_id, "metric-1");
    assert_eq!(all[0].event.metric.as_ref().unwrap().exemplars.len(), 1);

    let replay = recovered.replay(1, 10).unwrap();
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].event.event_id, "log-1");
}

#[tokio::test]
async fn http_ingest_and_standard_operational_routes_share_the_journal_contract() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = service_http::standard_probe_routes(state.clone(), Some(state.clone()), openapi)
        .merge(router(state));

    for path in ["/healthz", "/readyz", "/metrics", "/openapi.json", "/docs"] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "{path}");
    }

    let body = serde_json::to_vec(&event("metric-http", SignalKind::Metric)).unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
// HANDWRITE-END
