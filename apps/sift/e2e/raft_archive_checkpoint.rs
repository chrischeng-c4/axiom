use std::{collections::BTreeMap, sync::Arc, time::Duration};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use raft_runtime::RaftStateMachine;
use sha2::{Digest, Sha256};
use sift::{
    projection::{LogQuery, ProjectionRuntime, PROJECTION_LOGGING_STORE},
    storage::archive,
    DurableJournal, EventEnvelope, EventQuery, ServiceState, SignalKind,
};
use storage_object::ObjectStore;
use tower::ServiceExt;

fn log(index: u64) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "raft-checkpoint",
        "test",
        format!("checkpoint-{index}"),
        SignalKind::Log,
        serde_json::json!({"message":"manifest backed raft checkpoint"}),
    );
    let occurred_at = chrono::DateTime::parse_from_rfc3339("2026-08-01T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc)
        + chrono::Duration::seconds(index as i64);
    event.occurred_at = occurred_at.to_rfc3339();
    event.observed_at.clone_from(&event.occurred_at);
    event.resource = BTreeMap::from([("service.name".into(), "raft-checkpoint".into())]);
    event
}

fn log_at(index: u64, occurred_at: chrono::DateTime<chrono::Utc>) -> EventEnvelope {
    let mut event = log(index);
    event.occurred_at = occurred_at.to_rfc3339();
    event.observed_at.clone_from(&event.occurred_at);
    event
}

#[tokio::test]
async fn committed_retention_fence_survives_restart_and_blocks_only_queries() {
    let data = tempfile::tempdir().unwrap();
    let journal = Arc::new(DurableJournal::open(data.path()).unwrap());
    let state_machine = sift::durability::SiftStateMachine::new(journal.clone());
    let command = serde_json::to_vec(&serde_json::json!({
        "kind": "retention_fence",
        "fence": {
            "source_manifest_uri": "gs://sift-retention/fenced-manifest.json",
            "source_manifest_sha256": "a".repeat(64),
            "target_generation": 1,
            "evaluate_at": "2026-09-01T00:00:00Z"
        }
    }))
    .unwrap();
    state_machine.apply_local(1, &command).unwrap();
    drop(state_machine);
    drop(journal);

    let state = Arc::new(ServiceState::open(data.path()).unwrap());
    state.journal().append(log(1)).unwrap();
    let app = sift::router(state);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/query")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "version": 1,
                        "project": "raft-checkpoint",
                        "signal": {"kind": "logs"},
                        "mode": "sync"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()["retry-after"], "1");
    let body = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["error"], "retention_checkpoint_pending");
    assert_eq!(body["retryable"], true);
}

#[test]
fn committed_local_segments_make_a_small_voter_checkpoint() {
    let source_dir = tempfile::tempdir().unwrap();
    let source_journal = Arc::new(DurableJournal::open(source_dir.path()).unwrap());
    let source = sift::durability::SiftStateMachine::new(source_journal.clone());
    for index in 1..=200_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        source.apply(index, &command).unwrap();
        source.take_append_outcomes(index);
    }
    archive::archive_journal_local(&source_journal).unwrap();
    let mut checkpoint = Vec::new();
    source.snapshot_at(200, &mut checkpoint).unwrap();
    assert!(checkpoint.starts_with(b"SIFTLCP1"));
    assert!(checkpoint.len() < 4_096);

    let follower_dir = tempfile::tempdir().unwrap();
    let follower_journal = Arc::new(DurableJournal::open(follower_dir.path()).unwrap());
    let follower = sift::durability::SiftStateMachine::new(follower_journal.clone());
    for index in 1..=200_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        follower.apply(index, &command).unwrap();
        follower.take_append_outcomes(index);
    }
    follower.restore(&mut checkpoint.as_slice()).unwrap();
    let follower_wal = follower_dir.path().join("wal/logs/events.framed");
    assert!(
        storage_durable::FramedLogReader::read_frames(follower_wal, 0)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn resident_checkpoint_is_small_and_refuses_a_behind_voter() {
    let source_dir = tempfile::tempdir().unwrap();
    let source_journal = Arc::new(DurableJournal::open(source_dir.path()).unwrap());
    let source = sift::durability::SiftStateMachine::new(source_journal.clone());
    for index in 1..=100_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        source.apply(index, &command).unwrap();
        source.take_append_outcomes(index);
    }
    source.prepare_resident_checkpoint(100, 100).unwrap();
    let mut checkpoint = Vec::new();
    source.snapshot_at(100, &mut checkpoint).unwrap();
    assert!(checkpoint.starts_with(b"SIFTRSD1"));
    assert!(checkpoint.len() < 4_096);

    let caught_up_dir = tempfile::tempdir().unwrap();
    let caught_up_journal = Arc::new(DurableJournal::open(caught_up_dir.path()).unwrap());
    let caught_up = sift::durability::SiftStateMachine::new(caught_up_journal);
    for index in 1..=100_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        caught_up.apply(index, &command).unwrap();
        caught_up.take_append_outcomes(index);
    }
    caught_up.restore(&mut checkpoint.as_slice()).unwrap();

    let behind_dir = tempfile::tempdir().unwrap();
    let behind_journal = Arc::new(DurableJournal::open(behind_dir.path()).unwrap());
    let behind = sift::durability::SiftStateMachine::new(behind_journal);
    for index in 1..=99_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        behind.apply(index, &command).unwrap();
        behind.take_append_outcomes(index);
    }
    assert!(behind.restore(&mut checkpoint.as_slice()).is_err());
}

