use std::{collections::BTreeMap, sync::Arc, time::Duration};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{SecondsFormat, Utc};
use raft_runtime::RaftStateMachine;
use sift::{durability, storage::archive, DurableJournal, EventEnvelope, EventQuery, SignalKind};

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

        let reopened = Arc::new(DurableJournal::open(data.path()).unwrap());
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
        let reopened_machine = durability::SiftStateMachine::new(reopened.clone());
        let reuse_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![event("expire-log", SignalKind::Log, now)],
            &expiry_time.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        reopened_machine.apply_local(1, &reuse_command).unwrap();
        let appended = reopened_machine.take_append_outcomes(1).unwrap().remove(0);
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
        let retried =
            archive::expire_committed_events_at(&retry_journal, now + chrono::Duration::days(2))
                .unwrap();
        assert_eq!(retried.expired_events, 1);
        assert_eq!(retried.retained_events, 1);
        assert!(
            retry_journal
                .append(event("retry-expired", SignalKind::Log, now))
                .is_err(),
            "a damaged active receipt must fail closed even though retention is independent"
        );
        drop(retry_journal);
        assert!(
            DurableJournal::open(retry_data.path()).is_err(),
            "a special file at the dedupe root must block restart"
        );

        std::fs::remove_file(&dedupe_root).unwrap();
        std::fs::create_dir(&dedupe_root).unwrap();
        let retry_journal = Arc::new(DurableJournal::open(retry_data.path()).unwrap());
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
        let retry_machine = durability::SiftStateMachine::new(retry_journal.clone());
        let retry_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![event("retry-expired", SignalKind::Log, now)],
            &(now + chrono::Duration::days(2)).to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        retry_machine.apply_local(1, &retry_command).unwrap();
        let retried_append = retry_machine.take_append_outcomes(1).unwrap().remove(0);
        assert_eq!(retried_append.cursor, 3);
        assert!(!retried_append.duplicate);

        let reused_data = tempfile::tempdir().unwrap();
        let reused_journal = Arc::new(DurableJournal::open(reused_data.path()).unwrap());
        let reused_machine = durability::SiftStateMachine::new(reused_journal.clone());
        let old_acceptance = now - chrono::Duration::days(181);
        let old_event = event("retention-reused-id", SignalKind::Log, old_acceptance);
        let old_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![old_event],
            &old_acceptance.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        reused_machine.apply_local(1, &old_command).unwrap();
        assert!(!reused_machine.take_append_outcomes(1).unwrap()[0].duplicate);
        archive::archive_journal_gcs(&reused_journal, "gs://sift-retention/reused-id").unwrap();

        let recent_event = event("retention-reused-id", SignalKind::Log, now);
        let recent_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![recent_event.clone()],
            &now.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        reused_machine.apply_local(2, &recent_command).unwrap();
        let recent = reused_machine.take_append_outcomes(2).unwrap();
        assert!(!recent[0].duplicate);
        assert_eq!(recent[0].raw_cursor, 2);

        archive::expire_committed_events_at(&reused_journal, now).unwrap();
        let retry_time = now + chrono::Duration::minutes(1);
        let retry_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![recent_event],
            &retry_time.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        reused_machine.apply_local(3, &retry_command).unwrap();
        let retry = reused_machine.take_append_outcomes(3).unwrap();
        assert!(
            retry[0].duplicate,
            "expiring the old use of an ID must keep the recent six-hour acknowledgement"
        );
        assert_eq!(retry[0].raw_cursor, 2);
        assert_eq!(reused_journal.total_event_count(), 1);

        let boundary_data = tempfile::tempdir().unwrap();
        let boundary_journal = Arc::new(DurableJournal::open(boundary_data.path()).unwrap());
        let boundary_machine = durability::SiftStateMachine::new(boundary_journal.clone());
        let boundary_event = event(
            "retention-active-receipt",
            SignalKind::Log,
            now - chrono::Duration::days(180) + chrono::Duration::minutes(1),
        );
        let boundary_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![boundary_event.clone()],
            &now.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        boundary_machine.apply_local(1, &boundary_command).unwrap();
        let accepted = boundary_machine.take_append_outcomes(1).unwrap();
        assert!(!accepted[0].duplicate);
        archive::archive_journal_gcs(&boundary_journal, "gs://sift-retention/active-receipt")
            .unwrap();

        let boundary_expiry = archive::expire_committed_events_at(
            &boundary_journal,
            now + chrono::Duration::minutes(2),
        )
        .unwrap();
        assert_eq!(boundary_journal.total_event_count(), 0);
        let boundary_retry = durability::encode_raft_batch_at_for_diagnostics(
            vec![boundary_event],
            &(now + chrono::Duration::minutes(3)).to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        boundary_machine.apply_local(2, &boundary_retry).unwrap();
        let retried = boundary_machine.take_append_outcomes(2).unwrap();
        assert!(
            retried[0].duplicate,
            "telemetry retention must not shorten the exact six-hour retry window"
        );
        assert_eq!(retried[0].raw_cursor, 1);
        assert_eq!(retried[0].acknowledged_at, accepted[0].acknowledged_at);
        assert_eq!(boundary_journal.total_event_count(), 0);

        drop(boundary_machine);
        drop(boundary_journal);
        let boundary_dedupe = boundary_data.path().join("indexes/dedupe");
        std::fs::remove_dir_all(&boundary_dedupe).unwrap();
        std::fs::create_dir(&boundary_dedupe).unwrap();
        let rebuilt_boundary = Arc::new(DurableJournal::open(boundary_data.path()).unwrap());
        let rebuilt_machine = durability::SiftStateMachine::new(rebuilt_boundary.clone());
        rebuilt_machine.apply_local(3, &boundary_retry).unwrap();
        let rebuilt_retry = rebuilt_machine.take_append_outcomes(3).unwrap();
        assert!(rebuilt_retry[0].duplicate);
        assert_eq!(rebuilt_retry[0].raw_cursor, 1);
        assert_eq!(
            rebuilt_retry[0].acknowledged_at,
            accepted[0].acknowledged_at
        );
        assert_eq!(rebuilt_boundary.total_event_count(), 0);

        let boundary_restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&boundary_expiry.manifest_uri, boundary_restore.path()).unwrap();
        let restored_boundary = Arc::new(DurableJournal::open(boundary_restore.path()).unwrap());
        let restored_machine = durability::SiftStateMachine::new(restored_boundary.clone());
        restored_machine.apply_local(1, &boundary_retry).unwrap();
        let restored_retry = restored_machine.take_append_outcomes(1).unwrap();
        assert!(restored_retry[0].duplicate);
        assert_eq!(restored_retry[0].raw_cursor, 1);
        assert_eq!(
            restored_retry[0].acknowledged_at,
            accepted[0].acknowledged_at
        );
        assert_eq!(restored_boundary.total_event_count(), 0);

        let receipt_only_data = tempfile::tempdir().unwrap();
        let receipt_only_journal =
            Arc::new(DurableJournal::open(receipt_only_data.path()).unwrap());
        let receipt_only_machine = durability::SiftStateMachine::new(receipt_only_journal.clone());
        let receipt_event = event(
            "receipt-only-expiry",
            SignalKind::Log,
            now - chrono::Duration::days(180) + chrono::Duration::hours(4),
        );
        let receipt_retry_event = event("receipt-only-expiry", SignalKind::Log, now);
        let receipt_acceptance = now;
        let receipt_command = durability::encode_raft_batch_at_for_diagnostics(
            vec![receipt_event.clone()],
            &receipt_acceptance.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        receipt_only_machine
            .apply_local(1, &receipt_command)
            .unwrap();
        receipt_only_machine.take_append_outcomes(1).unwrap();
        let receipt_archive =
            archive::archive_journal_gcs(&receipt_only_journal, "gs://sift-retention/receipt-only")
                .unwrap();
        assert_eq!(receipt_archive.manifest.dedupe_receipt_count, 0);
        let payload_cleanup_at = now + chrono::Duration::hours(5);
        let payload_cleanup =
            archive::expire_committed_events_at(&receipt_only_journal, payload_cleanup_at).unwrap();
        assert_eq!(payload_cleanup.expired_events, 1);
        let receipt_status = archive::committed_status(receipt_only_data.path())
            .unwrap()
            .unwrap();
        let receipt_manifest: archive::ArchiveManifest = serde_json::from_slice(
            &service_backup::fetch_backup_object(&receipt_status.manifest_uri).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt_manifest.event_count, 0);
        assert_eq!(receipt_manifest.dedupe_receipt_count, 1);
        assert!(!archive::retention_due_at(receipt_only_data.path(), payload_cleanup_at).unwrap());
        let receipt_retry = durability::encode_raft_batch_at_for_diagnostics(
            vec![receipt_retry_event.clone()],
            &payload_cleanup_at.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        receipt_only_machine.apply_local(2, &receipt_retry).unwrap();
        assert!(receipt_only_machine.take_append_outcomes(2).unwrap()[0].duplicate);

        let receipt_cleanup_at = now + chrono::Duration::hours(7);
        assert!(
            archive::retention_due_at(receipt_only_data.path(), receipt_cleanup_at).unwrap(),
            "the lifecycle scheduler must wake for an expired receipt even with no telemetry rows"
        );
        let receipt_cleanup =
            archive::expire_committed_events_at(&receipt_only_journal, receipt_cleanup_at).unwrap();
        assert_eq!(receipt_cleanup.expired_events, 0);
        let receipt_status = archive::committed_status(receipt_only_data.path())
            .unwrap()
            .unwrap();
        let receipt_manifest: archive::ArchiveManifest = serde_json::from_slice(
            &service_backup::fetch_backup_object(&receipt_status.manifest_uri).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt_manifest.event_count, 0);
        assert_eq!(
            receipt_manifest.dedupe_receipt_count, 0,
            "six-hour receipt cleanup must run even when no telemetry row reaches 180 days"
        );
        let receipt_retry = durability::encode_raft_batch_at_for_diagnostics(
            vec![receipt_retry_event],
            &receipt_cleanup_at.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        receipt_only_machine.apply_local(3, &receipt_retry).unwrap();
        assert!(!receipt_only_machine.take_append_outcomes(3).unwrap()[0].duplicate);

        // Reproduce a crash-equivalent late failure after the replacement
        // manifest receipt is durable but before the local journal head is.
        // The same live process must reconcile that commit after the file
        // problem is repaired. A restart must not be required.
        let late_data = tempfile::tempdir().unwrap();
        let late_journal = DurableJournal::open(late_data.path()).unwrap();
        late_journal
            .append(event(
                "late-expired",
                SignalKind::Log,
                now - chrono::Duration::days(179),
            ))
            .unwrap();
        late_journal
            .append(event("late-hot", SignalKind::Log, now))
            .unwrap();
        archive::archive_journal_gcs(&late_journal, "gs://sift-retention/late-head").unwrap();
        let head_path = late_data.path().join("control/journal-head.json");
        std::fs::remove_file(&head_path).unwrap();
        std::fs::create_dir(&head_path).unwrap();
        assert!(archive::expire_committed_events_at(
            &late_journal,
            now + chrono::Duration::days(2),
        )
        .is_err());
        assert!(late_journal.query(EventQuery::default()).is_err());
        std::fs::remove_dir(&head_path).unwrap();
        let reconciled =
            archive::expire_committed_events_at(&late_journal, now + chrono::Duration::days(2))
                .unwrap();
        assert_eq!(reconciled.expired_events, 0);
        assert_eq!(late_journal.total_event_count(), 1);
        assert_eq!(
            late_journal.query(EventQuery::default()).unwrap()[0]
                .event
                .event_id,
            "late-hot"
        );

        let bounded_data = tempfile::tempdir().unwrap();
        let bounded_journal = Arc::new(DurableJournal::open(bounded_data.path()).unwrap());
        for index in 0..130_u64 {
            bounded_journal
                .append(event(
                    &format!("bounded-expired-{index}"),
                    SignalKind::Log,
                    now - chrono::Duration::days(179),
                ))
                .unwrap();
            bounded_journal.storage().seal_all().unwrap();
        }
        bounded_journal
            .append(event("bounded-hot", SignalKind::Log, now))
            .unwrap();
        archive::archive_journal_gcs(&bounded_journal, "gs://sift-retention/bounded").unwrap();
        let bounded_status = archive::committed_status(bounded_data.path())
            .unwrap()
            .unwrap();
        let bounded_machine = durability::SiftStateMachine::new(bounded_journal.clone());
        let fence = serde_json::to_vec(&serde_json::json!({
            "kind": "retention_fence",
            "fence": {
                "source_manifest_uri": bounded_status.manifest_uri,
                "source_manifest_sha256": bounded_status.manifest_sha256,
                "target_generation": 1,
                "evaluate_at": (now + chrono::Duration::days(2)).to_rfc3339()
            }
        }))
        .unwrap();
        bounded_machine.apply_local(1, &fence).unwrap();
        let bounded_expiry = now + chrono::Duration::days(2);
        let first_batch =
            archive::expire_committed_events_at(&bounded_journal, bounded_expiry).unwrap();
        assert!(first_batch.expired_events > 0);
        assert!(
            first_batch.expired_events <= 64,
            "one retention pass must have a fixed segment bound"
        );
        assert!(
            bounded_journal.total_event_count() > 1,
            "an unfinished bounded scan must leave retained work for its next pass"
        );
        let first_status = archive::committed_status(bounded_data.path())
            .unwrap()
            .unwrap();
        assert!(first_status.retention_scan_pending);
        let (_, partial_raw_cursor, _) = bounded_machine.capture_archive_prefix().unwrap();
        bounded_machine
            .prepare_archive_checkpoint(1, partial_raw_cursor)
            .unwrap();
        let mut partial_checkpoint = Vec::new();
        bounded_machine
            .snapshot_at(1, &mut partial_checkpoint)
            .unwrap();
        let bounded_follower_data = tempfile::tempdir().unwrap();
        let bounded_follower_journal =
            Arc::new(DurableJournal::open(bounded_follower_data.path()).unwrap());
        let bounded_follower = durability::SiftStateMachine::new(bounded_follower_journal.clone());
        bounded_follower.apply_local(1, &fence).unwrap();
        bounded_follower
            .restore(&mut partial_checkpoint.as_slice())
            .unwrap();
        assert!(bounded_follower.retention_fence_pending_for_diagnostics());
        assert!(
            bounded_follower_journal
                .query(EventQuery::default())
                .is_err(),
            "a follower must keep queries fenced after a partial retention checkpoint"
        );
        bounded_machine
            .clear_retention_fence_after_checkpoint_for_diagnostics(
                first_status.retention_generation,
            )
            .unwrap();
        assert!(bounded_machine.retention_fence_pending_for_diagnostics());
        let suffix = durability::encode_raft_batch_at_for_diagnostics(
            vec![event("bounded-suffix", SignalKind::Log, now)],
            &now.to_rfc3339_opts(SecondsFormat::Nanos, true),
        )
        .unwrap();
        bounded_machine.apply_local(2, &suffix).unwrap();
        bounded_machine.take_append_outcomes(2).unwrap();
        bounded_follower.apply_local(2, &suffix).unwrap();
        bounded_follower.take_append_outcomes(2).unwrap();
        assert!(
            archive::archive_journal_gcs(&bounded_journal, "gs://sift-retention/bounded")
                .unwrap_err()
                .to_string()
                .contains("bounded retention must complete"),
            "ordinary archive must not checkpoint a suffix over a partial retention catalog"
        );
        let mut expired = first_batch.expired_events;
        let mut passes = 1_u64;
        while bounded_journal.total_event_count() > 2 {
            let batch =
                archive::expire_committed_events_at(&bounded_journal, bounded_expiry).unwrap();
            assert!(batch.expired_events <= 64);
            expired = expired.saturating_add(batch.expired_events);
            passes = passes.saturating_add(1);
            assert!(
                passes <= 4,
                "bounded retention did not make durable progress"
            );
        }
        assert_eq!(expired, 130);
        assert_eq!(bounded_journal.total_event_count(), 2);
        let final_status = archive::committed_status(bounded_data.path())
            .unwrap()
            .unwrap();
        assert!(!final_status.retention_scan_pending);
        bounded_machine
            .prepare_archive_checkpoint(1, partial_raw_cursor)
            .unwrap();
        let mut final_checkpoint = Vec::new();
        bounded_machine
            .snapshot_at(1, &mut final_checkpoint)
            .unwrap();
        bounded_follower
            .restore(&mut final_checkpoint.as_slice())
            .unwrap();
        assert!(!bounded_follower.retention_fence_pending_for_diagnostics());
        assert_eq!(
            bounded_follower_journal
                .query(EventQuery::default())
                .unwrap()
                .len(),
            2,
            "the final retention checkpoint must clear the follower query fence"
        );
        let clear = durability::encode_clear_retention_fence_for_diagnostics(
            final_status.retention_generation,
        )
        .unwrap();
        bounded_machine.apply_local(3, &clear).unwrap();
        assert!(!bounded_machine.retention_fence_pending_for_diagnostics());
        assert_eq!(
            bounded_journal
                .query(EventQuery::default())
                .unwrap()
                .into_iter()
                .map(|event| event.event.event_id)
                .collect::<Vec<_>>(),
            vec!["bounded-hot", "bounded-suffix"]
        );
        let bounded_complete =
            archive::archive_journal_gcs(&bounded_journal, "gs://sift-retention/bounded").unwrap();
        let bounded_restore = tempfile::tempdir().unwrap();
        archive::restore_gcs(&bounded_complete.manifest_uri, bounded_restore.path()).unwrap();
        assert_eq!(
            DurableJournal::open(bounded_restore.path())
                .unwrap()
                .total_event_count(),
            2
        );
    })
    .await
    .unwrap();

    std::env::remove_var("STORAGE_EMULATOR_HOST");
    emulator.abort();
}
