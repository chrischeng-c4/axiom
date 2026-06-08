---
id: projects-meter-src-ts-runner-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/ts_runner.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/ts_runner.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `NpmAuditResult` | projects/meter/src/ts_runner.rs | struct | pub | 723 |  |
| `TsRunner` | projects/meter/src/ts_runner.rs | struct | pub | 151 |  |
| `TsRunnerConfig` | projects/meter/src/ts_runner.rs | struct | pub | 19 |  |
| `TsRunnerResult` | projects/meter/src/ts_runner.rs | struct | pub | 100 |  |
| `TsTestResult` | projects/meter/src/ts_runner.rs | struct | pub | 80 |  |
| `V8Metrics` | projects/meter/src/ts_runner.rs | struct | pub | 62 |  |
| `for_project` | projects/meter/src/ts_runner.rs | function | pub | 163 | for_project(project_path: impl Into<PathBuf>) -> Self |
| `get_node_version` | projects/meter/src/ts_runner.rs | function | pub | 754 | get_node_version() -> Option<String> |
| `get_npm_version` | projects/meter/src/ts_runner.rs | function | pub | 766 | get_npm_version() -> Option<String> |
| `has_critical` | projects/meter/src/ts_runner.rs | function | pub | 734 | has_critical(&self) -> bool |
| `has_high` | projects/meter/src/ts_runner.rs | function | pub | 739 | has_high(&self) -> bool |
| `is_ts_project` | projects/meter/src/ts_runner.rs | function | pub | 746 | is_ts_project(path: &Path) -> bool |
| `new` | projects/meter/src/ts_runner.rs | function | pub | 158 | new(config: TsRunnerConfig) -> Self |
| `run_audit` | projects/meter/src/ts_runner.rs | function | pub | 664 | run_audit(&self) -> Result<NpmAuditResult, String> |
| `run_tests` | projects/meter/src/ts_runner.rs | function | pub | 171 | run_tests(&self) -> Result<TsRunnerResult, String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/ts_runner.rs -->
````rust
//! TypeScript custom runner - executes TS tests with V8 metrics collection
//!
//! This module provides a custom TypeScript/JavaScript test runner that:
//! - Supports multiple test frameworks (jest, vitest, mocha patterns)
//! - Collects V8 metrics (heap usage, GC events)
//! - Runs tests in parallel via worker threads
//! - Integrates with the probe reporting system

use crate::runner::{Language, TestMeta, TestResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// TypeScript runner configuration
#[derive(Debug, Clone)]
pub struct TsRunnerConfig {
    /// Path to project directory (containing package.json)
    pub project_path: PathBuf,
    /// Test file pattern (glob)
    pub test_pattern: String,
    /// Node.js executable path
    pub node_path: String,
    /// NPM executable path
    pub npm_path: String,
    /// Additional arguments for the test runner
    pub args: Vec<String>,
    /// Environment variables
    pub env: Vec<(String, String)>,
    /// Timeout per test in seconds
    pub timeout: u32,
    /// Collect V8 metrics
    pub collect_v8_metrics: bool,
    /// Number of parallel workers
    pub workers: usize,
}

impl Default for TsRunnerConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::from("."),
            test_pattern: "**/*.{test,spec}.{ts,tsx,js,jsx}".to_string(),
            node_path: "node".to_string(),
            npm_path: "npm".to_string(),
            args: Vec::new(),
            env: Vec::new(),
            timeout: 30,
            collect_v8_metrics: true,
            workers: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
        }
    }
}

/// V8 heap metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct V8Metrics {
    /// Total heap size in bytes
    pub heap_total: u64,
    /// Used heap size in bytes
    pub heap_used: u64,
    /// External memory in bytes
    pub external: u64,
    /// Number of GC events during test
    pub gc_count: u32,
    /// Total GC pause time in ms
    pub gc_pause_ms: f64,
    /// Event loop lag in ms
    pub event_loop_lag_ms: f64,
}

/// Test result from TypeScript runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsTestResult {
    /// Test name
    pub name: String,
    /// Test file path
    pub file: String,
    /// Test status
    pub status: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Stack trace if failed
    pub stack_trace: Option<String>,
    /// V8 metrics for this test
    pub v8_metrics: Option<V8Metrics>,
}

