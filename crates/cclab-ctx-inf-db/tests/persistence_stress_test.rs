//! Concurrent-write stress for `PersistenceHandle` (Phase-2 hardening g).
//! R2a 100×1000 ops (identity + no silent loss), R2b 50×500 fat ops
//! (channel-full drop path), R2c 10×100 ops (durable reopen),
//! R2d concurrent log+flush. Tests use `PersistenceHandle` directly
//! (engine `persistence` field is `pub(crate)`, spec forbids prod-code
//! touches). Durability verified by reopening via
//! `CtxInfEngine::open_with_stats` → `RecoveryManager`.

use cclab_ctx_inf_db::storage::handle::PersistenceStats;
use cclab_ctx_inf_db::{
    CtxInfEngine, Entity, EntityType, GraphOp, PersistenceConfig, PersistenceHandle, PropertyValue,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────

/// Spawn `count` writers via `std::thread::scope`. Scope lets the body
/// borrow `&PersistenceHandle` without `Arc`.
fn spawn_writers<F>(handle: &PersistenceHandle, count: usize, per_thread_ops: usize, body: F)
where
    F: Fn(&PersistenceHandle, usize, usize) + Sync,
{
    let body = &body;
    thread::scope(|s| {
        for tid in 0..count {
            s.spawn(move || body(handle, tid, per_thread_ops));
        }
    });
}

/// Block until `ops_appended_total >= ops_logged_total` or `timeout`
/// elapses. Drops are NOT in the bg drain queue (they failed at try_send),
/// so we wait only on admitted ops to be appended.
fn await_drain(handle: &PersistenceHandle, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        let s = handle.stats();
        if s.ops_appended_total >= s.ops_logged_total {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

/// ~4 KiB entity used by R2b to slow drain via per-append cost +
/// frequent rotations at the for_testing 64 KiB ceiling.
fn fat_entity(name: &str) -> Entity {
    Entity::new(EntityType::Person, name)
        .with_property("blob", PropertyValue::String("x".repeat(4096)))
}

/// **Invariant**: `logged + dropped == 100_000` and every admitted op
/// survives reopen. **Primary target**: logged/appended/dropped
/// accounting identity + post-reopen `entity_count`. **Sizing**: 100
/// threads × 1000 ops vs 10_000-slot channel; small ops, drops should
/// be minimal but the identity must hold even if a few drops occur.
#[test]
fn test_concurrent_ingest_100_threads_1000_ops_no_data_loss() {
    let temp = TempDir::new().unwrap();
    let handle = PersistenceHandle::new(PersistenceConfig::for_testing(temp.path())).unwrap();

    let start = Instant::now();
    spawn_writers(&handle, 100, 1000, |h, tid, n| {
        for i in 0..n {
            let e = Entity::new(EntityType::Person, format!("p-{}-{}", tid, i));
            h.log_operation(GraphOp::CreateEntity { entity: e });
        }
    });
    eprintln!("R2a: 100×1000 spawn+join = {:?}", start.elapsed());

    handle.flush();
    assert!(
        await_drain(&handle, Duration::from_secs(20)),
        "drain did not complete in 20s — likely deadlock"
    );
    handle.flush();
    thread::sleep(Duration::from_millis(50));

    let s1: PersistenceStats = handle.stats();
    assert_eq!(
        s1.ops_logged_total + s1.ops_dropped_on_full,
        100_000,
        "every try_send must be accounted for: logged={} dropped={}",
        s1.ops_logged_total,
        s1.ops_dropped_on_full
    );
    assert_eq!(
        s1.ops_logged_total, s1.ops_appended_total,
        "after drain, every admitted op must be appended"
    );
    let dropped = s1.ops_dropped_on_full;
    let appended = s1.ops_appended_total;
    handle.shutdown().unwrap();

    let (engine, rstats) =
        CtxInfEngine::open_with_stats(PersistenceConfig::for_testing(temp.path())).unwrap();
    assert_eq!(engine.stats().entity_count as u64, appended);
    assert_eq!(
        engine.stats().entity_count as u64 + dropped,
        100_000,
        "no silent loss beyond the {} drops counted at try_send time",
        dropped
    );
    assert_eq!(rstats.corrupted_entries, 0);
    engine.shutdown().unwrap();
}

/// **Invariant**: saturated channel trips `ops_dropped_on_full > 0` and
/// `logged + dropped == 25_000`. **Primary target**: `ops_dropped_on_full`
/// (the field the R4e stub gave up on driving reliably). **Sizing**: 50
/// threads × 500 ops = 25_000 attempts, 2.5× channel capacity; ~4 KiB
/// fat entities + 64 KiB rotation threshold force hundreds of file-
/// create rotations that stall the bg thread enough to fill the channel.
#[test]
fn test_concurrent_ingest_exercises_channel_full_path() {
    let temp = TempDir::new().unwrap();
    let handle = PersistenceHandle::new(PersistenceConfig::for_testing(temp.path())).unwrap();

    let start = Instant::now();
    spawn_writers(&handle, 50, 500, |h, tid, n| {
        for i in 0..n {
            let e = fat_entity(&format!("fat-{}-{}", tid, i));
            h.log_operation(GraphOp::CreateEntity { entity: e });
        }
    });
    eprintln!("R2b: 50×500 fat spawn+join = {:?}", start.elapsed());
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "exceeded 30s hard timeout"
    );

    let s = handle.stats();
    assert!(
        s.ops_dropped_on_full > 0,
        "channel-full path was not exercised — logged={} dropped={} \
         appended={} buffered={}",
        s.ops_logged_total,
        s.ops_dropped_on_full,
        s.ops_appended_total,
        s.ops_buffered
    );
    assert_eq!(
        s.ops_logged_total + s.ops_dropped_on_full,
        25_000,
        "all 25_000 try_sends must be accounted for"
    );

    handle.shutdown().unwrap();
}

/// **Invariant**: non-saturating load → zero drops + every op flushed +
/// every op replayed on reopen. **Primary target**: `ops_dropped_on_full
/// == 0`, `flushes_performed_total >= 1`, post-reopen `entity_count ==
/// 1000`. **Sizing**: 10 threads × 100 ops = 1000 ops, 10× below
/// channel capacity; for_testing 10ms auto-flush ensures >=1 flush.
#[test]
fn test_concurrent_ingest_then_flush_is_durable() {
    let temp = TempDir::new().unwrap();
    let handle = PersistenceHandle::new(PersistenceConfig::for_testing(temp.path())).unwrap();

    let start = Instant::now();
    spawn_writers(&handle, 10, 100, |h, tid, n| {
        for i in 0..n {
            let e = Entity::new(EntityType::Person, format!("p-{}-{}", tid, i));
            h.log_operation(GraphOp::CreateEntity { entity: e });
        }
    });
    eprintln!("R2c: 10×100 spawn+join = {:?}", start.elapsed());
    handle.flush();
    assert!(
        await_drain(&handle, Duration::from_secs(5)),
        "drain did not complete in 5s"
    );
    thread::sleep(Duration::from_millis(50));

    let s = handle.stats();
    assert_eq!(
        s.ops_dropped_on_full, 0,
        "1000 ops must not saturate 10k channel"
    );
    assert_eq!(s.ops_logged_total, 1000);
    assert_eq!(s.ops_appended_total, 1000);
    assert!(
        s.flushes_performed_total >= 1,
        "expected >=1 flush, got {}",
        s.flushes_performed_total
    );

    handle.shutdown().unwrap();

    let (engine, _) =
        CtxInfEngine::open_with_stats(PersistenceConfig::for_testing(temp.path())).unwrap();
    assert_eq!(engine.stats().entity_count, 1000);
    engine.shutdown().unwrap();
}

/// **Invariant**: concurrent log+flush → flush counter accounting holds
/// and every logged op is durable through reopen. **Primary target**:
/// `flushes_requested + flushes_dropped > 0`, `flushes_performed >= 1`,
/// no panic/deadlock, post-reopen `entity_count == admitted_at_stop`.
/// **Sizing**: 20 threads (10 log, 10 flush) × 2s wall-clock.
#[test]
fn test_concurrent_flush_coherence() {
    let temp = TempDir::new().unwrap();
    let handle =
        Arc::new(PersistenceHandle::new(PersistenceConfig::for_testing(temp.path())).unwrap());
    let stop = Arc::new(AtomicBool::new(false));
    let logged_count = Arc::new(AtomicU64::new(0));
    let test_start = Instant::now();

    thread::scope(|s| {
        for tid in 0..10 {
            let h = Arc::clone(&handle);
            let stop = Arc::clone(&stop);
            let counter = Arc::clone(&logged_count);
            s.spawn(move || {
                let mut i = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    h.log_operation(GraphOp::CreateEntity {
                        entity: Entity::new(EntityType::Person, format!("p-{}-{}", tid, i)),
                    });
                    counter.fetch_add(1, Ordering::Relaxed);
                    i += 1;
                }
            });
        }
        for _ in 0..10 {
            let h = Arc::clone(&handle);
            let stop = Arc::clone(&stop);
            s.spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    h.flush();
                    thread::sleep(Duration::from_micros(100));
                }
            });
        }
        thread::sleep(Duration::from_secs(2));
        stop.store(true, Ordering::Relaxed);
    });
    let elapsed = test_start.elapsed();
    eprintln!("R2d: 20-thread log+flush window = {:?}", elapsed);
    assert!(
        elapsed < Duration::from_secs(10),
        "exceeded 10s hard timeout"
    );

    let attempted_at_stop = logged_count.load(Ordering::Relaxed);
    assert!(
        await_drain(&handle, Duration::from_secs(20)),
        "drain did not complete in 20s"
    );
    let s = handle.stats();
    assert!(
        s.flushes_requested_total + s.flushes_dropped_total > 0,
        "20 flush threads × 2s must have produced >0 flush try_sends"
    );
    assert!(s.flushes_performed_total >= 1);
    assert_eq!(
        s.ops_logged_total + s.ops_dropped_on_full,
        attempted_at_stop,
        "every log attempt must be admitted or counted as dropped"
    );

    Arc::try_unwrap(handle)
        .map_err(|_| "handle has refs")
        .unwrap()
        .shutdown()
        .unwrap();

    let (engine, _) =
        CtxInfEngine::open_with_stats(PersistenceConfig::for_testing(temp.path())).unwrap();
    assert_eq!(
        engine.stats().entity_count as u64,
        s.ops_logged_total,
        "every admitted op must be durable; admitted={} entity_count={} dropped={}",
        s.ops_logged_total,
        engine.stats().entity_count,
        s.ops_dropped_on_full
    );
    assert_eq!(
        engine.stats().entity_count as u64 + s.ops_dropped_on_full,
        attempted_at_stop,
        "no silent loss beyond counted drops; attempted={} entity_count={} dropped={}",
        attempted_at_stop,
        engine.stats().entity_count,
        s.ops_dropped_on_full
    );
    engine.shutdown().unwrap();
}
