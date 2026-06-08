use cclab_kv::KvEngine;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_kv_engine(c: &mut Criterion) {
    c.bench_function("kv_engine_new", |b| {
        b.iter(|| {
            let engine = KvEngine::new();
            black_box(engine);
        });
    });
}

criterion_group!(benches, benchmark_kv_engine);
criterion_main!(benches);
