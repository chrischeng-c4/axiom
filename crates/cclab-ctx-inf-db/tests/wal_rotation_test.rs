//! WAL rotation threshold + multi-file recovery invariants (R2a/b/c/d) plus
//! snapshot-position-rotation correctness tests (R3b / R3c) added alongside
//! the Strategy-B fix for
//! `bug-ctx-inf-db-snapshot-wal-position-file-relative-across-rotation`.
//!
//! Naming: `wal-current.log` (active) + `wal-<timestamp-token>.log` (rotated);
//! per `cclab-wal/src/writer.rs`.

use cclab_ctx_inf_db::storage::snapshot::{SnapshotLoader, SNAPSHOT_MAGIC};
use cclab_ctx_inf_db::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// ── Helpers (R3) ─────────────────────────────────────────────────────

fn setup_persistence_for_testing() -> (TempDir, PersistenceConfig) {
    let temp = TempDir::new().unwrap();
    let config = PersistenceConfig::for_testing(temp.path());
    (temp, config)
}

fn wal_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|s| s.starts_with("wal-") && s.ends_with(".log"))
        })
        .collect();
    files.sort();
    files
}

fn count_wal_files(dir: &Path) -> usize {
    wal_files(dir).len()
}

/// ~4 KiB property blob so ~20 entities crosses the 64 KiB threshold.
fn make_fat_entity(name: &str) -> Entity {
    let blob: String = "x".repeat(4 * 1024);
    Entity::new(EntityType::Person, name).with_property("blob", PropertyValue::String(blob))
}

/// Spawn engine, write `count` fat entities, flush, shutdown. Returns
/// `(insert_order_ids, names_by_id)`.
fn write_fat_entities_and_shutdown(
    config: PersistenceConfig,
    count: usize,
) -> (Vec<EntityId>, HashMap<EntityId, String>) {
    let engine = CtxInfEngine::with_persistence(config).unwrap();
    let mut ids = Vec::with_capacity(count);
    let mut names = HashMap::with_capacity(count);
    for i in 0..count {
        let name = format!("Fat-{:03}", i);
        let e = engine.create_entity(make_fat_entity(&name)).unwrap();
        names.insert(e.id, name);
        ids.push(e.id);
    }
    engine.flush();
    // Drain the channel + let any pending rotation complete.
    thread::sleep(Duration::from_millis(250));
    engine.shutdown().unwrap();
    (ids, names)
}

// ── R2a: rotation fires at threshold ────────────────────────────────

#[test]
fn test_rotation_fires_at_wal_max_file_size() {
    let (_temp, config) = setup_persistence_for_testing();
    let dir = config.data_dir.clone();

    // 20 × ~4 KiB = ~80 KiB serialised → crosses 64 KiB once. Staying below
    // two rotations keeps the assertion focused on the first threshold
    // crossing; `WalWriter::rotate` now uses a unique timestamp token.
    let (_ids, _names) = write_fat_entities_and_shutdown(config, 20);

    let files = wal_files(&dir);
    assert!(
        files.len() >= 2,
        "expected >= 2 WAL files after crossing 64 KiB threshold, got {}: {:?}",
        files.len(),
        files
    );

    let active = dir.join("wal-current.log");
    assert!(active.exists(), "active wal-current.log must exist");

    // Rotated file size = pre-rotation position, just past threshold.
    let mut saw_near_threshold = false;
    for p in files.iter().filter(|p| *p != &active) {
        let len = fs::metadata(p).unwrap().len();
        assert!(
            len <= 64 * 1024 + 8 * 1024,
            "rotated {:?} too big: {}",
            p,
            len
        );
        if len >= 60 * 1024 {
            saw_near_threshold = true;
        }
    }
    assert!(saw_near_threshold, "some rotated file must be near 64 KiB");
}

// ── R2b: recovery replays multi-file WAL in order ───────────────────

