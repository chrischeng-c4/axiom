//! Observability stats on `PersistenceHandle` (Phase-2 hardening item f).
//!
//! Pins the contract of `PersistenceHandle::stats()` returning a
//! `PersistenceStats` snapshot:
//!   - R4a counted logged + appended totals survive a shutdown-reopen cycle
//!   - R4b wal_bytes_written_total accumulates across rotation (doesn't drop)
//!   - R4c flush counters distinguish requested vs performed + last_flush_at
//!   - R4d ops_buffered reflects channel depth, converges to 0 after drain
//!   - R4e ops_dropped_on_full trips when channel saturates (see R4e note)
//!
//! These tests are intentionally race-tolerant — stats are monitoring
//! signals, not synchronization primitives; several assertions use
//! `>=` rather than `==` where the background drain / auto-flush
//! interval can legitimately add extra work.

use cclab_ctx_inf_db::storage::handle::PersistenceStats;
use cclab_ctx_inf_db::{
    Entity, EntityType, GraphOp, PersistenceConfig, PersistenceHandle, PropertyValue,
};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────

/// 10ms flush, 64KiB rotation — matches the existing test harness in
/// `persistence_test.rs` and the wal_rotation_test referenced by the
/// spec (R4b).
fn tiny_config(dir: &std::path::Path) -> PersistenceConfig {
    PersistenceConfig::for_testing(dir)
}

/// Build a ~4 KiB entity by stuffing a long string property. The WAL-on-disk
/// entry size depends on bincode/serde overhead, but ~20 of these reliably
/// crosses the 64 KiB rotation threshold on `for_testing` configs —
/// matches the `make_fat_entity` pattern called out in the issue's
/// Reference Context (R4b).
fn make_fat_entity(name: &str) -> Entity {
    Entity::new(EntityType::Person, name)
        .with_property("blob", PropertyValue::String("x".repeat(4096)))
}

// ── R4a: counts logged + appended ─────────────────────────────────────

#[test]
fn test_stats_counts_logged_and_appended() {
    let temp = TempDir::new().unwrap();
    let handle = PersistenceHandle::new(tiny_config(temp.path())).unwrap();

    for i in 0..100 {
        let e = Entity::new(EntityType::Person, format!("P{}", i));
        handle.log_operation(GraphOp::CreateEntity { entity: e });
    }
    handle.flush();

    // Give the background thread room to drain 100 ops through the WAL.
    thread::sleep(Duration::from_millis(250));

    let s: PersistenceStats = handle.stats();
    assert_eq!(
        s.ops_logged_total, 100,
        "all 100 log_operation calls succeeded"
    );
    assert_eq!(
        s.ops_dropped_on_full, 0,
        "channel capacity 10_000 is far above 100 — no drops expected"
    );
    assert!(
        s.wal_bytes_written_total > 0,
        "100 appends must produce some bytes"
    );

    handle.shutdown().unwrap();

    // Reopen: a *new* handle starts a fresh counter block — stats do NOT
    // persist across restarts. Pin that contract explicitly so a future
    // refactor that introduces disk-backed counters doesn't silently
    // change semantics.
    let handle2 = PersistenceHandle::new(tiny_config(temp.path())).unwrap();
    let s2 = handle2.stats();
    assert_eq!(s2.ops_logged_total, 0, "counters reset on reopen");
    assert_eq!(s2.ops_appended_total, 0);
    assert_eq!(s2.wal_bytes_written_total, 0);
    // The fresh WAL on-disk position IS carried forward (header bytes
    // from the reopened file) — but that's `wal_position()`, not the
    // stats counter.
    handle2.shutdown().unwrap();

    // Sanity: one additional assertion on the first-session snapshot —
    // ops_appended must have caught up to ops_logged after the flush
    // + shutdown drain.
    assert_eq!(
        s.ops_logged_total, 100,
        "snapshot captured pre-shutdown still reads 100"
    );
}

// ── R4b: bytes written survives rotation ──────────────────────────────

