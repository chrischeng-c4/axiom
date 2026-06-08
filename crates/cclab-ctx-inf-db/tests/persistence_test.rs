//! Integration tests for Phase 2 persistence: WAL, snapshot, recovery.
//!
//! Covers:
//!   - WAL roundtrip (write GraphOps, read them back)
//!   - Snapshot create + load roundtrip
//!   - Full lifecycle: create persistent engine → insert → shutdown → reopen → verify
//!   - WAL-only recovery (no snapshot)
//!   - Snapshot + WAL-delta recovery
//!   - Corrupted WAL: inject garbage and verify graceful degradation

use cclab_ctx_inf_db::*;
use cclab_wal::{find_wal_files, WalConfig, WalReader, WalWriter};
use chrono::{TimeZone, Utc};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn tiny_config(dir: &std::path::Path) -> PersistenceConfig {
    // 10ms flush so tests don't sleep too long.
    PersistenceConfig::for_testing(dir)
}

// ── WAL roundtrip ────────────────────────────────────────────────────

#[test]
fn test_wal_roundtrip() {
    let temp = TempDir::new().unwrap();

    let entity = Entity::new(EntityType::Person, "Alice");
    let entity_id = entity.id;
    let other = Entity::new(EntityType::Person, "Bob");
    let other_id = other.id;
    let rel = Relation::new(RelationType::MetWith, entity_id, other_id);
    let rel_id = rel.id;

    let ops = vec![
        GraphOp::CreateEntity {
            entity: entity.clone(),
        },
        GraphOp::CreateEntity {
            entity: other.clone(),
        },
        GraphOp::CreateRelation {
            relation: rel.clone(),
        },
        GraphOp::DeleteRelation {
            id: rel_id,
            tx_to: Utc::now(),
        },
    ];

    // Write.
    let mut writer = WalWriter::<GraphOp>::new(temp.path(), WalConfig::default()).unwrap();
    for op in &ops {
        writer.append(op).unwrap();
    }
    writer.flush().unwrap();
    drop(writer);

    // Read back.
    let mut reader = WalReader::<GraphOp>::new(temp.path().join("wal-current.log")).unwrap();
    let entries = reader.read_all().unwrap();

    assert_eq!(entries.len(), 4);
    match &entries[0].op {
        GraphOp::CreateEntity { entity: e } => {
            assert_eq!(e.id, entity_id);
            assert_eq!(e.name, "Alice");
        }
        _ => panic!("expected CreateEntity"),
    }
    match &entries[3].op {
        GraphOp::DeleteRelation { id, .. } => assert_eq!(*id, rel_id),
        _ => panic!("expected DeleteRelation"),
    }
}

// ── Snapshot roundtrip ───────────────────────────────────────────────

#[test]
fn test_snapshot_roundtrip() {
    let temp = TempDir::new().unwrap();

    // Build an in-memory engine with some state.
    let engine = CtxInfEngine::new();
    let alice = engine
        .create_entity(Entity::new(EntityType::Person, "Alice"))
        .unwrap();
    let bob = engine
        .create_entity(Entity::new(EntityType::Person, "Bob"))
        .unwrap();
    let _rel = engine
        .create_relation(Relation::new(RelationType::MetWith, alice.id, bob.id))
        .unwrap();

    // Use the writer directly. v2 snapshots carry (wal_file_timestamp,
    // wal_position_in_file) instead of the legacy single wal_position.
    let path = cclab_ctx_inf_db::storage::snapshot::SnapshotWriter::create(
        &engine,
        temp.path(),
        1_700_000_000, // wal_file_timestamp
        12345,         // wal_position_in_file
    )
    .unwrap();
    assert!(path.exists());

    // Load.
    let (data, replay_start) =
        cclab_ctx_inf_db::storage::snapshot::SnapshotLoader::load_latest(temp.path())
            .unwrap()
            .expect("snapshot should be present");

    assert_eq!(
        replay_start,
        cclab_ctx_inf_db::storage::snapshot::WalReplayStart::FromPoint {
            wal_file_timestamp: 1_700_000_000,
            wal_position_in_file: 12345,
        }
    );
    assert_eq!(data.entities.len(), 2);
    assert_eq!(data.relations.len(), 1);

    let names: Vec<_> = data.entities.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));
}

// ── Full lifecycle: create → persist → shutdown → reopen ─────────────