#[test]
fn test_recovery_replays_multiple_wal_files_in_order() {
    let (_temp, config) = setup_persistence_for_testing();
    let dir = config.data_dir.clone();
    let (ids, names) = write_fat_entities_and_shutdown(config.clone(), 20);

    assert!(count_wal_files(&dir) >= 2, "precondition: rotation fired");

    // Reopen — no snapshot → recovery replays every WAL file in
    // `find_wal_files` sort order (wal-<timestamp-token>.log sorts before
    // wal-current.log lexically).
    let (engine, stats) = CtxInfEngine::open_with_stats(config).unwrap();
    assert!(!stats.snapshot_loaded);
    assert_eq!(
        stats.wal_entries_replayed,
        ids.len(),
        "every WAL entry across all files must be replayed"
    );
    assert_eq!(stats.corrupted_entries, 0);

    for id in &ids {
        let e = engine
            .get_entity(*id)
            .unwrap_or_else(|_| panic!("entity {} missing after multi-file replay", id));
        assert_eq!(&e.name, names.get(id).unwrap());
        assert!(matches!(e.entity_type, EntityType::Person));
    }

    let people = engine.entities_by_type(&EntityType::Person);
    assert_eq!(people.len(), ids.len(), "no dup, no gap in type_index");

    engine.shutdown().unwrap();
}

// ── R2c: snapshot wal_position coherent across rotation ─────────────

// R5: fails on `ctx_inf_db`. `create_snapshot` stores `wal_writer.position()`
// (per-file byte offset); `replay_wal` applies that threshold against each
// file's reader position independently. Post-rotation the snapshot position
// is small (active file reset to header=32+N), then reused against the larger
// rotated-out file — pre-snap entries past the threshold get re-replayed.
// State stays correct (idempotent `apply_op::CreateEntity`) but
// `wal_entries_replayed` inflates (local repro: 15 vs 5 expected).
// Proposed: `bug-ctx-inf-db-snapshot-wal-position-file-relative-across-rotation`
#[test]
fn test_snapshot_wal_position_coherent_across_rotation() {
    let (_temp, config) = setup_persistence_for_testing();
    let dir = config.data_dir.clone();

    let pre_snap_ids;
    let post_snap_ids;
    {
        let engine = CtxInfEngine::with_persistence(config.clone()).unwrap();
        let mut ids = Vec::new();
        for i in 0..20 {
            let e = engine
                .create_entity(make_fat_entity(&format!("Pre-{:03}", i)))
                .unwrap();
            ids.push(e.id);
        }
        engine.flush();
        thread::sleep(Duration::from_millis(250));
        assert!(
            count_wal_files(&dir) >= 2,
            "rotation must fire pre-snapshot"
        );
        pre_snap_ids = ids;

        let _snap_path = engine.create_snapshot().unwrap();

        let mut more = Vec::new();
        for i in 0..5 {
            let e = engine
                .create_entity(make_fat_entity(&format!("Post-{:03}", i)))
                .unwrap();
            more.push(e.id);
        }
        post_snap_ids = more;

        engine.flush();
        thread::sleep(Duration::from_millis(150));
        engine.shutdown().unwrap();
    }

    let (engine, stats) = CtxInfEngine::open_with_stats(config).unwrap();
    assert!(stats.snapshot_loaded);
    assert_eq!(stats.snapshot_entities, 20);
    assert_eq!(
        stats.wal_entries_replayed, 5,
        "post-snapshot WAL delta must be exactly 5; got {} (pre-snap re-replayed?)",
        stats.wal_entries_replayed
    );

    assert_eq!(
        engine.stats().entity_count,
        25,
        "25 unique entities (20 snapshot + 5 WAL delta)"
    );
    for id in pre_snap_ids.iter().chain(post_snap_ids.iter()) {
        assert!(engine.get_entity(*id).is_ok(), "{} missing post-reopen", id);
    }
    let people = engine.entities_by_type(&EntityType::Person);
    assert_eq!(people.len(), 25, "no duplicate index entries");

    engine.shutdown().unwrap();
}

// ── R2d: rotated-out file fsynced + complete ────────────────────────

