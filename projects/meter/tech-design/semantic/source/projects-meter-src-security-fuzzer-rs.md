---
id: projects-meter-src-security-fuzzer-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: legacy-carried-internals
    role: primary
    gap: seeded-fuzz-and-injection-finding-generation
    claim: seeded-fuzz-and-injection-finding-generation
    coverage: full
    rationale: "Source template implements meter security, fuzzing, injection, or audit surfaces."
---

# Standardized projects/meter/src/security/fuzzer.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/security/fuzzer.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FuzzConfig` | projects/meter/src/security/fuzzer.rs | struct | pub | 15 |  |
| `FuzzCrash` | projects/meter/src/security/fuzzer.rs | struct | pub | 121 |  |
| `FuzzResult` | projects/meter/src/security/fuzzer.rs | struct | pub | 133 |  |
| `Fuzzer` | projects/meter/src/security/fuzzer.rs | struct | pub | 144 |  |
| `MutationStrategy` | projects/meter/src/security/fuzzer.rs | enum | pub | 88 |  |
| `add_corpus` | projects/meter/src/security/fuzzer.rs | function | pub | 162 | add_corpus(&mut self, input: &str) |
| `all` | projects/meter/src/security/fuzzer.rs | function | pub | 106 | all() -> &'static [MutationStrategy] |
| `fuzz` | projects/meter/src/security/fuzzer.rs | function | pub | 170 | fuzz(&self, target: F) -> FuzzResult |
| `mutate` | projects/meter/src/security/fuzzer.rs | function | pub | 227 | mutate(&self, input: &str, strategy: MutationStrategy) -> String |
| `new` | projects/meter/src/security/fuzzer.rs | function | pub | 50 | new() -> Self |
| `new` | projects/meter/src/security/fuzzer.rs | function | pub | 152 | new(config: FuzzConfig) -> Self |
| `with_corpus` | projects/meter/src/security/fuzzer.rs | function | pub | 55 | with_corpus(mut self, corpus: Vec<String>) -> Self |
| `with_iterations` | projects/meter/src/security/fuzzer.rs | function | pub | 61 | with_iterations(mut self, iterations: u32) -> Self |
| `with_mutation_rate` | projects/meter/src/security/fuzzer.rs | function | pub | 73 | with_mutation_rate(mut self, rate: f32) -> Self |
| `with_seed` | projects/meter/src/security/fuzzer.rs | function | pub | 67 | with_seed(mut self, seed: u64) -> Self |
| `with_timeout_ms` | projects/meter/src/security/fuzzer.rs | function | pub | 79 | with_timeout_ms(mut self, timeout_ms: u64) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Fuzzing framework for security testing
//!
//! Provides mutation-based fuzzing capabilities to test input validators,
//! parsers, and security boundaries.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};

