//! Engine perf-gate benchmark (issue #126).
//!
//! Deterministic, network-free microbenchmarks of the sharded KV engine — the
//! part keep owns and can ratchet. Run via `cargo bench -p keep` or, as the
//! gate, `meter bench --target projects/keep` (delegates cargo bench; with a
//! baseline it folds regressions to a non-zero exit). The `concurrent_*` group
//! is the important one: it guards the multi-core SET scaling fix (a global-lock
//! regression would collapse it, exactly what the gate must catch).

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use keep::{KvEngine, KvKey, KvValue};

const KEYSPACE: usize = 10_000;

fn key(i: usize) -> KvKey {
    KvKey::new(format!("benchkey:{}", i % KEYSPACE)).unwrap()
}

fn val() -> KvValue {
    KvValue::String("x".repeat(64))
}

/// Single-threaded per-op latency for the core engine operations.
fn single_ops(c: &mut Criterion) {
    let engine = KvEngine::with_shards(256);
    // Prime so GET/CAS/INCR hit existing keys.
    for i in 0..KEYSPACE {
        engine.set(&key(i), val(), None).unwrap();
    }

    let mut g = c.benchmark_group("engine_single");
    g.throughput(Throughput::Elements(1));

    g.bench_function("set", |b| {
        let mut i = 0usize;
        b.iter(|| {
            engine.set(&key(i), val(), None).unwrap();
            i += 1;
        })
    });
    g.bench_function("get", |b| {
        let mut i = 0usize;
        b.iter(|| {
            let _ = engine.get(&key(i));
            i += 1;
        })
    });
    g.bench_function("incr", |b| {
        let k = KvKey::new("counter").unwrap();
        b.iter(|| {
            let _ = engine.incr(&k, 1).unwrap();
        })
    });
    g.bench_function("cas", |b| {
        b.iter_batched(
            || (),
            |_| {
                // expected matches the primed value, so the swap applies.
                let _ = engine.cas(&key(0), &val(), val(), None);
            },
            BatchSize::SmallInput,
        )
    });
    g.finish();
}

/// Batched ops (claim-check bulk path).
fn batch_ops(c: &mut Criterion) {
    let engine = KvEngine::with_shards(256);
    const B: usize = 100;
    let keys: Vec<KvKey> = (0..B).map(key).collect();
    let refs: Vec<&KvKey> = keys.iter().collect();

    let mut g = c.benchmark_group("engine_batch100");
    g.throughput(Throughput::Elements(B as u64));
    g.bench_function("mset", |b| {
        b.iter(|| {
            let pairs: Vec<(&KvKey, KvValue)> = keys.iter().map(|k| (k, val())).collect();
            engine.mset(&pairs, None).unwrap();
        })
    });
    // Prime for mget.
    let pairs: Vec<(&KvKey, KvValue)> = keys.iter().map(|k| (k, val())).collect();
    engine.mset(&pairs, None).unwrap();
    g.bench_function("mget", |b| {
        b.iter(|| {
            let _ = engine.mget(&refs);
        })
    });
    g.finish();
}

/// Multi-core aggregate throughput — the regression guard for the SET
/// contention fix. Measures wall time for `ops_per_thread * threads` ops spread
/// across N threads; a global-lock regression makes this collapse vs single-core.
fn concurrent_throughput(c: &mut Criterion) {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let ops_per_thread = 50_000usize;
    let total = threads * ops_per_thread;

    let mut g = c.benchmark_group("engine_concurrent");
    g.throughput(Throughput::Elements(total as u64));
    g.sample_size(20);

    g.bench_function(format!("set_{threads}threads"), |b| {
        b.iter_custom(|iters| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iters {
                let engine = Arc::new(KvEngine::with_shards(256));
                let start = Instant::now();
                let mut hs = Vec::with_capacity(threads);
                for t in 0..threads {
                    let engine = engine.clone();
                    hs.push(thread::spawn(move || {
                        for i in 0..ops_per_thread {
                            let k = KvKey::new(format!("k:{t}:{i}")).unwrap();
                            engine.set(&k, KvValue::Int(i as i64), None).unwrap();
                        }
                    }));
                }
                for h in hs {
                    h.join().unwrap();
                }
                elapsed += start.elapsed();
            }
            elapsed
        })
    });
    g.finish();
}

criterion_group!(benches, single_ops, batch_ops, concurrent_throughput);
criterion_main!(benches);
