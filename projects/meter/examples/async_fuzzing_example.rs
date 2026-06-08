// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-examples-async-fuzzing-example-rs.md#source
// CODEGEN-BEGIN
use meter::{AsyncFuzzConfig, AsyncFuzzer};

#[tokio::main]
async fn main() {
    println!("=== Async Fuzzing Example ===\n");

    // Example 1: Basic async function fuzzing
    println!("1. Fuzzing async validation function");
    let config = AsyncFuzzConfig::new()
        .with_iterations(100)
        .with_seed(42)
        .with_corpus(vec![
            "valid".to_string(),
            "test".to_string(),
            "input".to_string(),
        ]);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer
        .fuzz_async(|input| async move {
            // Simulate async validation
            tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;

            if input.contains('\0') {
                Err("Null byte detected".to_string())
            } else if input.len() > 100 {
                Err("Input too long".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    println!("  Iterations: {}", result.iterations);
    println!("  Crashes: {}", result.crashes.len());
    println!("  Duration: {}ms\n", result.duration_ms);

    if !result.crashes.is_empty() {
        println!("  Sample crashes:");
        for (i, crash) in result.crashes.iter().take(3).enumerate() {
            let display_input = crash
                .input
                .chars()
                .take(20)
                .collect::<String>()
                .replace('\0', "\\0")
                .replace('\n', "\\n")
                .replace('\r', "\\r");
            println!(
                "    {}. [{}] {} - Error: {}",
                i + 1,
                crash.iteration,
                display_input,
                crash.error
            );
        }
        println!();
    }

    // Example 2: Parallel fuzzing
    println!("2. Parallel fuzzing with 8 concurrent tasks");
    let config = AsyncFuzzConfig::new()
        .with_iterations(200)
        .with_concurrent_mutations(8)
        .with_timeout_ms(100)
        .with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    let start = std::time::Instant::now();
    let result = fuzzer
        .fuzz_parallel(|input| async move {
            tokio::time::sleep(tokio::time::Duration::from_micros(50)).await;

            if input.starts_with("DROP") || input.starts_with("DELETE") {
                Err("SQL injection attempt".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    let elapsed = start.elapsed();

    println!("  Iterations: {}", result.iterations);
    println!("  Crashes: {}", result.crashes.len());
    println!("  Elapsed: {}ms", elapsed.as_millis());
    println!(
        "  Throughput: {:.1} iterations/sec\n",
        result.iterations as f64 / elapsed.as_secs_f64()
    );

    // Example 3: Testing timeout handling
    println!("3. Testing timeout handling");
    let config = AsyncFuzzConfig::new()
        .with_iterations(10)
        .with_timeout_ms(10)
        .with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    let result = fuzzer
        .fuzz_async(|_input| async {
            // This will timeout
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(())
        })
        .await;

    println!("  Iterations: {}", result.iterations);
    println!("  Timeouts: {}", result.crashes.len());
    println!("  Duration: {}ms\n", result.duration_ms);

    // Example 4: Adding inputs to corpus
    println!("4. Dynamic corpus building");
    let config = AsyncFuzzConfig::new().with_iterations(50).with_seed(42);

    let mut fuzzer = AsyncFuzzer::new(config);

    // Add interesting test cases
    fuzzer.add_corpus("admin");
    fuzzer.add_corpus("user@example.com");
    fuzzer.add_corpus("<script>alert(1)</script>");
    fuzzer.add_corpus("'; DROP TABLE users--");

    let result = fuzzer
        .fuzz_async(|input| async move {
            // Simulate XSS/SQL injection detection
            if input.contains("<script>") || input.contains("DROP TABLE") {
                Err("Security violation".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    println!("  Iterations: {}", result.iterations);
    println!("  Security violations found: {}", result.crashes.len());
    println!("  Duration: {}ms\n", result.duration_ms);

    // Example 5: Reproducible fuzzing
    println!("5. Reproducible fuzzing with seed");
    let seed = 12345;

    let config1 = AsyncFuzzConfig::new().with_iterations(30).with_seed(seed);
    let mut fuzzer1 = AsyncFuzzer::new(config1);

    let result1 = fuzzer1
        .fuzz_async(|input| async move {
            if input.len() > 50 {
                Err("Too long".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    let config2 = AsyncFuzzConfig::new().with_iterations(30).with_seed(seed);
    let mut fuzzer2 = AsyncFuzzer::new(config2);

    let result2 = fuzzer2
        .fuzz_async(|input| async move {
            if input.len() > 50 {
                Err("Too long".to_string())
            } else {
                Ok(())
            }
        })
        .await;

    println!("  Run 1 crashes: {}", result1.crashes.len());
    println!("  Run 2 crashes: {}", result2.crashes.len());
    println!(
        "  Reproducible: {}",
        result1.crashes.len() == result2.crashes.len()
    );
    println!();

    println!("=== Example Complete ===");
}
// CODEGEN-END
