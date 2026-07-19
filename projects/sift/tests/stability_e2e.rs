// HANDWRITE-BEGIN gap="sift-stability-evidence" tracker="1607" reason="Exercise readiness, drain, bounded ingestion, and journal recovery through the Sift service router."
use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sift::{openapi, router, DurableJournal, EventEnvelope, EventQuery, ServiceState, SignalKind};
use tower::ServiceExt;

fn event(sequence: usize) -> EventEnvelope {
    let mut event = EventEnvelope::new(
        format!("stability-{sequence}"),
        SignalKind::Log,
        serde_json::json!({"message": format!("bounded burst {sequence}")}),
    );
    event
        .resource
        .insert("service.name".to_string(), "sift-stability".to_string());
    event
}

#[tokio::test]
async fn bounded_ingest_burst_survives_drain_and_journal_reopen() {
    let data_dir = tempfile::tempdir().expect("temporary Sift data directory");
    let state = Arc::new(ServiceState::open(data_dir.path()).expect("open Sift state"));
    let app = service_http::standard_probe_routes(state.clone(), Some(state.clone()), openapi)
        .merge(router(state.clone()));

    for sequence in 0..128 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/events")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&event(sequence)).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "event {sequence}");
    }

    state.start_drain();
    let readiness = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(readiness.status(), StatusCode::SERVICE_UNAVAILABLE);

    drop(app);
    drop(state);
    let reopened = DurableJournal::open(data_dir.path()).expect("reopen journal after drain");
    let rows = reopened
        .query(EventQuery {
            limit: 128,
            ..EventQuery::default()
        })
        .expect("query durable burst");
    assert_eq!(
        rows.len(),
        128,
        "bounded burst must remain durable after reopen"
    );
    assert_eq!(rows.first().unwrap().event.event_id, "stability-0");
    assert_eq!(rows.last().unwrap().event.event_id, "stability-127");
}
// HANDWRITE-END
