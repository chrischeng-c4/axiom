---
id: projects-meter-src-runner-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/runner.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/runner.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Language` | projects/meter/src/runner.rs | enum | pub | 64 |  |
| `ProfileMetrics` | projects/meter/src/runner.rs | struct | pub | 187 |  |
| `RunnerConfig` | projects/meter/src/runner.rs | struct | pub | 298 |  |
| `TestMeta` | projects/meter/src/runner.rs | struct | pub | 88 |  |
| `TestResult` | projects/meter/src/runner.rs | struct | pub | 201 |  |
| `TestRunner` | projects/meter/src/runner.rs | struct | pub | 333 |  |
| `TestStatus` | projects/meter/src/runner.rs | enum | pub | 38 |  |
| `TestSummary` | projects/meter/src/runner.rs | struct | pub | 432 |  |
| `TestType` | projects/meter/src/runner.rs | enum | pub | 14 |  |
| `all_passed` | projects/meter/src/runner.rs | function | pub | 450 | all_passed(&self) -> bool |
| `config` | projects/meter/src/runner.rs | function | pub | 356 | config(&self) -> &RunnerConfig |
| `default_runner` | projects/meter/src/runner.rs | function | pub | 351 | default_runner() -> Self |
| `error` | projects/meter/src/runner.rs | function | pub | 260 | error(meta: TestMeta, duration_ms: u64, error: impl Into<String>) -> Self |
| `failed` | projects/meter/src/runner.rs | function | pub | 234 | failed(meta: TestMeta, duration_ms: u64, error: impl Into<String>) -> Self |
| `has_tag` | projects/meter/src/runner.rs | function | pub | 179 | has_tag(&self, tag: &str) -> bool |
| `is_failed` | projects/meter/src/runner.rs | function | pub | 290 | is_failed(&self) -> bool |
| `is_passed` | projects/meter/src/runner.rs | function | pub | 285 | is_passed(&self) -> bool |
| `is_skipped` | projects/meter/src/runner.rs | function | pub | 174 | is_skipped(&self) -> bool |
| `language` | projects/meter/src/runner.rs | function | pub | 150 | language(mut self, language: Language) -> Self |
| `new` | projects/meter/src/runner.rs | function | pub | 112 | new(name: impl Into<String>) -> Self |
| `new` | projects/meter/src/runner.rs | function | pub | 342 | new(config: RunnerConfig) -> Self |
| `pass_rate` | projects/meter/src/runner.rs | function | pub | 455 | pass_rate(&self) -> f64 |
| `passed` | projects/meter/src/runner.rs | function | pub | 221 | passed(meta: TestMeta, duration_ms: u64) -> Self |
| `record` | projects/meter/src/runner.rs | function | pub | 367 | record(&mut self, result: TestResult) |
| `results` | projects/meter/src/runner.rs | function | pub | 372 | results(&self) -> &[TestResult] |
| `run_tests_parallel` | projects/meter/src/runner.rs | function | pub | 477 | run_tests_parallel(results: Vec<TestResult>, config: RunnerConfig) -> Vec<TestResult> |
| `should_run` | projects/meter/src/runner.rs | function | pub | 402 | should_run(&self, meta: &TestMeta) -> bool |
| `skip` | projects/meter/src/runner.rs | function | pub | 168 | skip(mut self, reason: impl Into<String>) -> Self |
| `skipped` | projects/meter/src/runner.rs | function | pub | 247 | skipped(meta: TestMeta, reason: impl Into<String>) -> Self |
| `start` | projects/meter/src/runner.rs | function | pub | 361 | start(&mut self) |
| `summary` | projects/meter/src/runner.rs | function | pub | 384 | summary(&self) -> TestSummary |
| `total_duration` | projects/meter/src/runner.rs | function | pub | 377 | total_duration(&self) -> Duration |
| `with_language` | projects/meter/src/runner.rs | function | pub | 128 | with_language(name: impl Into<String>, language: Language) -> Self |
| `with_profile_metrics` | projects/meter/src/runner.rs | function | pub | 279 | with_profile_metrics(mut self, metrics: ProfileMetrics) -> Self |
| `with_stack_trace` | projects/meter/src/runner.rs | function | pub | 273 | with_stack_trace(mut self, trace: impl Into<String>) -> Self |
| `with_tags` | projects/meter/src/runner.rs | function | pub | 162 | with_tags(mut self, tags: Vec<String>) -> Self |
| `with_timeout` | projects/meter/src/runner.rs | function | pub | 156 | with_timeout(mut self, timeout: f64) -> Self |
| `with_type` | projects/meter/src/runner.rs | function | pub | 144 | with_type(mut self, test_type: TestType) -> Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Test runner - discovery, execution, and scheduling

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;

/// Test type categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub enum TestType {
    /// Standard unit test
    #[default]
    Unit,
    /// Performance profiling test
    Profile,
    /// Security/fuzzing test
    Security,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestType::Unit => write!(f, "unit"),
            TestType::Profile => write!(f, "profile"),
            TestType::Security => write!(f, "security"),
        }
    }
}

/// Test execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed (assertion failed)
    Failed,
    /// Test was skipped
    Skipped,
    /// Test encountered an error (exception, timeout, etc.)
    Error,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStatus::Passed => write!(f, "PASSED"),
            TestStatus::Failed => write!(f, "FAILED"),
            TestStatus::Skipped => write!(f, "SKIPPED"),
            TestStatus::Error => write!(f, "ERROR"),
        }
    }
}

/// Programming language for multi-language test support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub enum Language {
    /// Python (default)
    #[default]
    Python,
    /// Rust
    Rust,
    /// TypeScript/JavaScript
    TypeScript,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "python"),
            Language::Rust => write!(f, "rust"),
            Language::TypeScript => write!(f, "typescript"),
        }
    }
}

/// Metadata for a test function
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub struct TestMeta {
    /// Test function name
    pub name: String,
    /// Full qualified name (module.class.method)
    pub full_name: String,
    /// Test type
    pub test_type: TestType,
    /// Programming language
    pub language: Language,
    /// Timeout in seconds
    pub timeout: Option<f64>,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Skip reason (if skipped)
    pub skip_reason: Option<String>,
    /// Source file path
    pub file_path: Option<String>,
    /// Line number in source
    pub line_number: Option<u32>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl TestMeta {
    /// Create new test metadata
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            full_name: name.clone(),
            name,
            test_type: TestType::Unit,
            language: Language::default(),
            timeout: None,
            tags: Vec::new(),
            skip_reason: None,
            file_path: None,
            line_number: None,
        }
    }

    /// Create test metadata with a specific language
    pub fn with_language(name: impl Into<String>, language: Language) -> Self {
        let name = name.into();
        Self {
            full_name: name.clone(),
            name,
            test_type: TestType::Unit,
            language,
            timeout: None,
            tags: Vec::new(),
            skip_reason: None,
            file_path: None,
            line_number: None,
        }
    }

    /// Set test type
    pub fn with_type(mut self, test_type: TestType) -> Self {
        self.test_type = test_type;
        self
    }

    /// Set language (builder pattern)
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: f64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set skip reason
    pub fn skip(mut self, reason: impl Into<String>) -> Self {
        self.skip_reason = Some(reason.into());
        self
    }

    /// Check if test should be skipped
    pub fn is_skipped(&self) -> bool {
        self.skip_reason.is_some()
    }

    /// Check if test has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Profile metrics for performance tests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub struct ProfileMetrics {
    /// Number of iterations run
    pub iterations: u32,
    /// Average CPU time per iteration (ms)
    pub avg_cpu_time_ms: f64,
    /// Peak memory usage (bytes)
    pub peak_memory_bytes: u64,
    /// Rust-Python boundary overhead (ms)
    pub boundary_overhead_ms: f64,
}

/// Test result after execution
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub struct TestResult {
    /// Test metadata
    pub meta: TestMeta,
    /// Execution status
    pub status: TestStatus,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message (if failed or error)
    pub error: Option<String>,
    /// Stack trace (if available)
    pub stack_trace: Option<String>,
    /// Profile metrics (for profile tests)
    pub profile_metrics: Option<ProfileMetrics>,
    /// Timestamp when test started
    pub started_at: String,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl TestResult {
    /// Create a passed test result
    pub fn passed(meta: TestMeta, duration_ms: u64) -> Self {
        Self {
            meta,
            status: TestStatus::Passed,
            duration_ms,
            error: None,
            stack_trace: None,
            profile_metrics: None,
            started_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a failed test result
    pub fn failed(meta: TestMeta, duration_ms: u64, error: impl Into<String>) -> Self {
        Self {
            meta,
            status: TestStatus::Failed,
            duration_ms,
            error: Some(error.into()),
            stack_trace: None,
            profile_metrics: None,
            started_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a skipped test result
    pub fn skipped(meta: TestMeta, reason: impl Into<String>) -> Self {
        Self {
            meta,
            status: TestStatus::Skipped,
            duration_ms: 0,
            error: Some(reason.into()),
            stack_trace: None,
            profile_metrics: None,
            started_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create an error test result
    pub fn error(meta: TestMeta, duration_ms: u64, error: impl Into<String>) -> Self {
        Self {
            meta,
            status: TestStatus::Error,
            duration_ms,
            error: Some(error.into()),
            stack_trace: None,
            profile_metrics: None,
            started_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add stack trace
    pub fn with_stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    /// Add profile metrics
    pub fn with_profile_metrics(mut self, metrics: ProfileMetrics) -> Self {
        self.profile_metrics = Some(metrics);
        self
    }

    /// Check if test passed
    pub fn is_passed(&self) -> bool {
        self.status == TestStatus::Passed
    }

    /// Check if test failed
    pub fn is_failed(&self) -> bool {
        self.status == TestStatus::Failed
    }
}

/// Test runner configuration
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub struct RunnerConfig {
    /// Filter by test type
    pub test_type: Option<TestType>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Filter by name pattern
    pub name_pattern: Option<String>,
    /// Fail fast (stop on first failure)
    pub fail_fast: bool,
    /// Verbose output
    pub verbose: bool,
    /// Parallel execution
    pub parallel: bool,
    /// Max parallel workers
    pub max_workers: usize,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            test_type: None,
            tags: Vec::new(),
            name_pattern: None,
            fail_fast: false,
            verbose: false,
            parallel: false,
            max_workers: 4,
        }
    }
}

/// Test runner - orchestrates test discovery and execution
#[derive(Debug)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub struct TestRunner {
    config: RunnerConfig,
    results: Vec<TestResult>,
    start_time: Option<Instant>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl TestRunner {
    /// Create a new test runner
    pub fn new(config: RunnerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            start_time: None,
        }
    }

    /// Create with default config
    pub fn default_runner() -> Self {
        Self::new(RunnerConfig::default())
    }

    /// Get a reference to the config
    pub fn config(&self) -> &RunnerConfig {
        &self.config
    }

    /// Start the test run
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.results.clear();
    }

    /// Record a test result
    pub fn record(&mut self, result: TestResult) {
        self.results.push(result);
    }

    /// Get all results
    pub fn results(&self) -> &[TestResult] {
        &self.results
    }

    /// Get total duration
    pub fn total_duration(&self) -> Duration {
        self.start_time
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Get summary counts
    pub fn summary(&self) -> TestSummary {
        let mut summary = TestSummary::default();

        for result in &self.results {
            match result.status {
                TestStatus::Passed => summary.passed += 1,
                TestStatus::Failed => summary.failed += 1,
                TestStatus::Skipped => summary.skipped += 1,
                TestStatus::Error => summary.errors += 1,
            }
            summary.total_duration_ms += result.duration_ms;
        }

        summary.total = self.results.len();
        summary
    }

    /// Check if test should run based on filters
    pub fn should_run(&self, meta: &TestMeta) -> bool {
        // Check test type filter
        if let Some(filter_type) = self.config.test_type {
            if meta.test_type != filter_type {
                return false;
            }
        }

        // Check tag filter
        if !self.config.tags.is_empty() {
            let has_matching_tag = self.config.tags.iter().any(|t| meta.has_tag(t));
            if !has_matching_tag {
                return false;
            }
        }

        // Check name pattern
        if let Some(ref pattern) = self.config.name_pattern {
            if !meta.name.contains(pattern) && !meta.full_name.contains(pattern) {
                return false;
            }
        }

        true
    }
}

/// Summary of test results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub struct TestSummary {
    /// Total tests
    pub total: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Skipped tests
    pub skipped: usize,
    /// Error tests
    pub errors: usize,
    /// Total duration in ms
    pub total_duration_ms: u64,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
impl TestSummary {
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.errors == 0
    }

    /// Get pass rate (0.0 - 1.0)
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            return 1.0;
        }
        let denominator = self.total - self.skipped;
        if denominator == 0 {
            // All tests were skipped - consider as 100% pass rate
            return 1.0;
        }
        self.passed as f64 / denominator as f64
    }
}

// ============================================================================
// Parallel Test Execution with Tokio
// ============================================================================

/// Execute test results in parallel using Tokio
///
/// This function spawns concurrent tasks to process test results with a semaphore
/// limiting max concurrency.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-runner-rs.md#source
pub async fn run_tests_parallel(results: Vec<TestResult>, config: RunnerConfig) -> Vec<TestResult> {
    if !config.parallel || results.is_empty() {
        // Sequential execution
        return results;
    }

    // Create semaphore to limit concurrency
    let sem = Arc::new(Semaphore::new(config.max_workers));

    // Spawn tasks for each result
    let mut tasks: Vec<JoinHandle<TestResult>> = Vec::new();

    for result in results {
        let sem_clone = sem.clone();

        let task = tokio::spawn(async move {
            // Acquire permit (blocks if at max_workers)
            let _permit = sem_clone.acquire().await.unwrap();

            // Return the result (actual test execution happens in  layer)
            result
        });

        tasks.push(task);
    }

    // Collect results
    if config.fail_fast {
        collect_with_fail_fast(tasks).await
    } else {
        futures::future::join_all(tasks)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect()
    }
}

/// Collect results with fail-fast support
///
/// Stops collecting and returns as soon as the first failure is detected
async fn collect_with_fail_fast(tasks: Vec<JoinHandle<TestResult>>) -> Vec<TestResult> {
    let (tx, mut rx) = mpsc::channel(1);

    // Spawn monitor tasks
    for task in tasks {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let result = task.await.unwrap();
            let _ = tx_clone.send(result).await;
        });
    }

    drop(tx); // Close sender

    let mut results = Vec::new();
    while let Some(result) = rx.recv().await {
        if !result.is_passed() {
            // First failure: return immediately
            results.push(result);
            return results;
        }
        results.push(result);
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_creation() {
        let meta = TestMeta::new("test_example")
            .with_type(TestType::Unit)
            .with_timeout(5.0)
            .with_tags(vec!["unit".to_string(), "fast".to_string()]);

        assert_eq!(meta.name, "test_example");
        assert_eq!(meta.test_type, TestType::Unit);
        assert_eq!(meta.timeout, Some(5.0));
        assert!(meta.has_tag("unit"));
        assert!(meta.has_tag("fast"));
        assert!(!meta.has_tag("slow"));
    }

    #[test]
    fn test_result_creation() {
        let meta = TestMeta::new("test_pass");
        let result = TestResult::passed(meta, 100);

        assert!(result.is_passed());
        assert_eq!(result.duration_ms, 100);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_runner_summary() {
        let mut runner = TestRunner::default_runner();
        runner.start();

        runner.record(TestResult::passed(TestMeta::new("test1"), 10));
        runner.record(TestResult::passed(TestMeta::new("test2"), 20));
        runner.record(TestResult::failed(
            TestMeta::new("test3"),
            30,
            "assertion failed",
        ));
        runner.record(TestResult::skipped(
            TestMeta::new("test4"),
            "not implemented",
        ));

        let summary = runner.summary();
        assert_eq!(summary.total, 4);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 1);
        assert!(!summary.all_passed());
        // pass_rate excludes skipped: 2 passed / 3 non-skipped = 0.666...
        assert!((summary.pass_rate() - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_pass_rate_edge_cases() {
        // All skipped - should not panic (divide by zero bug fix)
        let all_skipped = TestSummary {
            total: 5,
            passed: 0,
            failed: 0,
            errors: 0,
            skipped: 5,
            total_duration_ms: 0,
        };
        assert_eq!(all_skipped.pass_rate(), 1.0);

        // Empty - no tests
        let empty = TestSummary {
            total: 0,
            passed: 0,
            failed: 0,
            errors: 0,
            skipped: 0,
            total_duration_ms: 0,
        };
        assert_eq!(empty.pass_rate(), 1.0);

        // All passed (no skipped)
        let all_passed = TestSummary {
            total: 3,
            passed: 3,
            failed: 0,
            errors: 0,
            skipped: 0,
            total_duration_ms: 100,
        };
        assert_eq!(all_passed.pass_rate(), 1.0);

        // Mixed with some skipped
        let mixed = TestSummary {
            total: 10,
            passed: 6,
            failed: 2,
            errors: 0,
            skipped: 2,
            total_duration_ms: 500,
        };
        // 6 passed / 8 non-skipped = 0.75
        assert_eq!(mixed.pass_rate(), 0.75);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/runner.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/runner.rs` captured during meter full-codegen standardization.
```
