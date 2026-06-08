---
id: projects-meter-examples-boundary-tracing-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: embedded-profiler-api
    claim: embedded-profiler-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/examples/boundary_tracing.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/examples/boundary_tracing.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/examples/boundary_tracing.rs -->
````rust
//! Example demonstrating binding boundary tracing
//!
//! Run with: cargo run -p cclab-probe --example boundary_tracing

use meter::performance::boundary::{BoundaryMetrics, BoundaryTracer};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Binding Boundary Tracing Example ===\n");

    // Example 1: Basic single operation tracing
    println!("Example 1: Single Operation Tracing");
    println!("{}", "-".repeat(50));
    basic_tracing_example();
    println!();

    // Example 2: Global metrics aggregation
    println!("Example 2: Global Metrics Aggregation");
    println!("{}", "-".repeat(50));
    global_metrics_example();
    println!();

    // Example 3: Thread-safe concurrent tracing
    println!("Example 3: Concurrent Operations");
    println!("{}", "-".repeat(50));
    concurrent_tracing_example();
    println!();
}

fn basic_tracing_example() {
    let mut tracer = BoundaryTracer::new("insert_many");

    // Simulate Phase 1: Extract (GIL held)
    tracer.start_extract();
    thread::sleep(Duration::from_micros(500));
    tracer.end_extract();

    // Simulate Phase 2: Convert (GIL released)
    tracer.start_convert();
    tracer.record_gil_release();
    thread::sleep(Duration::from_micros(2000));
    tracer.end_convert();

    // Simulate Phase 3: Network (GIL released)
    tracer.start_network();
    thread::sleep(Duration::from_micros(5000));
    tracer.end_network();

    // Simulate Phase 4: Materialize (GIL held)
    tracer.start_materialize();
    thread::sleep(Duration::from_micros(300));
    tracer.end_materialize();

    tracer.set_doc_count(1000);
    tracer.set_parallel(true);

    let timing = tracer.finish();
    println!("{}", timing.format());
    println!("\nMetrics:");
    println!(
        "  GIL held time: {}µs ({:.1}%)",
        timing.gil_held_us(),
        timing.gil_held_percent()
    );
    println!("  GIL released time: {}µs", timing.gil_released_us());
    println!("  Per document: {:.2}µs/doc", timing.per_doc_us());
}

fn global_metrics_example() {
    let metrics = Arc::new(BoundaryMetrics::new());

    // Simulate multiple operations
    for i in 0..5 {
        let mut tracer = BoundaryTracer::new(format!("operation_{}", i));

        tracer.start_extract();
        thread::sleep(Duration::from_micros(100));
        tracer.end_extract();

        tracer.start_convert();
        tracer.record_gil_release();
        thread::sleep(Duration::from_micros(500));
        tracer.end_convert();

        tracer.start_network();
        thread::sleep(Duration::from_micros(1000));
        tracer.end_network();

        tracer.start_materialize();
        thread::sleep(Duration::from_micros(50));
        tracer.end_materialize();

        tracer.set_doc_count(100);
        let timing = tracer.finish();
        metrics.record(&timing);
    }

    // Print aggregated metrics
    println!("Total operations: {}", metrics.operation_count());
    println!("Total documents: {}", metrics.doc_count());
    println!("Total GIL releases: {}", metrics.gil_release_count());
    println!("\nAverage timings:");
    println!("  Extract: {:.2}µs", metrics.avg_extract_us());
    println!("  Convert: {:.2}µs", metrics.avg_convert_us());
    println!("  Network: {:.2}µs", metrics.avg_network_us());
    println!("  Materialize: {:.2}µs", metrics.avg_materialize_us());

    println!("\nSnapshot:");
    let snapshot = metrics.snapshot();
    for (key, value) in snapshot {
        println!("  {}: {}", key, value);
    }
}

fn concurrent_tracing_example() {
    let metrics = Arc::new(BoundaryMetrics::new());
    let mut handles = vec![];

    // Spawn 4 threads, each simulating operations
    for thread_id in 0..4 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for op_id in 0..10 {
                let mut tracer = BoundaryTracer::new(format!("thread_{}_op_{}", thread_id, op_id));

                tracer.start_extract();
                thread::sleep(Duration::from_micros(50));
                tracer.end_extract();

                tracer.start_convert();
                tracer.record_gil_release();
                thread::sleep(Duration::from_micros(200));
                tracer.end_convert();

                tracer.start_network();
                thread::sleep(Duration::from_micros(500));
                tracer.end_network();

                tracer.start_materialize();
                thread::sleep(Duration::from_micros(30));
                tracer.end_materialize();

                tracer.set_doc_count(50);
                let timing = tracer.finish();
                metrics_clone.record(&timing);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Concurrent operations completed!");
    println!("Total operations: {}", metrics.operation_count());
    println!("Total documents: {}", metrics.doc_count());
    println!("Total GIL releases: {}", metrics.gil_release_count());
    println!("\nThis demonstrates thread-safe metrics collection across");
    println!("multiple concurrent binding operations.");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/examples/boundary_tracing.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/examples/boundary_tracing.rs` captured during meter full-codegen standardization.
```