#[test]
fn test_full_persistence_lifecycle() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    // Phase 1: create engine, add data, shutdown.
    let alice_id;
    let bob_id;
    let rel_id;
    {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();

        let alice = engine
            .create_entity(Entity::new(EntityType::Person, "Alice").with_temporal(
                TemporalRange::from(Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).unwrap()),
            ))
            .unwrap();
        alice_id = alice.id;

        let bob = engine
            .create_entity(Entity::new(EntityType::Person, "Bob"))
            .unwrap();
        bob_id = bob.id;

        let rel = engine
            .create_relation(
                Relation::new(RelationType::MetWith, alice_id, bob_id).with_confidence(0.9),
            )
            .unwrap();
        rel_id = rel.id;

        engine.flush();
        // Sleep briefly so the background thread drains the channel.
        thread::sleep(Duration::from_millis(100));
        engine.shutdown().unwrap();
    }

    // Phase 2: reopen, verify everything came back.
    {
        let (engine, stats) = CtxInfEngine::open_with_stats(tiny_config(&dir)).unwrap();
        assert_eq!(stats.wal_entries_replayed, 3);
        assert!(!stats.snapshot_loaded);

        let alice = engine.get_entity(alice_id).unwrap();
        assert_eq!(alice.name, "Alice");
        assert_eq!(
            alice.temporal.valid_from,
            Some(Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).unwrap())
        );

        let bob = engine.get_entity(bob_id).unwrap();
        assert_eq!(bob.name, "Bob");

        let rel = engine.get_relation(rel_id).unwrap();
        assert_eq!(rel.confidence, 0.9);
        assert_eq!(rel.source, alice_id);
        assert_eq!(rel.target, bob_id);

        // Verify adjacency was rebuilt.
        let neighbors = engine
            .neighbors(alice_id, Direction::Outgoing, None)
            .unwrap();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].1.id, bob_id);

        engine.shutdown().unwrap();
    }
}

// ── WAL-only recovery (no snapshot) ──────────────────────────────────

#[test]
fn test_recovery_wal_only() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    // Write a WAL by hand.
    let alice = Entity::new(EntityType::Person, "Alice");
    let alice_id = alice.id;
    let bob = Entity::new(EntityType::Person, "Bob");
    let bob_id = bob.id;
    let rel = Relation::new(RelationType::MetWith, alice_id, bob_id);

    {
        let mut writer = WalWriter::<GraphOp>::new(&dir, WalConfig::default()).unwrap();
        writer
            .append(&GraphOp::CreateEntity { entity: alice })
            .unwrap();
        writer
            .append(&GraphOp::CreateEntity { entity: bob })
            .unwrap();
        writer
            .append(&GraphOp::CreateRelation { relation: rel })
            .unwrap();
        writer.flush().unwrap();
    }

    // Recover (no snapshot).
    let (engine, stats) = RecoveryManager::recover(&dir).unwrap();
    assert!(!stats.snapshot_loaded);
    assert_eq!(stats.wal_entries_replayed, 3);
    assert_eq!(stats.corrupted_entries, 0);

    assert_eq!(engine.get_entity(alice_id).unwrap().name, "Alice");
    assert_eq!(engine.get_entity(bob_id).unwrap().name, "Bob");

    let neighbors = engine
        .neighbors(alice_id, Direction::Outgoing, None)
        .unwrap();
    assert_eq!(neighbors.len(), 1);
}

// ── Snapshot + WAL-delta recovery ────────────────────────────────────

#[test]
fn test_recovery_snapshot_plus_wal_delta() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let alice_id;
    let bob_id;
    let charlie_id;

    {
        // Create engine, add 2 entities, snapshot, then add a third.
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();
        let alice = engine
            .create_entity(Entity::new(EntityType::Person, "Alice"))
            .unwrap();
        alice_id = alice.id;
        let bob = engine
            .create_entity(Entity::new(EntityType::Person, "Bob"))
            .unwrap();
        bob_id = bob.id;

        // Wait for ops to drain to WAL, then snapshot.
        thread::sleep(Duration::from_millis(100));
        let _snap = engine.create_snapshot().unwrap();

        // Add another entity *after* the snapshot — should replay from WAL.
        let charlie = engine
            .create_entity(Entity::new(EntityType::Person, "Charlie"))
            .unwrap();
        charlie_id = charlie.id;

        thread::sleep(Duration::from_millis(100));
        engine.shutdown().unwrap();
    }

    // Reopen and verify all three entities are present.
    {
        let (engine, stats) = CtxInfEngine::open_with_stats(tiny_config(&dir)).unwrap();
        assert!(stats.snapshot_loaded);
        assert_eq!(stats.snapshot_entities, 2);
        // Charlie comes from the WAL delta after the snapshot.
        assert!(
            stats.wal_entries_replayed >= 1,
            "expected at least 1 WAL entry replayed, got {}",
            stats.wal_entries_replayed
        );

        assert_eq!(engine.get_entity(alice_id).unwrap().name, "Alice");
        assert_eq!(engine.get_entity(bob_id).unwrap().name, "Bob");
        assert_eq!(engine.get_entity(charlie_id).unwrap().name, "Charlie");

        // type_index should not double-count: querying Person should give exactly 3.
        let people = engine.entities_by_type(&EntityType::Person);
        assert_eq!(
            people.len(),
            3,
            "type_index should contain Alice, Bob, Charlie exactly once each"
        );

        engine.shutdown().unwrap();
    }
}

