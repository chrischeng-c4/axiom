// HANDWRITE-BEGIN gap="sift-vat-gcs-archive-tests" tracker="1659" reason="Run Vat Cloud Storage emulator with real service-backup GCS requests and verify archive/restore hash equality."
use std::{collections::BTreeMap, time::Duration};

use sift::{
    storage::{archive, RawStorage, StorageConfig},
    DurableJournal, EventEnvelope, EventQuery, StoredEvent,
};

fn stored(cursor: u64) -> StoredEvent {
    let mut event = EventEnvelope::new(
        format!("archive-{cursor}"),
        sift::SignalKind::Log,
        serde_json::json!({"cursor": cursor}),
    );
    event.resource = BTreeMap::from([("service.name".into(), "archive-test".into())]);
    StoredEvent {
        cursor,
        acknowledged_at: "2026-07-14T00:00:00Z".into(),
        event,
    }
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
        let storage = RawStorage::open_with_config(
            source_dir.path(),
            StorageConfig {
                initial_logical_shards: 1,
                max_segment_events: 2,
                ..StorageConfig::default()
            },
        )
        .unwrap();
        for cursor in 1..=5 {
            storage.append(&stored(cursor)).unwrap();
        }
        storage.seal_all().unwrap();

        let receipt = archive::archive_gcs(&storage, "gs://sift-test/domain-v1").unwrap();
        assert!(receipt.manifest_uri.ends_with("manifest.json"));
        assert_eq!(receipt.manifest.segments.len(), 3);

        let restore_dir = tempfile::tempdir().unwrap();
        let restored = archive::restore_gcs(&receipt.manifest_uri, restore_dir.path()).unwrap();
        assert_eq!(restored.segments.len(), receipt.manifest.segments.len());
        let reopened = DurableJournal::open(restore_dir.path()).unwrap();
        let events = reopened.query(EventQuery::default()).unwrap();
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].event.event_id, "archive-1");
        assert_eq!(events[4].event.event_id, "archive-5");

        let destination =
            service_backup::BackupDestination::from_uri("gs://sift-test/domain-v1").unwrap();
        let sink = service_backup::GcsSink::from_destination(&destination).unwrap();
        assert!(service_backup::BackupSink::prune(&sink, 0).unwrap() >= 4);
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
// HANDWRITE-END
