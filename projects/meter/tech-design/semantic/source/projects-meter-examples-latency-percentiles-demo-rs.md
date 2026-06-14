---
id: projects-meter-examples-latency-percentiles-demo-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: benchmark-regression-api
    claim: benchmark-regression-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/examples/latency_percentiles_demo.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/examples/latency_percentiles_demo.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
use meter::baseline::{
    FileBaselineStore, PercentileType, RegressionDetector, RegressionThresholds,
};
use meter::benchmark::{AdaptiveBenchmarkConfig, BenchmarkEnvironment, Benchmarker};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Latency Percentiles Demo ===\n");

    let benchmarker = Benchmarker::default_config();

    // Part 1: Uniform Distribution (consistent latency)
    println!("## Part 1: Uniform Distribution Benchmark");
    println!("Simulating API with consistent 10ms ± 2ms latency...\n");

    let uniform_result = benchmarker.run_adaptive(
        "uniform_latency",
        || {
            // Simulate consistent latency (10ms ± 2ms)
            let jitter = (rand::random::<f64>() * 4.0) - 2.0;
            thread::sleep(Duration::from_micros((10_000.0 + jitter) as u64));
        },
        AdaptiveBenchmarkConfig::quick(),
    );

    println!("Results:");
    println!("{}", uniform_result.stats.format());
    println!("\n📊 Analysis:");
    println!(
        "  Tail Ratio: {:.2}x (uniform distributions typically show ~1.2-1.5x)",
        uniform_result.stats.tail_latency_ratio
    );
    println!(
        "  P99.9: {:.2}ms (close to max, indicating consistent performance)",
        uniform_result.stats.p999_ms
    );

    // Part 2: Skewed Distribution (occasional spikes)
    println!("\n\n## Part 2: Skewed Distribution Benchmark");
    println!("Simulating API with occasional tail latency spikes (20% slow)...\n");

    let skewed_result = benchmarker.run_adaptive(
        "skewed_latency",
        || {
            // 80% fast (10ms), 20% slow (50ms)
            if rand::random::<f64>() < 0.80 {
                thread::sleep(Duration::from_millis(10));
            } else {
                thread::sleep(Duration::from_millis(50)); // 5x slower!
            }
        },
        AdaptiveBenchmarkConfig::quick(),
    );

    println!("Results:");
    println!("{}", skewed_result.stats.format());
    println!("\n📊 Analysis:");
    println!(
        "  Tail Ratio: {:.2}x (skewed distributions show >2.0x, indicating tail latency issues)",
        skewed_result.stats.tail_latency_ratio
    );
    println!(
        "  P99.9: {:.2}ms (much higher than median, confirming tail spikes)",
        skewed_result.stats.p999_ms
    );
    println!(
        "  P50 vs P99: {:.2}ms vs {:.2}ms ({:.0}% difference)",
        skewed_result.stats.median_ms,
        skewed_result.stats.p99_ms,
        ((skewed_result.stats.p99_ms - skewed_result.stats.median_ms)
            / skewed_result.stats.median_ms)
            * 100.0
    );

    // Part 3: Baseline Comparison with Percentile Regression
    println!("\n\n## Part 3: Percentile-Based Regression Detection");
    println!("Simulating a code change that introduces tail latency...\n");

    let store = FileBaselineStore::default_store();
    let env = BenchmarkEnvironment::default();

    // Save uniform as baseline (before the code change)
    println!("Step 1: Save baseline (before code change - uniform distribution)...");
    // Rename to simulate the same benchmark
    let mut baseline_for_save = uniform_result.clone();
    baseline_for_save.name = "api_endpoint".to_string();
    let baseline_results = vec![baseline_for_save];
    if let Err(e) = store.save_baseline("percentile_demo", &baseline_results, &env) {
        eprintln!("Failed to save baseline: {}", e);
        return;
    }
    println!("✅ Baseline saved\n");

    // Rename skewed result to match (simulating the same endpoint after code change)
    println!("Step 2: Simulate code change that introduces tail latency...");
    let mut current_result = skewed_result.clone();
    current_result.name = "api_endpoint".to_string();
    println!("✅ Code change deployed\n");

    println!("Step 3: Compare current performance against baseline...\n");

    // Mean-based comparison
    println!("### Mean-Based Regression Detection:");
    let baseline = match store.load_baseline("percentile_demo", "latest") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to load baseline: {}", e);
            return;
        }
    };

    let mean_thresholds = RegressionThresholds::default().with_percentile(PercentileType::Mean);
    let mean_report = RegressionDetector::detect_regressions(
        &baseline,
        &[current_result.clone()],
        &mean_thresholds,
    );

    let baseline_benchmarks = baseline.benchmarks().expect("expected benchmark baseline");
    println!(
        "  Baseline Mean: {:.2}ms",
        baseline_benchmarks[0].stats.mean_ms
    );
    println!("  Current Mean:  {:.2}ms", current_result.stats.mean_ms);
    println!(
        "  Change: {:+.1}%",
        ((current_result.stats.mean_ms - baseline_benchmarks[0].stats.mean_ms)
            / baseline_benchmarks[0].stats.mean_ms)
            * 100.0
    );

    if mean_report.regressions.is_empty() {
        println!("  ✅ No regression detected");
    } else {
        println!("  ⚠️  Regression detected!");
        for reg in &mean_report.regressions {
            println!(
                "    - {}: {:?} ({:.1}ms → {:.1}ms, {:+.1}%)",
                reg.name,
                reg.severity,
                reg.baseline_value_ms,
                reg.current_value_ms,
                reg.percent_change
            );
        }
    }

    // P95-based comparison (with stricter thresholds)
    println!("\n### P95-Based Regression Detection (5%/15% thresholds):");
    let p95_thresholds = RegressionThresholds {
        warning_threshold_percent: 5.0,
        failure_threshold_percent: 15.0,
        percentile_type: PercentileType::P95,
        ..Default::default()
    };
    let p95_report = RegressionDetector::detect_regressions(
        &baseline,
        &[current_result.clone()],
        &p95_thresholds,
    );

    println!(
        "  Baseline P95: {:.2}ms",
        baseline_benchmarks[0].stats.p95_ms
    );
    println!("  Current P95:  {:.2}ms", current_result.stats.p95_ms);
    println!(
        "  Change: {:+.1}%",
        ((current_result.stats.p95_ms - baseline_benchmarks[0].stats.p95_ms)
            / baseline_benchmarks[0].stats.p95_ms)
            * 100.0
    );

    if !p95_report.regressions.is_empty() {
        println!("  ⚠️  Regression detected!");
        for reg in &p95_report.regressions {
            println!(
                "    - {}: {:?} ({:.1}ms → {:.1}ms, {:+.1}%)",
                reg.name,
                reg.severity,
                reg.baseline_value_ms,
                reg.current_value_ms,
                reg.percent_change
            );
        }
    } else {
        println!("  ✅ No regression detected");
    }

    // P99-based comparison (with stricter thresholds)
    println!("\n### P99-Based Regression Detection (5%/15% thresholds):");
    let p99_thresholds = RegressionThresholds {
        warning_threshold_percent: 5.0,
        failure_threshold_percent: 15.0,
        percentile_type: PercentileType::P99,
        ..Default::default()
    };
    let p99_report = RegressionDetector::detect_regressions(
        &baseline,
        &[current_result.clone()],
        &p99_thresholds,
    );

    println!(
        "  Baseline P99: {:.2}ms",
        baseline_benchmarks[0].stats.p99_ms
    );
    println!("  Current P99:  {:.2}ms", current_result.stats.p99_ms);
    println!(
        "  Change: {:+.1}%",
        ((current_result.stats.p99_ms - baseline_benchmarks[0].stats.p99_ms)
            / baseline_benchmarks[0].stats.p99_ms)
            * 100.0
    );

    if !p99_report.regressions.is_empty() {
        println!("  ⚠️  Regression detected!");
        for reg in &p99_report.regressions {
            println!(
                "    - {}: {:?} ({:.1}ms → {:.1}ms, {:+.1}%)",
                reg.name,
                reg.severity,
                reg.baseline_value_ms,
                reg.current_value_ms,
                reg.percent_change
            );
        }
    } else {
        println!("  ✅ No regression detected");
    }

    // Part 4: Tail Latency Ratio Analysis
    println!("\n\n## Part 4: Tail Latency Ratio as Quick Health Check");
    println!("Comparing tail ratios between uniform and skewed distributions:\n");

    println!("  Uniform Distribution:");
    println!(
        "    - Tail Ratio: {:.2}x",
        uniform_result.stats.tail_latency_ratio
    );
    println!(
        "    - P50: {:.2}ms, P99: {:.2}ms",
        uniform_result.stats.median_ms, uniform_result.stats.p99_ms
    );
    println!("    - Status: ✅ Healthy (ratio < 2.0x)");

    println!("\n  Skewed Distribution:");
    println!(
        "    - Tail Ratio: {:.2}x",
        skewed_result.stats.tail_latency_ratio
    );
    println!(
        "    - P50: {:.2}ms, P99: {:.2}ms",
        skewed_result.stats.median_ms, skewed_result.stats.p99_ms
    );
    println!("    - Status: ⚠️  Investigate (ratio > 2.0x indicates tail latency issues)");

    println!("\n  Rule of Thumb:");
    println!("    - Ratio 1.0-1.5x: Excellent consistency");
    println!("    - Ratio 1.5-2.0x: Acceptable variation");
    println!("    - Ratio 2.0-3.0x: Monitor closely");
    println!("    - Ratio >3.0x:    Action required (investigate outliers)");

    // Summary
    println!("\n\n## Summary");
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ Percentile-based regression detection is crucial for APIs     │");
    println!("│ where tail latency impacts user experience.                   │");
    println!("│                                                                │");
    println!("│ In this example, mean DID catch the regression because 20%    │");
    println!("│ slow requests heavily impacted the average. But with <5%      │");
    println!("│ slow requests, mean stays normal while P95/P99 catch issues!  │");
    println!("│                                                                │");
    println!("│ Recommendation: Use P95 or P99 for production services.       │");
    println!("└────────────────────────────────────────────────────────────────┘");

    println!("\n✨ Key Takeaways:");
    println!("  1. Uniform distribution: Tail Ratio ~1.0-1.5x (excellent!)");
    println!("  2. Skewed distribution: Tail Ratio >2.0x (investigate!)");
    println!("  3. Histogram shows visual distribution shape");
    println!("  4. P999/P9999 catch extreme outliers (important for SLAs)");
    println!("  5. P95/P99 regression detection >> mean-based detection");
    println!("  6. Tail ratio provides quick health check without baselines");
    println!("  7. Default thresholds: 5% warning, 15% critical");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/examples/latency_percentiles_demo.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/examples/latency_percentiles_demo.rs` captured during meter full-codegen standardization.
```
