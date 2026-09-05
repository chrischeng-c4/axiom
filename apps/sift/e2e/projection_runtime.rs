// HANDWRITE-BEGIN gap="sift-projection-runtime-tests" tracker="1660" reason="Verify idempotency, atomic restart, lag waits, and fresh/live rebuild equality."
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use sift::{
    projection::{
        ProjectionRuntime, METRIC_SCHEMA_VERSION, PROJECTION_LOGGING_STORE,
        PROJECTION_METRIC_STORE, PROJECTION_TRACE_STORE,
    },
    DurableJournal, EventEnvelope, MetricPoint, MetricTemporality, SignalKind,
};

fn event(id: &str, message: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "projection-project",
        "test",
        id,
        SignalKind::Log,
        serde_json::json!({"message": message}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "projection-test".into())]);
    event
}

fn metric_event(id: &str) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "projection-project",
        "test",
        id,
        SignalKind::Metric,
        serde_json::json!({}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "projection-test".into())]);
    event.metric = Some(MetricPoint {
        name: "projection.requests".into(),
        value: 1.0,
        stale: false,
        unit: Some("1".into()),
        temporality: MetricTemporality::Gauge,
        exemplars: Vec::new(),
    });
    event
}

#[tokio::test]
async fn projection_checkpoint_is_idempotent_atomic_and_restartable() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal.append(event("evt-1", "database timeout")).unwrap();

    let runtime = ProjectionRuntime::open(temp.path(), journal.clone()).unwrap();
    assert_eq!(
        runtime.projection_names(),
        vec![
            PROJECTION_LOGGING_STORE.to_string(),
            PROJECTION_METRIC_STORE.to_string(),
            PROJECTION_TRACE_STORE.to_string(),
        ]
    );
    assert_eq!(runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap(), 1);
    let first_digest = runtime.semantic_digest(PROJECTION_LOGGING_STORE).unwrap();
    assert_eq!(runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap(), 1);
    assert_eq!(
        runtime.semantic_digest(PROJECTION_LOGGING_STORE).unwrap(),
        first_digest
    );

    drop(runtime);
    let reopened = ProjectionRuntime::open(temp.path(), journal.clone()).unwrap();
    assert_eq!(
        reopened.current_cursor(PROJECTION_LOGGING_STORE).unwrap(),
        1
    );
    assert_eq!(
        reopened.semantic_digest(PROJECTION_LOGGING_STORE).unwrap(),
        first_digest
    );

    let state_path = temp
        .path()
        .join("indexes")
        .join(PROJECTION_LOGGING_STORE)
        .join("state.json");
    let envelope: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&state_path).unwrap()).unwrap();
    assert_eq!(envelope["checkpoint"]["cursor"], 1);
    assert!(envelope["checkpoint"]["state_sha256"].as_str().is_some());
    assert!(envelope["state_base64"].as_str().is_some());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(state_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[tokio::test]
async fn min_cursor_wait_returns_typed_lag_then_wakes_after_projection_publish() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    let runtime = ProjectionRuntime::open(temp.path(), journal.clone()).unwrap();

    let lag = runtime
        .wait_for_min_cursor(PROJECTION_LOGGING_STORE, 1, Duration::from_millis(10))
        .await
        .unwrap_err();
    assert_eq!(lag.error, "projection_lag");
    assert_eq!(lag.projection, PROJECTION_LOGGING_STORE);
    assert_eq!(lag.required_cursor, 1);
    assert_eq!(lag.current_cursor, 0);
    assert!(lag.retryable);
    assert_eq!(lag.retry_after_seconds, 1);

    journal.append(event("evt-1", "database timeout")).unwrap();
    assert_eq!(runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap(), 1);
    assert_eq!(
        runtime
            .wait_for_min_cursor(PROJECTION_LOGGING_STORE, 1, Duration::from_secs(1))
            .await
            .unwrap(),
        1
    );
}

