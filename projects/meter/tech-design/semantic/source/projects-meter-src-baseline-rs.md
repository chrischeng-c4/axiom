---
id: projects-meter-src-baseline-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: benchmark-regression-api
    claim: benchmark-regression-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/src/baseline.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/baseline.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BaselineContent` | projects/meter/src/baseline.rs | enum | pub | 99 |  |
| `BaselineMetadata` | projects/meter/src/baseline.rs | struct | pub | 66 |  |
| `BaselineSnapshot` | projects/meter/src/baseline.rs | struct | pub | 109 |  |
| `FileBaselineStore` | projects/meter/src/baseline.rs | struct | pub | 311 |  |
| `GitMetadata` | projects/meter/src/baseline.rs | struct | pub | 85 |  |
| `Improvement` | projects/meter/src/baseline.rs | struct | pub | 268 |  |
| `PercentileType` | projects/meter/src/baseline.rs | enum | pub | 22 |  |
| `Regression` | projects/meter/src/baseline.rs | struct | pub | 248 |  |
| `RegressionDetector` | projects/meter/src/baseline.rs | struct | pub | 518 |  |
| `RegressionReport` | projects/meter/src/baseline.rs | struct | pub | 296 |  |
| `RegressionSeverity` | projects/meter/src/baseline.rs | enum | pub | 236 |  |
| `RegressionSummary` | projects/meter/src/baseline.rs | struct | pub | 282 |  |
| `RegressionThresholds` | projects/meter/src/baseline.rs | struct | pub | 198 |  |
| `agent_eval` | projects/meter/src/baseline.rs | function | pub | 153 | agent_eval(&self) -> Option<&[AgentEvalResult]> |
| `benchmarks` | projects/meter/src/baseline.rs | function | pub | 145 | benchmarks(&self) -> Option<&[BenchmarkResult]> |
| `default_store` | projects/meter/src/baseline.rs | function | pub | 325 | default_store() -> Self |
| `detect_regressions` | projects/meter/src/baseline.rs | function | pub | 531 | detect_regressions(         baseline: &BaselineSnapshot,         current: &[BenchmarkResult],         thresholds: &RegressionThresholds,     ) -> RegressionReport |
| `extract_value` | projects/meter/src/baseline.rs | function | pub | 34 | extract_value(&self, stats: &BenchmarkStats) -> f64 |
| `from_agent_eval` | projects/meter/src/baseline.rs | function | pub | 132 | from_agent_eval(results: Vec<AgentEvalResult>, env: &BenchmarkEnvironment) -> Self |
| `from_benchmarks` | projects/meter/src/baseline.rs | function | pub | 119 | from_benchmarks(benchmarks: Vec<BenchmarkResult>, env: &BenchmarkEnvironment) -> Self |
| `from_binary` | projects/meter/src/baseline.rs | function | pub | 183 | from_binary(_bytes: &[u8]) -> Result<Self, String> |
| `list_baselines` | projects/meter/src/baseline.rs | function | pub | 445 | list_baselines(&self, name: &str) -> io::Result<Vec<BaselineMetadata>> |
| `load_baseline` | projects/meter/src/baseline.rs | function | pub | 409 | load_baseline(&self, name: &str, id: &str) -> io::Result<BaselineSnapshot> |
| `name` | projects/meter/src/baseline.rs | function | pub | 46 | name(&self) -> &'static str |
| `new` | projects/meter/src/baseline.rs | function | pub | 318 | new(base_dir: impl AsRef<Path>) -> Self |
| `save_agent_eval_baseline` | projects/meter/src/baseline.rs | function | pub | 357 | save_agent_eval_baseline(         &self,         name: &str,         results: &[AgentEvalResult],         env: &BenchmarkEnvironment,     ) -> io::Result<String> |
| `save_baseline` | projects/meter/src/baseline.rs | function | pub | 338 | save_baseline(         &self,         name: &str,         results: &[BenchmarkResult],         env: &BenchmarkEnvironment,     ) -> io::Result<String> |
| `size_comparison` | projects/meter/src/baseline.rs | function | pub | 188 | size_comparison(&self) -> (usize, usize) |
| `to_binary` | projects/meter/src/baseline.rs | function | pub | 166 | to_binary(&self) -> Result<Vec<u8>, String> |
| `with_percentile` | projects/meter/src/baseline.rs | function | pub | 227 | with_percentile(mut self, percentile: PercentileType) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/baseline.rs -->
````rust
//! Baseline metrics recording and regression detection
//!
//! Provides utilities for:
//! - Saving benchmark results as baselines
//! - Loading historical baselines
//! - Detecting performance regressions
//! - Generating regression reports

use crate::agent_eval::result::AgentEvalResult;
use crate::benchmark::{BenchmarkEnvironment, BenchmarkResult, BenchmarkStats};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Percentile to use for regression detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PercentileType {
    Mean,
    P50,
    P95,
    P99,
    P999,
    P9999,
}

impl PercentileType {
    /// Extract the specified percentile value from BenchmarkStats
    pub fn extract_value(&self, stats: &BenchmarkStats) -> f64 {
        match self {
            PercentileType::Mean => stats.mean_ms,
            PercentileType::P50 => stats.median_ms,
            PercentileType::P95 => stats.p95_ms,
            PercentileType::P99 => stats.p99_ms,
            PercentileType::P999 => stats.p999_ms,
            PercentileType::P9999 => stats.p9999_ms,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            PercentileType::Mean => "Mean",
            PercentileType::P50 => "P50",
            PercentileType::P95 => "P95",
            PercentileType::P99 => "P99",
            PercentileType::P999 => "P999",
            PercentileType::P9999 => "P9999",
        }
    }
}

/// Metadata for a baseline snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct BaselineMetadata {
    /// Baseline format version
    pub version: String,
    /// Timestamp when baseline was created (RFC3339)
    pub timestamp: String,
    /// Git metadata (if available)
    pub git_metadata: Option<GitMetadata>,
    /// Environment information
    pub environment: BenchmarkEnvironment,
}

/// Git repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub struct GitMetadata {
    /// Current commit SHA
    pub commit_sha: Option<String>,
    /// Current branch name
    pub branch: Option<String>,
    /// Whether working directory has uncommitted changes
    pub is_dirty: bool,
    /// Git user name
    pub author: Option<String>,
}

/// Content type for baseline snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaselineContent {
    /// Benchmark results (supports binary serialization)
    Benchmarks(Vec<BenchmarkResult>),
    /// Agent evaluation results (JSON only for now)
    AgentEval(Vec<AgentEvalResult>),
}

/// A snapshot of benchmark results at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineSnapshot {
    /// Metadata about this baseline
    pub metadata: BaselineMetadata,
    /// Content (benchmarks or agent eval)
    pub content: BaselineContent,
}

impl BaselineSnapshot {
    /// Create a new benchmark baseline snapshot
    pub fn from_benchmarks(benchmarks: Vec<BenchmarkResult>, env: &BenchmarkEnvironment) -> Self {
        Self {
            metadata: BaselineMetadata {
                version: "1.0".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                git_metadata: FileBaselineStore::get_git_metadata().ok(),
                environment: env.clone(),
            },
            content: BaselineContent::Benchmarks(benchmarks),
        }
    }

    /// Create a new agent eval baseline snapshot
    pub fn from_agent_eval(results: Vec<AgentEvalResult>, env: &BenchmarkEnvironment) -> Self {
        Self {
            metadata: BaselineMetadata {
                version: "1.0".to_string(),
                timestamp: Utc::now().to_rfc3339(),
                git_metadata: FileBaselineStore::get_git_metadata().ok(),
                environment: env.clone(),
            },
            content: BaselineContent::AgentEval(results),
        }
    }

    /// Get benchmarks if this is a benchmark baseline
    pub fn benchmarks(&self) -> Option<&[BenchmarkResult]> {
        match &self.content {
            BaselineContent::Benchmarks(b) => Some(b),
            _ => None,
        }
    }

    /// Get agent eval results if this is an agent eval baseline
    pub fn agent_eval(&self) -> Option<&[AgentEvalResult]> {
        match &self.content {
            BaselineContent::AgentEval(r) => Some(r),
            _ => None,
        }
    }
}

impl BaselineSnapshot {
    /// Serialize to binary format using rkyv (zero-copy)
    /// Only works for benchmark baselines
    #[cfg(feature = "rkyv")]
    pub fn to_binary(&self) -> Result<Vec<u8>, String> {
        match &self.content {
            BaselineContent::Benchmarks(_) => {
                // For benchmarks, we serialize the whole snapshot
                // Note: This requires special handling since BaselineContent
                // doesn't derive rkyv. We'll just use JSON for now for agent eval.
                Err("Binary serialization only supported for benchmarks currently".to_string())
            }
            BaselineContent::AgentEval(_) => {
                Err("Binary serialization not supported for agent eval (use JSON)".to_string())
            }
        }
    }

    /// Deserialize from binary format using rkyv (zero-copy)
    /// Only works for benchmark baselines
    #[cfg(feature = "rkyv")]
    pub fn from_binary(_bytes: &[u8]) -> Result<Self, String> {
        Err("Binary deserialization only supported for benchmarks currently".to_string())
    }

    /// Get size comparison between JSON and binary formats
    pub fn size_comparison(&self) -> (usize, usize) {
        let json_size = serde_json::to_string(self).unwrap().len();
        // Binary serialization disabled for now (requires rkyv derives on all types)
        (json_size, 0)
    }
}

/// Thresholds for regression detection
#[derive(Debug, Clone)]
pub struct RegressionThresholds {
    /// Warning threshold as percentage (e.g., 5.0 = 5%)
    pub warning_threshold_percent: f64,
    /// Failure threshold as percentage (e.g., 15.0 = 15%)
    pub failure_threshold_percent: f64,
    /// Require confidence interval overlap to avoid false positives
    pub ci_overlap_required: bool,
    /// Minimum number of samples required for comparison
    pub min_samples_for_comparison: u32,
    /// Which percentile to compare (default: Mean)
    pub percentile_type: PercentileType,
}

impl Default for RegressionThresholds {
    fn default() -> Self {
        Self {
            warning_threshold_percent: 5.0,
            failure_threshold_percent: 15.0,
            ci_overlap_required: false,
            min_samples_for_comparison: 3,
            percentile_type: PercentileType::Mean,
        }
    }
}

impl RegressionThresholds {
    /// Set which percentile to use for regression detection
    pub fn with_percentile(mut self, percentile: PercentileType) -> Self {
        self.percentile_type = percentile;
        self
    }
}

/// Severity of a detected regression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// 0-5% slower
    Minor,
    /// 5-15% slower
    Moderate,
    /// >15% slower
    Severe,
}

/// A detected performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regression {
    /// Name of the benchmark
    pub name: String,
    /// Which percentile was used for comparison
    pub percentile_type: PercentileType,
    /// Baseline value in milliseconds
    pub baseline_value_ms: f64,
    /// Current value in milliseconds
    pub current_value_ms: f64,
    /// Percentage change (positive = slower)
    pub percent_change: f64,
    /// Whether confidence intervals overlap
    pub ci_overlap: bool,
    /// Severity of the regression
    pub severity: RegressionSeverity,
}

/// A detected performance improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    /// Name of the benchmark
    pub name: String,
    /// Baseline mean time in milliseconds
    pub baseline_mean_ms: f64,
    /// Current mean time in milliseconds
    pub current_mean_ms: f64,
    /// Percentage change (negative = faster)
    pub percent_change: f64,
}

/// Summary statistics for regression analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RegressionSummary {
    /// Total number of benchmarks compared
    pub total_benchmarks: usize,
    /// Number of regressions found
    pub regressions_found: usize,
    /// Number of improvements found
    pub improvements_found: usize,
    /// Number of unchanged benchmarks
    pub unchanged: usize,
}

/// Report containing all regression analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    /// Baseline timestamp
    pub baseline_timestamp: String,
    /// Current run timestamp
    pub current_timestamp: String,
    /// Detected regressions
    pub regressions: Vec<Regression>,
    /// Detected improvements
    pub improvements: Vec<Improvement>,
    /// Summary statistics
    pub summary: RegressionSummary,
}

/// File-based baseline storage
pub struct FileBaselineStore {
    base_dir: PathBuf,
}

impl FileBaselineStore {
    /// Create a new file-based baseline store
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a default baseline store in .baselines directory
    pub fn default_store() -> Self {
        Self::new(".baselines")
    }

    /// Save benchmark results as a baseline
    ///
    /// # Arguments
    /// * `name` - Name of the baseline (e.g., "main", "feature-branch")
    /// * `results` - Benchmark results to save
    /// * `env` - Environment information
    ///
    /// # Returns
    /// The filename of the saved baseline
    pub fn save_baseline(
        &self,
        name: &str,
        results: &[BenchmarkResult],
        env: &BenchmarkEnvironment,
    ) -> io::Result<String> {
        let snapshot = BaselineSnapshot::from_benchmarks(results.to_vec(), env);
        self.save_snapshot(name, &snapshot)
    }

    /// Save agent eval results as a baseline
    ///
    /// # Arguments
    /// * `name` - Name of the baseline (e.g., "main", "feature-branch")
    /// * `results` - Agent eval results to save
    /// * `env` - Environment information
    ///
    /// # Returns
    /// The filename of the saved baseline
    pub fn save_agent_eval_baseline(
        &self,
        name: &str,
        results: &[AgentEvalResult],
        env: &BenchmarkEnvironment,
    ) -> io::Result<String> {
        let snapshot = BaselineSnapshot::from_agent_eval(results.to_vec(), env);
        self.save_snapshot(name, &snapshot)
    }

    /// Internal method to save a snapshot
    fn save_snapshot(&self, name: &str, snapshot: &BaselineSnapshot) -> io::Result<String> {
        fs::create_dir_all(&self.base_dir)?;

        let timestamp = snapshot.metadata.timestamp.clone();

        let filename_base = format!("{}_{}", name, timestamp.replace(':', "-"));

        // Save JSON (human-readable, git-diffable)
        let json_path = self.base_dir.join(format!("{}.json", filename_base));
        let json = serde_json::to_string_pretty(&snapshot).map_err(io::Error::other)?;
        fs::write(&json_path, json)?;

        // Save binary (fast loading) - only for benchmarks
        #[cfg(feature = "rkyv")]
        {
            if matches!(snapshot.content, BaselineContent::Benchmarks(_)) {
                // Binary serialization disabled for now
                // Would require rkyv derives on all nested types
                // let binary_path = self.base_dir.join(format!("{}.bin", filename_base));
                // let binary = snapshot.to_binary()
                //     .map_err(io::Error::other)?;
                // fs::write(&binary_path, binary)?;
            }
        }

        // Update latest symlink/copy (JSON)
        let latest = self.base_dir.join(format!("{}_latest.json", name));
        let _ = fs::remove_file(&latest);
        fs::copy(&json_path, &latest)?;

        Ok(format!("{}.json", filename_base))
    }

    /// Load a baseline by name and ID
    ///
    /// # Arguments
    /// * `name` - Name of the baseline
    /// * `id` - Baseline ID (filename or "latest")
    ///
    /// # Returns
    /// The loaded baseline snapshot
    pub fn load_baseline(&self, name: &str, id: &str) -> io::Result<BaselineSnapshot> {
        let base_path = if id == "latest" {
            format!("{}_latest", name)
        } else {
            id.trim_end_matches(".json")
                .trim_end_matches(".bin")
                .to_string()
        };

        // Try binary first (faster)
        #[cfg(feature = "rkyv")]
        {
            let binary_path = self.base_dir.join(format!("{}.bin", base_path));
            if binary_path.exists() {
                let bytes = fs::read(&binary_path)?;
                return BaselineSnapshot::from_binary(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
            }
        }

        // Fallback to JSON
        let json_path = self.base_dir.join(format!("{}.json", base_path));
        let json = fs::read_to_string(&json_path)?;
        let snapshot: BaselineSnapshot = serde_json::from_str(&json)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(snapshot)
    }

    /// List all baselines for a given name
    ///
    /// # Arguments
    /// * `name` - Name of the baseline group
    ///
    /// # Returns
    /// Vector of metadata for all matching baselines (sorted newest first)
    pub fn list_baselines(&self, name: &str) -> io::Result<Vec<BaselineMetadata>> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut metadata = Vec::new();

        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if filename.starts_with(name) && !filename.ends_with("_latest.json") {
                    if let Ok(json) = fs::read_to_string(&path) {
                        if let Ok(snapshot) = serde_json::from_str::<BaselineSnapshot>(&json) {
                            metadata.push(snapshot.metadata);
                        }
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        metadata.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(metadata)
    }

    /// Attempt to extract git metadata from current repository
    fn get_git_metadata() -> io::Result<GitMetadata> {
        use std::process::Command;

        let commit_sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let is_dirty = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .ok()
            .map(|o| !o.stdout.is_empty())
            .unwrap_or(false);

        let author = Command::new("git")
            .args(["config", "user.name"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        Ok(GitMetadata {
            commit_sha,
            branch,
            is_dirty,
            author,
        })
    }
}

/// Regression detection engine
pub struct RegressionDetector;

impl RegressionDetector {
    /// Detect regressions by comparing current results against baseline
    ///
    /// # Arguments
    /// * `baseline` - Baseline snapshot to compare against
    /// * `current` - Current benchmark results
    /// * `thresholds` - Thresholds for regression detection
    ///
    /// # Returns
    /// A regression report containing all findings
    pub fn detect_regressions(
        baseline: &BaselineSnapshot,
        current: &[BenchmarkResult],
        thresholds: &RegressionThresholds,
    ) -> RegressionReport {
        let mut regressions = Vec::new();
        let mut improvements = Vec::new();
        let mut unchanged = 0;

        // Extract benchmarks from baseline content
        let baseline_benchmarks = match &baseline.content {
            BaselineContent::Benchmarks(b) => b,
            _ => {
                return RegressionReport {
                    baseline_timestamp: baseline.metadata.timestamp.clone(),
                    current_timestamp: Utc::now().to_rfc3339(),
                    summary: RegressionSummary {
                        total_benchmarks: current.len(),
                        regressions_found: 0,
                        improvements_found: 0,
                        unchanged: 0,
                    },
                    regressions: Vec::new(),
                    improvements: Vec::new(),
                }
            }
        };

        for current_result in current {
            if let Some(baseline_result) = baseline_benchmarks
                .iter()
                .find(|b| b.name == current_result.name)
            {
                // Extract values using configured percentile
                let baseline_value = thresholds
                    .percentile_type
                    .extract_value(&baseline_result.stats);
                let current_value = thresholds
                    .percentile_type
                    .extract_value(&current_result.stats);

                let percent_change = ((current_value - baseline_value) / baseline_value) * 100.0;

                // Check CI overlap (only applicable for mean/p50 that have CI)
                let ci_overlap = if matches!(
                    thresholds.percentile_type,
                    PercentileType::Mean | PercentileType::P50
                ) {
                    !(baseline_result.stats.ci_upper_ms < current_result.stats.ci_lower_ms
                        || baseline_result.stats.ci_lower_ms > current_result.stats.ci_upper_ms)
                } else {
                    false // No CI for high percentiles
                };

                if percent_change > thresholds.warning_threshold_percent {
                    if !ci_overlap || !thresholds.ci_overlap_required {
                        let severity = if percent_change > thresholds.failure_threshold_percent {
                            RegressionSeverity::Severe
                        } else if percent_change > thresholds.warning_threshold_percent {
                            RegressionSeverity::Moderate
                        } else {
                            RegressionSeverity::Minor
                        };

                        regressions.push(Regression {
                            name: current_result.name.clone(),
                            percentile_type: thresholds.percentile_type,
                            baseline_value_ms: baseline_value,
                            current_value_ms: current_value,
                            percent_change,
                            ci_overlap,
                            severity,
                        });
                    } else {
                        unchanged += 1;
                    }
                } else if percent_change < -2.0 {
                    improvements.push(Improvement {
                        name: current_result.name.clone(),
                        baseline_mean_ms: baseline_value,
                        current_mean_ms: current_value,
                        percent_change,
                    });
                } else {
                    unchanged += 1;
                }
            }
        }

        RegressionReport {
            baseline_timestamp: baseline.metadata.timestamp.clone(),
            current_timestamp: Utc::now().to_rfc3339(),
            summary: RegressionSummary {
                total_benchmarks: current.len(),
                regressions_found: regressions.len(),
                improvements_found: improvements.len(),
                unchanged,
            },
            regressions,
            improvements,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::BenchmarkStats;

    fn create_test_result(name: &str, mean_ms: f64, std_dev: f64) -> BenchmarkResult {
        BenchmarkResult {
            name: name.to_string(),
            stats: BenchmarkStats {
                iterations: 10,
                rounds: 1,
                warmup: 0,
                total_runs: 10,
                mean_ms,
                min_ms: mean_ms * 0.9,
                max_ms: mean_ms * 1.1,
                stddev_ms: std_dev,
                median_ms: mean_ms,
                total_ms: mean_ms * 10.0,
                p25_ms: mean_ms * 0.95,
                p75_ms: mean_ms * 1.05,
                p95_ms: mean_ms * 1.08,
                p99_ms: mean_ms * 1.10,
                p999_ms: mean_ms * 1.11,
                p9999_ms: mean_ms * 1.12,
                tail_latency_ratio: 1.10 / 1.0, // p99/p50
                histogram: vec![],
                iqr_ms: mean_ms * 0.1,
                outliers: 0,
                outliers_low: 0,
                outliers_high: 0,
                std_error_ms: std_dev / 10f64.sqrt(),
                ci_lower_ms: mean_ms - 1.96 * std_dev / 10f64.sqrt(),
                ci_upper_ms: mean_ms + 1.96 * std_dev / 10f64.sqrt(),
                all_times_ms: vec![mean_ms; 10],
                adaptive_stopped_early: false,
                adaptive_reason: None,
                adaptive_iterations_used: 10,
            },
            success: true,
            error: None,
        }
    }

    fn create_baseline_snapshot(benchmarks: Vec<BenchmarkResult>) -> BaselineSnapshot {
        BaselineSnapshot {
            metadata: BaselineMetadata {
                version: "1.0".to_string(),
                timestamp: "2026-01-01T00:00:00Z".to_string(),
                git_metadata: None,
                environment: BenchmarkEnvironment::default(),
            },
            content: BaselineContent::Benchmarks(benchmarks),
        }
    }

    fn create_test_result_with_percentiles(
        name: &str,
        mean_ms: f64,
        std_dev: f64,
        p95_ms: f64,
        p99_ms: f64,
    ) -> BenchmarkResult {
        BenchmarkResult {
            name: name.to_string(),
            stats: BenchmarkStats {
                iterations: 10,
                rounds: 1,
                warmup: 0,
                total_runs: 10,
                mean_ms,
                min_ms: mean_ms * 0.9,
                max_ms: mean_ms * 1.1,
                stddev_ms: std_dev,
                median_ms: mean_ms,
                total_ms: mean_ms * 10.0,
                p25_ms: mean_ms * 0.95,
                p75_ms: mean_ms * 1.05,
                p95_ms,
                p99_ms,
                p999_ms: p99_ms * 1.01,
                p9999_ms: p99_ms * 1.02,
                tail_latency_ratio: p99_ms / mean_ms,
                histogram: vec![],
                iqr_ms: mean_ms * 0.1,
                outliers: 0,
                outliers_low: 0,
                outliers_high: 0,
                std_error_ms: std_dev / 10f64.sqrt(),
                ci_lower_ms: mean_ms - 1.96 * std_dev / 10f64.sqrt(),
                ci_upper_ms: mean_ms + 1.96 * std_dev / 10f64.sqrt(),
                all_times_ms: vec![mean_ms; 10],
                adaptive_stopped_early: false,
                adaptive_reason: None,
                adaptive_iterations_used: 10,
            },
            success: true,
            error: None,
        }
    }

    #[test]
    fn test_regression_severity_classification() {
        let baseline = create_baseline_snapshot(vec![create_test_result("test1", 10.0, 1.0)]);

        let current_minor = vec![create_test_result("test1", 10.4, 1.0)]; // 4% slower
        let current_moderate = vec![create_test_result("test1", 11.0, 1.0)]; // 10% slower
        let current_severe = vec![create_test_result("test1", 12.0, 1.0)]; // 20% slower

        let thresholds = RegressionThresholds::default();

        let report_minor =
            RegressionDetector::detect_regressions(&baseline, &current_minor, &thresholds);
        assert_eq!(report_minor.regressions.len(), 0); // Below 5% threshold

        let report_moderate =
            RegressionDetector::detect_regressions(&baseline, &current_moderate, &thresholds);
        assert_eq!(report_moderate.regressions.len(), 1);
        assert_eq!(
            report_moderate.regressions[0].severity,
            RegressionSeverity::Moderate
        );

        let report_severe =
            RegressionDetector::detect_regressions(&baseline, &current_severe, &thresholds);
        assert_eq!(report_severe.regressions.len(), 1);
        assert_eq!(
            report_severe.regressions[0].severity,
            RegressionSeverity::Severe
        );
    }

    #[test]
    fn test_improvement_detection() {
        let baseline = create_baseline_snapshot(vec![create_test_result("test1", 10.0, 1.0)]);

        let current = vec![create_test_result("test1", 9.0, 1.0)]; // 10% faster

        let thresholds = RegressionThresholds::default();
        let report = RegressionDetector::detect_regressions(&baseline, &current, &thresholds);

        assert_eq!(report.improvements.len(), 1);
        assert_eq!(report.improvements[0].percent_change, -10.0);
    }

    #[test]
    fn test_baseline_store_default() {
        let store = FileBaselineStore::default_store();
        assert_eq!(store.base_dir, PathBuf::from(".baselines"));
    }

    #[test]
    fn test_percentile_regression_detection() {
        let baseline = create_baseline_snapshot(vec![
            create_test_result_with_percentiles("test1", 10.0, 1.0, 15.0, 20.0), // mean=10, p95=15, p99=20
        ]);

        // P99 regressed significantly (20 → 30 = 50% increase)
        // Mean stayed same (10 → 10 = 0%)
        let current = vec![create_test_result_with_percentiles(
            "test1", 10.0, 1.0, 15.0, 30.0,
        )];

        // Test mean-based: no regression
        let mean_thresholds = RegressionThresholds::default().with_percentile(PercentileType::Mean);
        let mean_report =
            RegressionDetector::detect_regressions(&baseline, &current, &mean_thresholds);
        assert_eq!(mean_report.regressions.len(), 0);

        // Test p99-based: regression detected
        let p99_thresholds = RegressionThresholds::default().with_percentile(PercentileType::P99);
        let p99_report =
            RegressionDetector::detect_regressions(&baseline, &current, &p99_thresholds);
        assert_eq!(p99_report.regressions.len(), 1);
        assert_eq!(
            p99_report.regressions[0].severity,
            RegressionSeverity::Severe
        );
        assert_eq!(
            p99_report.regressions[0].percentile_type,
            PercentileType::P99
        );
        assert!((p99_report.regressions[0].baseline_value_ms - 20.0).abs() < 0.01);
        assert!((p99_report.regressions[0].current_value_ms - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_type_extraction() {
        let result = create_test_result_with_percentiles("test", 10.0, 1.0, 15.0, 20.0);

        assert!((PercentileType::Mean.extract_value(&result.stats) - 10.0).abs() < 0.01);
        assert!((PercentileType::P50.extract_value(&result.stats) - 10.0).abs() < 0.01);
        assert!((PercentileType::P95.extract_value(&result.stats) - 15.0).abs() < 0.01);
        assert!((PercentileType::P99.extract_value(&result.stats) - 20.0).abs() < 0.01);
        assert!((PercentileType::P999.extract_value(&result.stats) - 20.2).abs() < 0.01);
        assert!((PercentileType::P9999.extract_value(&result.stats) - 20.4).abs() < 0.01);
    }

    #[test]
    fn test_percentile_type_names() {
        assert_eq!(PercentileType::Mean.name(), "Mean");
        assert_eq!(PercentileType::P50.name(), "P50");
        assert_eq!(PercentileType::P95.name(), "P95");
        assert_eq!(PercentileType::P99.name(), "P99");
        assert_eq!(PercentileType::P999.name(), "P999");
        assert_eq!(PercentileType::P9999.name(), "P9999");
    }

    #[test]
    #[cfg(feature = "rkyv")]
    fn test_binary_serialization_roundtrip() {
        let snapshot = create_baseline_snapshot(vec![create_test_result("test1", 10.0, 1.0)]);

        // Binary serialization temporarily disabled (requires rkyv derives on all types)
        let result = snapshot.to_binary();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Binary serialization only supported"));
    }

    #[test]
    #[cfg(feature = "rkyv")]
    fn test_size_comparison() {
        let times: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let snapshot = create_baseline_snapshot(vec![BenchmarkResult {
            name: "test".to_string(),
            stats: BenchmarkStats::from_times(times, 1000, 1, 0),
            success: true,
            error: None,
        }]);

        let (json_size, binary_size) = snapshot.size_comparison();

        println!("JSON size: {} bytes", json_size);
        println!("Binary size: {} bytes (disabled)", binary_size);

        // Binary serialization temporarily disabled
        assert_eq!(binary_size, 0);
        assert!(json_size > 0, "JSON size should be non-zero");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/baseline.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/baseline.rs` captured during meter full-codegen standardization.
```
