//! Criterion-based benchmark harness for Mamba JIT (R1).
//!
//! Uses criterion for statistically rigorous measurements with confidence
//! intervals, outlier detection, and HTML reports. Wraps the Mamba JIT
//! compilation-and-execution path for each benchmark workload.
//!
//! Run with: `cargo bench -p mamba`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use mamba::bench::{BenchKind, BenchRunner, BenchSuite};

/// Benchmark all workloads in the built-in suite under the Mamba JIT using criterion.
fn bench_mamba_suite(c: &mut Criterion) {
    let suite = BenchSuite::builtin();
    let runner = BenchRunner::mamba_only();

    let mut group = c.benchmark_group("mamba_jit");

    for bench in &suite.benchmarks {
        group.bench_with_input(
            BenchmarkId::new(bench.kind.label(), bench.name),
            bench,
            |b, bench| {
                b.iter(|| {
                    // Run one iteration of the benchmark via the Mamba JIT.
                    // black_box prevents the compiler from optimising away the call.
                    let result = runner.run_mamba(black_box(bench));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark numeric workloads (tight integer loops).
fn bench_numeric(c: &mut Criterion) {
    let suite = BenchSuite::builtin();
    let runner = BenchRunner::mamba_only();

    let mut group = c.benchmark_group("numeric");

    for bench in suite.filter_kind(BenchKind::Numeric) {
        group.bench_with_input(
            BenchmarkId::from_parameter(bench.name),
            bench,
            |b, bench| {
                b.iter(|| black_box(runner.run_mamba(black_box(bench))));
            },
        );
    }

    group.finish();
}

/// Benchmark recursive workloads (function call overhead).
fn bench_recursion(c: &mut Criterion) {
    let suite = BenchSuite::builtin();
    let runner = BenchRunner::mamba_only();

    let mut group = c.benchmark_group("recursion");

    for bench in suite.filter_kind(BenchKind::Recursion) {
        group.bench_with_input(
            BenchmarkId::from_parameter(bench.name),
            bench,
            |b, bench| {
                b.iter(|| black_box(runner.run_mamba(black_box(bench))));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_mamba_suite, bench_numeric, bench_recursion);
criterion_main!(benches);
