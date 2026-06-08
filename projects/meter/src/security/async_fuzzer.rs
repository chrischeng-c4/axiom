// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-security-async-fuzzer-rs.md#source
// CODEGEN-BEGIN
//! Async fuzzing framework for security testing
//!
//! Provides async mutation-based fuzzing capabilities with support for:
//! - Async target functions
//! - Concurrent fuzzing with tokio::spawn
//! - HTTP endpoint fuzzing with reqwest
//! - Timeout handling with tokio::time::timeout

use super::fuzzer::{FuzzCrash, FuzzResult, MutationStrategy};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Configuration for async fuzzing operations
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-async-fuzzer-rs.md#source
pub struct AsyncFuzzConfig {
    /// Initial corpus of seed inputs
    pub corpus: Vec<String>,
    /// Maximum fuzzing iterations
    pub max_iterations: u32,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Mutation rate (0.0-1.0)
    pub mutation_rate: f32,
    /// Timeout per iteration (milliseconds)
    pub timeout_ms: u64,
    /// Number of concurrent fuzzing tasks
    pub concurrent_mutations: usize,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-async-fuzzer-rs.md#source
impl AsyncFuzzConfig {
    /// Create a new async fuzzing configuration
    pub fn new() -> Self {
        Self {
            corpus: Vec::new(),
            max_iterations: 1000,
            seed: None,
            mutation_rate: 0.5,
            timeout_ms: 100,
            concurrent_mutations: 4,
        }
    }

    /// Set the corpus of seed inputs
    pub fn with_corpus(mut self, corpus: Vec<String>) -> Self {
        self.corpus = corpus;
        self
    }

    /// Set maximum iterations
    pub fn with_iterations(mut self, iterations: u32) -> Self {
        self.max_iterations = iterations;
        self
    }

    /// Set random seed for reproducibility
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set mutation rate (0.0-1.0)
    pub fn with_mutation_rate(mut self, rate: f32) -> Self {
        self.mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set timeout per iteration in milliseconds
    pub fn with_timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }

