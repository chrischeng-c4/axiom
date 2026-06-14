---
id: projects-meter-examples-adaptive-benchmark-demo-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: benchmark-regression-api
    claim: benchmark-regression-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/examples/adaptive_benchmark_demo.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/examples/adaptive_benchmark_demo.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Example demonstrating adaptive benchmark sampling
//!
//! Run with: cargo run --example adaptive_benchmark_demo

use meter::benchmark::{AdaptiveBenchmarkConfig, BenchmarkConfig, Benchmarker};

fn fast_operation() -> u64 {
    // Very fast, consistent operation
    let mut sum = 0u64;
    for i in 0..100 {
        sum = sum.wrapping_add(i);
    }
    sum
}

fn slow_operation() -> u64 {
    // Slower operation with some variation
    let mut sum = 0u64;
    for i in 0..10_000 {
        sum = sum.wrapping_add(i);
    }
    sum
}

fn variable_operation() -> u64 {
    // Operation with high variance
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut sum = 0u64;
    let iterations = 1000 + (count % 5000);
    for i in 0..iterations {
        sum = sum.wrapping_add(i);
    }
    sum
}

fn main() {
    println!("=== Adaptive Benchmark Demonstration ===\n");

    let benchmarker = Benchmarker::default_config();

    // 1. Fast operation with adaptive sampling
    println!("1. Fast operation (adaptive with default config):");
    let adaptive_config = AdaptiveBenchmarkConfig::default();
    let result = benchmarker.run_adaptive("fast_adaptive", fast_operation, adaptive_config);
    println!(
        "   Iterations used: {}",
        result.stats.adaptive_iterations_used
    );
    println!("   Stopped early: {}", result.stats.adaptive_stopped_early);
    if let Some(reason) = &result.stats.adaptive_reason {
        println!("   Stop reason: {}", reason);
    }
    println!(
        "   Mean: {:.3}ms ± {:.3}ms",
        result.stats.mean_ms, result.stats.stddev_ms
    );
    println!();

    // 2. Fast operation with fixed iterations (for comparison)
    println!("2. Fast operation (fixed 100 iterations):");
    let fixed_config = BenchmarkConfig::new(100, 1, 3);
    let fixed_benchmarker = Benchmarker::new(fixed_config);
    let result = fixed_benchmarker.run("fast_fixed", fast_operation);
    println!("   Iterations used: {}", result.stats.total_runs);
    println!(
        "   Mean: {:.3}ms ± {:.3}ms",
        result.stats.mean_ms, result.stats.stddev_ms
    );
    println!();

    // 3. Slow operation with adaptive sampling
    println!("3. Slow operation (adaptive with quick config):");
    let quick_config = AdaptiveBenchmarkConfig::quick();
    let result = benchmarker.run_adaptive("slow_adaptive", slow_operation, quick_config);
    println!(
        "   Iterations used: {}",
        result.stats.adaptive_iterations_used
    );
    println!("   Stopped early: {}", result.stats.adaptive_stopped_early);
    if let Some(reason) = &result.stats.adaptive_reason {
        println!("   Stop reason: {}", reason);
    }
    println!(
        "   Mean: {:.3}ms ± {:.3}ms",
        result.stats.mean_ms, result.stats.stddev_ms
    );
    println!();

    // 4. Variable operation with strict convergence
    println!("4. Variable operation (adaptive with thorough config):");
    let thorough_config = AdaptiveBenchmarkConfig::thorough().with_max_iterations(1000); // Limit to prevent long runtime
    let result = benchmarker.run_adaptive("variable_adaptive", variable_operation, thorough_config);
    println!(
        "   Iterations used: {}",
        result.stats.adaptive_iterations_used
    );
    println!("   Stopped early: {}", result.stats.adaptive_stopped_early);
    if let Some(reason) = &result.stats.adaptive_reason {
        println!("   Stop reason: {}", reason);
    }
    println!(
        "   Mean: {:.3}ms ± {:.3}ms",
        result.stats.mean_ms, result.stats.stddev_ms
    );
    println!(
        "   CV: {:.2}%",
        (result.stats.stddev_ms / result.stats.mean_ms) * 100.0
    );
    println!();

    // 5. Custom adaptive configuration with timeout
    println!("5. Slow operation with timeout (100ms):");
    let timeout_config = AdaptiveBenchmarkConfig::new()
        .with_min_iterations(5)
        .with_max_iterations(10_000)
        .with_timeout_ms(100.0);
    let result = benchmarker.run_adaptive("slow_timeout", slow_operation, timeout_config);
    println!(
        "   Iterations used: {}",
        result.stats.adaptive_iterations_used
    );
    println!("   Stopped early: {}", result.stats.adaptive_stopped_early);
    if let Some(reason) = &result.stats.adaptive_reason {
        println!("   Stop reason: {}", reason);
    }
    println!(
        "   Mean: {:.3}ms ± {:.3}ms",
        result.stats.mean_ms, result.stats.stddev_ms
    );
    println!();

    println!("=== Summary ===");
    println!("Adaptive sampling automatically adjusts iteration count based on:");
    println!("  - Coefficient of Variation (CV) - measures consistency");
    println!("  - Confidence Interval (CI) width - measures precision");
    println!("  - Timeout constraints");
    println!("\nBenefits:");
    println!("  ✓ Fast operations converge quickly (fewer iterations)");
    println!("  ✓ Slow operations use appropriate sample size");
    println!("  ✓ Variable operations get more samples for accuracy");
    println!("  ✓ Prevents excessive runtime with timeouts");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/examples/adaptive_benchmark_demo.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/examples/adaptive_benchmark_demo.rs` captured during meter full-codegen standardization.
```