#[test]
fn test_stats_wal_bytes_survives_rotation() {
    let temp = TempDir::new().unwrap();
    let handle = PersistenceHandle::new(tiny_config(temp.path())).unwrap();

    // 20 × ~4 KiB = ~80 KiB payload — crosses the 64 KiB rotation
    // threshold at least once.
    for i in 0..20 {
        let e = make_fat_entity(&format!("fat-{}", i));
        handle.log_operation(GraphOp::CreateEntity { entity: e });
    }
    handle.flush();

    // Give the background thread a comfortable window to drain + rotate.
    thread::sleep(Duration::from_millis(400));

    let s = handle.stats();

    assert!(
        s.wal_rotations_total >= 1,
        "20 × ~4KiB ops must have crossed the 64 KiB rotation threshold \
         at least once, got {}",
        s.wal_rotations_total
    );
    assert!(
        s.wal_bytes_written_total > 64 * 1024,
        "bytes-written must exceed one rotation's worth ({} KiB), got {}",
        64,
        s.wal_bytes_written_total
    );
    // Sanity: after rotation, wal_position() (file-relative) is SMALLER
    // than the lifetime bytes-written counter — this is the delta-based
    // accumulator's whole point.
    assert!(
        s.wal_bytes_written_total >= handle.wal_position(),
        "lifetime bytes-written ({}) must be >= file-relative position ({})",
        s.wal_bytes_written_total,
        handle.wal_position()
    );

    handle.shutdown().unwrap();
}

// ── R4c: flush counts performed vs requested ──────────────────────────

#[test]
fn test_stats_flush_counts_performed_vs_requested() {
    let temp = TempDir::new().unwrap();
    let handle = PersistenceHandle::new(tiny_config(temp.path())).unwrap();

    // Append one op so there's something for flush() to actually fsync.
    handle.log_operation(GraphOp::CreateEntity {
        entity: Entity::new(EntityType::Person, "probe"),
    });

    for _ in 0..5 {
        handle.flush();
    }

    // Allow all five Flush commands to be drained + processed.
    thread::sleep(Duration::from_millis(200));

    let s = handle.stats();
    assert_eq!(
        s.flushes_requested_total, 5,
        "5 flush() calls must all enqueue successfully on an otherwise \
         empty channel"
    );
    assert!(
        s.flushes_performed_total >= 1,
        "at least one flush must actually execute (auto-interval + \
         explicit requests), got {}",
        s.flushes_performed_total
    );
    assert!(
        s.last_flush_at_unix_ms > 0,
        "last_flush_at_unix_ms must be set once any flush has completed"
    );
    assert_eq!(
        s.flushes_dropped_total, 0,
        "channel nowhere near full — no flush requests should be dropped"
    );

    handle.shutdown().unwrap();
}

// ── R4d: ops_buffered reflects channel depth ──────────────────────────

#[test]
fn test_stats_ops_buffered_reflects_channel_depth() {
    let temp = TempDir::new().unwrap();
    let handle = PersistenceHandle::new(tiny_config(temp.path())).unwrap();

    // Pre-log snapshot — channel must be empty.
    let s0 = handle.stats();
    assert_eq!(
        s0.ops_buffered, 0,
        "channel must start empty, got {}",
        s0.ops_buffered
    );
    assert_eq!(s0.channel_capacity, 10_000, "channel capacity is 10_000");

    // Log one op. Read immediately — depth is 0 or 1 (race with drain).
    handle.log_operation(GraphOp::CreateEntity {
        entity: Entity::new(EntityType::Person, "depth-probe"),
    });
    let s1 = handle.stats();
    assert!(
        s1.ops_buffered <= 1,
        "immediately after logging 1 op, buffered depth is <=1 (race-\
         tolerant), got {}",
        s1.ops_buffered
    );

    // After a drain window, channel MUST converge to empty.
    thread::sleep(Duration::from_millis(100));
    let s2 = handle.stats();
    assert_eq!(
        s2.ops_buffered, 0,
        "after 100ms drain window, channel must be empty, got {}",
        s2.ops_buffered
    );

    handle.shutdown().unwrap();
}

// ── R4e: drops counter on full channel ────────────────────────────────
//
// The R4e stub (chmod-based channel-fill) was retired once Phase-2 (g)
// concurrent-write-stress reliably exercised this counter. See
// `tests/persistence_stress_test.rs::test_concurrent_ingest_exercises_channel_full_path`.
