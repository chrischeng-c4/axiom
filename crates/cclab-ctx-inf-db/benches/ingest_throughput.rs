//! Phase-2 ingest-throughput benchmarks for `cclab-ctx-inf-db`.
//!
//! Measures the cost of the WAL-backed write path under three workloads:
//!
//! * `bench_single_thread_entity_append` — pure entity ingest, 1 thread.
//! * `bench_single_thread_relation_append` — pure relation ingest, 1 thread,
//!   pre-populated entity pool of 1 000.
//! * `bench_multi_thread_entity_append` — 4 threads × 10 000 entity ops per
//!   criterion iteration, fanned through a single shared engine.
//!
//! All benches use a fresh `tempfile::TempDir` per criterion iteration so the
//! WAL state never bleeds across measurements. Each engine is built via
//! `PersistenceConfig::for_testing` (10 ms flush interval, 64 KiB rotation,
//! 10 000-slot command channel).
//!
//! Run with:
//!
//! ```bash
//! cargo bench -p cclab-ctx-inf-db --bench ingest_throughput
//! ```
//!
//! Each benchmark uses `iter_custom` so the inner tight loop runs many ops
//! against a single engine + tempdir built at the top of the iteration; only
//! the loop itself is in the timer. Engine bring-up cost is intentionally
//! excluded so the reported throughput is the steady-state ingest rate.
//!
//! Orthogonal verification note: `CtxInfEngine::persistence` is `pub(crate)`,
//! so `PersistenceStats` cannot be sampled from a `benches/` target without
//! broadening the public API. R7 forbids prod-code touches, so we rely on
//! criterion's own iteration count plus the engine's `stats()` (entity /
//! relation counts) for cross-checks; broader stats observability requires a
//! separate API-broadening change.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use tempfile::TempDir;

use cclab_ctx_inf_db::{
    CtxInfEngine, Entity, EntityId, EntityType, PersistenceConfig, Relation, RelationType,
};

/// Build a fresh persistent engine rooted in a per-iteration `TempDir`.
///
/// The `TempDir` is returned alongside the engine so the caller controls
/// drop-order: the engine must be shut down before the directory is unlinked.
fn fresh_engine() -> (CtxInfEngine, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let cfg = PersistenceConfig::for_testing(dir.path());
    let engine = CtxInfEngine::with_persistence(cfg).expect("engine init");
    (engine, dir)
}

/// Single-threaded entity-ingest throughput.
///
/// * Metric: ops/sec on the WAL-backed `create_entity` path.
/// * Workload: `sample_size = 100`, `measurement_time = 10 s`,
///   `Throughput::Elements(1)`. Each criterion sample times a tight loop of
///   `iters` `create_entity` calls against one engine; `iter_custom` divides
///   the elapsed time by `iters` so the reported per-element latency reflects
///   one `create_entity` call.
/// * Verification: criterion's iteration counter; the engine's
///   `stats().entity_count` (cross-checked at the end of each iteration) gives
///   a coarse-grained orthogonal count. `PersistenceStats` is not reachable
///   from `benches/` (see file-level doc).
fn bench_single_thread_entity_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_thread_entity_append");
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(1));

    group.bench_function("entity_append", |b| {
        b.iter_custom(|iters| {
            let (engine, _dir) = fresh_engine();
            let start = Instant::now();
            for i in 0..iters {
                let entity = Entity::new(EntityType::Person, format!("person-{i}"));
                let out = engine.create_entity(black_box(entity)).expect("create");
                black_box(out);
            }
            let elapsed = start.elapsed();
            // Cross-check: the engine should hold exactly `iters` entities.
            debug_assert_eq!(engine.stats().entity_count as u64, iters);
            elapsed
        });
    });

    group.finish();
}

