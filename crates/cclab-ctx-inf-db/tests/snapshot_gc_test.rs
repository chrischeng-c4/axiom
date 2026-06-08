//! Integration tests for snapshot garbage collection.
//!
//! Verifies that `cleanup_old_snapshots` is wired into the post-snapshot path
//! of `CtxInfEngine::create_snapshot` and honors `PersistenceConfig::snapshot_keep_count`.
//!
//! Scenarios (issue: snapshot_keep_count is decorative — no auto-cleanup):
//!   R3a - keeps only N after N+X snapshot calls
//!   R3b - keeps the newest (highest wal_position / LSN) N
//!   R3c - below-threshold is a no-op
//!   R3d - survives recovery: engine reopens, newest snapshot loads, state is intact

use cclab_ctx_inf_db::storage::snapshot::{find_snapshot_files, SnapshotHeader, WalReplayStart};
use cclab_ctx_inf_db::*;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────

/// Build a `PersistenceConfig` with the given `snapshot_keep_count`.
/// Other values follow `for_testing` tuning (fast flush, small rotation).
fn config_with_keep(dir: &Path, keep: usize) -> PersistenceConfig {
    let mut cfg = PersistenceConfig::for_testing(dir);
    cfg.snapshot_keep_count = keep;
    cfg
}

/// Spin up a persistent engine rooted at a fresh TempDir.
fn setup_with_keep_count(keep: usize) -> (TempDir, CtxInfEngine) {
    let temp = TempDir::new().unwrap();
    let cfg = config_with_keep(temp.path(), keep);
    let engine = CtxInfEngine::with_persistence(cfg).unwrap();
    (temp, engine)
}

/// Count `.snap` files in `dir`.
fn count_snap_files(dir: &Path) -> usize {
    find_snapshot_files(dir).unwrap().len()
}

/// List `.snap` files with their parsed wal_position_in_file (from header) in filename-sort order.
fn snap_wal_positions(dir: &Path) -> Vec<(PathBuf, u64)> {
    let mut out = Vec::new();
    for path in find_snapshot_files(dir).unwrap() {
        let file = File::open(&path).unwrap();
        let mut reader = BufReader::new(file);
        let (_header, replay_start) = SnapshotHeader::read(&mut reader).unwrap();
        let pos = match replay_start {
            WalReplayStart::FromPoint {
                wal_position_in_file,
                ..
            } => wal_position_in_file,
            WalReplayStart::FullReplay => 0,
        };
        out.push((path, pos));
    }
    out
}

/// Insert one entity and snapshot; returns the created entity id.
/// Sleeps briefly so the WAL channel drains AND so the next snapshot
/// filename (nanosecond timestamp) is distinct from the prior one.
fn insert_and_snapshot(engine: &CtxInfEngine, name: &str) -> EntityId {
    let e = engine
        .create_entity(Entity::new(EntityType::Person, name))
        .unwrap();
    // Allow WAL background thread to advance wal_position.
    thread::sleep(Duration::from_millis(50));
    engine.create_snapshot().unwrap();
    // Ensure next snapshot filename (nanos timestamp) differs.
    thread::sleep(Duration::from_millis(5));
    e.id
}

// ── R3a: keeps only N ────────────────────────────────────────────────

#[test]
fn test_snapshot_gc_keeps_only_n() {
    let (temp, engine) = setup_with_keep_count(2);
    let dir = temp.path().to_path_buf();

    // Insert one entity before each of 5 snapshots.
    for i in 0..5 {
        insert_and_snapshot(&engine, &format!("P{}", i));
    }

    engine.shutdown().unwrap();

    assert_eq!(
        count_snap_files(&dir),
        2,
        "exactly 2 .snap files should remain after 5 snapshots with keep=2"
    );
}

// ── R3b: keeps the newest (highest wal_position) ─────────────────────

