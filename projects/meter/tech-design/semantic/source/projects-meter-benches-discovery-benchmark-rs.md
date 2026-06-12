---
id: projects-meter-benches-discovery-benchmark-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: benchmark-regression-api
    claim: benchmark-regression-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/benches/discovery_benchmark.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/benches/discovery_benchmark.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/benches/discovery_benchmark.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/benches/discovery_benchmark.rs` captured during meter full-codegen standardization.
```