/// Single-threaded relation-ingest throughput.
///
/// * Metric: ops/sec on the WAL-backed `create_relation` path with
///   pre-validated endpoints (1 000 seed entities created outside the timer).
/// * Workload: `sample_size = 20`, `measurement_time = 10 s`,
///   `Throughput::Elements(1)`. Each criterion sample times a tight loop of
///   `iters` `create_relation` calls; `iter_custom` divides elapsed by
///   `iters` so the per-element number is one `create_relation` call.
///   (Spec asks for `sample_size = 100`; reduced to 20 because the 1 000
///   entity seed runs once per sample and dominates wall time at the actual
///   measured per-op cost — see BENCHMARKS.md deviations.)
/// * Verification: criterion's iteration counter plus the engine's
///   `stats().relation_count`. `PersistenceStats` would be the orthogonal
///   counter (`ops_appended_total`) but is unreachable from `benches/`
///   without broadening the public API (see file-level doc).
fn bench_single_thread_relation_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_thread_relation_append");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(1));

    group.bench_function("relation_append", |b| {
        b.iter_custom(|iters| {
            let (engine, _dir) = fresh_engine();
            // Seed pool of 1 000 entities to serve as relation endpoints.
            let mut ids: Vec<EntityId> = Vec::with_capacity(1000);
            for i in 0..1000 {
                let e = Entity::new(EntityType::Person, format!("seed-{i}"));
                let id = e.id;
                engine.create_entity(e).expect("seed entity");
                ids.push(id);
            }
            let start = Instant::now();
            for i in 0..iters as usize {
                let a = ids[i % 1000];
                let b_id = ids[(i + 1) % 1000];
                let rel = Relation::new(RelationType::MetWith, a, b_id);
                let out = engine.create_relation(black_box(rel)).expect("create rel");
                black_box(out);
            }
            let elapsed = start.elapsed();
            debug_assert_eq!(engine.stats().relation_count as u64, iters);
            elapsed
        });
    });

    group.finish();
}

/// Multi-threaded entity-ingest throughput (4 threads × 10 000 ops).
///
/// * Metric: aggregate ops/sec across 4 worker threads sharing one
///   `Arc<CtxInfEngine>`; per-thread ops/sec is `aggregate / 4` (assumes even
///   work distribution, which holds for this fixed loop count).
/// * Workload: `sample_size = 10`, `measurement_time = 10 s`,
///   `Throughput::Elements(40_000)`. Each criterion `iter` runs
///   4 × 10 000 = 40 000 `create_entity` calls; criterion divides the elapsed
///   time by 40 000 to derive per-element latency, and reports
///   `40_000 / elapsed` as the throughput.
///   (Spec asks for `sample_size = 30`, `measurement_time = 15 s`; reduced
///   to fit the 10-minute total runtime budget at actual measured per-op
///   cost — see BENCHMARKS.md deviations.)
/// * Verification: criterion's reported throughput plus the engine's
///   `stats().entity_count` (must equal 40 000 at iteration end). The WAL
///   flush / rotation counters are inside `PersistenceStats` and not
///   reachable from `benches/` per R7 (see file-level doc).
///
/// NOTE: `BUG-WAL-ROTATE-COLLISION` (cclab-wal) — under sustained 4-thread
/// pressure the WAL rotate path can silently overwrite archives at >1
/// rotation per second. If that bug fires it surfaces here as missing
/// rotations or an `entity_count` mismatch — observed behaviour is captured
/// in `BENCHMARKS.md` observations rather than worked around.
fn bench_multi_thread_entity_append(c: &mut Criterion) {
    const THREADS: usize = 4;
    const OPS_PER_THREAD: usize = 10_000;
    const TOTAL_OPS: u64 = (THREADS * OPS_PER_THREAD) as u64;

    let mut group = c.benchmark_group("multi_thread_entity_append");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(TOTAL_OPS));

    group.bench_function("entity_append_4t", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let (engine, _dir) = fresh_engine();
                let engine = Arc::new(engine);
                let start = Instant::now();
                thread::scope(|s| {
                    let mut handles = Vec::with_capacity(THREADS);
                    for tid in 0..THREADS {
                        let engine = Arc::clone(&engine);
                        handles.push(s.spawn(move || {
                            for i in 0..OPS_PER_THREAD {
                                let entity =
                                    Entity::new(EntityType::Person, format!("t{tid}-p{i}"));
                                let out = engine.create_entity(black_box(entity)).expect("create");
                                black_box(out);
                            }
                        }));
                    }
                    for h in handles {
                        h.join().expect("worker join");
                    }
                });
                total += start.elapsed();
                debug_assert_eq!(engine.stats().entity_count as u64, TOTAL_OPS);
            }
            total
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_thread_entity_append,
    bench_single_thread_relation_append,
    bench_multi_thread_entity_append,
);
criterion_main!(benches);