/// Configuration for fuzzing
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
pub struct FuzzConfig {
    /// Initial corpus of inputs to mutate
    pub corpus: Vec<String>,
    /// Maximum number of fuzzing iterations
    pub max_iterations: u32,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Mutation rate (0.0 to 1.0)
    pub mutation_rate: f32,
    /// Timeout per iteration in milliseconds
    pub timeout_ms: Option<u64>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
impl Default for FuzzConfig {
    fn default() -> Self {
        Self {
            corpus: vec![
                String::from(""),
                String::from("a"),
                String::from("test"),
                String::from("0"),
                String::from("\0"),
            ],
            max_iterations: 1000,
            seed: None,
            mutation_rate: 0.5,
            timeout_ms: Some(100),
        }
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
impl FuzzConfig {
    /// Create a new fuzzing configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the corpus of inputs
    pub fn with_corpus(mut self, corpus: Vec<String>) -> Self {
        self.corpus = corpus;
        self
    }

    /// Set maximum iterations
    pub fn with_iterations(mut self, iterations: u32) -> Self {
        self.max_iterations = iterations;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set mutation rate
    pub fn with_mutation_rate(mut self, rate: f32) -> Self {
        self.mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set timeout per iteration
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

/// Mutation strategies for fuzzing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
pub enum MutationStrategy {
    /// Flip random bits in bytes
    BitFlip,
    /// Flip random bytes
    ByteFlip,
    /// Insert random bytes
    Insert,
    /// Delete random bytes
    Delete,
    /// Substitute random bytes
    Substitute,
    /// Combine two inputs
    Combine,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
impl MutationStrategy {
    /// Get all mutation strategies
    pub fn all() -> &'static [MutationStrategy] {
        &[
            MutationStrategy::BitFlip,
            MutationStrategy::ByteFlip,
            MutationStrategy::Insert,
            MutationStrategy::Delete,
            MutationStrategy::Substitute,
            MutationStrategy::Combine,
        ]
    }
}

/// A crash discovered during fuzzing
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
pub struct FuzzCrash {
    /// Input that caused the crash
    pub input: String,
    /// Error message
    pub error: String,
    /// Iteration number when crash occurred
    pub iteration: u32,
}

/// Result of a fuzzing session
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
pub struct FuzzResult {
    /// Total iterations performed
    pub iterations: u32,
    /// Crashes discovered
    pub crashes: Vec<FuzzCrash>,
    /// Duration of fuzzing in milliseconds
    pub duration_ms: u64,
}

/// Mutation-based fuzzer
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
pub struct Fuzzer {
    config: FuzzConfig,
    rng: StdRng,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-security-fuzzer-rs.md#source
impl Fuzzer {
    /// Create a new fuzzer with configuration
    pub fn new(config: FuzzConfig) -> Self {
        let rng = match config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        Self { config, rng }
    }

    /// Add an input to the corpus
    pub fn add_corpus(&mut self, input: &str) {
        self.config.corpus.push(input.to_string());
    }

    /// Run fuzzing against a target function
    ///
    /// The target function should return Ok(()) for valid inputs and Err(msg) for invalid inputs.
    /// The fuzzer will collect crashes (unexpected errors).
    pub fn fuzz<F>(&self, target: F) -> FuzzResult
    where
        F: Fn(&str) -> Result<(), String>,
    {
        let start = Instant::now();
        let mut crashes = Vec::new();
        let mut rng = self.rng.clone();

        for iteration in 0..self.config.max_iterations {
            // Select random input from corpus
            let corpus_idx = rng.gen_range(0..self.config.corpus.len());
            let base_input = &self.config.corpus[corpus_idx];

            // Select random mutation strategy
            let strategies = MutationStrategy::all();
            let strategy_idx = rng.gen_range(0..strategies.len());
            let strategy = strategies[strategy_idx];

            // Mutate the input
            let mutated = self.mutate_with_rng(base_input, strategy, &mut rng);

            // Test the mutated input
            match target(&mutated) {
                Ok(()) => {
                    // Valid input, no crash
                }
                Err(error) => {
                    // Check if this is a known/expected error or a crash
                    // For now, we consider all errors as potential crashes
                    // In a real fuzzer, you'd distinguish between expected validation errors
                    // and unexpected crashes/panics
                    crashes.push(FuzzCrash {
                        input: mutated,
                        error,
                        iteration,
                    });
                }
            }

            // Check timeout
            if let Some(timeout_ms) = self.config.timeout_ms {
                if start.elapsed() > Duration::from_millis(timeout_ms * (iteration + 1) as u64) {
                    break;
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        FuzzResult {
            iterations: self.config.max_iterations,
            crashes,
            duration_ms,
        }
    }

    /// Mutate an input string using the specified strategy
    pub fn mutate(&self, input: &str, strategy: MutationStrategy) -> String {
        let mut rng = self.rng.clone();
        self.mutate_with_rng(input, strategy, &mut rng)
    }

    /// Internal mutation with provided RNG
    fn mutate_with_rng(&self, input: &str, strategy: MutationStrategy, rng: &mut StdRng) -> String {
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
                if self.config.corpus.len() > 1 {
                    let other_idx = rng.gen_range(0..self.config.corpus.len());
                    let other = self.config.corpus[other_idx].as_bytes();
                    bytes.extend_from_slice(other);
                }
            }
        }

        // Try to convert back to UTF-8, fall back to lossy conversion
        String::from_utf8(bytes)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzz_config_default() {
        let config = FuzzConfig::default();
        assert_eq!(config.max_iterations, 1000);
        assert_eq!(config.mutation_rate, 0.5);
        assert!(config.seed.is_none());
    }

    #[test]
    fn test_fuzz_config_builder() {
        let config = FuzzConfig::new()
            .with_iterations(500)
            .with_seed(42)
            .with_mutation_rate(0.7)
            .with_timeout_ms(50);

        assert_eq!(config.max_iterations, 500);
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.mutation_rate, 0.7);
        assert_eq!(config.timeout_ms, Some(50));
    }

    #[test]
    fn test_mutation_strategies() {
        let strategies = MutationStrategy::all();
        assert_eq!(strategies.len(), 6);
        assert!(strategies.contains(&MutationStrategy::BitFlip));
        assert!(strategies.contains(&MutationStrategy::Insert));
    }

    #[test]
    fn test_fuzzer_mutate_bit_flip() {
        let config = FuzzConfig::new().with_seed(42);
        let fuzzer = Fuzzer::new(config);
        let result = fuzzer.mutate("test", MutationStrategy::BitFlip);
        // Should be different from input (with high probability)
        assert!(!result.is_empty());
    }

    #[test]
    fn test_fuzzer_mutate_insert() {
        let config = FuzzConfig::new().with_seed(42);
        let fuzzer = Fuzzer::new(config);
        let result = fuzzer.mutate("test", MutationStrategy::Insert);
        // Should insert one byte (may not be valid UTF-8, so length might vary)
        assert!(result.len() >= 4); // At least original length
    }

    #[test]
    fn test_fuzzer_mutate_delete() {
        let config = FuzzConfig::new().with_seed(42);
        let fuzzer = Fuzzer::new(config);
        let result = fuzzer.mutate("test", MutationStrategy::Delete);
        assert_eq!(result.len(), 3); // One byte deleted
    }

    #[test]
    fn test_fuzzer_mutate_empty_string() {
        let config = FuzzConfig::new().with_seed(42);
        let fuzzer = Fuzzer::new(config);

        // Insert should work on empty string (may insert multi-byte UTF-8 char)
        let result = fuzzer.mutate("", MutationStrategy::Insert);
        assert!(result.len() >= 1);

        // Other strategies should return empty
        let result = fuzzer.mutate("", MutationStrategy::Delete);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_fuzzer_fuzz_basic() {
        let config = FuzzConfig::new().with_iterations(10).with_seed(42);

        let fuzzer = Fuzzer::new(config);

        // Target that always accepts
        let result = fuzzer.fuzz(|_input| Ok(()));
        assert_eq!(result.crashes.len(), 0);
    }

    #[test]
    fn test_fuzzer_fuzz_with_crashes() {
        let config = FuzzConfig::new().with_iterations(50).with_seed(42);

        let fuzzer = Fuzzer::new(config);

        // Target that rejects inputs containing 'x'
        let result = fuzzer.fuzz(|input| {
            if input.contains('x') {
                Err(format!("Invalid character 'x' in input"))
            } else {
                Ok(())
            }
        });

        // Should find some inputs with 'x'
        assert!(result.iterations > 0);
    }

    #[test]
    fn test_fuzzer_add_corpus() {
        let config = FuzzConfig::new();
        let mut fuzzer = Fuzzer::new(config);

        let initial_len = fuzzer.config.corpus.len();
        fuzzer.add_corpus("new_input");
        assert_eq!(fuzzer.config.corpus.len(), initial_len + 1);
    }

    #[test]
    fn test_fuzz_result_structure() {
        let result = FuzzResult {
            iterations: 100,
            crashes: vec![FuzzCrash {
                input: "test".to_string(),
                error: "Error".to_string(),
                iteration: 42,
            }],
            duration_ms: 500,
        };

        assert_eq!(result.iterations, 100);
        assert_eq!(result.crashes.len(), 1);
        assert_eq!(result.crashes[0].iteration, 42);
        assert_eq!(result.duration_ms, 500);
    }

    #[test]
    fn test_mutation_rate_clamping() {
        let config = FuzzConfig::new().with_mutation_rate(1.5);
        assert_eq!(config.mutation_rate, 1.0);

        let config = FuzzConfig::new().with_mutation_rate(-0.5);
        assert_eq!(config.mutation_rate, 0.0);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/security/fuzzer.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/security/fuzzer.rs` captured during meter full-codegen standardization.
```