#[test]
fn test_snapshot_gc_keeps_the_newest() {
    let (temp, engine) = setup_with_keep_count(2);
    let dir = temp.path().to_path_buf();

    // Capture wal_positions of each snapshot as it's taken.
    let mut observed_positions: Vec<u64> = Vec::new();
    for i in 0..5 {
        engine
            .create_entity(Entity::new(EntityType::Person, format!("P{}", i)))
            .unwrap();
        // Let WAL advance.
        thread::sleep(Duration::from_millis(50));
        engine.create_snapshot().unwrap();
        // Read header of the newest snapshot just written.
        let latest = find_snapshot_files(&dir)
            .unwrap()
            .into_iter()
            .last()
            .unwrap();
        let mut reader = BufReader::new(File::open(&latest).unwrap());
        let (_header, replay_start) = SnapshotHeader::read(&mut reader).unwrap();
        let pos = match replay_start {
            WalReplayStart::FromPoint {
                wal_position_in_file,
                ..
            } => wal_position_in_file,
            WalReplayStart::FullReplay => 0,
        };
        observed_positions.push(pos);
        thread::sleep(Duration::from_millis(5));
    }

    engine.shutdown().unwrap();

    let remaining = snap_wal_positions(&dir);
    assert_eq!(remaining.len(), 2, "keep=2, 5 snapshots → 2 remain");

    // wal_position is monotonically non-decreasing across snapshots.
    // The two highest observed positions MUST match the two remaining on disk.
    let mut observed_sorted = observed_positions.clone();
    observed_sorted.sort_unstable();
    let expected_top_two = &observed_sorted[observed_sorted.len() - 2..];

    let mut remaining_positions: Vec<u64> = remaining.iter().map(|(_, p)| *p).collect();
    remaining_positions.sort_unstable();

    assert_eq!(
        remaining_positions, expected_top_two,
        "the two remaining snapshots must be the two highest-LSN ones (observed: {:?})",
        observed_positions
    );

    // And the lowest observed position must NOT be in the remaining set.
    let lowest = observed_sorted[0];
    assert!(
        !remaining_positions.contains(&lowest),
        "the oldest snapshot (wal_position {}) should have been GC'd",
        lowest
    );
}

// ── R3c: below threshold is a no-op ──────────────────────────────────

#[test]
fn test_snapshot_gc_below_threshold_is_noop() {
    let (temp, engine) = setup_with_keep_count(5);
    let dir = temp.path().to_path_buf();

    for i in 0..3 {
        insert_and_snapshot(&engine, &format!("P{}", i));
    }

    engine.shutdown().unwrap();

    assert_eq!(
        count_snap_files(&dir),
        3,
        "3 snapshots under keep=5 threshold → all 3 must remain"
    );
}

// ── R3d: survives recovery ───────────────────────────────────────────

#[test]
fn test_snapshot_gc_survives_recovery() {
    let temp = TempDir::new().unwrap();
    let dir = temp.path().to_path_buf();

    let mut created_ids: Vec<EntityId> = Vec::new();

    {
        let cfg = config_with_keep(&dir, 2);
        let engine = CtxInfEngine::with_persistence(cfg).unwrap();

        for i in 0..5 {
            let e = engine
                .create_entity(Entity::new(EntityType::Person, format!("P{}", i)))
                .unwrap();
            created_ids.push(e.id);
            thread::sleep(Duration::from_millis(50));
            engine.create_snapshot().unwrap();
            thread::sleep(Duration::from_millis(5));
        }

        engine.shutdown().unwrap();
    }

    // GC should have left exactly 2 files on disk.
    assert_eq!(
        count_snap_files(&dir),
        2,
        "pre-reopen: keep=2 invariant must hold"
    );

    // Capture the wal_positions of the two surviving snapshots — the loader
    // must pick the highest of these on reopen.
    let remaining_before = snap_wal_positions(&dir);
    let max_wal_pos_on_disk = remaining_before
        .iter()
        .map(|(_, p)| *p)
        .max()
        .expect("must have at least one snapshot");

    // Reopen the engine. Recovery must load the newest snapshot + replay any
    // WAL delta and rebuild the full 5-entity state.
    let cfg = config_with_keep(&dir, 2);
    let (engine, stats) = CtxInfEngine::open_with_stats(cfg).unwrap();
    assert!(
        stats.snapshot_loaded,
        "recovery must pick up the remaining newest snapshot"
    );
    // The newest survivor must be loadable — at minimum, entity_count of the
    // loaded snapshot + any replayed WAL delta must reconstruct all 5 entities.
    // (The RecoveryManager stats do not expose snapshot_wal_position directly;
    // we indirectly assert by checking post-recovery entity count below.)
    let _ = max_wal_pos_on_disk;

    // All 5 entities must still be reachable — snapshot covers the last state,
    // any ops after the last snapshot come from WAL replay.
    let people = engine.entities_by_type(&EntityType::Person);
    assert_eq!(
        people.len(),
        5,
        "all 5 entities must be present post-recovery"
    );
    for id in &created_ids {
        assert!(
            engine.get_entity(*id).is_ok(),
            "entity {:?} missing after recovery",
            id
        );
    }

    engine.shutdown().unwrap();
}
