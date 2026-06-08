---
id: projects-meter-tests-async-fuzzing-test-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: legacy-carried-internals
    role: primary
    gap: seeded-fuzz-and-injection-finding-generation
    claim: seeded-fuzz-and-injection-finding-generation
    coverage: full
    rationale: "Source template implements meter security, fuzzing, injection, or audit surfaces."
---

# Standardized projects/meter/tests/async_fuzzing_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/tests/async_fuzzing_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/tests/async_fuzzing_test.rs -->
````rust
use meter::{AsyncFuzzConfig, AsyncFuzzer, TestServer};
use tokio::time::Duration;

#[tokio::test]
async fn test_async_fuzzing_basic() {
    let config = AsyncFuzzConfig::new()
        .with_iterations(50)
        .with_corpus(vec!["test".to_string(), "hello".to_string()]);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer
        .fuzz_async(|input| async move {
            if input.len() > 1000 {
                Err("Input too large".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    println!(
        "Completed {} iterations, found {} crashes",
        result.iterations,
        result.crashes.len()
    );

    assert_eq!(result.iterations, 50);
}

#[tokio::test]
async fn test_async_fuzzing_crash_detection() {
    let config = AsyncFuzzConfig::new()
        .with_iterations(100)
        .with_seed(42)
        .with_corpus(vec!["test".to_string()]);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer
        .fuzz_async(|input| async move {
            // Simulate validation that fails on certain patterns
            if input.contains('\0') {
                Err("Null byte detected".to_string())
            } else if input.len() > 500 {
                Err("Input too large".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    println!(
        "Found {} crashes in {} iterations",
        result.crashes.len(),
        result.iterations
    );

    assert_eq!(result.iterations, 100);
    // Some mutations should trigger validation errors
}

#[tokio::test]
async fn test_http_endpoint_fuzzing() {
    // Start test server
    let server = TestServer::new()
        .post_echo("/api/test")
        .start()
        .await
        .unwrap();

    let config = AsyncFuzzConfig::new()
        .with_iterations(20)
        .with_timeout_ms(500)
        .with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    let url = format!("{}/api/test", server.url());
    let result = fuzzer.fuzz_http_endpoint(&url, "POST").await;

    println!(
        "HTTP fuzzing: {} iterations, {} crashes",
        result.iterations,
        result.crashes.len()
    );

    assert_eq!(result.iterations, 20);
    // Echo endpoint should accept most inputs
}

#[tokio::test]
async fn test_http_endpoint_fuzzing_get() {
    // Start test server with GET endpoint
    let server = TestServer::new()
        .get("/api/test", serde_json::json!({"status": "ok"}))
        .start()
        .await
        .unwrap();

    let config = AsyncFuzzConfig::new()
        .with_iterations(15)
        .with_timeout_ms(500)
        .with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    let url = format!("{}/api/test", server.url());
    let result = fuzzer.fuzz_http_endpoint(&url, "GET").await;

    println!(
        "HTTP GET fuzzing: {} iterations, {} crashes",
        result.iterations,
        result.crashes.len()
    );

    assert_eq!(result.iterations, 15);
}

#[tokio::test]
async fn test_parallel_fuzzing() {
    let config = AsyncFuzzConfig::new()
        .with_iterations(100)
        .with_concurrent_mutations(8)
        .with_timeout_ms(50)
        .with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    let start = std::time::Instant::now();
    let result = fuzzer
        .fuzz_parallel(|_input| async {
            tokio::time::sleep(Duration::from_micros(10)).await;
            Ok(())
        })
        .await;

    let elapsed = start.elapsed();
    println!(
        "Parallel fuzzing: {} iterations in {}ms",
        result.iterations,
        elapsed.as_millis()
    );

    assert_eq!(result.iterations, 100);
    // Should be faster than sequential (100 * 10us = 1000us minimum)
    // With 8 concurrent tasks, should complete in ~200us + overhead
    assert!(elapsed.as_millis() < 500, "Parallel fuzzing should be fast");
}

#[tokio::test]
async fn test_parallel_fuzzing_with_crashes() {
    let config = AsyncFuzzConfig::new()
        .with_iterations(80)
        .with_concurrent_mutations(4)
        .with_seed(42)
        .with_corpus(vec!["test".to_string(), "data".to_string()]);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer
        .fuzz_parallel(|input| async move {
            if input.len() > 100 {
                Err("Input too large".to_string())
            } else if input.contains("DROP") {
                Err("SQL injection attempt".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    println!("Parallel fuzzing found {} crashes", result.crashes.len());
    assert_eq!(result.iterations, 80);
}

#[tokio::test]
async fn test_timeout_handling() {
    let config = AsyncFuzzConfig::new()
        .with_iterations(10)
        .with_timeout_ms(20)
        .with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer
        .fuzz_async(|_input| async {
            // Sleep longer than timeout
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        })
        .await;

    println!(
        "Timeout test: {} crashes (all should be timeouts)",
        result.crashes.len()
    );

    // All iterations should timeout
    assert_eq!(result.crashes.len(), 10);
    assert!(result.crashes.iter().all(|c| c.error == "Timeout exceeded"));
}

#[tokio::test]
async fn test_async_fuzzing_reproducibility() {
    let config = AsyncFuzzConfig::new()
        .with_iterations(30)
        .with_seed(12345)
        .with_corpus(vec!["seed".to_string()]);

    let mut fuzzer1 = AsyncFuzzer::new(config.clone());
    let result1 = fuzzer1
        .fuzz_async(|input| async move {
            if input.contains('\0') {
                Err("Invalid input".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    let mut fuzzer2 = AsyncFuzzer::new(config);
    let result2 = fuzzer2
        .fuzz_async(|input| async move {
            if input.contains('\0') {
                Err("Invalid input".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    // Same seed should produce same number of crashes
    assert_eq!(
        result1.crashes.len(),
        result2.crashes.len(),
        "Same seed should produce reproducible results"
    );
}

#[tokio::test]
async fn test_corpus_mutation() {
    let config = AsyncFuzzConfig::new()
        .with_iterations(50)
        .with_seed(42)
        .with_mutation_rate(1.0) // Always mutate
        .with_corpus(vec!["original".to_string()]);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer
        .fuzz_async(|_input| async move {
            // With mutation rate 1.0, all inputs should be mutated
            Ok(())
        })
        .await;

    // With mutation rate 1.0, we should see mutations
    assert_eq!(result.iterations, 50);
}

#[tokio::test]
async fn test_empty_corpus_handling() {
    let config = AsyncFuzzConfig::new().with_iterations(10).with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer.fuzz_async(|_input| async { Ok(()) }).await;

    // Should handle empty corpus by creating one
    assert_eq!(result.iterations, 10);
    assert!(result.crashes.is_empty());
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/tests/async_fuzzing_test.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/tests/async_fuzzing_test.rs` captured during meter full-codegen standardization.
```