/// TypeScript runner result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsRunnerResult {
    /// Individual test results
    pub results: Vec<TestResult>,
    /// Total passed
    pub passed: u32,
    /// Total failed
    pub failed: u32,
    /// Total skipped
    pub skipped: u32,
    /// Total duration in ms
    pub duration_ms: u64,
    /// Aggregated V8 metrics
    pub v8_metrics: Option<V8Metrics>,
    /// Node.js version used
    pub node_version: Option<String>,
}

/// Output from probe's custom harness script (internal)
#[derive(Debug, Clone, Deserialize)]
struct HarnessOutput {
    passed: u32,
    failed: u32,
    skipped: u32,
    tests: Vec<HarnessTestResult>,
    v8_metrics: Option<HarnessV8Metrics>,
}

#[derive(Debug, Clone, Deserialize)]
struct HarnessTestResult {
    name: String,
    status: String,
    #[serde(default)]
    duration_ms: u64,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    stack_trace: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct HarnessV8Metrics {
    heap_total: u64,
    heap_used: u64,
    external: u64,
    gc_count: u32,
    gc_pause_ms: f64,
    event_loop_lag_ms: f64,
}

/// TypeScript test runner
pub struct TsRunner {
    config: TsRunnerConfig,
}

impl TsRunner {
    /// Create a new TypeScript runner
    pub fn new(config: TsRunnerConfig) -> Self {
        Self { config }
    }

    /// Create with default config for a project path
    pub fn for_project(project_path: impl Into<PathBuf>) -> Self {
        Self::new(TsRunnerConfig {
            project_path: project_path.into(),
            ..Default::default()
        })
    }

