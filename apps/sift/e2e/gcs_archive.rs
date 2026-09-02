// HANDWRITE-BEGIN gap="sift-vat-gcs-archive-tests" tracker="1659" reason="Run Vat Cloud Storage emulator with real service-backup GCS requests and verify archive/restore hash equality."
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{SecondsFormat, Utc};
use sha2::Digest;
use sift::{
    durability, storage::archive, DurableJournal, EventEnvelope, EventQuery, MetricPoint,
    MetricTemporality, SignalKind, StoredEvent,
};
use storage_object::{ObjectStore, PutCondition};

fn stored(cursor: u64) -> StoredEvent {
    let signal = match cursor % 3 {
        1 => SignalKind::Log,
        2 => SignalKind::Metric,
        _ => SignalKind::Span,
    };
    let mut event = EventEnvelope::new(
        format!("archive-{cursor}"),
        signal,
        serde_json::json!({"cursor": cursor, "signal": signal}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "archive-test".into())]);
    if signal == SignalKind::Metric {
        event.metric = Some(MetricPoint {
            name: "archive_test_total".into(),
            value: cursor as f64,
            stale: false,
            unit: None,
            temporality: MetricTemporality::Cumulative,
            exemplars: Vec::new(),
        });
    }
    StoredEvent {
        cursor,
        acknowledged_at: "2026-07-14T00:00:00Z".into(),
        event,
    }
}