#[tokio::test]
async fn resident_checkpoint_carries_the_retention_fence_to_a_voter() {
    let source_dir = tempfile::tempdir().unwrap();
    let source_journal = Arc::new(DurableJournal::open(source_dir.path()).unwrap());
    let source = sift::durability::SiftStateMachine::new(source_journal);
    let append = serde_json::to_vec(&serde_json::json!({
        "kind":"append_events",
        "events":[log(1)]
    }))
    .unwrap();
    source.apply(1, &append).unwrap();
    source.take_append_outcomes(1);
    let fence = serde_json::to_vec(&serde_json::json!({
        "kind": "retention_fence",
        "fence": {
            "source_manifest_uri": "gs://sift-retention/resident-manifest.json",
            "source_manifest_sha256": "b".repeat(64),
            "target_generation": 1,
            "evaluate_at": "2026-09-01T00:00:00Z"
        }
    }))
    .unwrap();
    source.apply(2, &fence).unwrap();
    source.prepare_resident_checkpoint(2, 1).unwrap();
    let mut checkpoint = Vec::new();
    source.snapshot_at(2, &mut checkpoint).unwrap();

    let follower_dir = tempfile::tempdir().unwrap();
    let follower_journal = Arc::new(DurableJournal::open(follower_dir.path()).unwrap());
    let follower = sift::durability::SiftStateMachine::new(follower_journal.clone());
    follower.apply(1, &append).unwrap();
    follower.take_append_outcomes(1);
    follower.restore(&mut checkpoint.as_slice()).unwrap();
    drop(follower);
    drop(follower_journal);

    let state = Arc::new(ServiceState::open(follower_dir.path()).unwrap());
    let response = sift::router(state)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/query")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "version": 1,
                        "project": "raft-checkpoint",
                        "signal": {"kind": "logs"},
                        "mode": "sync"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn sealed_raft_prefix_can_compact_while_newer_ingest_continues() {
    let source_dir = tempfile::tempdir().unwrap();
    let source_journal = Arc::new(DurableJournal::open(source_dir.path()).unwrap());
    let source = sift::durability::SiftStateMachine::new(source_journal.clone());
    for index in 1..=100_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        source.apply(index, &command).unwrap();
        source.take_append_outcomes(index);
    }

    let (applied_index, raw_cursor, segments) = source.capture_archive_prefix().unwrap();
    assert_eq!((applied_index, raw_cursor), (100, 100));

    for index in 101..=120_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        source.apply(index, &command).unwrap();
        source.take_append_outcomes(index);
    }
    archive::archive_journal_local_captured(&source_journal, raw_cursor, segments).unwrap();
    source
        .prepare_local_checkpoint(applied_index, raw_cursor)
        .unwrap();

    let mut checkpoint = Vec::new();
    source.snapshot_at(applied_index, &mut checkpoint).unwrap();
    assert!(checkpoint.starts_with(b"SIFTLCP1"));
    assert_eq!(source.applied_index(), 120);
    assert_eq!(source_journal.total_event_count(), 120);

    let follower_dir = tempfile::tempdir().unwrap();
    let follower_journal = Arc::new(DurableJournal::open(follower_dir.path()).unwrap());
    let follower = sift::durability::SiftStateMachine::new(follower_journal.clone());
    for index in 1..=120_u64 {
        let command = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(index)]
        }))
        .unwrap();
        follower.apply(index, &command).unwrap();
        follower.take_append_outcomes(index);
    }
    follower.restore(&mut checkpoint.as_slice()).unwrap();
    assert_eq!(follower.applied_index(), 120);
    assert_eq!(follower_journal.total_event_count(), 120);
    let tail = follower_journal
        .query(EventQuery {
            after: 119,
            limit: 10,
            ..EventQuery::default()
        })
        .unwrap();
    assert_eq!(tail[0].event.event_id, "checkpoint-120");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn committed_gcs_manifest_makes_a_small_restorable_raft_checkpoint() {
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

    tokio::task::spawn_blocking(|| {
        let source_dir = tempfile::tempdir().unwrap();
        let source_journal = Arc::new(DurableJournal::open(source_dir.path()).unwrap());
        let source = sift::durability::SiftStateMachine::new(source_journal.clone());
        for index in 1..=200_u64 {
            let command = serde_json::to_vec(&serde_json::json!({
                "kind":"append_events",
                "events":[log(index)]
            }))
            .unwrap();
            source.apply(index, &command).unwrap();
            source.take_append_outcomes(index);
        }
        let receipt =
            archive::archive_journal_gcs(&source_journal, "gs://sift-checkpoint/raft").unwrap();
        let expected_manifest_hash = hex::encode(Sha256::digest(
            service_backup::fetch_backup_object(&receipt.manifest_uri).unwrap(),
        ));
        assert_eq!(expected_manifest_hash, receipt.manifest_sha256);

        let mut checkpoint = Vec::new();
        source.snapshot_at(200, &mut checkpoint).unwrap();
        assert!(checkpoint.starts_with(b"SIFTRCP1"));
        assert!(checkpoint.len() < 4_096);

        let follower_dir = tempfile::tempdir().unwrap();
        let follower_journal = Arc::new(DurableJournal::open(follower_dir.path()).unwrap());
        let follower = sift::durability::SiftStateMachine::new(follower_journal.clone());
        for index in 1..=220_u64 {
            let command = serde_json::to_vec(&serde_json::json!({
                "kind":"append_events",
                "events":[log(index)]
            }))
            .unwrap();
            follower.apply(index, &command).unwrap();
            follower.take_append_outcomes(index);
        }
        let follower_wal = follower_dir.path().join("wal/logs/events.framed");
        assert!(
            !storage_durable::FramedLogReader::read_frames(&follower_wal, 0)
                .unwrap()
                .is_empty()
        );
        follower.restore(&mut checkpoint.as_slice()).unwrap();
        assert_eq!(
            storage_durable::FramedLogReader::read_frames(&follower_wal, 0)
                .unwrap()
                .len(),
            20,
            "archive adoption must keep the valid suffix after the checkpoint"
        );
        assert_eq!(follower.applied_index(), 220);
        assert_eq!(follower_journal.total_event_count(), 220);
        assert_eq!(
            follower_journal
                .query(EventQuery {
                    after: 219,
                    limit: 10,
                    ..EventQuery::default()
                })
                .unwrap()[0]
                .event
                .event_id,
            "checkpoint-220"
        );

        let restore_dir = tempfile::tempdir().unwrap();
        let restore_journal = Arc::new(DurableJournal::open(restore_dir.path()).unwrap());
        let restored = sift::durability::SiftStateMachine::new(restore_journal.clone());
        restored.restore(&mut checkpoint.as_slice()).unwrap();
        assert_eq!(restored.applied_index(), 200);
        assert_eq!(restore_journal.total_event_count(), 200);
        let local_tail = restore_journal
            .query(EventQuery {
                after: 199,
                limit: 10,
                ..EventQuery::default()
            })
            .unwrap();
        assert!(
            local_tail.is_empty(),
            "a voter restore must not retain an entirely cold segment on its PVC"
        );
        let mut cold_tail = Vec::new();
        let replay = archive::replay_committed_events(
            restore_dir.path(),
            SignalKind::Log,
            "raft-checkpoint",
            Some("test"),
            None,
            None,
            |event| {
                if event.cursor > 199 {
                    cold_tail.push(event.event.event_id);
                }
                Ok(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(replay.replayed, 200);
        assert_eq!(cold_tail, vec!["checkpoint-200"]);

        let partial_dir = tempfile::tempdir().unwrap();
        let partial_journal = Arc::new(DurableJournal::open(partial_dir.path()).unwrap());
        let partial = sift::durability::SiftStateMachine::new(partial_journal.clone());
        let first = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log(1)]
        }))
        .unwrap();
        partial.apply(1, &first).unwrap();
        partial.take_append_outcomes(1);
        partial.restore(&mut checkpoint.as_slice()).unwrap();
        assert_eq!(partial.applied_index(), 200);
        assert_eq!(partial_journal.total_event_count(), 200);

        let delta_source_dir = tempfile::tempdir().unwrap();
        let delta_source_journal = Arc::new(DurableJournal::open(delta_source_dir.path()).unwrap());
        let delta_source = sift::durability::SiftStateMachine::new(delta_source_journal.clone());
        let delta_now = chrono::Utc::now();
        for (index, event) in [
            (1, log_at(1, delta_now - chrono::Duration::days(179))),
            (2, log_at(2, delta_now)),
        ] {
            let command = serde_json::to_vec(&serde_json::json!({
                "kind":"append_events",
                "events":[event]
            }))
            .unwrap();
            delta_source.apply(index, &command).unwrap();
            delta_source.take_append_outcomes(index);
        }
        archive::archive_journal_gcs(
            &delta_source_journal,
            "gs://sift-checkpoint/delta-no-restore",
        )
        .unwrap();
        let mut source_checkpoint = Vec::new();
        delta_source.snapshot_at(2, &mut source_checkpoint).unwrap();
        let delta_follower_dir = tempfile::tempdir().unwrap();
        let delta_follower_journal =
            Arc::new(DurableJournal::open(delta_follower_dir.path()).unwrap());
        let delta_follower =
            sift::durability::SiftStateMachine::new(delta_follower_journal.clone());
        delta_follower
            .restore(&mut source_checkpoint.as_slice())
            .unwrap();
        assert_eq!(delta_follower_journal.total_event_count(), 2);

        let delta_expiration = archive::expire_committed_events_at(
            &delta_source_journal,
            delta_now + chrono::Duration::days(2),
        )
        .unwrap();
        assert_eq!(delta_expiration.expired_events, 1);
        let delta_status = archive::committed_status(delta_source_dir.path())
            .unwrap()
            .unwrap();
        let delta_barrier = sift::durability::encode_archive_checkpoint_barrier_for_diagnostics(
            delta_status.retention_generation,
            delta_status.manifest_uri,
            delta_status.manifest_sha256,
        )
        .unwrap();
        delta_source.apply_local(3, &delta_barrier).unwrap();
        delta_follower.apply_local(3, &delta_barrier).unwrap();
        let target_root: archive::ArchiveManifest = serde_json::from_slice(
            &service_backup::fetch_backup_object(&delta_expiration.manifest_uri).unwrap(),
        )
        .unwrap();
        let target_manifest = archive::inspect_archive_catalog(&target_root).unwrap().0;
        let object_store = storage_object::GcsObjectStore::new("sift-checkpoint", "").unwrap();
        for segment in target_manifest {
            let key = segment
                .object_uri
                .strip_prefix("gs://sift-checkpoint/")
                .unwrap();
            object_store.delete(key).unwrap();
        }
        let mut delta_checkpoint = Vec::new();
        delta_source.snapshot_at(3, &mut delta_checkpoint).unwrap();
        delta_follower
            .restore(&mut delta_checkpoint.as_slice())
            .expect("a caught-up voter must apply the small retention delta without Parquet restore");
        assert_eq!(delta_follower_journal.total_event_count(), 1);
        assert_eq!(
            delta_follower_journal
                .query(EventQuery::default())
                .unwrap()
                .into_iter()
                .map(|event| event.event.event_id)
                .collect::<Vec<_>>(),
            vec!["checkpoint-2"]
        );

        let now = chrono::Utc::now();
        let retained_source_dir = tempfile::tempdir().unwrap();
        let retained_source_journal =
            Arc::new(DurableJournal::open(retained_source_dir.path()).unwrap());
        let retained_source =
            sift::durability::SiftStateMachine::new(retained_source_journal.clone());
        let retained_follower_dir = tempfile::tempdir().unwrap();
        let retained_follower_journal =
            Arc::new(DurableJournal::open(retained_follower_dir.path()).unwrap());
        let retained_follower =
            sift::durability::SiftStateMachine::new(retained_follower_journal.clone());
        for (index, event) in [
            (1, log_at(1, now - chrono::Duration::days(179))),
            (2, log_at(2, now)),
        ] {
            let source_command = serde_json::to_vec(&serde_json::json!({
                "kind":"append_events",
                "events":[event.clone()]
            }))
            .unwrap();
            retained_source.apply(index, &source_command).unwrap();
            retained_source.take_append_outcomes(index);
            let mut follower_event = event;
            if index == 2 {
                follower_event.payload["message"] =
                    serde_json::Value::String("split leader alternate row".into());
            }
            let follower_command = serde_json::to_vec(&serde_json::json!({
                "kind":"append_events",
                "events":[follower_event]
            }))
            .unwrap();
            retained_follower.apply(index, &follower_command).unwrap();
            retained_follower.take_append_outcomes(index);
        }
        let follower_projections = ProjectionRuntime::open(
            retained_follower_dir.path(),
            retained_follower_journal.clone(),
        )
        .unwrap();
        follower_projections
            .catch_up(PROJECTION_LOGGING_STORE)
            .unwrap();
        assert_eq!(
            follower_projections
                .query_logs(&LogQuery::for_project("raft-checkpoint"))
                .unwrap()
                .records
                .len(),
            2
        );

        archive::archive_journal_gcs(&retained_source_journal, "gs://sift-checkpoint/retention")
            .unwrap();
        let expiration = archive::expire_committed_events_at(
            &retained_source_journal,
            now + chrono::Duration::days(2),
        )
        .unwrap();
        assert_eq!(expiration.expired_events, 1);
        archive::archive_journal_gcs(
            &retained_follower_journal,
            "gs://sift-checkpoint/split-retention",
        )
        .unwrap();
        let alternate_expiration = archive::expire_committed_events_at(
            &retained_follower_journal,
            now + chrono::Duration::days(2),
        )
        .unwrap();
        assert_eq!(alternate_expiration.expired_events, 1);
        assert_ne!(alternate_expiration.manifest_uri, expiration.manifest_uri);
        let retention_status = archive::committed_status(retained_source_dir.path())
            .unwrap()
            .unwrap();
        let barrier = sift::durability::encode_archive_checkpoint_barrier_for_diagnostics(
            retention_status.retention_generation,
            retention_status.manifest_uri.clone(),
            retention_status.manifest_sha256.clone(),
        )
        .unwrap();
        retained_source.apply(3, &barrier).unwrap();
        retained_follower.apply(3, &barrier).unwrap();
        let suffix = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log_at(3, now)]
        }))
        .unwrap();
        retained_follower.apply(4, &suffix).unwrap();
        retained_follower.take_append_outcomes(4);

        let crash_dir = tempfile::tempdir().unwrap();
        let crash_journal = Arc::new(DurableJournal::open(crash_dir.path()).unwrap());
        let crash_state = sift::durability::SiftStateMachine::new(crash_journal.clone());
        for (index, event) in [
            (1, log_at(1, now - chrono::Duration::days(179))),
            (2, log_at(2, now)),
        ] {
            let command = serde_json::to_vec(&serde_json::json!({
                "kind":"append_events",
                "events":[event]
            }))
            .unwrap();
            crash_state.apply(index, &command).unwrap();
            crash_state.take_append_outcomes(index);
        }
        let crash_projections =
            ProjectionRuntime::open(crash_dir.path(), crash_journal.clone()).unwrap();
        crash_projections
            .catch_up(PROJECTION_LOGGING_STORE)
            .unwrap();
        drop(crash_projections);
        drop(crash_state);
        drop(crash_journal);
        std::fs::copy(
            retained_source_dir
                .path()
                .join("control/archive-commit.json"),
            crash_dir.path().join("control/archive-commit.json"),
        )
        .unwrap();
        let crash_recovered = Arc::new(DurableJournal::open(crash_dir.path()).unwrap());
        let crash_recovered_projections =
            ProjectionRuntime::open(crash_dir.path(), crash_recovered.clone()).unwrap();
        assert_eq!(crash_recovered.total_event_count(), 1);
        assert_eq!(
            crash_recovered_projections
                .query_logs(&LogQuery::for_project("raft-checkpoint"))
                .unwrap()
                .records
                .iter()
                .map(|record| record.event_id.as_str())
                .collect::<Vec<_>>(),
            vec!["checkpoint-2"]
        );

        let mut retention_checkpoint = Vec::new();
        retained_source
            .snapshot_at(3, &mut retention_checkpoint)
            .unwrap();

        let ahead_dir = tempfile::tempdir().unwrap();
        let ahead_journal = Arc::new(DurableJournal::open(ahead_dir.path()).unwrap());
        let ahead = sift::durability::SiftStateMachine::new(ahead_journal.clone());
        for (index, event) in [
            (1, log_at(1, now - chrono::Duration::days(179))),
            (2, log_at(2, now)),
        ] {
            let command = serde_json::to_vec(&serde_json::json!({
                "kind":"append_events",
                "events":[event]
            }))
            .unwrap();
            ahead.apply(index, &command).unwrap();
            ahead.take_append_outcomes(index);
        }
        let ahead_fence = serde_json::to_vec(&serde_json::json!({
            "kind": "retention_fence",
            "fence": {
                "source_manifest_uri": retention_status.manifest_uri,
                "source_manifest_sha256": retention_status.manifest_sha256,
                "target_generation": retention_status.retention_generation,
                "evaluate_at": (now + chrono::Duration::days(2)).to_rfc3339()
            }
        }))
        .unwrap();
        ahead.apply(3, &ahead_fence).unwrap();
        ahead.apply(4, &suffix).unwrap();
        ahead.take_append_outcomes(4);
        assert!(ahead.retention_fence_pending_for_diagnostics());
        ahead
            .restore(&mut retention_checkpoint.as_slice())
            .unwrap();
        assert!(
            !ahead.retention_fence_pending_for_diagnostics(),
            "an authoritative completed retention generation must clear an older fence even when the follower has a newer append index"
        );
        assert_eq!(ahead_journal.query(EventQuery::default()).unwrap().len(), 2);

        let follower_wal = retained_follower_dir.path().join("wal/logs/events.framed");
        let wal_frames_before =
            storage_durable::FramedLogReader::read_frames(&follower_wal, 0).unwrap();
        assert!(!wal_frames_before.is_empty());
        let dedupe_root = retained_follower_dir.path().join("indexes/dedupe");
        std::fs::remove_dir_all(&dedupe_root).unwrap();
        std::fs::write(&dedupe_root, b"injected late rebuild failure").unwrap();
        let error = retained_follower
            .restore(&mut retention_checkpoint.as_slice())
            .expect_err("late dedupe failure must refuse the archive checkpoint");
        assert!(!error.to_string().is_empty());
        assert_eq!(
            storage_durable::FramedLogReader::read_frames(&follower_wal, 0).unwrap(),
            wal_frames_before,
            "a late restore failure must retain the recoverable WAL"
        );
        assert!(retained_follower_journal
            .query(EventQuery::default())
            .unwrap_err()
            .to_string()
            .contains("requires archive recovery"));
        std::fs::remove_file(&dedupe_root).unwrap();
        std::fs::create_dir(&dedupe_root).unwrap();
        drop(follower_projections);
        drop(retained_follower);
        drop(retained_follower_journal);

        let restarted_journal =
            Arc::new(DurableJournal::open(retained_follower_dir.path()).unwrap());
        let restarted = sift::durability::SiftStateMachine::new(restarted_journal.clone());
        let restarted_projections =
            ProjectionRuntime::open(retained_follower_dir.path(), restarted_journal.clone())
                .unwrap();
        restarted
            .restore(&mut retention_checkpoint.as_slice())
            .unwrap();
        assert!(
            retained_follower_dir
                .path()
                .join("control/archive-gc-pending.json")
                .exists(),
            "a voter must inherit the manifest cleanup plan before it can become leader"
        );

        assert_eq!(restarted_journal.total_event_count(), 2);
        assert_eq!(
            restarted_journal
                .query(EventQuery {
                    signal: Some(SignalKind::Log),
                    after: 0,
                    limit: 10,
                })
                .unwrap()
                .into_iter()
                .map(|event| event.event.event_id)
                .collect::<Vec<_>>(),
            vec!["checkpoint-2", "checkpoint-3"]
        );
        assert_eq!(
            restarted_journal.query(EventQuery::default()).unwrap()[0]
                .event
                .payload["message"],
            "manifest backed raft checkpoint",
            "same-generation manifest identity changes require full reconciliation"
        );
        assert_eq!(
            restarted_projections
                .query_logs(&LogQuery::for_project("raft-checkpoint"))
                .unwrap()
                .records
                .iter()
                .map(|record| record.event_id.as_str())
                .collect::<Vec<_>>(),
            vec!["checkpoint-2", "checkpoint-3"]
        );

        let carried = archive::archive_journal_gcs(
            &retained_source_journal,
            "gs://sift-checkpoint/retention",
        )
        .unwrap();
        let carried_gc = archive::inspect_archive_gc_plan(&carried.manifest).unwrap();
        assert!(
            !carried_gc.is_empty(),
            "an ordinary archive must carry unfinished cleanup work"
        );
        let rebound_pending: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                retained_source_dir
                    .path()
                    .join("control/archive-gc-pending.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            rebound_pending["replacement_manifest_uri"],
            carried.manifest_uri
        );

        assert!(
            archive::finalize_archive_gc_after_checkpoint(retained_source_dir.path()).unwrap() > 0
        );
        assert!(!retained_source_dir
            .path()
            .join("control/archive-gc-pending.json")
            .exists());
        let next = serde_json::to_vec(&serde_json::json!({
            "kind":"append_events",
            "events":[log_at(3, now)]
        }))
        .unwrap();
        retained_source.apply(4, &next).unwrap();
        retained_source.take_append_outcomes(4);
        let next_archive = archive::archive_journal_gcs(
            &retained_source_journal,
            "gs://sift-checkpoint/retention",
        )
        .unwrap();
        let next_gc = archive::inspect_archive_gc_plan(&next_archive.manifest).unwrap();
        assert!(next_gc.contains(&carried.manifest_uri));
        assert!(next_gc.iter().all(|uri| {
            uri == &carried.manifest_uri
                || uri.contains("/gc-plan/pages/")
                || uri.contains("/catalog/pages/")
        }), "a later archive must clean only its predecessor root and retired catalog pages after the old plan completed: {next_gc:?}");
        retained_source
            .prepare_archive_checkpoint_without_gc(4, 3)
            .unwrap();
        let mut no_gc_checkpoint = Vec::new();
        retained_source
            .snapshot_at(4, &mut no_gc_checkpoint)
            .unwrap();
        let no_gc_dir = tempfile::tempdir().unwrap();
        let no_gc_journal = Arc::new(DurableJournal::open(no_gc_dir.path()).unwrap());
        let no_gc_follower = sift::durability::SiftStateMachine::new(no_gc_journal);
        no_gc_follower
            .restore(&mut no_gc_checkpoint.as_slice())
            .unwrap();
        assert!(
            !no_gc_dir
                .path()
                .join("control/archive-gc-pending.json")
                .exists(),
            "a quorum-only archive checkpoint must not carry deletion authority"
        );
        retained_source.prepare_archive_checkpoint(4, 3).unwrap();
        let mut next_checkpoint = Vec::new();
        retained_source
            .snapshot_at(4, &mut next_checkpoint)
            .unwrap();
        restarted.restore(&mut next_checkpoint.as_slice()).unwrap();
        let follower_pending_bytes = std::fs::read(
            retained_follower_dir
                .path()
                .join("control/archive-gc-pending.json"),
        )
        .unwrap();
        assert!(follower_pending_bytes.len() < 64 * 1024);
        let follower_pending: serde_json::Value =
            serde_json::from_slice(&follower_pending_bytes).unwrap();
        assert_eq!(
            follower_pending["replacement_manifest_uri"],
            next_archive.manifest_uri
        );
        assert_eq!(
            follower_pending["gc_plan_uri"],
            serde_json::json!(next_archive.manifest.gc_plan_uri),
            "a follower must inherit the immutable paged cleanup plan"
        );
        assert_eq!(
            follower_pending["gc_plan_root"]["entry_count"],
            serde_json::json!(next_gc.len()),
        );
        assert!(follower_pending["cursor"].is_null());
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
