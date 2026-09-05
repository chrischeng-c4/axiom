use std::{collections::BTreeMap, os::unix::fs::PermissionsExt, sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{SecondsFormat, Utc};
use raft_runtime::RaftStateMachine;
use sha2::{Digest, Sha256};
use sift::{storage::archive, DurableJournal, EventEnvelope, EventQuery, SignalKind};

fn log(id: &str, occurred_at: chrono::DateTime<Utc>, attachment: bool) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "follower-blob-gc",
        "test",
        id,
        SignalKind::Log,
        serde_json::json!({"message": id}),
    );
    event.occurred_at = occurred_at.to_rfc3339_opts(SecondsFormat::Nanos, true);
    event.observed_at.clone_from(&event.occurred_at);
    event.resource = BTreeMap::from([("service.name".into(), "blob-gc".into())]);
    if attachment {
        event.payload["attachment_base64"] =
            serde_json::Value::String(BASE64.encode(vec![23_u8; 70_000]));
    }
    event
}

fn apply(machine: &sift::durability::SiftStateMachine, index: u64, event: EventEnvelope) {
    let command = serde_json::to_vec(&serde_json::json!({
        "kind": "append_events",
        "events": [event]
    }))
    .unwrap();
    machine.apply_local(index, &command).unwrap();
    machine.take_append_outcomes(index).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follower_persists_and_resumes_local_blob_gc_from_an_archive_checkpoint() {
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
        let now = Utc::now();
        let source_dir = tempfile::tempdir().unwrap();
        let source_journal = Arc::new(DurableJournal::open(source_dir.path()).unwrap());
        let source = sift::durability::SiftStateMachine::new(source_journal.clone());
        let follower_dir = tempfile::tempdir().unwrap();
        let follower_journal = Arc::new(DurableJournal::open(follower_dir.path()).unwrap());
        let follower = sift::durability::SiftStateMachine::new(follower_journal.clone());

        for (index, event) in [
            (
                1,
                log(
                    "expired-attachment",
                    now - chrono::Duration::days(179),
                    true,
                ),
            ),
            (2, log("retained-log", now, false)),
        ] {
            apply(&source, index, event.clone());
            apply(&follower, index, event);
        }
        assert_eq!(source_journal.storage().blob_paths().unwrap().len(), 1);
        assert_eq!(follower_journal.storage().blob_paths().unwrap().len(), 1);

        archive::archive_journal_gcs(&source_journal, "gs://sift-follower-gc/source").unwrap();
        archive::archive_journal_gcs(&follower_journal, "gs://sift-follower-gc/follower").unwrap();
        let expired =
            archive::expire_committed_events_at(&source_journal, now + chrono::Duration::days(2))
                .unwrap();
        assert_eq!(expired.expired_events, 1);

        let status = archive::committed_status(source_dir.path())
            .unwrap()
            .unwrap();
        let barrier = sift::durability::encode_archive_checkpoint_barrier_for_diagnostics(
            status.retention_generation,
            status.manifest_uri,
            status.manifest_sha256,
        )
        .unwrap();
        source.apply_local(3, &barrier).unwrap();
        follower.apply_local(3, &barrier).unwrap();
        let mut checkpoint = Vec::new();
        source.snapshot_at(3, &mut checkpoint).unwrap();

        follower.restore(&mut checkpoint.as_slice()).unwrap();
        assert_eq!(follower_journal.total_event_count(), 1);
        assert_eq!(
            follower_journal.query(EventQuery::default()).unwrap()[0]
                .event
                .event_id,
            "retained-log"
        );
        assert!(
            follower_journal.storage().blob_paths().unwrap().is_empty(),
            "every voter must remove an unreferenced local blob after it adopts the manifest"
        );
        let pending = follower_dir.path().join("control/local-blob-gc.json");
        assert!(
            !pending.exists(),
            "an all-voter checkpoint must finish local blob GC before it acknowledges install"
        );
        let complete = follower_dir
            .path()
            .join("control/local-blob-gc-complete.json");
        assert!(
            complete.exists(),
            "the follower must retain a durable completion receipt"
        );

        drop(follower);
        drop(follower_journal);
        let reopened = DurableJournal::open(follower_dir.path()).unwrap();
        assert_eq!(reopened.total_event_count(), 1);
        assert!(reopened.storage().blob_paths().unwrap().is_empty());
        assert!(
            !pending.exists(),
            "restart must not recreate an already completed local blob GC plan"
        );
        drop(reopened);

        // Reproduce an acknowledged WAL suffix whose deferred segment append
        // failed. Local blob GC must repair and scan that canonical WAL before
        // it removes a content hash reused by the suffix.
        let wal_only_dir = tempfile::tempdir().unwrap();
        let wal_only = DurableJournal::open_with_resident_limit(wal_only_dir.path(), 1).unwrap();
        wal_only
            .append(log(
                "wal-old-attachment",
                now - chrono::Duration::days(179),
                true,
            ))
            .unwrap();
        wal_only.append(log("wal-retained", now, false)).unwrap();
        archive::archive_journal_gcs(&wal_only, "gs://sift-follower-gc/wal-only").unwrap();
        let expired =
            archive::expire_committed_events_at(&wal_only, now + chrono::Duration::days(2))
                .unwrap();
        assert!(wal_only.storage().blob_paths().unwrap().is_empty());
        let manifest: archive::ArchiveManifest = serde_json::from_slice(
            &service_backup::fetch_backup_object(&expired.manifest_uri).unwrap(),
        )
        .unwrap();
        let status = archive::committed_status(wal_only_dir.path())
            .unwrap()
            .unwrap();
        let bytes = vec![23_u8; 70_000];
        let hash = format!("sha256:{}", hex::encode(Sha256::digest(&bytes)));

        let traces_root = wal_only_dir.path().join("segments/traces");
        std::fs::remove_dir_all(&traces_root).unwrap();
        std::fs::write(&traces_root, b"force deferred trace segment failure").unwrap();
        let mut reused = EventEnvelope::for_project(
            "follower-blob-gc",
            "test",
            "wal-only-reuse",
            SignalKind::Span,
            serde_json::json!({
                "trace_id": "11111111111111111111111111111111",
                "span_id": "2222222222222222",
                "name": "wal-only-reuse",
                "attachment_base64": BASE64.encode(&bytes),
            }),
        );
        reused.occurred_at = now.to_rfc3339_opts(SecondsFormat::Nanos, true);
        reused.observed_at.clone_from(&reused.occurred_at);
        reused.resource = BTreeMap::from([("service.name".into(), "blob-gc".into())]);
        wal_only.append(reused).unwrap();
        let suffix_head = wal_only
            .append(log("evict-wal-only-resident", now, false))
            .unwrap()
            .cursor;

        std::fs::remove_file(&traces_root).unwrap();
        std::fs::create_dir_all(&traces_root).unwrap();
        std::fs::set_permissions(&traces_root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let pending = wal_only_dir.path().join("control/local-blob-gc.json");
        std::fs::write(
            &pending,
            serde_json::to_vec_pretty(&serde_json::json!({
                "format_version": 3,
                "replacement_manifest_uri": status.manifest_uri,
                "replacement_manifest_sha256": status.manifest_sha256,
                "gc_plan_uri": manifest.gc_plan_uri.unwrap(),
                "gc_plan_root": manifest.gc_plan_root.unwrap(),
                "plan_cursor": null,
                "plan_exhausted": false,
                "candidates": [hash],
                "scan_start_cursor": manifest.raft_snapshot_index,
                "scanned_through_cursor": manifest.raft_snapshot_index,
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&pending, std::fs::Permissions::from_mode(0o600)).unwrap();

        let (removed, complete) = archive::resume_local_blob_gc_batch(&wal_only, 1, 1).unwrap();
        assert!(!complete);
        assert_eq!(removed, 0);
        assert_eq!(wal_only.storage().blob_paths().unwrap().len(), 1);
        let mut progress: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&pending).unwrap()).unwrap();
        assert_eq!(
            progress["scanned_through_cursor"].as_u64().unwrap(),
            suffix_head
        );
        assert_ne!(
            progress["scanned_through_cursor"], progress["scan_start_cursor"],
            "the next GC candidate batch must not restart the retained suffix scan"
        );
        progress["plan_exhausted"] = serde_json::Value::Bool(true);
        progress["candidates"] = serde_json::json!([hash]);
        std::fs::write(&pending, serde_json::to_vec_pretty(&progress).unwrap()).unwrap();
        std::fs::set_permissions(&pending, std::fs::Permissions::from_mode(0o600)).unwrap();

        let (removed, complete) = archive::resume_local_blob_gc_batch(&wal_only, 1, 1).unwrap();
        assert!(complete);
        assert_eq!(removed, 0);
        assert_eq!(wal_only.storage().blob_paths().unwrap().len(), 1);
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
