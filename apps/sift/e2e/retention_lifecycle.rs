use std::{collections::BTreeMap, time::Duration};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{SecondsFormat, Utc};
use sift::{storage::archive, DurableJournal, EventEnvelope, EventQuery, SignalKind};

fn event(id: &str, signal: SignalKind, occurred_at: chrono::DateTime<Utc>) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "retention-project",
        "prod",
        id,
        signal,
        serde_json::json!({"message": id}),
    );
    event.occurred_at = occurred_at.to_rfc3339_opts(SecondsFormat::Nanos, true);
    event.observed_at.clone_from(&event.occurred_at);
    event.resource = BTreeMap::from([("service.name".into(), "retention-test".into())]);
    event
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hot_eviction_and_180_day_expiration_preserve_restore_and_cursor_identity() {
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
        assert_eq!(
            journal
                .append(event(
                    "hot-span",
                    SignalKind::Span,
                    now - chrono::Duration::days(29)
                ))
                .unwrap()
                .cursor,
            1
        );
        let mut expiring = event(
            "expire-log",
            SignalKind::Log,
            now - chrono::Duration::days(179),
        );
        expiring.payload["attachment_base64"] =
            serde_json::Value::String(BASE64.encode(vec![42_u8; 70_000]));
        assert_eq!(journal.append(expiring).unwrap().cursor, 2);
        assert_eq!(
            journal
                .append(event("hot-log", SignalKind::Log, now))
                .unwrap()
                .cursor,
            3
        );
        assert_eq!(journal.storage().blob_paths().unwrap().len(), 1);
        let first = archive::archive_journal_gcs(&journal, "gs://sift-retention/mvp").unwrap();
        let mut suffix = event("suffix-log", SignalKind::Log, now);
        suffix.payload["attachment_base64"] =
            serde_json::Value::String(BASE64.encode(vec![24_u8; 70_000]));
        assert_eq!(
            journal.append(suffix).unwrap().cursor,
            4,
            "retention must preserve events accepted after the archived prefix"
        );

        let eviction = archive::evict_committed_cold_segments_at(&journal, now).unwrap();
        assert_eq!(eviction.evicted_segments, 0);
        assert_eq!(eviction.evicted_events, 0);
        let mut archived_ids = Vec::new();
        let replay = archive::replay_committed_events(
            data.path(),
            SignalKind::Log,
            "retention-project",
            Some("prod"),
            None,
            None,
            |event| {
                archived_ids.push(event.event.event_id);
                Ok(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(replay.replayed, 2);
        assert_eq!(archived_ids, ["expire-log", "hot-log"]);

        let expiry_time = now + chrono::Duration::days(2);
        let expiry = archive::expire_committed_events_at(&journal, expiry_time).unwrap();
        assert_eq!(expiry.expired_events, 1);
        assert_eq!(expiry.retained_events, 2);
        assert_eq!(expiry.replaced_segments, 1);
        assert_eq!(expiry.removed_segments, 0);
        assert_ne!(expiry.manifest_uri, first.manifest_uri);
        assert!(
            service_backup::fetch_backup_object(&first.manifest_uri).is_ok(),
            "the prior manifest must remain until the replacement checkpoint is durable"
        );
        let pending_path = data.path().join("control/archive-gc-pending.json");
        let pending_bytes = std::fs::read(&pending_path).unwrap();
        let mut wrong_identity: serde_json::Value = serde_json::from_slice(&pending_bytes).unwrap();
        wrong_identity["replacement_manifest_uri"] =
            serde_json::Value::String(first.manifest_uri.clone());
        wrong_identity["replacement_manifest_sha256"] =
            serde_json::Value::String(first.manifest_sha256.clone());
        std::fs::write(
            &pending_path,
            serde_json::to_vec_pretty(&wrong_identity).unwrap(),
        )
        .unwrap();
        assert!(
            archive::finalize_archive_gc_after_checkpoint(data.path()).is_err(),
            "GC must refuse an intent that is not bound to the committed replacement manifest"
        );
        assert!(
            service_backup::fetch_backup_object(&first.manifest_uri).is_ok(),
            "an identity mismatch must not delete the prior recovery source"
        );
        std::fs::write(&pending_path, pending_bytes).unwrap();
        assert!(
            archive::finalize_archive_gc_after_checkpoint(data.path()).unwrap() > 0,
            "the post-checkpoint barrier must delete queued archive objects"
        );
        assert!(service_backup::fetch_backup_object(&first.manifest_uri).is_err());
        assert_eq!(
            journal.storage().blob_paths().unwrap().len(),
            1,
            "retention must remove the expired blob but preserve a concurrent suffix blob"
        );
        let local_logs = journal
            .query(EventQuery {
                signal: Some(SignalKind::Log),
                after: 0,
                limit: 10,
            })
            .unwrap();
        let local_log_ids = local_logs
            .iter()
            .map(|stored| stored.event.event_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            local_log_ids,
            ["hot-log", "suffix-log"],
            "mixed local segments must be rewritten without expired rows"
        );
        let suffix_ref = &local_logs
            .iter()
            .find(|stored| stored.event.event_id == "suffix-log")
            .unwrap()
            .event
            .blob_refs[0];
        assert_eq!(
            journal.storage().read_blob(&suffix_ref.hash).unwrap().len(),
            70_000
        );
        assert!(
            journal
                .query(EventQuery {
                    signal: Some(SignalKind::Span),
                    after: 0,
                    limit: 10,
                })
                .unwrap()
                .is_empty(),
            "a 29-day span must become cold after the two-day clock advance"
        );

        archived_ids.clear();
        let replay = archive::replay_committed_events(
            data.path(),
            SignalKind::Log,
            "retention-project",
            Some("prod"),
            None,
            None,
            |event| {
                archived_ids.push(event.event.event_id);
                Ok(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(replay.replayed, 1);
        assert_eq!(archived_ids, ["hot-log"]);

        let repeated = archive::expire_committed_events_at(&journal, expiry_time).unwrap();
        assert_eq!(repeated.expired_events, 0);
        assert_eq!(repeated.retained_events, 2);
        assert_eq!(repeated.manifest_uri, expiry.manifest_uri);

        let restored_data = tempfile::tempdir().unwrap();
        let restored_manifest =
            archive::restore_gcs(&expiry.manifest_uri, restored_data.path()).unwrap();
        assert_eq!(restored_manifest.event_count, 2);
        assert_eq!(restored_manifest.raft_snapshot_index, 3);
        assert_eq!(restored_manifest.retained_watermarks.logs, 3);
        assert_eq!(restored_manifest.retained_watermarks.traces, 1);
        let restored = DurableJournal::open(restored_data.path()).unwrap();
        assert_eq!(restored.total_event_count(), 2);
        let restored_append = restored
            .append(event("restored-next", SignalKind::Log, now))
            .unwrap();
        assert_eq!(restored_append.cursor, 4);
        drop(journal);

        let reopened = DurableJournal::open(data.path()).unwrap();
        assert_eq!(reopened.total_event_count(), 3);
        let reopened_log_ids = reopened
            .query(EventQuery {
                signal: Some(SignalKind::Log),
                after: 0,
                limit: 10,
            })
            .unwrap()
            .into_iter()
            .map(|stored| stored.event.event_id)
            .collect::<Vec<_>>();
        assert_eq!(reopened_log_ids, ["hot-log", "suffix-log"]);
        let appended = reopened
            .append(event("expire-log", SignalKind::Log, now))
            .unwrap();
        assert_eq!(appended.cursor, 5, "expiration must not reuse cursor 4");
        assert!(!appended.duplicate, "expired event IDs must leave dedupe");

        let retry_data = tempfile::tempdir().unwrap();
        let retry_journal = DurableJournal::open(retry_data.path()).unwrap();
        retry_journal
            .append(event(
                "retry-expired",
                SignalKind::Log,
                now - chrono::Duration::days(179),
            ))
            .unwrap();
        retry_journal
            .append(event("retry-hot", SignalKind::Log, now))
            .unwrap();
        archive::archive_journal_gcs(&retry_journal, "gs://sift-retention/retry").unwrap();

        let dedupe_root = retry_data.path().join("indexes/dedupe");
        std::fs::remove_dir_all(&dedupe_root).unwrap();
        std::fs::write(&dedupe_root, b"injected late dedupe failure").unwrap();
        archive::expire_committed_events_at(&retry_journal, now + chrono::Duration::days(2))
            .expect_err("a late dedupe rebuild failure must fail closed");
        assert!(retry_journal
            .query(EventQuery::default())
            .unwrap_err()
            .to_string()
            .contains("requires archive recovery"));

        std::fs::remove_file(&dedupe_root).unwrap();
        std::fs::create_dir(&dedupe_root).unwrap();
        let retried =
            archive::expire_committed_events_at(&retry_journal, now + chrono::Duration::days(2))
                .unwrap();
        assert_eq!(retried.expired_events, 0);
        assert_eq!(retried.retained_events, 1);
        assert_eq!(retry_journal.total_event_count(), 1);
        assert_eq!(
            retry_journal
                .query(EventQuery::default())
                .unwrap()
                .into_iter()
                .map(|stored| stored.event.event_id)
                .collect::<Vec<_>>(),
            ["retry-hot"]
        );
        let retried_append = retry_journal
            .append(event("retry-expired", SignalKind::Log, now))
            .unwrap();
        assert_eq!(retried_append.cursor, 3);
        assert!(!retried_append.duplicate);
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
