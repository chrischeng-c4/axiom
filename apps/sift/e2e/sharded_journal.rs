// HANDWRITE-BEGIN gap="sift-sharded-journal-tests" tracker="1659" reason="Verify deterministic routing, future-only epochs, blob ordering, sealing/movement, torn tails, and compatibility recovery."
use std::{collections::BTreeMap, fs::OpenOptions, io::Write};

use sift::{
    storage::{RawStorage, StorageConfig, VIRTUAL_BUCKETS},
    EventEnvelope, SignalKind, StoredEvent,
};

fn stored(cursor: u64, id: &str) -> StoredEvent {
    let mut event = EventEnvelope::new(id, SignalKind::Log, serde_json::json!({"cursor": cursor}));
    event.resource = BTreeMap::from([("service.name".into(), "shard-test".into())]);
    StoredEvent {
        cursor,
        acknowledged_at: "2026-07-14T00:00:00Z".into(),
        event,
    }
}

#[test]
fn bucket_routes_are_deterministic_and_epoch_changes_are_future_only() {
    let temp = tempfile::tempdir().unwrap();
    let storage = RawStorage::open(temp.path()).unwrap();
    let before = storage.route("stable-event", 1);
    assert!(usize::from(before.bucket) < VIRTUAL_BUCKETS);
    assert_eq!(before, storage.route("stable-event", 99));

    let mut next = (0..VIRTUAL_BUCKETS)
        .map(|bucket| ((bucket + 1) % 16) as u16)
        .collect::<Vec<_>>();
    next[usize::from(before.bucket)] = (before.shard + 1) % 16;
    storage.activate_epoch(100, next).unwrap();

    let historical = storage.route("stable-event", 99);
    let future = storage.route("stable-event", 100);
    assert_eq!(historical.epoch, 1);
    assert_eq!(historical.shard, before.shard);
    assert_eq!(future.epoch, 2);
    assert_ne!(future.shard, before.shard);
    drop(storage);

    let reopened = RawStorage::open(temp.path()).unwrap();
    assert_eq!(reopened.route("stable-event", 99), historical);
    assert_eq!(reopened.route("stable-event", 100), future);
}

#[test]
fn torn_active_tail_is_truncated_and_sealed_segments_move_without_rewrite() {
    let temp = tempfile::tempdir().unwrap();
    let storage = RawStorage::open_with_config(
        temp.path(),
        StorageConfig {
            max_segment_events: 10,
            ..StorageConfig::default()
        },
    )
    .unwrap();
    let first_route = storage.route("one", 1);
    let second_id = (0..10_000)
        .map(|candidate| format!("two-{candidate}"))
        .find(|candidate| storage.route(candidate, 2).shard == first_route.shard)
        .unwrap();
    storage.append(&stored(1, "one")).unwrap();
    storage.append(&stored(2, &second_id)).unwrap();
    let active = storage.active_segment_paths();
    assert_eq!(active.len(), 1);
    drop(storage);
    OpenOptions::new()
        .append(true)
        .open(&active[0])
        .unwrap()
        .write_all(b"torn-tail")
        .unwrap();

    let recovered = RawStorage::open_with_config(
        temp.path(),
        StorageConfig {
            max_segment_events: 10,
            ..StorageConfig::default()
        },
    )
    .unwrap();
    assert_eq!(recovered.recovered_events().unwrap().len(), 2);
    let manifests = recovered.seal_all().unwrap();
    assert_eq!(manifests.len(), 1);
    let original = std::fs::read(&manifests[0].local_path).unwrap();
    let moved_dir = tempfile::tempdir().unwrap();
    let moved = recovered
        .move_segment(&manifests[0].segment_id, moved_dir.path())
        .unwrap();
    assert_eq!(std::fs::read(&moved.local_path).unwrap(), original);
    assert_eq!(moved.epoch, manifests[0].epoch);
    assert_eq!(moved.shard, manifests[0].shard);
    assert_eq!(recovered.recovered_events().unwrap().len(), 2);
}

#[test]
fn encoded_byte_limit_marks_an_active_segment_ready_to_seal() {
    let temp = tempfile::tempdir().unwrap();
    let event = stored(1, "byte-limit");
    let frame_bytes = serde_json::to_vec(&event).unwrap().len() + 16;
    let storage = RawStorage::open_with_config(
        temp.path(),
        StorageConfig {
            max_segment_events: 100,
            max_segment_bytes: frame_bytes,
            ..StorageConfig::default()
        },
    )
    .unwrap();

    storage.append(&event).unwrap();
    let manifests = storage.seal_ready().unwrap();

    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].event_count, 1);
    assert_eq!(manifests[0].bytes, frame_bytes as u64);
}
// HANDWRITE-END