#[test]
fn test_rotated_file_is_fsynced_and_complete() {
    let (_temp, config) = setup_persistence_for_testing();
    let dir = config.data_dir.clone();

    // Cross threshold, shut down, recover. If the rotating-out file had been
    // switched away from before fsync, its tail entries would be missing
    // (< ids.len() replayed) or CRC-corrupt (corrupted_entries > 0).
    let (ids, names) = write_fat_entities_and_shutdown(config.clone(), 20);
    assert!(count_wal_files(&dir) >= 2, "precondition: rotation fired");

    let (engine, stats) = CtxInfEngine::open_with_stats(config).unwrap();
    assert!(!stats.snapshot_loaded);
    assert_eq!(
        stats.corrupted_entries, 0,
        "no torn-tail CRC failures expected after clean rotation"
    );
    assert_eq!(
        stats.wal_entries_replayed,
        ids.len(),
        "all {} entries must survive rotation (rotated file not fsynced?)",
        ids.len()
    );

    for id in &ids {
        let e = engine.get_entity(*id).unwrap();
        assert_eq!(&e.name, names.get(id).unwrap());
    }

    engine.shutdown().unwrap();
}

// ── R3b: non-idempotent op (Delete) survives rotation + snapshot ────

/// Demonstrates correctness (not just counter-accuracy): a pre-snapshot
/// `Delete(E1)` must remain durable across a WAL rotation that pushes the
/// `Create(E1)` onto a rotated-out file and through a follow-up snapshot.
/// Before the Strategy-B fix the rotated-out file would be re-replayed from
/// its post-header byte, effectively resurrecting E1.
#[test]
fn test_non_idempotent_op_survives_rotation_plus_snapshot() {
    let (_temp, config) = setup_persistence_for_testing();
    let dir = config.data_dir.clone();

    let e1_id;
    let e2_id;
    let e3_id;
    {
        let engine = CtxInfEngine::with_persistence(config.clone()).unwrap();

        // Insert E1 (small — doesn't cause rotation).
        let e1 = engine
            .create_entity(Entity::new(EntityType::Person, "E1"))
            .unwrap();
        e1_id = e1.id;
        engine.flush();
        thread::sleep(Duration::from_millis(150));

        // Snapshot S0 — captures {E1}.
        let _s0 = engine.create_snapshot().unwrap();

        // Delete E1 — the non-idempotent op under test.
        engine.delete_entity(e1_id, false).unwrap();

        // Fill WAL past rotation threshold with ~20 fat dummies. This pushes
        // the rotated-out file above 64 KiB and forces a rotation — the
        // rotated-out file now contains [Create E1, Delete E1, dummies...].
        for i in 0..20 {
            engine
                .create_entity(make_fat_entity(&format!("Dummy-{:03}", i)))
                .unwrap();
        }
        engine.flush();
        thread::sleep(Duration::from_millis(300));
        assert!(
            count_wal_files(&dir) >= 2,
            "a rotation must have fired between S0 and S1 (have {} wal files)",
            count_wal_files(&dir)
        );

        // E2 in the NEW wal-current.log (post-rotation, pre-S1).
        let e2 = engine
            .create_entity(Entity::new(EntityType::Person, "E2"))
            .unwrap();
        e2_id = e2.id;
        engine.flush();
        thread::sleep(Duration::from_millis(150));

        // Snapshot S1 — captures {dummies, E2} (NOT E1 — it was deleted).
        let _s1 = engine.create_snapshot().unwrap();

        // E3 in wal-current.log after S1.
        let e3 = engine
            .create_entity(Entity::new(EntityType::Person, "E3"))
            .unwrap();
        e3_id = e3.id;
        engine.flush();
        thread::sleep(Duration::from_millis(150));
        engine.shutdown().unwrap();
    }

    // Reopen. Latest snapshot is S1; it skips wal-<T>.log (pre-S1) entirely
    // and applies a byte-offset skip to wal-current.log at the position
    // captured at S1-time, leaving only Create(E3) to replay.
    let (engine, stats) = CtxInfEngine::open_with_stats(config).unwrap();
    assert!(stats.snapshot_loaded, "S1 must be loaded on reopen");

    // E1 must be absent: the Delete was durable.
    assert!(
        engine.get_entity(e1_id).is_err(),
        "E1 resurrected — pre-snapshot Delete lost across rotation (bug regression)"
    );
    // E2 (pre-S1 but post-rotation) must survive.
    assert!(
        engine.get_entity(e2_id).is_ok(),
        "E2 missing — pre-S1 post-rotation Create not restored by snapshot"
    );
    // E3 (post-S1) must survive via WAL replay of wal-current.log suffix.
    assert!(
        engine.get_entity(e3_id).is_ok(),
        "E3 missing — post-S1 WAL delta not replayed"
    );

    engine.shutdown().unwrap();
}

