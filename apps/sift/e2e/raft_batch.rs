use std::{os::unix::fs::PermissionsExt, sync::Arc};

use service_http::MetricsProvider;
use sift::{EventEnvelope, EventQuery, ServiceState, SignalKind};

fn log(id: &str, message: impl Into<String>) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        id,
        SignalKind::Log,
        serde_json::json!({"message": message.into()}),
    );
    event
        .resource
        .insert("service.name".into(), "checkout".into());
    event
}

#[tokio::test]
async fn one_batch_has_one_commit_and_private_control_state() {
    let temp = tempfile::tempdir().unwrap();
    let state = ServiceState::open(temp.path()).unwrap();

    let results = state
        .append_batch(vec![log("log-1", "one"), log("log-2", "two")])
        .await
        .unwrap();
    assert_eq!(results.len(), 2);
    assert!(results[0].commit_index > 0);
    assert_eq!(results[0].commit_index, results[1].commit_index);
    assert_eq!(
        state.journal().query(EventQuery::default()).unwrap().len(),
        2
    );

    let control = temp.path().join("control/sift-control-state.json");
    assert!(control.is_file());
    assert_eq!(
        control.metadata().unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(!temp.path().join("sift-control-state.json").exists());
}

#[tokio::test]
async fn empty_and_oversize_batches_fail_without_writing() {
    let temp = tempfile::tempdir().unwrap();
    let state = ServiceState::open(temp.path()).unwrap();

    let empty = state
        .append_batch(Vec::new())
        .await
        .unwrap_err()
        .to_string();
    assert!(empty.contains("must not be empty"), "{empty}");

    let oversize = state
        .append_batch(
            (0..5_000)
                .map(|index| log(&format!("large-{index}"), "x".repeat(300)))
                .collect(),
        )
        .await
        .unwrap_err()
        .to_string();
    assert!(oversize.contains("1 MiB"), "{oversize}");
    assert!(state
        .journal()
        .query(EventQuery::default())
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn a_large_ingest_request_is_split_into_bounded_commits() {
    let temp = tempfile::tempdir().unwrap();
    let state = ServiceState::open(temp.path()).unwrap();
    let events = (0..300)
        .map(|index| log(&format!("split-{index}"), "x".repeat(4_000)))
        .collect::<Vec<_>>();

    let results = state.append_events(events).await.unwrap();
    assert_eq!(results.len(), 300);
    assert!(results
        .windows(2)
        .any(|pair| pair[0].commit_index != pair[1].commit_index));
    assert_eq!(
        state
            .journal()
            .query(EventQuery {
                limit: 1_000,
                ..EventQuery::default()
            })
            .unwrap()
            .len(),
        300
    );
}

#[tokio::test]
async fn one_signal_batch_has_one_durable_wal_frame_and_one_fsync() {
    let temp = tempfile::tempdir().unwrap();
    let state = ServiceState::open(temp.path()).unwrap();
    let events = (0..100)
        .map(|index| log(&format!("batched-{index}"), format!("message-{index}")))
        .collect::<Vec<_>>();

    let results = state.append_batch(events).await.unwrap();
    assert_eq!(results.len(), 100);
    assert!(results
        .iter()
        .all(|result| result.commit_index == results[0].commit_index));

    let frames = storage_durable::FramedLogReader::read_frames(
        temp.path().join("wal/logs/events.framed"),
        0,
    )
    .unwrap();
    assert_eq!(frames.len(), 1, "one Raft batch must be one WAL frame");
    assert!(state
        .render_metrics()
        .contains("sift_journal_fsync_total 1"));

    drop(state);
    let reopened = ServiceState::open(temp.path()).unwrap();
    assert_eq!(
        reopened
            .journal()
            .query(EventQuery {
                limit: 1_000,
                ..EventQuery::default()
            })
            .unwrap()
            .len(),
        100
    );
}

#[tokio::test]
async fn restart_keeps_raft_index_separate_from_multi_event_cursor() {
    let temp = tempfile::tempdir().unwrap();
    let state = ServiceState::open(temp.path()).unwrap();
    let first = state
        .append_batch(
            (0..100)
                .map(|index| log(&format!("restart-batch-{index}"), "before restart"))
                .collect(),
        )
        .await
        .unwrap();
    assert!(first.iter().all(|result| result.commit_index == 1));
    assert_eq!(
        state
            .journal()
            .query(EventQuery {
                limit: 1_000,
                ..EventQuery::default()
            })
            .unwrap()
            .len(),
        100
    );
    drop(state);

    let reopened = ServiceState::open(temp.path()).unwrap();
    let second = reopened
        .append_batch(vec![log("restart-next", "after restart")])
        .await
        .unwrap();
    assert_eq!(
        second[0].commit_index, 2,
        "one Raft entry may contain many events, so event cursor 100 must not become applied index 100"
    );
    assert_eq!(
        reopened
            .journal()
            .query(EventQuery {
                limit: 1_000,
                ..EventQuery::default()
            })
            .unwrap()
            .len(),
        101
    );
}

#[tokio::test]
async fn concurrent_requests_share_the_ten_millisecond_batch_window() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(ServiceState::open(temp.path()).unwrap());
    let barrier = Arc::new(tokio::sync::Barrier::new(3));

    let first = {
        let state = state.clone();
        let barrier = barrier.clone();
        tokio::spawn(async move {
            barrier.wait().await;
            state.append_events(vec![log("window-1", "one")]).await
        })
    };
    let second = {
        let state = state.clone();
        let barrier = barrier.clone();
        tokio::spawn(async move {
            barrier.wait().await;
            state.append_events(vec![log("window-2", "two")]).await
        })
    };
    barrier.wait().await;

    let first = first.await.unwrap().unwrap();
    let second = second.await.unwrap().unwrap();
    assert_eq!(first[0].commit_index, second[0].commit_index);
    let frames = storage_durable::FramedLogReader::read_frames(
        temp.path().join("wal/logs/events.framed"),
        0,
    )
    .unwrap();
    assert_eq!(frames.len(), 1);
}