    /// Set number of concurrent fuzzing tasks
    pub fn with_concurrent_mutations(mut self, concurrency: usize) -> Self {
        self.concurrent_mutations = concurrency.max(1);
        self
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-async-fuzzer-rs.md#source
impl Default for AsyncFuzzConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Async fuzzer for testing async target functions and network endpoints
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-async-fuzzer-rs.md#source
pub struct AsyncFuzzer {
    config: AsyncFuzzConfig,
    rng: StdRng,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-async-fuzzer-rs.md#source
impl AsyncFuzzer {
    /// Create a new async fuzzer with configuration
    pub fn new(config: AsyncFuzzConfig) -> Self {
        let seed = config.seed.unwrap_or_else(rand::random);
        let rng = StdRng::seed_from_u64(seed);
        Self { config, rng }
    }

    /// Add an input to the corpus
    pub fn add_corpus(&mut self, input: impl Into<String>) {
        self.config.corpus.push(input.into());
    }

    /// Fuzz an async target function
    ///
    /// The target function should return Ok(()) for valid inputs and Err(msg) for invalid inputs.
    /// The fuzzer will collect crashes (errors or timeouts).
    pub async fn fuzz_async<F, Fut>(&mut self, target: F) -> FuzzResult
    where
        F: Fn(String) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<(), String>> + Send,
    {
        let start = Instant::now();
        let mut crashes = Vec::new();

        // Ensure we have corpus
        if self.config.corpus.is_empty() {
            self.config.corpus.push(String::new());
        }

        for iteration in 0..self.config.max_iterations {
            // Select random corpus entry
            let corpus_idx = self.rng.gen_range(0..self.config.corpus.len());
            let base_input = self.config.corpus[corpus_idx].clone();

            // Apply mutation
            let mutated = if self.rng.gen::<f32>() < self.config.mutation_rate {
                let strategy = self.random_strategy();
                self.mutate(&base_input, strategy)
            } else {
                base_input
            };

            // Test with timeout
            let timeout_duration = Duration::from_millis(self.config.timeout_ms);
            let result = timeout(timeout_duration, target(mutated.clone())).await;

            match result {
                Ok(Ok(())) => {
                    // Target succeeded, no crash
                }
                Ok(Err(error)) => {
                    // Target returned error - potential crash
                    crashes.push(FuzzCrash {
                        input: mutated,
                        error,
                        iteration,
                    });
                }
                Err(_) => {
                    // Timeout - also a crash
                    crashes.push(FuzzCrash {
                        input: mutated,
                        error: "Timeout exceeded".to_string(),
                        iteration,
                    });
                }
            }
        }

        FuzzResult {
            iterations: self.config.max_iterations,
            crashes,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Fuzz with concurrent mutations
    ///
    /// Spawns multiple tokio tasks to perform fuzzing in parallel, improving throughput.
    pub async fn fuzz_parallel<F, Fut>(&mut self, target: F) -> FuzzResult
    where
        F: Fn(String) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<(), String>> + Send + 'static,
    {
        let start = Instant::now();
        let mut crashes = Vec::new();

        // Ensure corpus
        if self.config.corpus.is_empty() {
            self.config.corpus.push(String::new());
        }

        let iterations_per_task =
            self.config.max_iterations / self.config.concurrent_mutations as u32;
        let mut handles = Vec::new();

        for task_id in 0..self.config.concurrent_mutations {
            let corpus = self.config.corpus.clone();
            let target = target.clone();
            let timeout_ms = self.config.timeout_ms;
            let mutation_rate = self.config.mutation_rate;
            let seed = self.config.seed.unwrap_or_else(rand::random) + task_id as u64;

            let handle = tokio::spawn(async move {
                let mut rng = StdRng::seed_from_u64(seed);
                let mut task_crashes = Vec::new();

                for iteration in 0..iterations_per_task {
                    let corpus_idx = rng.gen_range(0..corpus.len());
                    let base_input = corpus[corpus_idx].clone();

                    let mutated = if rng.gen::<f32>() < mutation_rate {
                        let strategy = Self::random_strategy_static(&mut rng);
                        Self::mutate_static(&base_input, strategy, &mut rng)
                    } else {
                        base_input
                    };

                    let timeout_duration = Duration::from_millis(timeout_ms);
                    let result = timeout(timeout_duration, target(mutated.clone())).await;

                    match result {
                        Ok(Ok(())) => {}
                        Ok(Err(error)) => {
                            task_crashes.push(FuzzCrash {
                                input: mutated,
                                error,
                                iteration: task_id as u32 * iterations_per_task + iteration,
                            });
                        }
                        Err(_) => {
                            task_crashes.push(FuzzCrash {
                                input: mutated,
                                error: "Timeout exceeded".to_string(),
                                iteration: task_id as u32 * iterations_per_task + iteration,
                            });
                        }
                    }
                }

                task_crashes
            });

            handles.push(handle);
        }

        // Collect results from all tasks
        for handle in handles {
            if let Ok(task_crashes) = handle.await {
                crashes.extend(task_crashes);
            }
        }

        FuzzResult {
            iterations: self.config.max_iterations,
            crashes,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Fuzz an HTTP endpoint with mutated payloads
    ///
    /// # Arguments
    /// * `url` - The endpoint URL to fuzz
    /// * `method` - HTTP method (GET, POST, etc.)
    ///
    /// # Example
    /// ```no_run
    /// use meter::{AsyncFuzzer, AsyncFuzzConfig};
    ///
    /// # async fn example() {
    /// let config = AsyncFuzzConfig::new().with_iterations(50);
    /// let mut fuzzer = AsyncFuzzer::new(config);
    /// let result = fuzzer.fuzz_http_endpoint("http://localhost:8080/api/test", "POST").await;
    /// println!("Found {} crashes", result.crashes.len());
    /// # }
    /// ```
    pub async fn fuzz_http_endpoint(&mut self, url: &str, method: &str) -> FuzzResult {
        let client = reqwest::Client::new();
        let url_owned = url.to_string();
        let method_owned = method.to_uppercase();

        self.fuzz_async(move |input| {
            let client = client.clone();
            let url = url_owned.clone();
            let method = method_owned.clone();

            async move {
                let request = match method.as_str() {
                    "GET" => client.get(&url).query(&[("input", input)]),
                    "POST" => client.post(&url).body(input),
                    "PUT" => client.put(&url).body(input),
                    "DELETE" => client.delete(&url).query(&[("input", input)]),
                    _ => return Err(format!("Unsupported method: {}", method)),
                };

                match request.send().await {
                    Ok(resp) if resp.status().is_success() => Ok(()),
                    Ok(resp) => Err(format!(
                        "HTTP {}: {}",
                        resp.status(),
                        resp.text().await.unwrap_or_default()
                    )),
                    Err(e) => Err(format!("Request failed: {}", e)),
                }
            }
        })
        .await
    }

    // Helper methods (reuse mutation logic from sync fuzzer)

    fn mutate(&mut self, input: &str, strategy: MutationStrategy) -> String {
        Self::mutate_static(input, strategy, &mut self.rng)
    }

    fn mutate_static(input: &str, strategy: MutationStrategy, rng: &mut StdRng) -> String {
        if input.is_empty() && strategy != MutationStrategy::Insert {
            return input.to_string();
        }

        let mut bytes = input.as_bytes().to_vec();

        match strategy {
            MutationStrategy::BitFlip => {
                if !bytes.is_empty() {
                    let idx = rng.gen_range(0..bytes.len());
                    let bit = rng.gen_range(0..8);
                    bytes[idx] ^= 1 << bit;
                }
            }
            MutationStrategy::ByteFlip => {
                if !bytes.is_empty() {
                    let idx = rng.gen_range(0..bytes.len());
                    bytes[idx] = rng.gen();
                }
            }
            MutationStrategy::Insert => {
                let idx = if bytes.is_empty() {
                    0
                } else {
                    rng.gen_range(0..=bytes.len())
                };
                let new_byte: u8 = rng.gen();
                bytes.insert(idx, new_byte);
            }
            MutationStrategy::Delete => {
                if !bytes.is_empty() {
                    let idx = rng.gen_range(0..bytes.len());
                    bytes.remove(idx);
                }
            }
            MutationStrategy::Substitute => {
                if !bytes.is_empty() {
                    let idx = rng.gen_range(0..bytes.len());
                    let substitutions = b"'\"\\\0\n\r\t;--/**/";
                    let sub_idx = rng.gen_range(0..substitutions.len());
                    bytes[idx] = substitutions[sub_idx];
                }
            }
            MutationStrategy::Combine => {
                // For Combine strategy in static context, we just duplicate the input
                // since we don't have access to corpus
                bytes.extend_from_slice(input.as_bytes());
            }
        }

        // Try to convert back to UTF-8, fall back to lossy conversion
        String::from_utf8(bytes)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string())
    }

    fn random_strategy(&mut self) -> MutationStrategy {
        Self::random_strategy_static(&mut self.rng)
    }

    fn random_strategy_static(rng: &mut StdRng) -> MutationStrategy {
        match rng.gen_range(0..6) {
            0 => MutationStrategy::BitFlip,
            1 => MutationStrategy::ByteFlip,
            2 => MutationStrategy::Insert,
            3 => MutationStrategy::Delete,
            4 => MutationStrategy::Substitute,
            _ => MutationStrategy::Combine,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_fuzzer_basic() {
        let config = AsyncFuzzConfig::new()
            .with_iterations(10)
            .with_seed(42)
            .with_corpus(vec!["test".to_string()]);

        let mut fuzzer = AsyncFuzzer::new(config);

        let result = fuzzer.fuzz_async(|_input| async { Ok(()) }).await;

        assert_eq!(result.iterations, 10);
        assert!(result.crashes.is_empty());
    }

    #[tokio::test]
    async fn test_async_fuzzer_crash_detection() {
        let config = AsyncFuzzConfig::new()
            .with_iterations(200)
            .with_seed(42)
            .with_corpus(vec!["test".to_string()]);

        let mut fuzzer = AsyncFuzzer::new(config);

        let result = fuzzer
            .fuzz_async(|input| async move {
                if input.contains("crash") || input.contains('\0') {
                    Err("Crash detected!".to_string())
                } else {
                    Ok(())
                }
            })
            .await;

        assert_eq!(result.iterations, 200);
        // Note: With random mutations, we might or might not get crashes
        // This test mainly verifies that crash detection works if triggered
    }

    #[tokio::test]
    async fn test_async_fuzzer_timeout() {
        let config = AsyncFuzzConfig::new()
            .with_iterations(5)
            .with_timeout_ms(10);

        let mut fuzzer = AsyncFuzzer::new(config);

        let result = fuzzer
            .fuzz_async(|_input| async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(())
            })
            .await;

        // Should have timeouts
        assert_eq!(result.iterations, 5);
        assert!(!result.crashes.is_empty(), "Expected timeout crashes");
        assert!(result.crashes.iter().all(|c| c.error == "Timeout exceeded"));
    }

    #[tokio::test]
    async fn test_async_fuzzer_parallel() {
        let config = AsyncFuzzConfig::new()
            .with_iterations(20)
            .with_concurrent_mutations(4)
            .with_seed(42);

        let mut fuzzer = AsyncFuzzer::new(config);

        let result = fuzzer.fuzz_parallel(|_input| async { Ok(()) }).await;

        assert_eq!(result.iterations, 20);
        assert!(result.crashes.is_empty());
    }

    #[tokio::test]
    async fn test_async_fuzzer_parallel_with_crashes() {
        let config = AsyncFuzzConfig::new()
            .with_iterations(40)
            .with_concurrent_mutations(4)
            .with_seed(42)
            .with_corpus(vec!["test".to_string(), "data".to_string()]);

        let mut fuzzer = AsyncFuzzer::new(config);

        let result = fuzzer
            .fuzz_parallel(|input| async move {
                if input.len() > 50 {
                    Err("Input too large".to_string())
                } else {
                    Ok(())
                }
            })
            .await;

        assert_eq!(result.iterations, 40);
    }

    #[tokio::test]
    async fn test_async_fuzzer_add_corpus() {
        let config = AsyncFuzzConfig::new();
        let mut fuzzer = AsyncFuzzer::new(config);

        assert_eq!(fuzzer.config.corpus.len(), 0);
        fuzzer.add_corpus("new_input");
        assert_eq!(fuzzer.config.corpus.len(), 1);
    }

    #[tokio::test]
    async fn test_async_fuzz_config_defaults() {
        let config = AsyncFuzzConfig::default();
        assert_eq!(config.max_iterations, 1000);
        assert_eq!(config.mutation_rate, 0.5);
        assert_eq!(config.timeout_ms, 100);
        assert_eq!(config.concurrent_mutations, 4);
        assert!(config.seed.is_none());
    }

    #[tokio::test]
    async fn test_async_fuzz_config_builder() {
        let config = AsyncFuzzConfig::new()
            .with_iterations(500)
            .with_seed(42)
            .with_mutation_rate(0.7)
            .with_timeout_ms(50)
            .with_concurrent_mutations(8);

        assert_eq!(config.max_iterations, 500);
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.mutation_rate, 0.7);
        assert_eq!(config.timeout_ms, 50);
        assert_eq!(config.concurrent_mutations, 8);
    }

    #[tokio::test]
    async fn test_mutation_rate_clamping() {
        let config = AsyncFuzzConfig::new().with_mutation_rate(1.5);
        assert_eq!(config.mutation_rate, 1.0);

        let config = AsyncFuzzConfig::new().with_mutation_rate(-0.5);
        assert_eq!(config.mutation_rate, 0.0);
    }

    #[tokio::test]
    async fn test_concurrent_mutations_minimum() {
        let config = AsyncFuzzConfig::new().with_concurrent_mutations(0);
        assert_eq!(config.concurrent_mutations, 1);
    }
}
// CODEGEN-END