// ── Update + delete replay ───────────────────────────────────────────

#[test]
fn test_recovery_update_and_delete() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let kept_id;
    let updated_id;

    {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();
        let kept = engine
            .create_entity(Entity::new(EntityType::Person, "Kept"))
            .unwrap();
        kept_id = kept.id;

        let updated = engine
            .create_entity(Entity::new(EntityType::Person, "Original Name"))
            .unwrap();
        updated_id = updated.id;
        engine
            .update_entity(updated_id, 0, |e| e.name = "New Name".into())
            .unwrap();

        let doomed = engine
            .create_entity(Entity::new(EntityType::Person, "Doomed"))
            .unwrap();
        engine.delete_entity(doomed.id, false).unwrap();

        thread::sleep(Duration::from_millis(100));
        engine.shutdown().unwrap();
    }

    {
        let engine = CtxInfEngine::open(tiny_config(&dir)).unwrap();
        assert_eq!(engine.get_entity(kept_id).unwrap().name, "Kept");

        let updated = engine.get_entity(updated_id).unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.version, 1);

        // The "Doomed" entity was deleted — only Kept + updated should remain.
        let people = engine.entities_by_type(&EntityType::Person);
        assert_eq!(people.len(), 2);

        engine.shutdown().unwrap();
    }
}

// ── Corrupted WAL: graceful degradation ──────────────────────────────

#[test]
fn test_recovery_corrupted_wal() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let alice = Entity::new(EntityType::Person, "Alice");
    let alice_id = alice.id;
    let bob = Entity::new(EntityType::Person, "Bob");
    let bob_id = bob.id;

    // Write 2 valid entries.
    {
        let mut writer = WalWriter::<GraphOp>::new(&dir, WalConfig::default()).unwrap();
        writer
            .append(&GraphOp::CreateEntity { entity: alice })
            .unwrap();
        writer
            .append(&GraphOp::CreateEntity { entity: bob })
            .unwrap();
        writer.flush().unwrap();
    }

    // Corrupt the tail of the WAL — append junk bytes.
    {
        let wal_path = dir.join("wal-current.log");
        let mut file = OpenOptions::new().write(true).open(&wal_path).unwrap();
        file.seek(SeekFrom::End(0)).unwrap();
        file.write_all(&[0xFF; 256]).unwrap();
        file.sync_all().unwrap();
    }

    // Recover — should pick up the 2 valid entries, skip the garbage tail.
    let (engine, stats) = RecoveryManager::recover(&dir).unwrap();
    assert_eq!(
        stats.wal_entries_replayed, 2,
        "should replay both valid entries"
    );
    assert!(
        stats.corrupted_entries >= 1,
        "should detect at least one corrupted entry, got {}",
        stats.corrupted_entries
    );

    assert_eq!(engine.get_entity(alice_id).unwrap().name, "Alice");
    assert_eq!(engine.get_entity(bob_id).unwrap().name, "Bob");
}

// ── Empty data dir ───────────────────────────────────────────────────

#[test]
fn test_recovery_empty_dir() {
    let temp = TempDir::new().unwrap();
    let (engine, stats) = RecoveryManager::recover(temp.path()).unwrap();
    assert_eq!(stats.wal_entries_replayed, 0);
    assert_eq!(stats.snapshot_entities, 0);
    assert!(!stats.snapshot_loaded);
    assert_eq!(engine.stats().entity_count, 0);
}

// ── create_snapshot: wal_position is durable ─────────────────────────

