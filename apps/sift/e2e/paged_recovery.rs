use sift::{
    storage::{DedupeIndex, RawStorage, StorageConfig},
    DurableJournal, EventEnvelope, EventQuery, ServiceState, SignalKind, StoredEvent,
};

fn event(index: usize) -> EventEnvelope {
    let mut event = EventEnvelope::for_project(
        "project-a",
        "prod",
        format!("paged-{index:05}"),
        SignalKind::Log,
        serde_json::json!({"message": format!("message-{index}")}),
    );
    event
        .resource
        .insert("service.name".into(), "paged-recovery".into());
    event
}

fn stored(index: u64, message: &str) -> StoredEvent {
    let mut envelope = EventEnvelope::for_project(
        "restore-project",
        "prod",
        format!("restore-{index}"),
        SignalKind::Log,
        serde_json::json!({"message": message}),
    );
    envelope
        .resource
        .insert("service.name".into(), "bounded-restore".into());
    StoredEvent {
        cursor: index,
        acknowledged_at: "2026-01-01T00:00:00Z".into(),
        event: envelope,
    }
}

#[tokio::test]
async fn restart_pages_canonical_data_and_rebuilds_a_missing_disk_index() {
    const EVENTS: usize = 20_500;
    let temp = tempfile::tempdir().unwrap();
    let state = ServiceState::open(temp.path()).unwrap();
    for first in (0..EVENTS).step_by(1_000) {
        let last = (first + 1_000).min(EVENTS);
        state
            .append_events((first..last).map(event).collect())
            .await
            .unwrap();
    }
    state.finish_drain().await.unwrap();
    drop(state);
    let (dedupe, _) = DedupeIndex::open(temp.path()).unwrap();
    dedupe
        .flush_pending_at_for_diagnostics(chrono::Utc::now())
        .unwrap();
    drop(dedupe);

    let dedupe_root = temp.path().join("indexes/dedupe");
    let shard = std::fs::read_dir(&dedupe_root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .flat_map(|entry| {
            std::fs::read_dir(entry.path())
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|shard| shard.path())
        })
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("idx"))
        .expect("at least one dedupe shard");
    std::fs::remove_file(shard).unwrap();

    let journal = DurableJournal::open_with_resident_limit(temp.path(), 16).unwrap();
    assert!(journal.resident_event_count() <= 16);
    assert_eq!(journal.total_event_count(), EVENTS as u64);
    assert!(journal.append(event(0)).unwrap().duplicate);

    let mut count = 0;
    let mut after = 0;
    loop {
        let page = journal
            .query(EventQuery {
                after,
                limit: 3_000,
                ..EventQuery::default()
            })
            .unwrap();
        let Some(last) = page.last() else {
            break;
        };
        after = last.cursor;
        count += page.len();
    }
    assert_eq!(count, EVENTS);
    assert_eq!(after, EVENTS as u64);
}

#[test]
fn full_prefix_restore_is_bounded_by_one_segment_and_one_append_page() {
    const PREFIX: u64 = 20_000;
    const SUFFIX: u64 = 100;
    let retained_root = tempfile::tempdir().unwrap();
    let local_root = tempfile::tempdir().unwrap();
    let config = StorageConfig {
        max_segment_events: 257,
        max_segment_bytes: 2 * 1024 * 1024,
        ..StorageConfig::default()
    };
    let retained = RawStorage::open_with_config(retained_root.path(), config.clone()).unwrap();
    let local = RawStorage::open_with_config(local_root.path(), config).unwrap();

    for first in (1..=PREFIX).step_by(1_000) {
        let last = (first + 999).min(PREFIX);
        let expected = (first..=last)
            .map(|cursor| stored(cursor, "authoritative"))
            .collect::<Vec<_>>();
        let conflicting = (first..=last)
            .map(|cursor| stored(cursor, "stale-local"))
            .collect::<Vec<_>>();
        retained.append_batch(&expected).unwrap();
        local.append_batch(&conflicting).unwrap();
        retained.seal_ready().unwrap();
        local.seal_ready().unwrap();
    }
    let suffix = (PREFIX + 1..=PREFIX + SUFFIX)
        .map(|cursor| stored(cursor, "raft-suffix"))
        .collect::<Vec<_>>();
    local.append_batch(&suffix).unwrap();

    let stats = local.reconcile_retained_prefix(&retained, PREFIX).unwrap();
    assert!(stats.max_buffered_events <= 1_000, "{stats:?}");
    let rows = local
        .query_events(None, 0, (PREFIX + SUFFIX) as usize)
        .unwrap();
    assert_eq!(rows.len() as u64, PREFIX + SUFFIX);
    assert_eq!(rows[0].event.payload["message"], "authoritative");
    assert_eq!(rows.last().unwrap().event.payload["message"], "raft-suffix");
}
