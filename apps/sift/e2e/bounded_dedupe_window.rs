use chrono::{Duration, TimeZone, Utc};
use sift::{storage::DedupeIndex, EventEnvelope, SignalKind, StoredEvent};

fn copy_tree(source: &std::path::Path, target: &std::path::Path) {
    std::fs::create_dir(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &destination);
        } else {
            std::fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn stored(cursor: u64, acknowledged_at: chrono::DateTime<Utc>) -> StoredEvent {
    stored_with_id(cursor, acknowledged_at, &format!("dedupe-{cursor}"))
}

fn stored_with_id(
    cursor: u64,
    acknowledged_at: chrono::DateTime<Utc>,
    event_id: &str,
) -> StoredEvent {
    let mut event = EventEnvelope::for_project(
        "dedupe-window",
        "test",
        event_id,
        SignalKind::Log,
        serde_json::json!({"cursor": cursor}),
    );
    event
        .resource
        .insert("service.name".into(), "dedupe-window".into());
    StoredEvent {
        cursor,
        acknowledged_at: acknowledged_at.to_rfc3339(),
        event,
    }
}

#[test]
fn rebuild_page_ignores_an_expired_row_when_the_same_id_was_reused() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 30, 0).unwrap();
    let expired = now - Duration::hours(6) - Duration::minutes(25);
    let recent = now - Duration::minutes(10);
    let (index, _) = DedupeIndex::open_at(root.path(), now).unwrap();

    index
        .append_batch_at(
            &[
                stored_with_id(1, expired, "reused-id"),
                stored_with_id(2, recent, "reused-id"),
            ],
            now,
        )
        .unwrap();

    assert_eq!(
        index.lookup_at("dedupe-window", "reused-id", now).unwrap(),
        Some(2)
    );
    assert_eq!(index.stats_at(now).unwrap().entry_count, 1);
}

#[test]
fn one_restore_page_can_cross_an_acknowledgement_hour_and_reopen() {
    let root = tempfile::tempdir().unwrap();
    let before_hour = Utc.with_ymd_and_hms(2026, 1, 1, 11, 59, 59).unwrap();
    let after_hour = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 1).unwrap();
    let (index, _) = DedupeIndex::open_at(root.path(), after_hour).unwrap();
    index
        .append_batch_at(&[stored(1, before_hour), stored(2, after_hour)], after_hour)
        .unwrap();
    drop(index);

    let (reopened, stats) = DedupeIndex::open_at(root.path(), after_hour).unwrap();
    assert!(
        !stats.rebuild_required,
        "a valid cross-hour receipt page must not be classified as corrupt"
    );
    assert_eq!(
        reopened
            .lookup_at("dedupe-window", "dedupe-1", after_hour)
            .unwrap(),
        Some(1)
    );
    assert_eq!(
        reopened
            .lookup_at("dedupe-window", "dedupe-2", after_hour)
            .unwrap(),
        Some(2)
    );
}

#[test]
fn acknowledgement_defers_rebuildable_shard_io_to_maintenance() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 30, 0).unwrap();
    let (index, _) = DedupeIndex::open_at(root.path(), now).unwrap();
    let events = (1..=1_000)
        .map(|cursor| stored(cursor, now))
        .collect::<Vec<_>>();
    index.append_batch_at(&events, now).unwrap();

    let generation = root
        .path()
        .join("indexes/dedupe")
        .join(format!("g-{}", now.timestamp().div_euclid(60 * 60)));
    let shard_files_before = std::fs::read_dir(&generation)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("idx"))
        .count();
    assert_eq!(
        shard_files_before, 0,
        "acknowledgement must not synchronously touch hundreds of rebuildable shards"
    );
    assert_eq!(index.pending_entry_count_for_diagnostics(), 1_000);

    index.flush_pending_at_for_diagnostics(now).unwrap();
    assert_eq!(index.pending_entry_count_for_diagnostics(), 0);
    let shard_files_after = std::fs::read_dir(&generation)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("idx"))
        .count();
    assert!(shard_files_after > 0);
    assert_eq!(
        index.lookup_at("dedupe-window", "dedupe-500", now).unwrap(),
        Some(500)
    );
}

#[test]
fn background_maintenance_uses_applied_raft_time_not_the_process_clock() {
    let root = tempfile::tempdir().unwrap();
    let historical = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
    let (index, _) = DedupeIndex::open_at(root.path(), Utc::now()).unwrap();
    index
        .append_batch_at(
            &[stored_with_id(1, historical, "historical-id")],
            historical,
        )
        .unwrap();

    index
        .maintain_for_applied_time_for_diagnostics(true)
        .unwrap();
    assert_eq!(
        index
            .lookup_at(
                "dedupe-window",
                "historical-id",
                historical + Duration::hours(5)
            )
            .unwrap(),
        Some(1)
    );
}

