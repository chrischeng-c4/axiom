// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-examples-baseline-demo-rs.md#source
// CODEGEN-BEGIN
use meter::baseline::{FileBaselineStore, PercentileType, RegressionDetector, RegressionThresholds};
use meter::benchmark::{AdaptiveBenchmarkConfig, BenchmarkEnvironment, Benchmarker};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Baseline Metrics Demo ===\n");

    let benchmarker = Benchmarker::default_config();
    let store = FileBaselineStore::default_store();

    // Run initial benchmarks
    println!("Running initial benchmarks...");
    let mut results = Vec::new();

    let result1 = benchmarker.run_adaptive(
        "fast_operation",
        || thread::sleep(Duration::from_millis(10)),
        AdaptiveBenchmarkConfig::quick(),
    );
    results.push(result1);

    let result2 = benchmarker.run_adaptive(
        "medium_operation",
        || thread::sleep(Duration::from_millis(50)),
        AdaptiveBenchmarkConfig::quick(),
    );
    results.push(result2);

    println!("✅ Benchmarks complete\n");

    // Save as baseline
    println!("Saving baseline...");
    let env = BenchmarkEnvironment::default();
    match store.save_baseline("demo", &results, &env) {
        Ok(id) => println!("✅ Baseline saved: {}\n", id),
        Err(e) => {
            eprintln!("❌ Failed to save baseline: {}", e);
            return;
        }
    }

    // Simulate code changes - make operations slower
    println!("Simulating performance regression...");
    let mut new_results = Vec::new();

    let result1_slow = benchmarker.run_adaptive(
        "fast_operation",
        || thread::sleep(Duration::from_millis(12)), // 20% slower
        AdaptiveBenchmarkConfig::quick(),
    );
    new_results.push(result1_slow);

    let result2_improved = benchmarker.run_adaptive(
        "medium_operation",
        || thread::sleep(Duration::from_millis(45)), // 10% faster
        AdaptiveBenchmarkConfig::quick(),
    );
    new_results.push(result2_improved);

    println!("✅ New benchmarks complete\n");

    // Load baseline and detect regressions
    println!("Comparing against baseline...");
    match store.load_baseline("demo", "latest") {
        Ok(baseline) => {
            // Mean-based regression detection
            println!("\n=== Mean-based Regression Report ===");
            let thresholds = RegressionThresholds::default();
            let report =
                RegressionDetector::detect_regressions(&baseline, &new_results, &thresholds);

            println!("Total benchmarks: {}", report.summary.total_benchmarks);
            println!("Regressions found: {}", report.summary.regressions_found);
            println!("Improvements found: {}", report.summary.improvements_found);
            println!("Unchanged: {}\n", report.summary.unchanged);

            if !report.regressions.is_empty() {
                println!("⚠️  Regressions:");
                for reg in &report.regressions {
                    println!(
                        "  - {} ({:?}): {:.1}ms → {:.1}ms ({:+.1}%)",
                        reg.name,
                        reg.severity,
                        reg.baseline_value_ms,
                        reg.current_value_ms,
                        reg.percent_change
                    );
                }
                println!();
            }

            if !report.improvements.is_empty() {
                println!("✨ Improvements:");
                for imp in &report.improvements {
                    println!(
                        "  - {}: {:.1}ms → {:.1}ms ({:.1}% faster)",
                        imp.name,
                        imp.baseline_mean_ms,
                        imp.current_mean_ms,
                        imp.percent_change.abs()
                    );
                }
                println!();
            }

            // P99-based regression detection
            println!("=== P99 Regression Detection ===");
            let p99_thresholds =
                RegressionThresholds::default().with_percentile(PercentileType::P99);

            let p99_report =
                RegressionDetector::detect_regressions(&baseline, &new_results, &p99_thresholds);

            println!("Checking P99 latencies...");
            if !p99_report.regressions.is_empty() {
                println!("⚠️  P99 Regressions:");
                for reg in &p99_report.regressions {
                    println!(
                        "  - {}: {:.1}ms → {:.1}ms ({:+.1}%)",
                        reg.name, reg.baseline_value_ms, reg.current_value_ms, reg.percent_change
                    );
                }
            } else {
                println!("✅ No P99 regressions detected");
            }

            // P95-based regression detection
            println!("\n=== P95 Regression Detection ===");
            let p95_thresholds =
                RegressionThresholds::default().with_percentile(PercentileType::P95);

            let p95_report =
                RegressionDetector::detect_regressions(&baseline, &new_results, &p95_thresholds);

            println!("Checking P95 latencies...");
            if !p95_report.regressions.is_empty() {
                println!("⚠️  P95 Regressions:");
                for reg in &p95_report.regressions {
                    println!(
                        "  - {}: {:.1}ms → {:.1}ms ({:+.1}%)",
                        reg.name, reg.baseline_value_ms, reg.current_value_ms, reg.percent_change
                    );
                }
            } else {
                println!("✅ No P95 regressions detected");
            }
        }
        Err(e) => eprintln!("❌ Failed to load baseline: {}", e),
    }
}
// CODEGEN-END
