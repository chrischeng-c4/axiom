use std::{sync::Arc, time::Duration};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use chrono::{SecondsFormat, Utc};
use sift::{
    projection::PROJECTION_LOGGING_STORE, router, storage::archive, EventEnvelope, ServiceState,
    SignalKind,
};
use tower::ServiceExt;

async fn query(app: &axum::Router, start: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/query")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "version": 1,
                        "project": "cold-project",
                        "environment": "prod",
                        "time_range": {"start": start},
                        "signal": {
                            "kind": "logs",
                            "filter": {"op": "eq", "field": "event_id", "value": "cold-log"}
                        },
                        "limit": 10,
                        "mode": "sync"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cold_query_reports_archive_outage_instead_of_silent_complete_data() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let address_text = address.to_string();
    let emulator = tokio::spawn(async move {
        vat::emulator::serve(vat::emulator::Kind::CloudStorage, &address_text)
            .await
            .unwrap();
    });
    for _ in 0..100 {
        if tokio::net::TcpStream::connect(address).await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    std::env::set_var("STORAGE_EMULATOR_HOST", format!("http://{address}"));

    let data = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(data.path()).unwrap());
    let occurred = Utc::now() - chrono::Duration::days(31);
    let start = (occurred - chrono::Duration::days(1)).to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut event = EventEnvelope::for_project(
        "cold-project",
        "prod",
        "cold-log",
        SignalKind::Log,
        serde_json::json!({"message": "archived log"}),
    );
    event.occurred_at = occurred.to_rfc3339_opts(SecondsFormat::Nanos, true);
    event.observed_at.clone_from(&event.occurred_at);
    event
        .resource
        .insert("service.name".into(), "cold-service".into());
    state.journal().append(event).unwrap();
    let archive_state = state.clone();
    tokio::task::spawn_blocking(move || {
        archive::archive_journal_gcs(archive_state.journal(), "gs://sift-cold/query")
    })
    .await
    .unwrap()
    .expect("commit remote archive before the query");
    state
        .projections()
        .catch_up(PROJECTION_LOGGING_STORE)
        .expect("build the local projection before removing the raw hot copy");
    for manifest in state.journal().storage().seal_all().unwrap() {
        std::fs::remove_file(&manifest.local_path).unwrap();
    }
    let app = router(state);

    let available = query(&app, &start).await;
    assert_eq!(available["partial"], false);
    assert_eq!(available["data"]["records"][0]["event_id"], "cold-log");
    assert!(data
        .path()
        .join("archive-cache")
        .read_dir()
        .unwrap()
        .next()
        .is_some());

    emulator.abort();
    let _ = emulator.await;
    let unavailable = query(&app, &start).await;
    assert_eq!(unavailable["partial"], true);
    assert!(unavailable["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning
            .as_str()
            .unwrap()
            .to_ascii_lowercase()
            .contains("archive")));

    std::env::remove_var("STORAGE_EMULATOR_HOST");
}
