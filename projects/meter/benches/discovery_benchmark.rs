// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-benches-discovery-benchmark-rs.md#source
// CODEGEN-BEGIN
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use meter::discovery::{walk_files, DiscoveryConfig};
use std::path::PathBuf;

fn bench_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("test_discovery");

    // Test with different thread counts
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}threads", num_threads)),
            &num_threads,
            |b, &threads| {
                let config = DiscoveryConfig {
                    root_path: PathBuf::from("tests/"),
                    patterns: vec!["test_*.py".to_string(), "bench_*.py".to_string()],
                    exclusions: vec!["__pycache__".to_string(), ".git".to_string()],
                    max_depth: 10,
                    num_threads: threads,
                };

                b.iter(|| {
                    let files = walk_files(black_box(&config)).unwrap();
                    black_box(files);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_discovery);
criterion_main!(benches);
// CODEGEN-END
