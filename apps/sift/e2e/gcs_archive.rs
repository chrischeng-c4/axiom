// HANDWRITE-BEGIN gap="sift-vat-gcs-archive-tests" tracker="1659" reason="Run Vat Cloud Storage emulator with real service-backup GCS requests and verify archive/restore hash equality."
use std::{collections::BTreeMap, sync::Arc, time::Duration};

use sha2::Digest;
use sift::{
    storage::archive, DurableJournal, EventEnvelope, EventQuery, MetricPoint, MetricTemporality,
    SignalKind, StoredEvent,
};

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
        assert_eq!(receipt.manifest.format_version, 6);
        let manifest_json = serde_json::to_value(&receipt.manifest).unwrap();
        assert_eq!(
            manifest_json["source_cluster_id"],
            source_layout["cluster_id"]
        );
        assert_eq!(manifest_json["source_node_id"], source_layout["node_id"]);
        assert_eq!(manifest_json["raft_snapshot_index"], 5);
        assert_eq!(manifest_json["event_count"], 5);
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
        assert_eq!(
            receipt.manifest.segments.len(),
            3,
            "one initial shard per signal must produce one segment per signal"
        );
        assert!(receipt
            .manifest
            .segments
            .iter()
            .all(|segment| segment.object_uri.ends_with(".parquet")));
        assert_eq!(
            receipt
                .manifest
                .segments
                .iter()
                .map(|segment| segment.signal)
                .collect::<std::collections::BTreeSet<_>>(),
            [SignalKind::Log, SignalKind::Metric, SignalKind::Span]
                .into_iter()
                .collect()
        );
        for segment in &receipt.manifest.segments {
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
        assert_eq!(restored.segments.len(), receipt.manifest.segments.len());
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
        for prior in &receipt.manifest.segments {
            let reused = next
                .manifest
                .segments
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