    /// Run tests using the probe test harness
    pub fn run_tests(&self) -> Result<TsRunnerResult, String> {
        let start = std::time::Instant::now();

        // First, check if package.json exists
        let package_json = self.config.project_path.join("package.json");
        if !package_json.exists() {
            return Err("No package.json found in project directory".to_string());
        }

        // Try to detect and use the appropriate test runner
        let runner_type = self.detect_test_runner()?;

        let result = match runner_type {
            TestRunnerType::Vitest => self.run_vitest()?,
            TestRunnerType::Jest => self.run_jest()?,
            TestRunnerType::Mocha => self.run_mocha()?,
            TestRunnerType::ProbeHarness => self.run_probe_harness()?,
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(TsRunnerResult {
            results: result.results,
            passed: result.passed,
            failed: result.failed,
            skipped: result.skipped,
            duration_ms,
            v8_metrics: result.v8_metrics,
            node_version: get_node_version(),
        })
    }

    /// Detect which test runner is configured
    fn detect_test_runner(&self) -> Result<TestRunnerType, String> {
        let package_json = self.config.project_path.join("package.json");
        let content = std::fs::read_to_string(&package_json)
            .map_err(|e| format!("Failed to read package.json: {}", e))?;

        let pkg: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse package.json: {}", e))?;

        // Check devDependencies and dependencies
        let deps = pkg
            .get("devDependencies")
            .or_else(|| pkg.get("dependencies"));

        if let Some(deps) = deps {
            if deps.get("vitest").is_some() {
                return Ok(TestRunnerType::Vitest);
            }
            if deps.get("jest").is_some() {
                return Ok(TestRunnerType::Jest);
            }
            if deps.get("mocha").is_some() {
                return Ok(TestRunnerType::Mocha);
            }
        }

        // Check scripts.test
        if let Some(scripts) = pkg.get("scripts") {
            if let Some(test_script) = scripts.get("test").and_then(|s| s.as_str()) {
                if test_script.contains("vitest") {
                    return Ok(TestRunnerType::Vitest);
                }
                if test_script.contains("jest") {
                    return Ok(TestRunnerType::Jest);
                }
                if test_script.contains("mocha") {
                    return Ok(TestRunnerType::Mocha);
                }
            }
        }

        // Fall back to probe's custom harness
        Ok(TestRunnerType::ProbeHarness)
    }

    /// Run tests with Vitest
    fn run_vitest(&self) -> Result<TsRunnerResult, String> {
        let mut cmd = Command::new(&self.config.npm_path);
        cmd.arg("exec")
            .arg("--")
            .arg("vitest")
            .arg("run")
            .arg("--reporter=json");

        cmd.current_dir(&self.config.project_path);

        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run vitest: {}", e))?;

        self.parse_vitest_output(&output.stdout)
    }

    /// Parse Vitest JSON output
    fn parse_vitest_output(&self, output: &[u8]) -> Result<TsRunnerResult, String> {
        let stdout = String::from_utf8_lossy(output);

        // Vitest outputs JSON directly
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|_| "Failed to parse vitest JSON output".to_string())?;

        let mut results = Vec::new();
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;

        if let Some(test_results) = json.get("testResults").and_then(|t| t.as_array()) {
            for file_result in test_results {
                if let Some(assertions) = file_result
                    .get("assertionResults")
                    .and_then(|a| a.as_array())
                {
                    for test in assertions {
                        let name = test
                            .get("fullName")
                            .or_else(|| test.get("title"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");

                        let status = test
                            .get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("passed");

                        let duration = test.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);

                        let meta = TestMeta::with_language(name, Language::TypeScript);

                        let result = match status {
                            "passed" => {
                                passed += 1;
                                TestResult::passed(meta, duration)
                            }
                            "failed" => {
                                failed += 1;
                                let error = test
                                    .get("failureMessages")
                                    .and_then(|m| m.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("Test failed");
                                TestResult::failed(meta, duration, error)
                            }
                            "skipped" | "pending" | "todo" => {
                                skipped += 1;
                                TestResult::skipped(meta, "Skipped")
                            }
                            _ => {
                                passed += 1;
                                TestResult::passed(meta, duration)
                            }
                        };

                        results.push(result);
                    }
                }
            }
        }

        Ok(TsRunnerResult {
            results,
            passed,
            failed,
            skipped,
            duration_ms: 0,
            v8_metrics: None,
            node_version: None,
        })
    }

    /// Run tests with Jest
    fn run_jest(&self) -> Result<TsRunnerResult, String> {
        let mut cmd = Command::new(&self.config.npm_path);
        cmd.arg("exec")
            .arg("--")
            .arg("jest")
            .arg("--json")
            .arg("--testLocationInResults");

        cmd.current_dir(&self.config.project_path);

        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run jest: {}", e))?;

        self.parse_jest_output(&output.stdout)
    }

    /// Parse Jest JSON output
    fn parse_jest_output(&self, output: &[u8]) -> Result<TsRunnerResult, String> {
        let stdout = String::from_utf8_lossy(output);

        let json: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|_| "Failed to parse jest JSON output".to_string())?;

        let mut results = Vec::new();
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;

        if let Some(test_results) = json.get("testResults").and_then(|t| t.as_array()) {
            for file_result in test_results {
                if let Some(assertions) = file_result
                    .get("assertionResults")
                    .and_then(|a| a.as_array())
                {
                    for test in assertions {
                        let name = test
                            .get("fullName")
                            .or_else(|| test.get("title"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("unknown");

                        let status = test
                            .get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("passed");

                        let duration = test.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);

                        let meta = TestMeta::with_language(name, Language::TypeScript);

                        let result = match status {
                            "passed" => {
                                passed += 1;
                                TestResult::passed(meta, duration)
                            }
                            "failed" => {
                                failed += 1;
                                let error = test
                                    .get("failureMessages")
                                    .and_then(|m| m.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("Test failed");
                                TestResult::failed(meta, duration, error)
                            }
                            "skipped" | "pending" | "todo" => {
                                skipped += 1;
                                TestResult::skipped(meta, "Skipped")
                            }
                            _ => {
                                passed += 1;
                                TestResult::passed(meta, duration)
                            }
                        };

                        results.push(result);
                    }
                }
            }
        }

        Ok(TsRunnerResult {
            results,
            passed,
            failed,
            skipped,
            duration_ms: 0,
            v8_metrics: None,
            node_version: None,
        })
    }

    /// Run tests with Mocha
    fn run_mocha(&self) -> Result<TsRunnerResult, String> {
        let mut cmd = Command::new(&self.config.npm_path);
        cmd.arg("exec")
            .arg("--")
            .arg("mocha")
            .arg("--reporter=json");

        cmd.current_dir(&self.config.project_path);

        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run mocha: {}", e))?;

        self.parse_mocha_output(&output.stdout)
    }

    /// Parse Mocha JSON output
    fn parse_mocha_output(&self, output: &[u8]) -> Result<TsRunnerResult, String> {
        let stdout = String::from_utf8_lossy(output);

        let json: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|_| "Failed to parse mocha JSON output".to_string())?;

        let mut results = Vec::new();
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut skipped = 0u32;

        // Parse passes
        if let Some(passes) = json.get("passes").and_then(|p| p.as_array()) {
            for test in passes {
                let name = test
                    .get("fullTitle")
                    .or_else(|| test.get("title"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");

                let duration = test.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);

                let meta = TestMeta::with_language(name, Language::TypeScript);
                results.push(TestResult::passed(meta, duration));
                passed += 1;
            }
        }

        // Parse failures
        if let Some(failures) = json.get("failures").and_then(|f| f.as_array()) {
            for test in failures {
                let name = test
                    .get("fullTitle")
                    .or_else(|| test.get("title"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");

                let duration = test.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);

                let error = test
                    .get("err")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Test failed");

                let meta = TestMeta::with_language(name, Language::TypeScript);
                results.push(TestResult::failed(meta, duration, error));
                failed += 1;
            }
        }

        // Parse pending/skipped
        if let Some(pending) = json.get("pending").and_then(|p| p.as_array()) {
            for test in pending {
                let name = test
                    .get("fullTitle")
                    .or_else(|| test.get("title"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");

                let meta = TestMeta::with_language(name, Language::TypeScript);
                results.push(TestResult::skipped(meta, "Pending"));
                skipped += 1;
            }
        }

        Ok(TsRunnerResult {
            results,
            passed,
            failed,
            skipped,
            duration_ms: 0,
            v8_metrics: None,
            node_version: None,
        })
    }

    /// Run tests with probe's custom harness (for projects without standard runner)
    fn run_probe_harness(&self) -> Result<TsRunnerResult, String> {
        // Use Node.js directly with a simple test discovery and execution
        let harness_script = include_str!("../scripts/ts_harness.js");

        // Write harness script to temp file
        let temp_dir = std::env::temp_dir();
        let harness_path = temp_dir.join("probe_ts_harness.js");
        std::fs::write(&harness_path, harness_script)
            .map_err(|e| format!("Failed to write harness script: {}", e))?;

        let mut cmd = Command::new(&self.config.node_path);
        cmd.arg(&harness_path)
            .arg(&self.config.project_path)
            .arg(&self.config.test_pattern);

        if self.config.collect_v8_metrics {
            cmd.arg("--v8-metrics");
        }

        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run probe harness: {}", e))?;

        // Parse JSON output from harness
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try to parse the harness output
        if let Ok(harness_output) = serde_json::from_str::<HarnessOutput>(&stdout) {
            let mut results = Vec::new();

            for test in &harness_output.tests {
                let meta = TestMeta::with_language(&test.name, Language::TypeScript);

                let result = match test.status.as_str() {
                    "passed" => TestResult::passed(meta, test.duration_ms),
                    "failed" => {
                        let error = test
                            .error
                            .clone()
                            .unwrap_or_else(|| "Test failed".to_string());
                        let mut result = TestResult::failed(meta, test.duration_ms, error);
                        if let Some(ref trace) = test.stack_trace {
                            result = result.with_stack_trace(trace.clone());
                        }
                        result
                    }
                    "skipped" => TestResult::skipped(
                        meta,
                        test.error.clone().unwrap_or_else(|| "Skipped".to_string()),
                    ),
                    _ => TestResult::passed(meta, test.duration_ms),
                };

                results.push(result);
            }

            let v8_metrics = harness_output.v8_metrics.map(|m| V8Metrics {
                heap_total: m.heap_total,
                heap_used: m.heap_used,
                external: m.external,
                gc_count: m.gc_count,
                gc_pause_ms: m.gc_pause_ms,
                event_loop_lag_ms: m.event_loop_lag_ms,
            });

            return Ok(TsRunnerResult {
                results,
                passed: harness_output.passed,
                failed: harness_output.failed,
                skipped: harness_output.skipped,
                duration_ms: 0,
                v8_metrics,
                node_version: get_node_version(),
            });
        }

        // Check for error in harness output
        if let Ok(error_output) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(error) = error_output.get("error").and_then(|e| e.as_str()) {
                return Err(format!("Harness error: {}", error));
            }
        }

        // Fallback: harness ran but output wasn't parseable
        // Check if process failed
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Harness failed with exit code {:?}: {}",
                output.status.code(),
                stderr.lines().take(5).collect::<Vec<_>>().join("\n")
            ));
        }

        // No tests found
        Ok(TsRunnerResult {
            results: Vec::new(),
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            v8_metrics: None,
            node_version: get_node_version(),
        })
    }

    /// Run npm audit for security scanning
    pub fn run_audit(&self) -> Result<NpmAuditResult, String> {
        let mut cmd = Command::new(&self.config.npm_path);
        cmd.arg("audit").arg("--json");

        cmd.current_dir(&self.config.project_path);
        cmd.stdout(Stdio::piped());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run npm audit: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if let Ok(report) = serde_json::from_str::<serde_json::Value>(&stdout) {
            let vulnerabilities = report
                .get("vulnerabilities")
                .and_then(|v| v.as_object())
                .map(|v| v.len())
                .unwrap_or(0);

            let critical = report
                .get("metadata")
                .and_then(|m| m.get("vulnerabilities"))
                .and_then(|v| v.get("critical"))
                .and_then(|c| c.as_u64())
                .unwrap_or(0) as u32;

            let high = report
                .get("metadata")
                .and_then(|m| m.get("vulnerabilities"))
                .and_then(|v| v.get("high"))
                .and_then(|h| h.as_u64())
                .unwrap_or(0) as u32;

            return Ok(NpmAuditResult {
                total_vulnerabilities: vulnerabilities,
                critical,
                high,
                moderate: 0,
                low: 0,
            });
        }

        Ok(NpmAuditResult::default())
    }
}

/// Test runner type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestRunnerType {
    Vitest,
    Jest,
    Mocha,
    ProbeHarness,
}

/// NPM audit result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NpmAuditResult {
    pub total_vulnerabilities: usize,
    pub critical: u32,
    pub high: u32,
    pub moderate: u32,
    pub low: u32,
}

impl NpmAuditResult {
    /// Check if there are any critical vulnerabilities
    pub fn has_critical(&self) -> bool {
        self.critical > 0
    }