#[test]
fn exact_dedupe_is_restart_safe_inside_six_hours_and_bounded_across_180_days() {
    let root = tempfile::tempdir().unwrap();
    let start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let (index, _) = DedupeIndex::open_at(root.path(), start).unwrap();
    index.append_batch_at(&[stored(1, start)], start).unwrap();
    assert_eq!(
        index
            .lookup_at("dedupe-window", "dedupe-1", start + Duration::hours(6))
            .unwrap(),
        Some(1)
    );
    assert_eq!(
        index
            .lookup_at(
                "dedupe-window",
                "dedupe-1",
                start + Duration::hours(6) + Duration::nanoseconds(1)
            )
            .unwrap(),
        None,
        "the public six-hour window must not be rounded to an hourly bucket"
    );
    drop(index);

    let (index, _) = DedupeIndex::open_at(root.path(), start + Duration::hours(6)).unwrap();
    assert_eq!(
        index
            .lookup_at("dedupe-window", "dedupe-1", start + Duration::hours(6))
            .unwrap(),
        Some(1)
    );
    assert_eq!(
        index
            .lookup_at("dedupe-window", "dedupe-1", start + Duration::hours(8))
            .unwrap(),
        None
    );

    let mut cursor = 2_u64;
    let mut bytes_at_day_30 = 0_u64;
    for day in 0_i64..180 {
        let now = start + Duration::days(day) + Duration::hours(12);
        let events = (0..100)
            .map(|_| {
                let event = stored(cursor, now);
                cursor += 1;
                event
            })
            .collect::<Vec<_>>();
        index.append_batch_at(&events, now).unwrap();
        index.flush_pending_at_for_diagnostics(now).unwrap();
        if day == 29 {
            bytes_at_day_30 = index.disk_bytes().unwrap();
        }
    }
    let bytes_at_day_180 = index.disk_bytes().unwrap();
    assert!(bytes_at_day_30 > 0);
    assert!(
        bytes_at_day_180 <= bytes_at_day_30 + 16 * 1024,
        "dedupe bytes grew from {bytes_at_day_30} to {bytes_at_day_180} with cold retention"
    );
    assert!(index.active_generation_count() <= 7);
}

#[test]
fn replacement_recovers_after_backup_rename_and_validates_every_record() {
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
    let target_root = tempfile::tempdir().unwrap();
    let source_root = tempfile::tempdir().unwrap();
    let (target, _) = DedupeIndex::open_at(target_root.path(), now).unwrap();
    target
        .append_batch_at(&[stored_with_id(1, now, "old-id")], now)
        .unwrap();
    let (source, _) = DedupeIndex::open_at(source_root.path(), now).unwrap();
    source
        .append_batch_at(&[stored_with_id(2, now, "replacement-id")], now)
        .unwrap();
    drop(target);
    drop(source);

    let indexes = target_root.path().join("indexes");
    let target = indexes.join("dedupe");
    let stage = indexes.join(".dedupe-replace-stage");
    let backup = indexes.join(".dedupe-replace-backup");
    copy_tree(&source_root.path().join("indexes/dedupe"), &stage);
    let meta: serde_json::Value =
        serde_json::from_slice(&std::fs::read(stage.join("meta.json")).unwrap()).unwrap();
    std::fs::write(
        indexes.join(".dedupe-replace.json"),
        serde_json::to_vec(&serde_json::json!({
            "format_version": 1,
            "indexed_through_cursor": meta["indexed_through_cursor"],
            "content_sha256": meta["content_sha256"],
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::rename(&target, &backup).unwrap();

    let (recovered, stats) = DedupeIndex::open_at(target_root.path(), now).unwrap();
    assert!(!stats.rebuild_required);
    assert_eq!(
        recovered
            .lookup_at("dedupe-window", "replacement-id", now)
            .unwrap(),
        Some(2)
    );
    assert_eq!(
        recovered.lookup_at("dedupe-window", "old-id", now).unwrap(),
        None
    );
    assert!(!stage.exists());
    assert!(!backup.exists());
    assert!(!indexes.join(".dedupe-replace.json").exists());
    drop(recovered);

    let shard = std::fs::read_dir(target)
        .unwrap()
        .find_map(|entry| {
            let generation = entry.unwrap().path();
            generation.is_dir().then(|| {
                std::fs::read_dir(generation)
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path()
            })
        })
        .unwrap();
    let mut bytes = std::fs::read(&shard).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&shard, bytes).unwrap();
    let (rebuilt, stats) = DedupeIndex::open_at(target_root.path(), now).unwrap();
    assert!(stats.rebuild_required);
    assert_eq!(
        rebuilt
            .lookup_at("dedupe-window", "replacement-id", now)
            .unwrap(),
        None
    );
}