#[test]
fn projection_state_is_checkpointed_in_large_intervals_and_flushes_explicitly() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal.append(event("evt-0", "first")).unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal.clone()).unwrap();
    runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap();

    for index in 1..=10 {
        journal
            .append(event(&format!("evt-{index}"), "later"))
            .unwrap();
    }
    runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap();
    assert_eq!(
        runtime.current_cursor(PROJECTION_LOGGING_STORE).unwrap(),
        11
    );

    let path = temp
        .path()
        .join("indexes")
        .join(PROJECTION_LOGGING_STORE)
        .join("state.json");
    let before: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(before["checkpoint"]["cursor"], 1);

    runtime.persist_all().unwrap();
    let after: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(after["checkpoint"]["cursor"], 11);
}

#[test]
fn fresh_raw_rebuild_matches_and_installs_the_live_projection() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal.append(event("evt-1", "database timeout")).unwrap();
    journal
        .append(event("evt-2", "database recovered"))
        .unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal).unwrap();
    runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap();

    let result = runtime
        .rebuild_and_compare(PROJECTION_LOGGING_STORE)
        .unwrap();
    assert_eq!(result.source_cursor, 2);
    assert_eq!(result.rebuilt_cursor, 2);
    assert_eq!(result.live_digest, result.rebuilt_digest);
    assert!(result.equal);
    assert_eq!(runtime.current_cursor(PROJECTION_LOGGING_STORE).unwrap(), 2);
}

#[test]
fn corrupt_projection_snapshot_is_quarantined_and_rebuilt_from_journal() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal.append(event("evt-1", "database timeout")).unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal.clone()).unwrap();
    runtime.catch_up(PROJECTION_LOGGING_STORE).unwrap();
    runtime.persist_all().unwrap();
    let expected_digest = runtime.semantic_digest(PROJECTION_LOGGING_STORE).unwrap();
    drop(runtime);

    let state_dir = temp.path().join("indexes").join(PROJECTION_LOGGING_STORE);
    std::fs::write(state_dir.join("state.json"), b"{\"truncated\":").unwrap();

    let rebuilt = ProjectionRuntime::open(temp.path(), journal).unwrap();
    assert_eq!(rebuilt.current_cursor(PROJECTION_LOGGING_STORE).unwrap(), 1);
    assert_eq!(
        rebuilt.semantic_digest(PROJECTION_LOGGING_STORE).unwrap(),
        expected_digest
    );
    assert_eq!(
        std::fs::read_dir(&state_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("state.corrupt-"))
            .count(),
        1
    );
}

#[test]
fn schema_five_metric_checkpoint_is_quarantined_and_rebuilt_from_journal() {
    let temp = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(temp.path()).unwrap());
    journal.append(metric_event("metric-1")).unwrap();
    let runtime = ProjectionRuntime::open(temp.path(), journal.clone()).unwrap();
    runtime.catch_up(PROJECTION_METRIC_STORE).unwrap();
    runtime.persist_all().unwrap();
    let expected_digest = runtime.semantic_digest(PROJECTION_METRIC_STORE).unwrap();
    drop(runtime);

    let state_dir = temp.path().join("indexes").join(PROJECTION_METRIC_STORE);
    let state_path = state_dir.join("state.json");
    let mut envelope: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&state_path).unwrap()).unwrap();
    assert_eq!(
        envelope["checkpoint"]["schema_version"],
        METRIC_SCHEMA_VERSION
    );
    envelope["checkpoint"]["schema_version"] = serde_json::json!(5);
    std::fs::write(&state_path, serde_json::to_vec_pretty(&envelope).unwrap()).unwrap();

    let rebuilt = ProjectionRuntime::open(temp.path(), journal).unwrap();
    assert_eq!(rebuilt.current_cursor(PROJECTION_METRIC_STORE).unwrap(), 1);
    assert_eq!(
        rebuilt.semantic_digest(PROJECTION_METRIC_STORE).unwrap(),
        expected_digest
    );
    let rebuilt_envelope: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&state_path).unwrap()).unwrap();
    assert_eq!(
        rebuilt_envelope["checkpoint"]["schema_version"],
        METRIC_SCHEMA_VERSION
    );
    assert_eq!(
        std::fs::read_dir(&state_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("state.corrupt-"))
            .count(),
        1
    );
}

// HANDWRITE-END
