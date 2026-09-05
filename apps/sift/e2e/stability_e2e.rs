// HANDWRITE-BEGIN gap="sift-stability-evidence" tracker="1607" reason="Exercise readiness, drain, bounded ingestion, and journal recovery through the Sift service router."
use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sift::{openapi, router, DurableJournal, EventQuery, ServiceState};
use tower::ServiceExt;

fn log_record(sequence: usize) -> serde_json::Value {
    serde_json::json!({
        "body": {"stringValue": format!("bounded burst {sequence}")},
        "attributes": [{
            "key": "sift.event_id",
            "value": {"stringValue": format!("stability-{sequence}")}
        }]
    })
}

#[tokio::test]
async fn bounded_ingest_burst_survives_drain_and_journal_reopen() {
    let data_dir = tempfile::tempdir().expect("temporary Sift data directory");
    let state = Arc::new(ServiceState::open(data_dir.path()).expect("open Sift state"));
    let app = service_http::standard_probe_routes(state.clone(), Some(state.clone()), openapi)
        .merge(router(state.clone()));

    let payload = serde_json::json!({
        "resourceLogs": [{
            "resource": {"attributes": [{
                "key": "service.name",
                "value": {"stringValue": "sift-stability"}
            }]},
            "scopeLogs": [{
                "logRecords": (0..128).map(log_record).collect::<Vec<_>>()
            }]
        }]
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/logs")
                .header("content-type", "application/json")
                .header("x-sift-project", "stability")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

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
    state
        .finish_drain()
        .await
        .expect("finish accepted ingest batches during drain");

    drop(response);
    drop(readiness);
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
