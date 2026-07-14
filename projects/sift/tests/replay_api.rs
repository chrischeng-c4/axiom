// HANDWRITE-BEGIN gap="sift-replay-api-tests" tracker="1660" reason="Verify one-state-machine event/replay ordering, durable job restart, and API lifecycle."
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use sift::{router, AppendResult, EventEnvelope, ServiceState, SignalKind};
use tower::ServiceExt;

fn event(id: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "replay-project",
        "test",
        id,
        SignalKind::Log,
        serde_json::json!({"message": id}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "replay-test".into())]);
    event
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn write(app: &axum::Router, id: &str) -> AppendResult {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/events")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&event(id)).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    serde_json::from_value(body_json(response).await).unwrap()
}

#[tokio::test]
async fn replay_jobs_share_state_machine_without_creating_raw_cursor_gaps() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let app = router(state.clone());

    let first = write(&app, "before-replay").await;
    assert_eq!(first.raw_cursor, 1);
    assert_eq!(first.cursor, first.raw_cursor);
    assert_eq!(first.commit_index, 1);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/replays")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"projection":"event-index"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
    let started = body_json(response).await;
    let replay_id = started["id"].as_str().unwrap().to_string();
    let replay_commit = started["commit_index"].as_u64().unwrap();
    assert!(replay_commit > first.commit_index);

    let second = write(&app, "after-replay").await;
    assert_eq!(second.raw_cursor, 2);
    assert!(second.commit_index > replay_commit);

    let mut terminal = None;
    for _ in 0..100 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/v1/replays/{replay_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let job = body_json(response).await;
        if matches!(job["state"].as_str(), Some("completed" | "failed")) {
            terminal = Some(job);
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let terminal = terminal.expect("replay reaches a terminal state");
    assert_eq!(terminal["state"], "completed");
    assert_eq!(terminal["equal"], true);

    drop(app);
    drop(state);
    let reopened = Arc::new(ServiceState::open(temp.path()).unwrap());
    let response = router(reopened)
        .oneshot(
            Request::builder()
                .uri(format!("/v1/replays/{replay_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["state"], "completed");
}

#[tokio::test]
async fn replay_api_rejects_unknown_projection_and_returns_not_found() {
    let temp = tempfile::tempdir().unwrap();
    let app = router(Arc::new(ServiceState::open(temp.path()).unwrap()));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/replays")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"projection":"missing"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/replays/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

<!-- marker: sift-replay-api-tests path: projects/sift/tests/replay_api.rs reason: Verify one-state-machine event/replay ordering, durable job restart, and API lifecycle. -->
// HANDWRITE-END
