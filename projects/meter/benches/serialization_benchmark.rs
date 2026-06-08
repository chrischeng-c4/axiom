// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-benches-serialization-benchmark-rs.md#source
// CODEGEN-BEGIN
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use meter::baseline::BaselineSnapshot;
use meter::benchmark::{BenchmarkEnvironment, BenchmarkResult, BenchmarkStats};

fn create_test_snapshot(num_results: usize) -> BaselineSnapshot {
    let results: Vec<_> = (0..num_results)
        .map(|i| {
            let times: Vec<f64> = (0..1000).map(|j| j as f64).collect();
            BenchmarkResult {
                name: format!("benchmark_{}", i),
                stats: BenchmarkStats::from_times(times, 1000, 1, 0),
                success: true,
                error: None,
            }
        })
        .collect();

    let env = BenchmarkEnvironment::default();
    BaselineSnapshot::from_benchmarks(results, &env)
}

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    for size in [1, 10, 100].iter() {
        let snapshot = create_test_snapshot(*size);

        // JSON serialization
        group.bench_with_input(
            BenchmarkId::new("json_serialize", size),
            &snapshot,
            |b, s| {
                b.iter(|| {
                    let _json = serde_json::to_string(black_box(s)).unwrap();
                });
            },
        );

        // JSON deserialization
        let json = serde_json::to_string(&snapshot).unwrap();
        group.bench_with_input(BenchmarkId::new("json_deserialize", size), &json, |b, j| {
            b.iter(|| {
                let _: BaselineSnapshot = serde_json::from_str(black_box(j)).unwrap();
            });
        });

        // Binary serialization
        #[cfg(feature = "rkyv")]
        {
            group.bench_with_input(
                BenchmarkId::new("binary_serialize", size),
                &snapshot,
                |b, s| {
                    b.iter(|| {
                        let _binary = s.to_binary().unwrap();
                    });
                },
            );

            // Binary deserialization
            let binary = snapshot.to_binary().unwrap();
            group.bench_with_input(
                BenchmarkId::new("binary_deserialize", size),
                &binary,
                |b, bin| {
                    b.iter(|| {
                        let _ = BaselineSnapshot::from_binary(black_box(bin)).unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, benchmark_serialization);
criterion_main!(benches);
// CODEGEN-END