fn event_id_set_digest(ids: &[&str]) -> String {
    let mut digest = [0u8; 32];
    for id in ids {
        let item: [u8; 32] = sha2::Sha256::digest(id.as_bytes()).into();
        for (slot, byte) in digest.iter_mut().zip(item) {
            *slot ^= byte;
        }
    }
    hex::encode(digest)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vat_gcs_archive_manifest_is_written_last_and_cold_restore_is_equal() {
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
        let journal = DurableJournal::open(source_dir.path()).unwrap();
        for cursor in 1..=5 {
            journal.append(stored(cursor).event).unwrap();
        }
        let source_events = journal.query(EventQuery::default()).unwrap();
        let source_layout: serde_json::Value =
            serde_json::from_slice(&std::fs::read(source_dir.path().join("layout.json")).unwrap())
                .unwrap();

        let receipt = archive::archive_journal_gcs(&journal, "gs://sift-test/domain-v1").unwrap();
        assert!(receipt.manifest_uri.ends_with("manifest.json"));
        assert_eq!(receipt.manifest.format_version, 10);
        let manifest_json = serde_json::to_value(&receipt.manifest).unwrap();
        assert_eq!(
            manifest_json["source_cluster_id"],
            source_layout["cluster_id"]
        );
        assert_eq!(manifest_json["source_node_id"], source_layout["node_id"]);
        assert_eq!(manifest_json["raft_snapshot_index"], 5);
        assert_eq!(manifest_json["event_count"], 5);
        assert!(manifest_json.get("segments").is_none());
        assert!(manifest_json.get("blobs").is_none());
        assert!(manifest_json.get("gc_object_uris").is_none());
        let root_bytes = service_backup::fetch_backup_object(&receipt.manifest_uri).unwrap();
        assert!(root_bytes.len() < 64 * 1024);
        assert!(
            std::fs::metadata(source_dir.path().join("control/archive-commit.json"))
                .unwrap()
                .len()
                < 64 * 1024,
            "the local recovery receipt must stay bounded"
        );
        assert_eq!(manifest_json["event_id_digest_algorithm"], "xor-sha256-v1");
        assert_eq!(
            manifest_json["event_id_sha256"],
            event_id_set_digest(&[
                "archive-1",
                "archive-2",
                "archive-3",
                "archive-4",
                "archive-5",
            ])
        );
        let (receipt_segments, _) = archive::inspect_archive_catalog(&receipt.manifest).unwrap();
        assert_eq!(
            receipt_segments.len(),
            3,
            "one initial shard per signal must produce one segment per signal"
        );
        assert!(receipt_segments
            .iter()
            .all(|segment| segment.object_uri.ends_with(".parquet")));
        assert_eq!(
            receipt_segments
                .iter()
                .map(|segment| segment.signal)
                .collect::<std::collections::BTreeSet<_>>(),
            [SignalKind::Log, SignalKind::Metric, SignalKind::Span]
                .into_iter()
                .collect()
        );
        for segment in &receipt_segments {
            let bytes = service_backup::fetch_backup_object(&segment.object_uri).unwrap();
            assert_eq!(&bytes[..4], b"PAR1");
            assert_eq!(&bytes[bytes.len() - 4..], b"PAR1");
            assert_eq!(
                hex::encode(sha2::Sha256::digest(&bytes)),
                segment.parquet_sha256
            );
        }
        for signal in ["logs", "metrics", "traces"] {
            let path = source_dir
                .path()
                .join("wal")
                .join(signal)
                .join("events.framed");
            assert!(
                storage_durable::FramedLogReader::read_frames(path, 0)
                    .unwrap()
                    .is_empty(),
                "committed archive must compact the {signal} WAL"
            );
        }
        drop(journal);
        let reopened_source = DurableJournal::open(source_dir.path()).unwrap();
        assert_eq!(
            reopened_source.query(EventQuery::default()).unwrap(),
            source_events
        );
        drop(reopened_source);

        let restore_dir = tempfile::tempdir().unwrap();
        let restored = archive::restore_gcs(&receipt.manifest_uri, restore_dir.path()).unwrap();
        assert_eq!(restored.segment_count, receipt.manifest.segment_count);
        assert!(
            !restore_dir.path().join("snapshots/journal.json").exists(),
            "cold restore must not create a second full-journal JSON snapshot"
        );
        let reopened = DurableJournal::open(restore_dir.path()).unwrap();
        let events = reopened.query(EventQuery::default()).unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].event.event_id, "archive-1");
        assert_eq!(events[4].event.event_id, "archive-5");
        assert_eq!(events, source_events);
        drop(reopened);
        let restored_layout: serde_json::Value =
            serde_json::from_slice(&std::fs::read(restore_dir.path().join("layout.json")).unwrap())
                .unwrap();
        assert_ne!(restored_layout["cluster_id"], source_layout["cluster_id"]);
        assert_eq!(restored_layout["restored_from"], receipt.manifest_uri);
        assert!(
            archive::bootstrap_gcs_if_needed(&receipt.manifest_uri, restore_dir.path())
                .unwrap()
                .is_none(),
            "a pod restart must reuse the completed bootstrap"
        );

        let incremental_source = DurableJournal::open(source_dir.path()).unwrap();
        for cursor in 6..=7 {
            incremental_source.append(stored(cursor).event).unwrap();
        }
        let next =
            archive::archive_journal_gcs(&incremental_source, "gs://sift-test/domain-v1").unwrap();
        assert_eq!(next.manifest.event_count, 7);
        assert_eq!(next.manifest.raft_snapshot_index, 7);
        assert!(
            std::fs::metadata(source_dir.path().join("control/archive-commit.json"))
                .unwrap()
                .len()
                < 64 * 1024
        );
        assert!(
            std::fs::metadata(source_dir.path().join("control/archive-gc-pending.json"))
                .unwrap()
                .len()
                < 64 * 1024,
            "incremental archive cleanup progress must stay in a fixed-size root"
        );
        assert!(!archive::inspect_archive_gc_plan(&next.manifest)
            .unwrap()
            .is_empty());
        let (next_segments, _) = archive::inspect_archive_catalog(&next.manifest).unwrap();
        for prior in &receipt_segments {
            let reused = next_segments
                .iter()
                .find(|segment| segment.source.segment_id == prior.source.segment_id)
                .expect("the next manifest must keep every prior immutable segment");
            assert_eq!(
                reused.object_uri, prior.object_uri,
                "a later archive must not upload an unchanged segment again"
            );
        }
        let incremental_restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&next.manifest_uri, incremental_restore.path()).unwrap();
        let incremental = DurableJournal::open(incremental_restore.path()).unwrap();
        assert_eq!(incremental.query(EventQuery::default()).unwrap().len(), 7);
        drop(incremental);
        drop(incremental_source);

        let nonempty = tempfile::tempdir().unwrap();
        let marker = nonempty.path().join("do-not-overwrite");
        std::fs::write(&marker, b"owned").unwrap();
        let error = archive::restore_gcs(&receipt.manifest_uri, nonempty.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("empty data directory"));
        assert_eq!(std::fs::read(marker).unwrap(), b"owned");
        assert!(!nonempty.path().join("layout.json").exists());

        let interrupted_segment = &receipt_segments[0];
        let interrupted_bytes =
            service_backup::fetch_backup_object(&interrupted_segment.object_uri).unwrap();
        let interrupted_key = interrupted_segment
            .object_uri
            .strip_prefix("gs://sift-test/")
            .unwrap();
        let interrupted_store = storage_object::GcsObjectStore::new("sift-test", "").unwrap();
        interrupted_store.delete(interrupted_key).unwrap();
        let interrupted_restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&receipt.manifest_uri, interrupted_restore.path())
            .expect_err("a missing segment must interrupt cold restore");
        assert!(interrupted_restore
            .path()
            .join(".sift-restore.json")
            .exists());
        assert!(interrupted_restore
            .path()
            .join(".sift-restore-stage")
            .exists());
        interrupted_store
            .put(
                interrupted_key,
                &interrupted_bytes,
                "application/vnd.apache.parquet",
                PutCondition::Any,
            )
            .unwrap();
        assert!(archive::bootstrap_gcs_if_needed(
            &receipt.manifest_uri,
            interrupted_restore.path()
        )
        .unwrap()
        .is_some());
        assert!(!interrupted_restore
            .path()
            .join(".sift-restore.json")
            .exists());
        assert!(!interrupted_restore
            .path()
            .join(".sift-restore-stage")
            .exists());
        let resumed_restore = DurableJournal::open(interrupted_restore.path()).unwrap();
        assert_eq!(resumed_restore.total_event_count(), 5);
        drop(resumed_restore);

        let destination =
            service_backup::BackupDestination::from_uri("gs://sift-test/domain-v1").unwrap();
        let sink = service_backup::GcsSink::from_destination(&destination).unwrap();
        assert!(service_backup::BackupSink::prune(&sink, 0).unwrap() >= 4);

        let concurrent_dir = tempfile::tempdir().unwrap();
        let concurrent = Arc::new(DurableJournal::open(concurrent_dir.path()).unwrap());
        concurrent.append(stored(1).event).unwrap();
        let writer_journal = concurrent.clone();
        let writer = std::thread::spawn(move || {
            for cursor in 2..=120 {
                writer_journal.append(stored(cursor).event).unwrap();
            }
        });
        for _ in 0..4 {
            archive::archive_journal_gcs(&concurrent, "gs://sift-test/concurrent-prefix").unwrap();
        }
        writer.join().unwrap();
        let final_receipt =
            archive::archive_journal_gcs(&concurrent, "gs://sift-test/concurrent-prefix").unwrap();
        assert_eq!(final_receipt.manifest.raft_snapshot_index, 120);
        assert_eq!(final_receipt.manifest.event_count, 120);
        let concurrent_restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&final_receipt.manifest_uri, concurrent_restore.path()).unwrap();
        let restored_concurrent = DurableJournal::open(concurrent_restore.path()).unwrap();
        assert_eq!(
            restored_concurrent
                .query(EventQuery {
                    limit: 200,
                    ..EventQuery::default()
                })
                .unwrap()
                .len(),
            120
        );

        let bounded_source_dir = tempfile::tempdir().unwrap();
        let bounded_source = DurableJournal::open(bounded_source_dir.path()).unwrap();
        let mut cold = stored(1).event;
        let cold_time = Utc::now() - chrono::Duration::days(31);
        cold.occurred_at = cold_time.to_rfc3339_opts(SecondsFormat::Nanos, true);
        cold.observed_at.clone_from(&cold.occurred_at);
        cold.payload["attachment_base64"] =
            serde_json::Value::String(BASE64.encode(vec![7_u8; 2 * 1024 * 1024]));
        let cold_project = cold.project.clone();
        let cold_environment = cold.environment.clone();
        bounded_source.append(cold).unwrap();
        bounded_source.append(stored(2).event).unwrap();
        assert_eq!(bounded_source.storage().blob_paths().unwrap().len(), 1);
        let bounded_receipt =
            archive::archive_journal_gcs(&bounded_source, "gs://sift-test/bounded-restore")
                .unwrap();

        let bounded_restore_dir = tempfile::tempdir().unwrap();
        archive::restore_gcs(&bounded_receipt.manifest_uri, bounded_restore_dir.path()).unwrap();
        let bounded_restore = DurableJournal::open(bounded_restore_dir.path()).unwrap();
        assert_eq!(bounded_restore.total_event_count(), 2);
        assert_eq!(
            bounded_restore
                .query(EventQuery::default())
                .unwrap()
                .into_iter()
                .map(|event| event.event.event_id)
                .collect::<Vec<_>>(),
            vec!["archive-2"],
            "fresh restore must materialize only the 30-day hot set"
        );
        assert!(
            bounded_restore.storage().blob_paths().unwrap().is_empty(),
            "a blob referenced only by cold data must stay in GCS"
        );
        let duplicate = bounded_restore.append(stored(1).event).unwrap();
        assert!(
            duplicate.duplicate,
            "cold event IDs must remain deduplicated"
        );
        let mut cold_ids = Vec::new();
        let replay = archive::replay_committed_events(
            bounded_restore_dir.path(),
            SignalKind::Log,
            &cold_project,
            Some(&cold_environment),
            None,
            None,
            |event| {
                cold_ids.push(event.event.event_id);
                Ok(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(replay.replayed, 1);
        assert_eq!(cold_ids, ["archive-1"]);

        let rebuild_source_dir = tempfile::tempdir().unwrap();
        let rebuild_source = Arc::new(DurableJournal::open(rebuild_source_dir.path()).unwrap());
        let rebuild_machine = durability::SiftStateMachine::new(rebuild_source.clone());
        let recent_acceptance = Utc::now();
        let old_acceptance = recent_acceptance - chrono::Duration::hours(12);
        let mut old_event = stored(1).event;
        old_event.event_id = "dedupe-rebuild-old".into();
        old_event.occurred_at = old_acceptance.to_rfc3339_opts(SecondsFormat::Nanos, true);
        old_event.observed_at.clone_from(&old_event.occurred_at);
        let old_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![old_event],
            &old_acceptance.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        rebuild_machine.apply_local(1, &old_command).unwrap();
        rebuild_machine.take_append_outcomes(1).unwrap();
        let old_receipt =
            archive::archive_journal_gcs(&rebuild_source, "gs://sift-test/recent-rebuild").unwrap();
        let (old_segments, _) = archive::inspect_archive_catalog(&old_receipt.manifest).unwrap();
        assert_eq!(old_segments.len(), 1);

        let mut recent_event = stored(1).event;
        recent_event.event_id = "dedupe-rebuild-recent".into();
        let recent_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![recent_event.clone()],
            &recent_acceptance.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        rebuild_machine.apply_local(2, &recent_command).unwrap();
        rebuild_machine.take_append_outcomes(2).unwrap();
        archive::archive_journal_gcs(&rebuild_source, "gs://sift-test/recent-rebuild").unwrap();

        let old_segment = &old_segments[0];
        let old_key = old_segment
            .object_uri
            .strip_prefix("gs://sift-test/")
            .unwrap();
        let object_store = storage_object::GcsObjectStore::new("sift-test", "").unwrap();
        object_store.delete(old_key).unwrap();
        let old_cache = rebuild_source_dir
            .path()
            .join("archive-cache")
            .join(format!("{}.parquet", old_segment.parquet_sha256));
        if old_cache.exists() {
            std::fs::remove_file(old_cache).unwrap();
        }
        drop(rebuild_machine);
        drop(rebuild_source);
        std::fs::remove_dir_all(rebuild_source_dir.path().join("indexes/dedupe")).unwrap();

        let rebuilt = DurableJournal::open(rebuild_source_dir.path()).unwrap();
        let duplicate = rebuilt.append(recent_event).unwrap();
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.cursor, 2);

        let intent_source = tempfile::tempdir().unwrap();
        let intent_journal = DurableJournal::open(intent_source.path()).unwrap();
        intent_journal.append(stored(1).event).unwrap();
        let uploaded =
            archive::archive_gcs(intent_journal.storage(), "gs://sift-test/upload-intent").unwrap();
        let intent_path = intent_source
            .path()
            .join("control/archive-upload-intent.json");
        assert!(intent_path.exists());
        let object_store = storage_object::GcsObjectStore::new("sift-test", "").unwrap();
        let objects_before_retry = object_store.list("upload-intent/").unwrap().len();
        let recovered =
            archive::archive_journal_gcs(&intent_journal, "gs://sift-test/upload-intent").unwrap();
        assert_eq!(recovered.manifest_uri, uploaded.manifest_uri);
        assert_eq!(recovered.manifest_sha256, uploaded.manifest_sha256);
        assert_eq!(
            object_store.list("upload-intent/").unwrap().len(),
            objects_before_retry,
            "retry after a manifest-last crash must reuse the deterministic upload"
        );
        assert!(!intent_path.exists());
        assert!(intent_source
            .path()
            .join("control/archive-commit.json")
            .exists());

        // A manifest-last upload can finish remotely and then lose the local
        // receipt. If retention advances the source manifest first, the old
        // upload intent is an orphan. It must not block the next archive.
        let stale_source = tempfile::tempdir().unwrap();
        let stale_journal = DurableJournal::open(stale_source.path()).unwrap();
        let now = Utc::now();
        let mut old = stored(1).event;
        old.event_id = "stale-intent-expired".into();
        old.occurred_at =
            (now - chrono::Duration::days(179)).to_rfc3339_opts(SecondsFormat::Nanos, true);
        old.observed_at.clone_from(&old.occurred_at);
        stale_journal.append(old).unwrap();
        archive::archive_journal_gcs(&stale_journal, "gs://sift-test/stale-intent").unwrap();

        let mut suffix = stored(2).event;
        suffix.event_id = "stale-intent-retained".into();
        suffix.occurred_at = now.to_rfc3339_opts(SecondsFormat::Nanos, true);
        suffix.observed_at.clone_from(&suffix.occurred_at);
        stale_journal.append(suffix).unwrap();
        archive::archive_gcs(stale_journal.storage(), "gs://sift-test/stale-intent").unwrap();
        let stale_intent_path = stale_source
            .path()
            .join("control/archive-upload-intent.json");
        assert!(stale_intent_path.exists());

        archive::expire_committed_events_at(&stale_journal, now + chrono::Duration::days(2))
            .unwrap();
        let advanced =
            archive::archive_journal_gcs(&stale_journal, "gs://sift-test/stale-intent").unwrap();
        assert_eq!(advanced.manifest.event_count, 1);
        assert!(!stale_intent_path.exists());
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}

#[test]
fn archive_failure_never_compacts_the_wal() {
    let source_dir = tempfile::tempdir().unwrap();
    let journal = DurableJournal::open(source_dir.path()).unwrap();
    journal.append(stored(1).event).unwrap();
    let wal = source_dir
        .path()
        .join("wal")
        .join("logs")
        .join("events.framed");
    let before = std::fs::read(&wal).unwrap();

    let error = archive::archive_journal_gcs(&journal, "file:///not-a-gcs-destination")
        .unwrap_err()
        .to_string();
    assert!(error.contains("not a GCS"));
    assert_eq!(std::fs::read(&wal).unwrap(), before);
    assert!(!source_dir
        .path()
        .join("control/archive-commit.json")
        .exists());
}
// HANDWRITE-END