/// Regression test for the create_snapshot wal_position race (bug
/// `bug-ctx-inf-db-create-snapshot-captures-wal-position-before-async-flush`).
///
/// Before the fix, `create_snapshot` fired a non-blocking `Flush` command and
/// then read `wal_position()` — so the captured position could point past the
/// last fsynced byte. If the process crashed before the background thread
/// consumed the Flush, the recovered snapshot referenced a phantom WAL
/// position and replay silently dropped every in-flight op.
///
/// This test simulates that crash scenario by:
///   1. issuing N ops,
///   2. taking a snapshot (must block until the WAL is durable),
///   3. truncating the WAL file to the position recorded in the snapshot —
///      modeling a crash where every byte past the snapshot's position was
///      never persisted (lost from OS page cache),
///   4. reopening the engine and verifying recovery restores all N ops
///      exactly (no losses, no double-count).
#[test]
fn test_create_snapshot_wal_position_is_durable() {
    use cclab_ctx_inf_db::storage::snapshot::{
        find_snapshot_files, SnapshotHeader, WalReplayStart,
    };
    use std::fs::{File, OpenOptions};
    use std::io::BufReader;

    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    const N: usize = 12;
    let mut entity_ids = Vec::with_capacity(N);

    {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();
        for i in 0..N {
            let e = engine
                .create_entity(Entity::new(
                    EntityType::Person,
                    format!("pre-snapshot-{}", i),
                ))
                .unwrap();
            entity_ids.push(e.id);
        }

        // Take the snapshot. Post-fix: this MUST block until the background
        // thread has fsynced the WAL and captured the durable position.
        let snap_path = engine.create_snapshot().unwrap();
        assert!(snap_path.exists(), "snapshot file should have been written");

        // Drop the engine WITHOUT calling `shutdown()`. We still rely on Drop
        // to stop the thread so the WAL file is not held open, but we do NOT
        // treat that as a clean shutdown barrier — the crash simulation below
        // discards anything the background thread might have written after
        // the snapshot barrier point.
        drop(engine);
    }

    // ── Simulate crash: truncate the WAL file to the snapshot's recorded
    //    wal_position. Any bytes past that point model data that never
    //    reached disk (e.g. still in the OS page cache at kernel panic).
    let snapshot_paths = find_snapshot_files(&dir).unwrap();
    assert_eq!(
        snapshot_paths.len(),
        1,
        "exactly one snapshot should have been created"
    );
    let snap_path = &snapshot_paths[0];

    let recorded_wal_pos = {
        let file = File::open(snap_path).unwrap();
        let mut reader = BufReader::new(file);
        let (_header, replay_start) = SnapshotHeader::read(&mut reader).unwrap();
        match replay_start {
            WalReplayStart::FromPoint {
                wal_position_in_file,
                ..
            } => wal_position_in_file,
            WalReplayStart::FullReplay => {
                panic!("expected FromPoint replay start, got FullReplay");
            }
        }
    };
    assert!(
        recorded_wal_pos > 0,
        "snapshot wal_position should be past the WAL header (got 0)"
    );

    let wal_path = dir.join("wal-current.log");
    let actual_wal_len = std::fs::metadata(&wal_path).unwrap().len();
    assert!(
        recorded_wal_pos <= actual_wal_len,
        "BUG: snapshot recorded wal_position {} exceeds actual WAL file size {} — \
         fire-and-forget flush race has reappeared",
        recorded_wal_pos,
        actual_wal_len
    );

    // Truncate: discard any bytes past the snapshot's durable position.
    let file = OpenOptions::new().write(true).open(&wal_path).unwrap();
    file.set_len(recorded_wal_pos).unwrap();
    file.sync_all().unwrap();
    drop(file);

    // ── Reopen and verify: all N ops recorded before the snapshot must
    //    be restored. Because the snapshot captures engine state in-memory
    //    (not just the WAL position), all N entities come back via
    //    SnapshotLoader even though the WAL was truncated.
    let (engine, stats) = CtxInfEngine::open_with_stats(tiny_config(&dir)).unwrap();
    assert!(
        stats.snapshot_loaded,
        "snapshot must load cleanly after WAL truncation to its recorded position"
    );
    assert_eq!(
        stats.snapshot_entities, N,
        "snapshot must contain all {} entities issued before create_snapshot",
        N
    );
    // After truncation, the WAL tail exactly matches the snapshot position,
    // so replay finds zero entries past the snapshot — no losses, no dupes.
    assert_eq!(
        stats.wal_entries_replayed, 0,
        "WAL truncated to snapshot position — no tail entries to replay"
    );

    for (i, id) in entity_ids.iter().enumerate() {
        let e = engine
            .get_entity(*id)
            .unwrap_or_else(|_| panic!("entity {} (index {}) missing after recovery", id, i));
        assert_eq!(e.name, format!("pre-snapshot-{}", i));
    }

    let people = engine.entities_by_type(&EntityType::Person);
    assert_eq!(
        people.len(),
        N,
        "type_index should have all {} entities after recovery",
        N
    );

    engine.shutdown().unwrap();
}

// ── WAL files appear on disk ─────────────────────────────────────────

#[test]
fn test_wal_file_created() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    {
        let engine = CtxInfEngine::with_persistence(tiny_config(&dir)).unwrap();
        engine
            .create_entity(Entity::new(EntityType::Person, "Test"))
            .unwrap();
        engine.flush();
        thread::sleep(Duration::from_millis(100));
        engine.shutdown().unwrap();
    }

    let wal_files = find_wal_files(&dir).unwrap();
    assert!(!wal_files.is_empty(), "WAL file must exist on disk");
}