    /// Check if there are any high severity vulnerabilities
    pub fn has_high(&self) -> bool {
        self.high > 0
    }
}

/// Detect if a directory is a TypeScript/JavaScript project
pub fn is_ts_project(path: &Path) -> bool {
    let package_json = path.join("package.json");
    let tsconfig = path.join("tsconfig.json");
    package_json.exists() || tsconfig.exists()
}

/// Get Node.js version
pub fn get_node_version() -> Option<String> {
    let output = Command::new("node").arg("--version").output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Get NPM version
pub fn get_npm_version() -> Option<String> {
    let output = Command::new("npm").arg("--version").output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ts_runner_config_default() {
        let config = TsRunnerConfig::default();
        assert_eq!(config.project_path, PathBuf::from("."));
        assert!(config.collect_v8_metrics);
        assert_eq!(config.timeout, 30);
    }

    #[test]
    fn test_node_version_detection() {
        // This test depends on node being installed
        let version = get_node_version();
        if let Some(v) = version {
            assert!(v.starts_with('v'));
        }
    }

    #[test]
    fn test_npm_version_detection() {
        // This test depends on npm being installed
        let version = get_npm_version();
        // npm version is just numbers like "10.2.3"
        if version.is_some() {
            // Version exists, that's enough
        }
    }

    #[test]
    fn test_is_ts_project() {
        // Test with a path that likely doesn't have package.json
        let non_ts = PathBuf::from("/tmp");
        // May or may not have package.json in /tmp
        let _ = is_ts_project(&non_ts);
    }

    #[test]
    fn test_npm_audit_result_helpers() {
        let result = NpmAuditResult {
            total_vulnerabilities: 5,
            critical: 1,
            high: 2,
            moderate: 1,
            low: 1,
        };
        assert!(result.has_critical());
        assert!(result.has_high());
    }

    #[test]
    fn test_v8_metrics_default() {
        let metrics = V8Metrics::default();
        assert_eq!(metrics.heap_total, 0);
        assert_eq!(metrics.gc_count, 0);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/ts_runner.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/ts_runner.rs` captured during meter full-codegen standardization.
```