// ── R3c: v1 snapshot → graceful fallback to full WAL replay ─────────

/// Hand-craft a v1 snapshot file (legacy 72-byte header, byte-offset only)
/// pointing to a mid-file position, then open the engine. The v1 snapshot's
/// position is untrustworthy (see bug slug), so the loader must ignore it and
/// the recovery path must replay every WAL entry from scratch — asserting
/// `wal_entries_replayed` equals the total entry count (NOT a half-skip).
#[test]
fn test_snapshot_old_version_falls_back_to_full_replay() {
    let temp = TempDir::new().unwrap();
    let mut config = PersistenceConfig::for_testing(temp.path());
    // Roomy rotation threshold: we want N small entries in a single file so we
    // know `wal_entries_replayed == N` on full replay with no rotation noise.
    config.wal_max_file_size = 1024 * 1024; // 1 MiB
    let data_dir = config.data_dir.clone();

    const N: usize = 7;
    {
        let engine = CtxInfEngine::with_persistence(config.clone()).unwrap();
        for i in 0..N {
            engine
                .create_entity(Entity::new(EntityType::Person, &format!("V1-{}", i)))
                .unwrap();
        }
        engine.flush();
        thread::sleep(Duration::from_millis(200));
        engine.shutdown().unwrap();
    }

    // There must be WAL content on disk.
    let wal_files = fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|s| s.starts_with("wal-") && s.ends_with(".log"))
        })
        .count();
    assert!(wal_files >= 1, "need at least one WAL file on disk");

    // Hand-craft a v1 snapshot header: 72 bytes, with a mid-file wal_position
    // that — if trusted — would incorrectly skip the first few entries.
    // v1 layout:
    //   magic(4) + version(4) + created_at(8) + entity_count(8) + relation_count(8)
    //   + wal_position(8) + checksum(32)
    // The payload is an empty {entities:[], relations:[]} JSON, so checksum
    // must match SHA256 of that exact byte string.
    use sha2::{Digest, Sha256};
    let empty_payload = br#"{"entities":[],"relations":[]}"#;
    let mut hasher = Sha256::new();
    hasher.update(empty_payload);
    let checksum: [u8; 32] = hasher.finalize().into();

    let snap_path = data_dir.join("snapshot-0000000000000000001.snap");
    {
        let mut f = File::create(&snap_path).unwrap();
        f.write_all(SNAPSHOT_MAGIC).unwrap();
        f.write_all(&1u32.to_be_bytes()).unwrap(); // version 1
        f.write_all(&0i64.to_be_bytes()).unwrap(); // created_at
        f.write_all(&0u64.to_be_bytes()).unwrap(); // entity_count (empty)
        f.write_all(&0u64.to_be_bytes()).unwrap(); // relation_count (empty)
        f.write_all(&9999u64.to_be_bytes()).unwrap(); // legacy wal_position (would-be-trusted)
        f.write_all(&checksum).unwrap();
        f.write_all(empty_payload).unwrap();
        f.sync_all().unwrap();
    }

    // Sanity: the SnapshotLoader itself reports FullReplay for v1.
    let (_data, replay) = SnapshotLoader::load_latest(&data_dir)
        .unwrap()
        .expect("hand-crafted v1 snapshot present");
    assert_eq!(
        replay,
        cclab_ctx_inf_db::storage::snapshot::WalReplayStart::FullReplay,
        "v1 snapshot must yield WalReplayStart::FullReplay"
    );

    // Now drive the full recovery path through the engine.
    let (engine, stats) = CtxInfEngine::open_with_stats(config).unwrap();
    assert!(
        stats.snapshot_loaded,
        "snapshot was loaded (even though v1)"
    );
    assert_eq!(
        stats.snapshot_entities, 0,
        "v1 snapshot was written with 0 entities in payload"
    );
    assert_eq!(
        stats.wal_entries_replayed, N,
        "v1 fallback must replay every WAL entry (full replay), not half-skip \
         based on the untrusted legacy byte offset (got {} of {})",
        stats.wal_entries_replayed, N
    );
    assert_eq!(stats.corrupted_entries, 0);
    assert_eq!(
        engine.stats().entity_count,
        N,
        "post-replay entity count matches number of creates"
    );

    engine.shutdown().unwrap();
}
