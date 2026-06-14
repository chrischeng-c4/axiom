---
id: projects-meter-src-rust-runner-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/rust_runner.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/rust_runner.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AuditResult` | projects/meter/src/rust_runner.rs | struct | pub | 601 |  |
| `AuditWarning` | projects/meter/src/rust_runner.rs | struct | pub | 592 |  |
| `BenchEvent` | projects/meter/src/rust_runner.rs | struct | pub | 64 |  |
| `CargoTestEvent` | projects/meter/src/rust_runner.rs | enum | pub | 20 |  |
| `Package` | projects/meter/src/rust_runner.rs | struct | pub | 584 |  |
| `RustRunner` | projects/meter/src/rust_runner.rs | struct | pub | 124 |  |
| `RustRunnerConfig` | projects/meter/src/rust_runner.rs | struct | pub | 73 |  |
| `RustRunnerResult` | projects/meter/src/rust_runner.rs | struct | pub | 105 |  |
| `SuiteEvent` | projects/meter/src/rust_runner.rs | struct | pub | 35 |  |
| `TestEvent` | projects/meter/src/rust_runner.rs | struct | pub | 50 |  |
| `Vulnerability` | projects/meter/src/rust_runner.rs | struct | pub | 501 |  |
| `for_project` | projects/meter/src/rust_runner.rs | function | pub | 136 | for_project(project_path: impl Into<PathBuf>) -> Self |
| `get_cargo_version` | projects/meter/src/rust_runner.rs | function | pub | 649 | get_cargo_version() -> Option<String> |
| `get_rust_version` | projects/meter/src/rust_runner.rs | function | pub | 637 | get_rust_version() -> Option<String> |
| `has_critical` | projects/meter/src/rust_runner.rs | function | pub | 609 | has_critical(&self) -> bool |
| `has_vulnerabilities` | projects/meter/src/rust_runner.rs | function | pub | 619 | has_vulnerabilities(&self) -> bool |
| `is_rust_project` | projects/meter/src/rust_runner.rs | function | pub | 631 | is_rust_project(path: &Path) -> bool |
| `new` | projects/meter/src/rust_runner.rs | function | pub | 131 | new(config: RustRunnerConfig) -> Self |
| `run_audit` | projects/meter/src/rust_runner.rs | function | pub | 412 | run_audit(&self) -> Result<AuditResult, String> |
| `run_benchmarks` | projects/meter/src/rust_runner.rs | function | pub | 326 | run_benchmarks(&self) -> Result<RustRunnerResult, String> |
| `run_tests` | projects/meter/src/rust_runner.rs | function | pub | 144 | run_tests(&self) -> Result<RustRunnerResult, String> |
| `vulnerability_count` | projects/meter/src/rust_runner.rs | function | pub | 624 | vulnerability_count(&self) -> usize |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Rust runner integration - integrates with cargo test/bench/fuzz
//!
//! This module provides integration with the Rust toolchain for:
//! - Running tests via `cargo test --message-format=json`
//! - Running benchmarks via `cargo bench`
//! - Security scanning via `cargo-audit` and `cargo-fuzz`

use crate::runner::{Language, TestMeta, TestResult, TestStatus, TestType};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Rust test event from cargo test --message-format=json
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub enum CargoTestEvent {
    /// Test suite started
    #[serde(rename = "suite")]
    Suite(SuiteEvent),
    /// Individual test event
    #[serde(rename = "test")]
    Test(TestEvent),
    /// Benchmark result
    #[serde(rename = "bench")]
    Bench(BenchEvent),
}

/// Suite-level event
#[derive(Debug, Clone, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct SuiteEvent {
    pub event: String, // "started", "ok", "failed"
    #[serde(default)]
    pub test_count: Option<u32>,
    #[serde(default)]
    pub passed: Option<u32>,
    #[serde(default)]
    pub failed: Option<u32>,
    #[serde(default)]
    pub ignored: Option<u32>,
}

/// Individual test event
#[derive(Debug, Clone, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct TestEvent {
    pub event: String, // "started", "ok", "failed", "ignored"
    pub name: String,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stderr: Option<String>,
    #[serde(default)]
    pub exec_time: Option<f64>,
}

/// Benchmark event
#[derive(Debug, Clone, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct BenchEvent {
    pub name: String,
    pub median: u64,
    pub deviation: u64,
}

/// Cargo test runner configuration
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct RustRunnerConfig {
    /// Path to Cargo.toml or project directory
    pub project_path: PathBuf,
    /// Additional cargo test arguments
    pub args: Vec<String>,
    /// Test filter pattern
    pub filter: Option<String>,
    /// Run in release mode
    pub release: bool,
    /// Features to enable
    pub features: Vec<String>,
    /// Environment variables
    pub env: Vec<(String, String)>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
impl Default for RustRunnerConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::from("."),
            args: Vec::new(),
            filter: None,
            release: false,
            features: Vec::new(),
            env: Vec::new(),
        }
    }
}

/// Rust runner result
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct RustRunnerResult {
    /// Individual test results
    pub results: Vec<TestResult>,
    /// Total passed
    pub passed: u32,
    /// Total failed
    pub failed: u32,
    /// Total ignored/skipped
    pub ignored: u32,
    /// Total duration in ms
    pub duration_ms: u64,
    /// Compilation succeeded
    pub compiled: bool,
    /// Compilation error message (if any)
    pub compile_error: Option<String>,
}

/// Rust test runner
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct RustRunner {
    config: RustRunnerConfig,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
impl RustRunner {
    /// Create a new Rust runner
    pub fn new(config: RustRunnerConfig) -> Self {
        Self { config }
    }

    /// Create with default config for a project path
    pub fn for_project(project_path: impl Into<PathBuf>) -> Self {
        Self::new(RustRunnerConfig {
            project_path: project_path.into(),
            ..Default::default()
        })
    }

    /// Run cargo test and collect results
    pub fn run_tests(&self) -> Result<RustRunnerResult, String> {
        let start = std::time::Instant::now();

        // Build cargo test command
        let mut cmd = Command::new("cargo");
        cmd.arg("test")
            .arg("--message-format=json")
            .arg("--no-fail-fast");

        // Set working directory
        cmd.current_dir(&self.config.project_path);

        // Add filter if specified
        if let Some(ref filter) = self.config.filter {
            cmd.arg(filter);
        }

        // Add release flag
        if self.config.release {
            cmd.arg("--release");
        }

        // Add features
        if !self.config.features.is_empty() {
            cmd.arg("--features").arg(self.config.features.join(","));
        }

        // Add environment variables
        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        // Add additional args
        if !self.config.args.is_empty() {
            cmd.arg("--").args(&self.config.args);
        }

        // Run command and capture output
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn cargo test: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        // Spawn thread to drain stderr to prevent deadlock
        let stderr_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut stderr_content = String::new();
            for line in reader.lines() {
                if let Ok(line) = line {
                    stderr_content.push_str(&line);
                    stderr_content.push('\n');
                }
            }
            stderr_content
        });

        let reader = BufReader::new(stdout);
        let mut results = Vec::new();
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut ignored = 0u32;
        let mut compiled = true;
        let mut compile_error = None;

        // Parse JSON output line by line
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;

            // Try to parse as cargo message
            if let Ok(event) = serde_json::from_str::<CargoTestEvent>(&line) {
                match event {
                    CargoTestEvent::Test(test) => {
                        let result = self.test_event_to_result(&test);
                        match result.status {
                            TestStatus::Passed => passed += 1,
                            TestStatus::Failed | TestStatus::Error => failed += 1,
                            TestStatus::Skipped => ignored += 1,
                        }
                        results.push(result);
                    }
                    CargoTestEvent::Bench(bench) => {
                        // Convert bench to test result
                        let meta = TestMeta::with_language(&bench.name, Language::Rust)
                            .with_type(TestType::Profile);
                        let result = TestResult::passed(meta, bench.median / 1000); // ns to µs
                        results.push(result);
                    }
                    CargoTestEvent::Suite(_) => {
                        // Suite events are for summary, we track individually
                    }
                }
            } else if line.contains("\"reason\":\"compiler-message\"") {
                // Check for compilation errors
                if line.contains("\"level\":\"error\"") {
                    compiled = false;
                    // Extract error message
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(message) = msg.get("message").and_then(|m| m.get("message")) {
                            compile_error =
                                Some(message.as_str().unwrap_or("Compilation error").to_string());
                        }
                    }
                }
            }
        }

        // Wait for stderr thread to complete
        let stderr_output = stderr_handle
            .join()
            .map_err(|_| "Failed to join stderr thread".to_string())?;

        // If no compile error from JSON but stderr has content, use it
        if compile_error.is_none() && !stderr_output.trim().is_empty() && !compiled {
            compile_error = Some(stderr_output.lines().take(5).collect::<Vec<_>>().join("\n"));
        }

        // Wait for process to complete
        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for cargo test: {}", e))?;

        // Check exit status for compilation failure
        if !status.success() && results.is_empty() && compile_error.is_none() {
            compiled = false;
            compile_error = Some(format!(
                "cargo test failed with exit code: {:?}",
                status.code()
            ));
        }

        if !status.success() && compiled {
            // Process may have failed but tests completed
            tracing::debug!(
                "cargo test exited with non-zero status: {:?}",
                status.code()
            );
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(RustRunnerResult {
            results,
            passed,
            failed,
            ignored,
            duration_ms,
            compiled,
            compile_error,
        })
    }

    /// Convert a cargo test event to TestResult
    fn test_event_to_result(&self, event: &TestEvent) -> TestResult {
        let meta = TestMeta::with_language(&event.name, Language::Rust);
        let duration_ms = event.exec_time.map(|t| (t * 1000.0) as u64).unwrap_or(0);

        match event.event.as_str() {
            "ok" => TestResult::passed(meta, duration_ms),
            "failed" => {
                let error = event
                    .stdout
                    .clone()
                    .or_else(|| event.stderr.clone())
                    .unwrap_or_else(|| "Test failed".to_string());
                TestResult::failed(meta, duration_ms, error)
            }
            "ignored" => TestResult::skipped(meta, "Test ignored"),
            _ => TestResult::passed(meta, duration_ms),
        }
    }

    /// Run cargo bench and collect results
    pub fn run_benchmarks(&self) -> Result<RustRunnerResult, String> {
        let start = std::time::Instant::now();

        let mut cmd = Command::new("cargo");
        cmd.arg("bench").arg("--message-format=json");

        cmd.current_dir(&self.config.project_path);

        if let Some(ref filter) = self.config.filter {
            cmd.arg(filter);
        }

        if !self.config.features.is_empty() {
            cmd.arg("--features").arg(self.config.features.join(","));
        }

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn cargo bench: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child.stderr.take();

        // Spawn thread to drain stderr to prevent deadlock
        let stderr_handle = stderr.map(|stderr| {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    let _ = line; // Just drain, don't need content
                }
            })
        });

        let reader = BufReader::new(stdout);
        let mut results = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;

            if let Ok(CargoTestEvent::Bench(bench)) = serde_json::from_str(&line) {
                let meta = TestMeta::with_language(&bench.name, Language::Rust)
                    .with_type(TestType::Profile);

                // bench.median is in nanoseconds, convert to milliseconds
                let mut result = TestResult::passed(meta, bench.median / 1_000_000);

                // Add benchmark-specific metrics
                result.profile_metrics = Some(crate::runner::ProfileMetrics {
                    iterations: 1,
                    avg_cpu_time_ms: bench.median as f64 / 1_000_000.0,
                    peak_memory_bytes: 0,
                    boundary_overhead_ms: 0.0,
                });

                results.push(result);
            }
        }

        // Wait for stderr thread
        if let Some(handle) = stderr_handle {
            let _ = handle.join();
        }

        child
            .wait()
            .map_err(|e| format!("Failed to wait for cargo bench: {}", e))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(RustRunnerResult {
            results: results.clone(),
            passed: results.len() as u32,
            failed: 0,
            ignored: 0,
            duration_ms,
            compiled: true,
            compile_error: None,
        })
    }

    /// Run cargo-audit for security scanning
    pub fn run_audit(&self) -> Result<AuditResult, String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("audit").arg("--json");

        cmd.current_dir(&self.config.project_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // `cargo audit` exits NON-ZERO exactly when advisories are found, so the
        // report MUST be parsed regardless of exit status. A spawn failure means
        // the binary is absent.
        let output = cmd.output().map_err(|e| {
            format!("Failed to run cargo-audit: {e}. Install it with `cargo install cargo-audit`.")
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_audit_output(&stdout)
    }

    /// Parse `cargo audit --json` stdout into an [`AuditResult`].
    ///
    /// This is independent of the audit process exit status: `cargo audit`
    /// returns non-zero exactly when advisories are found, so keying parsing on
    /// success would silently drop every real vulnerability. A clean crate still
    /// emits a JSON report with an empty `vulnerabilities.list`, so a clean run
    /// naturally parses into an empty result. An unparseable run (e.g. the
    /// advisory DB could not be fetched) surfaces as an `Err` rather than a
    /// fake-clean result.
    fn parse_audit_output(stdout: &str) -> Result<AuditResult, String> {
        match serde_json::from_str::<AuditReport>(stdout) {
            Ok(report) => Ok(AuditResult {
                vulnerabilities: report.vulnerabilities.list,
                warnings: report.warnings.0,
            }),
            Err(e) => Err(format!("cargo-audit output not parseable as JSON: {e}")),
        }
    }
}

/// Cargo audit report structure
#[derive(Debug, Clone, Deserialize)]
struct AuditReport {
    vulnerabilities: VulnerabilityList,
    /// Real `cargo audit --json` emits `warnings` as a map keyed by warning
    /// kind, but a flat array is also accepted. `WarningsField` normalizes both
    /// (and absence) into a `Vec<AuditWarning>` without changing the public
    /// `AuditResult::warnings` type.
    #[serde(default)]
    warnings: WarningsField,
}

#[derive(Debug, Clone, Deserialize)]
struct VulnerabilityList {
    #[serde(default)]
    list: Vec<Vulnerability>,
}

/// Accepts `warnings` as either a map (`{ "unmaintained": [...] }`, the real
/// cargo-audit shape), a flat array, or absent — always yielding a flat
/// `Vec<AuditWarning>`.
#[derive(Debug, Clone, Default)]
struct WarningsField(Vec<AuditWarning>);

impl<'de> Deserialize<'de> for WarningsField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Map(std::collections::BTreeMap<String, Vec<AuditWarning>>),
            List(Vec<AuditWarning>),
        }
        Ok(match Raw::deserialize(deserializer)? {
            Raw::Map(m) => WarningsField(m.into_values().flatten().collect()),
            Raw::List(v) => WarningsField(v),
        })
    }
}

/// Security vulnerability from cargo-audit.
///
/// Real `cargo audit --json` nests advisory fields under an `advisory` object
/// and carries a top-level `package` object on each list entry. A custom
/// `Deserialize` reads that shape while ALSO accepting a flat shape (id/title/
/// etc. at the top level), so the public field set below is unchanged.
#[derive(Debug, Clone, Serialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct Vulnerability {
    /// Advisory ID (e.g., RUSTSEC-2021-0001)
    pub id: String,
    /// Package name
    pub package: Package,
    /// Severity level
    pub severity: Option<String>,
    /// Advisory title
    pub title: String,
    /// Advisory description
    pub description: String,
    /// CVE identifier if available
    pub cvss: Option<String>,
}

impl<'de> Deserialize<'de> for Vulnerability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Advisory fields, accepted either nested under `advisory` (real
        // cargo-audit) or flattened at the top level (compact/test shape).
        #[derive(Deserialize)]
        struct Advisory {
            id: String,
            title: String,
            #[serde(default)]
            description: String,
            #[serde(default)]
            severity: Option<String>,
            #[serde(default)]
            cvss: Option<String>,
        }
        #[derive(Deserialize)]
        struct Raw {
            // Top-level entry package object (present in both shapes).
            package: Package,
            // Real shape: nested advisory object.
            #[serde(default)]
            advisory: Option<Advisory>,
            // Flat shape: advisory fields hoisted to the top level.
            #[serde(default)]
            id: Option<String>,
            #[serde(default)]
            title: Option<String>,
            #[serde(default)]
            description: Option<String>,
            #[serde(default)]
            severity: Option<String>,
            #[serde(default)]
            cvss: Option<String>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let v = if let Some(adv) = raw.advisory {
            Vulnerability {
                id: adv.id,
                package: raw.package,
                severity: adv.severity,
                title: adv.title,
                description: adv.description,
                cvss: adv.cvss,
            }
        } else {
            Vulnerability {
                id: raw
                    .id
                    .ok_or_else(|| serde::de::Error::missing_field("id"))?,
                package: raw.package,
                severity: raw.severity,
                title: raw
                    .title
                    .ok_or_else(|| serde::de::Error::missing_field("title"))?,
                description: raw.description.unwrap_or_default(),
                cvss: raw.cvss,
            }
        };
        Ok(v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct Package {
    pub name: String,
    pub version: String,
}

/// Audit warning
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct AuditWarning {
    pub kind: String,
    pub package: Option<Package>,
    pub message: Option<String>,
}

/// Result of cargo-audit scan
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub struct AuditResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub warnings: Vec<AuditWarning>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
impl AuditResult {
    /// Check if there are any critical vulnerabilities
    pub fn has_critical(&self) -> bool {
        self.vulnerabilities.iter().any(|v| {
            v.severity
                .as_ref()
                .map(|s| s == "critical")
                .unwrap_or(false)
        })
    }

    /// Check if there are any vulnerabilities
    pub fn has_vulnerabilities(&self) -> bool {
        !self.vulnerabilities.is_empty()
    }

    /// Get vulnerability count
    pub fn vulnerability_count(&self) -> usize {
        self.vulnerabilities.len()
    }
}

/// Detect if a directory is a Rust project
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub fn is_rust_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
}

/// Get Rust toolchain version info
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub fn get_rust_version() -> Option<String> {
    let output = Command::new("rustc").arg("--version").output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Get cargo version
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-rust-runner-rs.md#source
pub fn get_cargo_version() -> Option<String> {
    let output = Command::new("cargo").arg("--version").output().ok()?;

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
    fn test_rust_runner_config_default() {
        let config = RustRunnerConfig::default();
        assert_eq!(config.project_path, PathBuf::from("."));
        assert!(!config.release);
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_rust_version_detection() {
        // This test depends on rustc being installed
        let version = get_rust_version();
        if let Some(v) = version {
            assert!(v.contains("rustc"));
        }
    }

    #[test]
    fn test_cargo_version_detection() {
        // This test depends on cargo being installed
        let version = get_cargo_version();
        if let Some(v) = version {
            assert!(v.contains("cargo"));
        }
    }

    #[test]
    fn test_is_rust_project() {
        // Test with a path that likely doesn't have Cargo.toml
        let non_rust = PathBuf::from("/tmp");
        assert!(!is_rust_project(&non_rust));
    }

    #[test]
    fn test_audit_result_helpers() {
        let result = AuditResult {
            vulnerabilities: vec![],
            warnings: vec![],
        };
        assert!(!result.has_vulnerabilities());
        assert!(!result.has_critical());
        assert_eq!(result.vulnerability_count(), 0);
    }

    #[test]
    fn parse_audit_output_surfaces_vulns() {
        // A vulnerable run: `cargo audit` would exit NON-ZERO here, but the
        // parser must surface the advisory anyway (the trust bug was keying
        // parsing on a successful exit status).
        let sample = r#"{
            "vulnerabilities": {
                "list": [
                    {
                        "id": "RUSTSEC-2020-0071",
                        "package": { "name": "time", "version": "0.1.45" },
                        "severity": "high",
                        "title": "Potential segfault in the time crate",
                        "description": "Unix-like operating systems may segfault.",
                        "cvss": "CVSS:3.1/AV:L/AC:H/PR:N/UI:N/S:C/C:N/I:N/A:H"
                    }
                ]
            }
        }"#;

        let result = RustRunner::parse_audit_output(sample).unwrap();
        assert!(
            result.vulnerabilities.len() >= 1,
            "parser must surface advisories independent of exit code"
        );
        assert_eq!(result.vulnerabilities[0].id, "RUSTSEC-2020-0071");
        assert_eq!(result.vulnerabilities[0].package.name, "time");
    }

    #[test]
    fn parse_audit_output_clean_is_empty() {
        // A clean crate: `cargo audit --json` still emits a JSON report with an
        // empty list. This must parse into an empty, non-error result.
        let sample = r#"{ "vulnerabilities": { "list": [] } }"#;
        let result = RustRunner::parse_audit_output(sample).unwrap();
        assert!(result.vulnerabilities.is_empty());
        assert!(!result.has_vulnerabilities());
    }

    #[test]
    fn test_test_event_parsing() {
        let json = r#"{"type":"test","event":"ok","name":"tests::test_example","exec_time":0.001}"#;
        let event: CargoTestEvent = serde_json::from_str(json).unwrap();

        match event {
            CargoTestEvent::Test(test) => {
                assert_eq!(test.name, "tests::test_example");
                assert_eq!(test.event, "ok");
                assert!(test.exec_time.is_some());
            }
            _ => panic!("Expected Test event"),
        }
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/rust_runner.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/rust_runner.rs` captured during meter full-codegen standardization.
```
