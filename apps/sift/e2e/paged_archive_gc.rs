use std::{collections::BTreeMap, time::Duration};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{SecondsFormat, Utc};
use sift::{storage::archive, DurableJournal, EventEnvelope, SignalKind};

fn event(id: &str, occurred_at: chrono::DateTime<Utc>, attachment: bool) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "paged-gc-project",
        "prod",
        id,
        SignalKind::Log,
        serde_json::json!({"message": id}),
    );
    event.occurred_at = occurred_at.to_rfc3339_opts(SecondsFormat::Nanos, true);
    event.observed_at.clone_from(&event.occurred_at);
    event.resource = BTreeMap::from([("service.name".into(), "paged-gc-test".into())]);
    if attachment {
        event.payload["attachment_base64"] =
            serde_json::Value::String(BASE64.encode(vec![17_u8; 70_000]));
    }
    event
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn archive_gc_resumes_from_a_small_durable_cursor_after_restart() {
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
        let data = tempfile::tempdir().unwrap();
        let journal = DurableJournal::open(data.path()).unwrap();
        journal
            .append(event(
                "expires-with-blob",
                now - chrono::Duration::days(179),
                true,
            ))
            .unwrap();
        journal.append(event("retained", now, false)).unwrap();
        archive::archive_journal_gcs(&journal, "gs://sift-paged-gc/mvp").unwrap();

        let expiry =
            archive::expire_committed_events_at(&journal, now + chrono::Duration::days(2)).unwrap();
        let manifest_bytes = service_backup::fetch_backup_object(&expiry.manifest_uri).unwrap();
        assert!(manifest_bytes.len() < 64 * 1024);
        let manifest: archive::ArchiveManifest = serde_json::from_slice(&manifest_bytes).unwrap();
        let plan = archive::inspect_archive_gc_plan(&manifest).unwrap();
        assert!(
            plan.len() >= 4,
            "expiration must queue the old root, catalog, segment, and blob objects"
        );
        assert!(plan
            .iter()
            .all(|uri| service_backup::fetch_backup_object(uri).is_ok()));

        let commit_path = data.path().join("control/archive-commit.json");
        let pending_path = data.path().join("control/archive-gc-pending.json");
        assert!(std::fs::metadata(&commit_path).unwrap().len() < 64 * 1024);
        let pending_before = std::fs::read(&pending_path).unwrap();
        assert!(pending_before.len() < 64 * 1024);
        let pending_before: serde_json::Value = serde_json::from_slice(&pending_before).unwrap();
        assert!(pending_before["cursor"].is_null());

        let (deleted, complete) =
            archive::finalize_archive_gc_batch_after_checkpoint(data.path(), 1).unwrap();
        assert_eq!(deleted, 1);
        assert!(!complete);
        let pending_after = std::fs::read(&pending_path).unwrap();
        assert!(pending_after.len() < 64 * 1024);
        let pending_after: serde_json::Value = serde_json::from_slice(&pending_after).unwrap();
        assert!(pending_after["cursor"].as_str().is_some());
        assert_eq!(
            plan.iter()
                .filter(|uri| service_backup::fetch_backup_object(uri).is_err())
                .count(),
            1
        );

        drop(journal);
        let reopened = DurableJournal::open(data.path()).unwrap();
        assert_eq!(reopened.total_event_count(), 1);
        drop(reopened);

        let mut total_deleted = deleted;
        let mut complete = complete;
        while !complete {
            let (batch_deleted, batch_complete) =
                archive::finalize_archive_gc_batch_after_checkpoint(data.path(), 1).unwrap();
            assert!(batch_deleted <= 1);
            total_deleted += batch_deleted;
            complete = batch_complete;
        }
        assert_eq!(total_deleted, plan.len());
        assert!(!pending_path.exists());
        assert!(plan
            .iter()
            .all(|uri| service_backup::fetch_backup_object(uri).is_err()));
        assert!(service_backup::fetch_backup_object(&expiry.manifest_uri).is_ok());

        let restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&expiry.manifest_uri, restore.path()).unwrap();
        let restored = DurableJournal::open(restore.path()).unwrap();
        assert_eq!(restored.total_event_count(), 1);
        drop(restored);

        // Reproduce a crash after a retention rewrite saved its deletion
        // intent but before the replacement manifest receipt became durable.
        // The next ordinary archive must discard that stale authority and
        // must never delete objects still referenced by its new catalog.
        let crash_data = tempfile::tempdir().unwrap();
        let crash_journal = DurableJournal::open(crash_data.path()).unwrap();
        crash_journal
            .append(event(
                "crash-expired",
                now - chrono::Duration::days(179),
                true,
            ))
            .unwrap();
        crash_journal
            .append(event("crash-retained", now, false))
            .unwrap();
        let crash_first =
            archive::archive_journal_gcs(&crash_journal, "gs://sift-paged-gc/crash").unwrap();
        let crash_commit_path = crash_data.path().join("control/archive-commit.json");
        let first_commit = std::fs::read(&crash_commit_path).unwrap();

        let stale_rewrite =
            archive::expire_committed_events_at(&crash_journal, now + chrono::Duration::days(2))
                .unwrap();
        assert_ne!(stale_rewrite.manifest_uri, crash_first.manifest_uri);
        let crash_pending_path = crash_data.path().join("control/archive-gc-pending.json");
        let stale_pending = std::fs::read(&crash_pending_path).unwrap();

        std::fs::write(&crash_commit_path, first_commit).unwrap();
        std::fs::write(&crash_pending_path, stale_pending).unwrap();
        crash_journal
            .append(event("crash-suffix", now, false))
            .unwrap();
        let recovered =
            archive::archive_journal_gcs(&crash_journal, "gs://sift-paged-gc/crash").unwrap();
        let live_plan = archive::inspect_archive_gc_plan(&recovered.manifest).unwrap();
        let (live_segments, live_blobs) =
            archive::inspect_archive_catalog(&recovered.manifest).unwrap();
        assert!(live_segments
            .iter()
            .all(|segment| !live_plan.contains(&segment.object_uri)));
        assert!(live_blobs
            .iter()
            .all(|blob| !live_plan.contains(&blob.object_uri)));
        archive::finalize_archive_gc_after_checkpoint(crash_data.path()).unwrap();

        let recovered_restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&recovered.manifest_uri, recovered_restore.path()).unwrap();
        let recovered_journal = DurableJournal::open(recovered_restore.path()).unwrap();
        assert_eq!(
            recovered_journal.total_event_count(),
            3,
            "stale pre-commit GC authority must not orphan the new archive catalog"
        );

        let precommit_data = tempfile::tempdir().unwrap();
        let precommit_journal = DurableJournal::open(precommit_data.path()).unwrap();
        precommit_journal
            .append(event("precommit-a", now, false))
            .unwrap();
        archive::archive_journal_gcs(&precommit_journal, "gs://sift-paged-gc/precommit").unwrap();
        precommit_journal
            .append(event("precommit-b", now, false))
            .unwrap();
        let precommit_b =
            archive::archive_journal_gcs(&precommit_journal, "gs://sift-paged-gc/precommit")
                .unwrap();
        let precommit_b_plan = archive::inspect_archive_gc_plan(&precommit_b.manifest).unwrap();
        assert!(!precommit_b_plan.is_empty());
        let precommit_commit_path = precommit_data.path().join("control/archive-commit.json");
        let precommit_pending_path = precommit_data
            .path()
            .join("control/archive-gc-pending.json");
        let precommit_staged_path = precommit_data.path().join("control/archive-gc-staged.json");
        let precommit_b_commit = std::fs::read(&precommit_commit_path).unwrap();
        let precommit_b_pending = std::fs::read(&precommit_pending_path).unwrap();

        precommit_journal
            .append(event("precommit-c", now, false))
            .unwrap();
        let precommit_wal_path = precommit_data.path().join("wal/logs/events.framed");
        let precommit_c_wal = std::fs::read(&precommit_wal_path).unwrap();
        archive::archive_journal_gcs(&precommit_journal, "gs://sift-paged-gc/precommit").unwrap();
        let precommit_c_pending = std::fs::read(&precommit_pending_path).unwrap();
        drop(precommit_journal);
        std::fs::write(&precommit_commit_path, &precommit_b_commit).unwrap();
        std::fs::write(&precommit_pending_path, &precommit_b_pending).unwrap();
        std::fs::write(&precommit_staged_path, &precommit_c_pending).unwrap();
        std::fs::write(&precommit_wal_path, precommit_c_wal).unwrap();

        let precommit_reopened = DurableJournal::open(precommit_data.path()).unwrap();
        assert!(!precommit_staged_path.exists());
        assert_eq!(
            std::fs::read(&precommit_pending_path).unwrap(),
            precommit_b_pending,
            "a pre-commit crash must preserve the last committed GC authority"
        );
        precommit_reopened
            .append(event("precommit-d", now, false))
            .unwrap();
        let precommit_d =
            archive::archive_journal_gcs(&precommit_reopened, "gs://sift-paged-gc/precommit")
                .unwrap();
        let precommit_d_plan = archive::inspect_archive_gc_plan(&precommit_d.manifest).unwrap();
        assert!(precommit_b_plan
            .iter()
            .all(|uri| precommit_d_plan.contains(uri)));
        archive::finalize_archive_gc_after_checkpoint(precommit_data.path()).unwrap();
        let precommit_restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&precommit_d.manifest_uri, precommit_restore.path()).unwrap();
        assert_eq!(
            DurableJournal::open(precommit_restore.path())
                .unwrap()
                .total_event_count(),
            4
        );

        let postcommit_data = tempfile::tempdir().unwrap();
        let postcommit_journal = DurableJournal::open(postcommit_data.path()).unwrap();
        postcommit_journal
            .append(event("postcommit-a", now, false))
            .unwrap();
        archive::archive_journal_gcs(&postcommit_journal, "gs://sift-paged-gc/postcommit").unwrap();
        postcommit_journal
            .append(event("postcommit-b", now, false))
            .unwrap();
        archive::archive_journal_gcs(&postcommit_journal, "gs://sift-paged-gc/postcommit").unwrap();
        let postcommit_pending_path = postcommit_data
            .path()
            .join("control/archive-gc-pending.json");
        let postcommit_staged_path = postcommit_data
            .path()
            .join("control/archive-gc-staged.json");
        let old_pending = std::fs::read(&postcommit_pending_path).unwrap();
        postcommit_journal
            .append(event("postcommit-c", now, false))
            .unwrap();
        archive::archive_journal_gcs(&postcommit_journal, "gs://sift-paged-gc/postcommit").unwrap();
        let new_pending = std::fs::read(&postcommit_pending_path).unwrap();
        std::fs::write(&postcommit_pending_path, old_pending).unwrap();
        std::fs::write(&postcommit_staged_path, &new_pending).unwrap();
        drop(postcommit_journal);

        let postcommit_reopened = DurableJournal::open(postcommit_data.path()).unwrap();
        assert_eq!(postcommit_reopened.total_event_count(), 3);
        assert!(!postcommit_staged_path.exists());
        assert_eq!(
            std::fs::read(&postcommit_pending_path).unwrap(),
            new_pending,
            "a post-commit crash must promote the staged GC authority"
        );
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
