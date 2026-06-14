// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
// CODEGEN-BEGIN
//! Test reporter - generates reports in various formats

use crate::agent_eval::result::{AgentEvalMetrics, AgentEvalResult};
use crate::runner::{TestResult, TestStatus, TestSummary, TestType};
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;

/// Report output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
pub enum ReportFormat {
    /// Markdown format (human-readable)
    #[default]
    Markdown,
    /// HTML format (interactive report)
    Html,
    /// JSON format (machine-parseable)
    Json,
    /// YAML format (human-readable, machine-parseable)
    Yaml,
    /// JUnit XML format (CI integration)
    JUnit,
    /// Console format (colored terminal output)
    Console,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
impl std::fmt::Display for ReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportFormat::Markdown => write!(f, "markdown"),
            ReportFormat::Html => write!(f, "html"),
            ReportFormat::Json => write!(f, "json"),
            ReportFormat::Yaml => write!(f, "yaml"),
            ReportFormat::JUnit => write!(f, "junit"),
            ReportFormat::Console => write!(f, "console"),
        }
    }
}

/// Full test report with all results and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
pub struct TestReport {
    /// Suite name
    pub suite_name: String,
    /// Report generation timestamp
    pub generated_at: String,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Summary statistics
    pub summary: TestSummary,
    /// Individual test results
    pub results: Vec<TestResult>,
    /// Environment info
    pub environment: EnvironmentInfo,
    /// Code coverage info (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageInfo>,
}

/// Environment information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
pub struct EnvironmentInfo {
    /// Python version
    pub python_version: Option<String>,
    /// Rust version
    pub rust_version: Option<String>,
    /// Platform (OS)
    pub platform: Option<String>,
    /// Hostname
    pub hostname: Option<String>,
}

/// Coverage information for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
pub struct FileCoverage {
    /// File path (relative to project root)
    pub path: String,
    /// Total number of statements
    pub statements: usize,
    /// Number of covered statements
    pub covered: usize,
    /// Lines that are not covered (line numbers)
    pub missing_lines: Vec<usize>,
    /// Coverage percentage for this file
    pub coverage_percent: f64,
}

/// Overall coverage summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
pub struct CoverageInfo {
    /// Total statements across all files
    pub total_statements: usize,
    /// Total covered statements
    pub covered_statements: usize,
    /// Overall coverage percentage
    pub coverage_percent: f64,
    /// Per-file coverage data
    pub files: Vec<FileCoverage>,
    /// Files with 0% coverage (not imported/executed)
    pub uncovered_files: Vec<String>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
impl TestReport {
    /// Create a new test report
    pub fn new(suite_name: impl Into<String>, results: Vec<TestResult>) -> Self {
        let suite_name = suite_name.into();
        let summary = calculate_summary(&results);
        let duration_ms = summary.total_duration_ms;

        Self {
            suite_name,
            generated_at: chrono::Utc::now().to_rfc3339(),
            duration_ms,
            summary,
            results,
            environment: EnvironmentInfo::default(),
            coverage: None,
        }
    }

    /// Create a test report from summary (for coverage-only reports)
    pub fn from_summary(suite_name: impl Into<String>, summary: TestSummary) -> Self {
        Self {
            suite_name: suite_name.into(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            duration_ms: summary.total_duration_ms,
            summary,
            results: Vec::new(),
            environment: EnvironmentInfo::default(),
            coverage: None,
        }
    }

    /// Set environment info
    pub fn with_environment(mut self, env: EnvironmentInfo) -> Self {
        self.environment = env;
        self
    }

    /// Set coverage info
    pub fn with_coverage(mut self, coverage: CoverageInfo) -> Self {
        self.coverage = Some(coverage);
        self
    }

    /// Set coverage info (mutable)
    pub fn set_coverage(&mut self, coverage: CoverageInfo) {
        self.coverage = Some(coverage);
    }

    /// Get results by test type
    pub fn results_by_type(&self, test_type: TestType) -> Vec<&TestResult> {
        self.results
            .iter()
            .filter(|r| r.meta.test_type == test_type)
            .collect()
    }

    /// Get failed results
    pub fn failed_results(&self) -> Vec<&TestResult> {
        self.results
            .iter()
            .filter(|r| r.status == TestStatus::Failed || r.status == TestStatus::Error)
            .collect()
    }
}

/// Calculate summary from results
fn calculate_summary(results: &[TestResult]) -> TestSummary {
    let mut summary = TestSummary::default();

    for result in results {
        match result.status {
            TestStatus::Passed => summary.passed += 1,
            TestStatus::Failed => summary.failed += 1,
            TestStatus::Skipped => summary.skipped += 1,
            TestStatus::Error => summary.errors += 1,
        }
        summary.total_duration_ms += result.duration_ms;
    }

    summary.total = results.len();
    summary
}

/// Agent evaluation report with all results and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
pub struct AgentEvalReport {
    /// Suite name
    pub suite_name: String,
    /// Report generation timestamp
    pub generated_at: String,
    /// Individual evaluation results
    pub results: Vec<AgentEvalResult>,
    /// Aggregated metrics
    pub metrics: AgentEvalMetrics,
    /// Environment info
    pub environment: EnvironmentInfo,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
impl AgentEvalReport {
    /// Create a new agent evaluation report
    pub fn new(
        suite_name: impl Into<String>,
        results: Vec<AgentEvalResult>,
        metrics: AgentEvalMetrics,
    ) -> Self {
        Self {
            suite_name: suite_name.into(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            results,
            metrics,
            environment: EnvironmentInfo::default(),
        }
    }

    /// Set environment info
    pub fn with_environment(mut self, env: EnvironmentInfo) -> Self {
        self.environment = env;
        self
    }

    /// Get failed results
    pub fn failed_results(&self) -> Vec<&AgentEvalResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.metrics.passed == self.metrics.total_cases
    }
}

/// Test reporter - generates reports in various formats
#[derive(Debug)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
pub struct Reporter {
    format: ReportFormat,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-reporter-rs.md#source
impl Reporter {
    /// Create a new reporter with specified format
    pub fn new(format: ReportFormat) -> Self {
        Self { format }
    }

    /// Create markdown reporter
    pub fn markdown() -> Self {
        Self::new(ReportFormat::Markdown)
    }

    /// Create JSON reporter
    pub fn json() -> Self {
        Self::new(ReportFormat::Json)
    }

    /// Create HTML reporter
    pub fn html() -> Self {
        Self::new(ReportFormat::Html)
    }

    /// Create JUnit reporter
    pub fn junit() -> Self {
        Self::new(ReportFormat::JUnit)
    }

    /// Create YAML reporter
    pub fn yaml() -> Self {
        Self::new(ReportFormat::Yaml)
    }

    /// Create Console reporter
    pub fn console() -> Self {
        Self::new(ReportFormat::Console)
    }

    /// Generate report string
    pub fn generate(&self, report: &TestReport) -> String {
        match self.format {
            ReportFormat::Markdown => self.generate_markdown(report),
            ReportFormat::Html => self.generate_html(report),
            ReportFormat::Json => self.generate_json(report),
            ReportFormat::Yaml => self.generate_yaml(report),
            ReportFormat::JUnit => self.generate_junit(report),
            ReportFormat::Console => self.generate_console(report),
        }
    }

    /// Generate Markdown report
    fn generate_markdown(&self, report: &TestReport) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "# Test Report: {}", report.suite_name).unwrap();
        writeln!(output).unwrap();

        // Metadata
        let status_emoji = if report.summary.all_passed() {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        };
        writeln!(
            output,
            "**Date**: {} | **Duration**: {:.2}s | **Status**: {} ({}/{})",
            &report.generated_at[..10],
            report.duration_ms as f64 / 1000.0,
            status_emoji,
            report.summary.passed,
            report.summary.total
        )
        .unwrap();
        writeln!(output).unwrap();

        // Summary table
        writeln!(output, "## Summary").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "| Type | Passed | Failed | Skipped | Duration |").unwrap();
        writeln!(output, "|------|--------|--------|---------|----------|").unwrap();

        for test_type in [TestType::Unit, TestType::Profile, TestType::Security] {
            let results = report.results_by_type(test_type);
            if !results.is_empty() {
                let passed = results
                    .iter()
                    .filter(|r| r.status == TestStatus::Passed)
                    .count();
                let failed = results
                    .iter()
                    .filter(|r| r.status == TestStatus::Failed || r.status == TestStatus::Error)
                    .count();
                let skipped = results
                    .iter()
                    .filter(|r| r.status == TestStatus::Skipped)
                    .count();
                let duration: u64 = results.iter().map(|r| r.duration_ms).sum();

                writeln!(
                    output,
                    "| {} | {} | {} | {} | {:.2}s |",
                    test_type,
                    passed,
                    failed,
                    skipped,
                    duration as f64 / 1000.0
                )
                .unwrap();
            }
        }
        writeln!(output).unwrap();

        // Coverage section
        if let Some(ref coverage) = report.coverage {
            writeln!(output, "## Coverage").unwrap();
            writeln!(output).unwrap();

            // Coverage badge/summary
            let coverage_emoji = if coverage.coverage_percent >= 80.0 {
                "🟢"
            } else if coverage.coverage_percent >= 60.0 {
                "🟡"
            } else {
                "🔴"
            };
            writeln!(
                output,
                "{} **{:.1}%** coverage ({}/{} statements)",
                coverage_emoji,
                coverage.coverage_percent,
                coverage.covered_statements,
                coverage.total_statements
            )
            .unwrap();
            writeln!(output).unwrap();

            // Per-file coverage table (only show files with < 100% coverage)
            let incomplete_files: Vec<_> = coverage
                .files
                .iter()
                .filter(|f| f.coverage_percent < 100.0)
                .collect();

            if !incomplete_files.is_empty() {
                writeln!(output, "### Files with Incomplete Coverage").unwrap();
                writeln!(output).unwrap();
                writeln!(output, "| File | Coverage | Statements | Missing Lines |").unwrap();
                writeln!(output, "|------|----------|------------|---------------|").unwrap();

                for file in incomplete_files {
                    let missing = if file.missing_lines.len() > 5 {
                        format!(
                            "{}, ... ({} more)",
                            file.missing_lines[..5]
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                            file.missing_lines.len() - 5
                        )
                    } else {
                        file.missing_lines
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    };

                    writeln!(
                        output,
                        "| {} | {:.1}% | {}/{} | {} |",
                        file.path, file.coverage_percent, file.covered, file.statements, missing
                    )
                    .unwrap();
                }
                writeln!(output).unwrap();
            }

            // Uncovered files
            if !coverage.uncovered_files.is_empty() {
                writeln!(output, "### Uncovered Files (0%)").unwrap();
                writeln!(output).unwrap();
                for file in &coverage.uncovered_files {
                    writeln!(output, "- {}", file).unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        // Failed tests section
        let failed = report.failed_results();
        if !failed.is_empty() {
            writeln!(output, "## Failed Tests").unwrap();
            writeln!(output).unwrap();

            for result in failed {
                writeln!(output, "### {} ❌", result.meta.name).unwrap();
                writeln!(output).unwrap();
                if let Some(ref error) = result.error {
                    writeln!(output, "**Error**: {}", error).unwrap();
                }
                if let Some(ref trace) = result.stack_trace {
                    writeln!(output, "```").unwrap();
                    writeln!(output, "{}", trace).unwrap();
                    writeln!(output, "```").unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        // Profile metrics section
        let profile_results: Vec<_> = report
            .results
            .iter()
            .filter(|r| r.profile_metrics.is_some())
            .collect();

        if !profile_results.is_empty() {
            writeln!(output, "## Profile Metrics").unwrap();
            writeln!(output).unwrap();
            writeln!(
                output,
                "| Test | Iterations | Avg CPU (ms) | Peak Memory | Boundary Overhead |"
            )
            .unwrap();
            writeln!(
                output,
                "|------|------------|--------------|-------------|-------------------|"
            )
            .unwrap();

            for result in profile_results {
                if let Some(ref metrics) = result.profile_metrics {
                    writeln!(
                        output,
                        "| {} | {} | {:.2} | {} | {:.2}ms |",
                        result.meta.name,
                        metrics.iterations,
                        metrics.avg_cpu_time_ms,
                        format_bytes(metrics.peak_memory_bytes),
                        metrics.boundary_overhead_ms
                    )
                    .unwrap();
                }
            }
            writeln!(output).unwrap();
        }

        // All tests details
        writeln!(output, "## All Tests").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "| Status | Test | Duration | Type |").unwrap();
        writeln!(output, "|--------|------|----------|------|").unwrap();

        for result in &report.results {
            let status_icon = match result.status {
                TestStatus::Passed => "✅",
                TestStatus::Failed => "❌",
                TestStatus::Skipped => "⏭️",
                TestStatus::Error => "💥",
            };
            writeln!(
                output,
                "| {} | {} | {}ms | {} |",
                status_icon, result.meta.name, result.duration_ms, result.meta.test_type
            )
            .unwrap();
        }
        writeln!(output).unwrap();

        // Environment info
        if report.environment.python_version.is_some() || report.environment.platform.is_some() {
            writeln!(output, "## Environment").unwrap();
            writeln!(output).unwrap();
            if let Some(ref py) = report.environment.python_version {
                writeln!(output, "- **Python**: {}", py).unwrap();
            }
            if let Some(ref rust) = report.environment.rust_version {
                writeln!(output, "- **Rust**: {}", rust).unwrap();
            }
            if let Some(ref platform) = report.environment.platform {
                writeln!(output, "- **Platform**: {}", platform).unwrap();
            }
            if let Some(ref host) = report.environment.hostname {
                writeln!(output, "- **Hostname**: {}", host).unwrap();
            }
        }

        output
    }

    /// Generate JSON report
    fn generate_json(&self, report: &TestReport) -> String {
        serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate HTML report
    fn generate_html(&self, report: &TestReport) -> String {
        let mut output = String::new();

        writeln!(output, "<!DOCTYPE html>").unwrap();
        writeln!(output, "<html lang=\"en\">").unwrap();
        writeln!(output, "<head>").unwrap();
        writeln!(output, "  <meta charset=\"UTF-8\">").unwrap();
        writeln!(
            output,
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
        )
        .unwrap();
        writeln!(
            output,
            "  <title>Test Report: {}</title>",
            report.suite_name
        )
        .unwrap();
        writeln!(output, "  <style>").unwrap();
        writeln!(output, "    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; background: #f5f5f5; }}").unwrap();
        writeln!(output, "    .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}").unwrap();
        writeln!(
            output,
            "    h1 {{ color: #333; border-bottom: 2px solid #4CAF50; padding-bottom: 10px; }}"
        )
        .unwrap();
        writeln!(output, "    h2 {{ color: #555; margin-top: 30px; }}").unwrap();
        writeln!(output, "    .status-passed {{ color: #4CAF50; }}").unwrap();
        writeln!(output, "    .status-failed {{ color: #f44336; }}").unwrap();
        writeln!(output, "    .status-skipped {{ color: #ff9800; }}").unwrap();
        writeln!(output, "    .status-error {{ color: #9c27b0; }}").unwrap();
        writeln!(
            output,
            "    table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}"
        )
        .unwrap();
        writeln!(
            output,
            "    th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}"
        )
        .unwrap();
        writeln!(
            output,
            "    th {{ background: #f8f9fa; font-weight: 600; }}"
        )
        .unwrap();
        writeln!(output, "    tr:hover {{ background: #f5f5f5; }}").unwrap();
        writeln!(
            output,
            "    .summary {{ display: flex; gap: 20px; margin: 20px 0; }}"
        )
        .unwrap();
        writeln!(
            output,
            "    .stat {{ padding: 20px; border-radius: 8px; text-align: center; flex: 1; }}"
        )
        .unwrap();
        writeln!(output, "    .stat-passed {{ background: #e8f5e9; }}").unwrap();
        writeln!(output, "    .stat-failed {{ background: #ffebee; }}").unwrap();
        writeln!(output, "    .stat-skipped {{ background: #fff3e0; }}").unwrap();
        writeln!(
            output,
            "    .stat-value {{ font-size: 2em; font-weight: bold; }}"
        )
        .unwrap();
        writeln!(output, "    .error-box {{ background: #ffebee; padding: 15px; border-radius: 4px; margin: 10px 0; }}").unwrap();
        writeln!(output, "    pre {{ background: #263238; color: #aed581; padding: 15px; border-radius: 4px; overflow-x: auto; }}").unwrap();
        writeln!(output, "  </style>").unwrap();
        writeln!(output, "</head>").unwrap();
        writeln!(output, "<body>").unwrap();
        writeln!(output, "<div class=\"container\">").unwrap();

        // Header
        let status_class = if report.summary.all_passed() {
            "status-passed"
        } else {
            "status-failed"
        };
        writeln!(
            output,
            "  <h1>Test Report: {} <span class=\"{}\">({})</span></h1>",
            report.suite_name,
            status_class,
            if report.summary.all_passed() {
                "PASSED"
            } else {
                "FAILED"
            }
        )
        .unwrap();
        writeln!(
            output,
            "  <p>Generated: {} | Duration: {:.2}s</p>",
            &report.generated_at[..10],
            report.duration_ms as f64 / 1000.0
        )
        .unwrap();

        // Summary cards
        writeln!(output, "  <div class=\"summary\">").unwrap();
        writeln!(
            output,
            "    <div class=\"stat stat-passed\"><div class=\"stat-value\">{}</div><div>Passed</div></div>",
            report.summary.passed
        )
        .unwrap();
        writeln!(
            output,
            "    <div class=\"stat stat-failed\"><div class=\"stat-value\">{}</div><div>Failed</div></div>",
            report.summary.failed + report.summary.errors
        )
        .unwrap();
        writeln!(
            output,
            "    <div class=\"stat stat-skipped\"><div class=\"stat-value\">{}</div><div>Skipped</div></div>",
            report.summary.skipped
        )
        .unwrap();
        writeln!(output, "  </div>").unwrap();

        // Coverage section
        if let Some(ref coverage) = report.coverage {
            let coverage_color = if coverage.coverage_percent >= 80.0 {
                "#4CAF50"
            } else if coverage.coverage_percent >= 60.0 {
                "#ff9800"
            } else {
                "#f44336"
            };

            writeln!(output, "  <h2>Coverage</h2>").unwrap();
            writeln!(
                output,
                "  <div style=\"background: linear-gradient(90deg, {} {}%, #e0e0e0 {}%); padding: 15px; border-radius: 8px; margin: 20px 0;\">",
                coverage_color,
                coverage.coverage_percent,
                coverage.coverage_percent
            )
            .unwrap();
            writeln!(
                output,
                "    <span style=\"font-size: 1.5em; font-weight: bold; color: white; text-shadow: 1px 1px 2px rgba(0,0,0,0.5);\">{:.1}%</span>",
                coverage.coverage_percent
            )
            .unwrap();
            writeln!(
                output,
                "    <span style=\"color: white; text-shadow: 1px 1px 2px rgba(0,0,0,0.5);\"> ({}/{} statements)</span>",
                coverage.covered_statements,
                coverage.total_statements
            )
            .unwrap();
            writeln!(output, "  </div>").unwrap();

            // Per-file coverage table
            let incomplete_files: Vec<_> = coverage
                .files
                .iter()
                .filter(|f| f.coverage_percent < 100.0)
                .collect();

            if !incomplete_files.is_empty() {
                writeln!(output, "  <h3>Files with Incomplete Coverage</h3>").unwrap();
                writeln!(output, "  <table>").unwrap();
                writeln!(
                    output,
                    "    <tr><th>File</th><th>Coverage</th><th>Statements</th><th>Missing Lines</th></tr>"
                )
                .unwrap();

                for file in incomplete_files {
                    let file_color = if file.coverage_percent >= 80.0 {
                        "#4CAF50"
                    } else if file.coverage_percent >= 60.0 {
                        "#ff9800"
                    } else {
                        "#f44336"
                    };

                    let missing = if file.missing_lines.len() > 5 {
                        format!(
                            "{}, ... ({} more)",
                            file.missing_lines[..5]
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                            file.missing_lines.len() - 5
                        )
                    } else {
                        file.missing_lines
                            .iter()
                            .map(|n| n.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    };

                    writeln!(
                        output,
                        "    <tr><td>{}</td><td style=\"color: {}\">{:.1}%</td><td>{}/{}</td><td><code>{}</code></td></tr>",
                        file.path, file_color, file.coverage_percent, file.covered, file.statements, missing
                    )
                    .unwrap();
                }

                writeln!(output, "  </table>").unwrap();
            }

            // Uncovered files
            if !coverage.uncovered_files.is_empty() {
                writeln!(output, "  <h3>Uncovered Files (0%)</h3>").unwrap();
                writeln!(output, "  <ul>").unwrap();
                for file in &coverage.uncovered_files {
                    writeln!(output, "    <li style=\"color: #f44336;\">{}</li>", file).unwrap();
                }
                writeln!(output, "  </ul>").unwrap();
            }
        }

        // Results table
        writeln!(output, "  <h2>Test Results</h2>").unwrap();
        writeln!(output, "  <table>").unwrap();
        writeln!(output, "    <tr><th>Status</th><th>Test</th><th>Type</th><th>Duration</th><th>Details</th></tr>").unwrap();

        for result in &report.results {
            let (status_icon, status_class) = match result.status {
                TestStatus::Passed => ("✅", "status-passed"),
                TestStatus::Failed => ("❌", "status-failed"),
                TestStatus::Skipped => ("⏭️", "status-skipped"),
                TestStatus::Error => ("💥", "status-error"),
            };
            let error_detail = result
                .error
                .as_ref()
                .map(|e| e.chars().take(50).collect::<String>())
                .unwrap_or_default();

            writeln!(
                output,
                "    <tr><td class=\"{}\">{}</td><td>{}</td><td>{}</td><td>{}ms</td><td>{}</td></tr>",
                status_class, status_icon, result.meta.name, result.meta.test_type, result.duration_ms, error_detail
            )
            .unwrap();
        }

        writeln!(output, "  </table>").unwrap();

        // Failed tests details
        let failed = report.failed_results();
        if !failed.is_empty() {
            writeln!(output, "  <h2>Failed Tests Details</h2>").unwrap();
            for result in failed {
                writeln!(output, "  <div class=\"error-box\">").unwrap();
                writeln!(output, "    <h3>{}</h3>", result.meta.name).unwrap();
                if let Some(ref error) = result.error {
                    writeln!(output, "    <p><strong>Error:</strong> {}</p>", error).unwrap();
                }
                if let Some(ref trace) = result.stack_trace {
                    writeln!(output, "    <pre>{}</pre>", trace).unwrap();
                }
                writeln!(output, "  </div>").unwrap();
            }
        }

        writeln!(output, "</div>").unwrap();
        writeln!(output, "</body>").unwrap();
        writeln!(output, "</html>").unwrap();

        output
    }

    /// Generate JUnit XML report (for CI integration)
    fn generate_junit(&self, report: &TestReport) -> String {
        let mut output = String::new();

        writeln!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>").unwrap();
        writeln!(
            output,
            "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"{}\" skipped=\"{}\" time=\"{:.3}\" timestamp=\"{}\">",
            escape_xml(&report.suite_name),
            report.summary.total,
            report.summary.failed,
            report.summary.errors,
            report.summary.skipped,
            report.duration_ms as f64 / 1000.0,
            report.generated_at
        )
        .unwrap();

        for result in &report.results {
            write!(
                output,
                "  <testcase name=\"{}\" classname=\"{}\" time=\"{:.3}\"",
                escape_xml(&result.meta.name),
                escape_xml(&result.meta.full_name),
                result.duration_ms as f64 / 1000.0
            )
            .unwrap();

            // Add file and line if available
            if let Some(ref file) = result.meta.file_path {
                write!(output, " file=\"{}\"", escape_xml(file)).unwrap();
            }
            if let Some(line) = result.meta.line_number {
                write!(output, " line=\"{}\"", line).unwrap();
            }

            writeln!(output, ">").unwrap();

            match result.status {
                TestStatus::Failed => {
                    let message = result.error.as_deref().unwrap_or("Assertion failed");
                    writeln!(
                        output,
                        "    <failure message=\"{}\">{}</failure>",
                        escape_xml(message),
                        escape_xml(result.stack_trace.as_deref().unwrap_or(""))
                    )
                    .unwrap();
                }
                TestStatus::Error => {
                    let message = result.error.as_deref().unwrap_or("Test error");
                    writeln!(
                        output,
                        "    <error type=\"Error\" message=\"{}\">{}</error>",
                        escape_xml(message),
                        escape_xml(result.stack_trace.as_deref().unwrap_or(""))
                    )
                    .unwrap();
                }
                TestStatus::Skipped => {
                    let message = result.error.as_deref().unwrap_or("Skipped");
                    writeln!(
                        output,
                        "    <skipped message=\"{}\" />",
                        escape_xml(message)
                    )
                    .unwrap();
                }
                TestStatus::Passed => {}
            }

            writeln!(output, "  </testcase>").unwrap();
        }

        writeln!(output, "</testsuite>").unwrap();

        output
    }

    /// Generate YAML report
    fn generate_yaml(&self, report: &TestReport) -> String {
        let mut output = String::new();

        writeln!(output, "# Test Report").unwrap();
        writeln!(output, "suite_name: \"{}\"", report.suite_name).unwrap();
        writeln!(output, "generated_at: \"{}\"", report.generated_at).unwrap();
        writeln!(output, "duration_ms: {}", report.duration_ms).unwrap();
        writeln!(output).unwrap();

        // Summary
        writeln!(output, "summary:").unwrap();
        writeln!(output, "  total: {}", report.summary.total).unwrap();
        writeln!(output, "  passed: {}", report.summary.passed).unwrap();
        writeln!(output, "  failed: {}", report.summary.failed).unwrap();
        writeln!(output, "  skipped: {}", report.summary.skipped).unwrap();
        writeln!(output, "  errors: {}", report.summary.errors).unwrap();
        writeln!(
            output,
            "  total_duration_ms: {}",
            report.summary.total_duration_ms
        )
        .unwrap();
        writeln!(output, "  pass_rate: {:.2}", report.summary.pass_rate()).unwrap();
        writeln!(output).unwrap();

        // Coverage (if present)
        if let Some(ref coverage) = report.coverage {
            writeln!(output, "coverage:").unwrap();
            writeln!(output, "  total_statements: {}", coverage.total_statements).unwrap();
            writeln!(
                output,
                "  covered_statements: {}",
                coverage.covered_statements
            )
            .unwrap();
            writeln!(
                output,
                "  coverage_percent: {:.2}",
                coverage.coverage_percent
            )
            .unwrap();
            if !coverage.files.is_empty() {
                writeln!(output, "  files:").unwrap();
                for file in &coverage.files {
                    writeln!(output, "    - path: \"{}\"", file.path).unwrap();
                    writeln!(output, "      statements: {}", file.statements).unwrap();
                    writeln!(output, "      covered: {}", file.covered).unwrap();
                    writeln!(
                        output,
                        "      coverage_percent: {:.2}",
                        file.coverage_percent
                    )
                    .unwrap();
                    if !file.missing_lines.is_empty() {
                        let missing: Vec<String> =
                            file.missing_lines.iter().map(|n| n.to_string()).collect();
                        writeln!(output, "      missing_lines: [{}]", missing.join(", ")).unwrap();
                    }
                }
            }
            if !coverage.uncovered_files.is_empty() {
                writeln!(output, "  uncovered_files:").unwrap();
                for file in &coverage.uncovered_files {
                    writeln!(output, "    - \"{}\"", file).unwrap();
                }
            }
            writeln!(output).unwrap();
        }

        // Results
        writeln!(output, "results:").unwrap();
        for result in &report.results {
            writeln!(output, "  - name: \"{}\"", result.meta.name).unwrap();
            writeln!(output, "    status: \"{}\"", result.status).unwrap();
            writeln!(output, "    duration_ms: {}", result.duration_ms).unwrap();
            writeln!(output, "    test_type: \"{}\"", result.meta.test_type).unwrap();
            if let Some(ref error) = result.error {
                writeln!(output, "    error: \"{}\"", error.replace('"', "\\\"")).unwrap();
            }
        }

        // Environment
        if report.environment.python_version.is_some() || report.environment.platform.is_some() {
            writeln!(output).unwrap();
            writeln!(output, "environment:").unwrap();
            if let Some(ref py) = report.environment.python_version {
                writeln!(output, "  python_version: \"{}\"", py).unwrap();
            }
            if let Some(ref rust) = report.environment.rust_version {
                writeln!(output, "  rust_version: \"{}\"", rust).unwrap();
            }
            if let Some(ref platform) = report.environment.platform {
                writeln!(output, "  platform: \"{}\"", platform).unwrap();
            }
            if let Some(ref host) = report.environment.hostname {
                writeln!(output, "  hostname: \"{}\"", host).unwrap();
            }
        }

        output
    }

    /// Generate Console report (colored terminal output)
    fn generate_console(&self, report: &TestReport) -> String {
        let mut output = String::new();

        // ANSI color codes
        const RESET: &str = "\x1b[0m";
        const BOLD: &str = "\x1b[1m";
        const GREEN: &str = "\x1b[32m";
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const CYAN: &str = "\x1b[36m";
        const DIM: &str = "\x1b[2m";

        // Header
        let status_color = if report.summary.all_passed() {
            GREEN
        } else {
            RED
        };
        let status_text = if report.summary.all_passed() {
            "PASSED"
        } else {
            "FAILED"
        };
        writeln!(
            output,
            "\n{}{}═══ Test Report: {} ═══{}",
            BOLD, status_color, report.suite_name, RESET
        )
        .unwrap();
        writeln!(
            output,
            "{}Status: {}{}{}  |  Duration: {:.2}s  |  Date: {}{}",
            DIM,
            status_color,
            status_text,
            DIM,
            report.duration_ms as f64 / 1000.0,
            &report.generated_at[..10],
            RESET
        )
        .unwrap();

        // Summary bar
        writeln!(output).unwrap();
        writeln!(output, "{}{}Summary:{}", BOLD, CYAN, RESET).unwrap();
        writeln!(
            output,
            "  {}✓ Passed: {}{}  {}✗ Failed: {}{}  {}⊘ Skipped: {}{}  {}! Errors: {}{}",
            GREEN,
            report.summary.passed,
            RESET,
            RED,
            report.summary.failed,
            RESET,
            YELLOW,
            report.summary.skipped,
            RESET,
            RED,
            report.summary.errors,
            RESET
        )
        .unwrap();

        // Coverage (if present)
        if let Some(ref coverage) = report.coverage {
            writeln!(output).unwrap();
            writeln!(output, "{}{}Coverage:{}", BOLD, CYAN, RESET).unwrap();

            let cov_color = if coverage.coverage_percent >= 80.0 {
                GREEN
            } else if coverage.coverage_percent >= 60.0 {
                YELLOW
            } else {
                RED
            };

            // Progress bar
            let bar_width = 40;
            let filled = (coverage.coverage_percent / 100.0 * bar_width as f64) as usize;
            let empty = bar_width - filled;
            writeln!(
                output,
                "  {}[{}{}{}] {}{:.1}%{} ({}/{} statements)",
                DIM,
                cov_color,
                "█".repeat(filled),
                "░".repeat(empty),
                cov_color,
                coverage.coverage_percent,
                RESET,
                coverage.covered_statements,
                coverage.total_statements
            )
            .unwrap();

            // Low coverage files
            let low_coverage: Vec<_> = coverage
                .files
                .iter()
                .filter(|f| f.coverage_percent < 50.0)
                .take(5)
                .collect();
            if !low_coverage.is_empty() {
                writeln!(output, "  {}Low coverage files:{}", DIM, RESET).unwrap();
                for file in low_coverage {
                    writeln!(
                        output,
                        "    {}{}{} {:.1}%{}",
                        RED, file.path, DIM, file.coverage_percent, RESET
                    )
                    .unwrap();
                }
            }
        }

        // Test results
        writeln!(output).unwrap();
        writeln!(output, "{}{}Test Results:{}", BOLD, CYAN, RESET).unwrap();
        for result in &report.results {
            let (icon, color) = match result.status {
                TestStatus::Passed => ("✓", GREEN),
                TestStatus::Failed => ("✗", RED),
                TestStatus::Skipped => ("⊘", YELLOW),
                TestStatus::Error => ("!", RED),
            };
            writeln!(
                output,
                "  {}{} {}{}  {}{}ms{}",
                color, icon, result.meta.name, RESET, DIM, result.duration_ms, RESET
            )
            .unwrap();

            if let Some(ref error) = result.error {
                writeln!(output, "    {}→ {}{}", DIM, error, RESET).unwrap();
            }
        }

        // Footer
        writeln!(output).unwrap();
        writeln!(
            output,
            "{}────────────────────────────────────────{}",
            DIM, RESET
        )
        .unwrap();
        writeln!(
            output,
            "{}Total: {} tests  |  Pass rate: {:.1}%{}",
            DIM,
            report.summary.total,
            report.summary.pass_rate() * 100.0,
            RESET
        )
        .unwrap();

        output
    }

    // ==================== Agent Evaluation Reporting ====================

    /// Generate agent evaluation report
    pub fn generate_agent_eval(&self, report: &AgentEvalReport) -> String {
        match self.format {
            ReportFormat::Markdown => self.generate_agent_eval_markdown(report),
            ReportFormat::Html => self.generate_agent_eval_html(report),
            ReportFormat::Json => self.generate_agent_eval_json(report),
            ReportFormat::Yaml => self.generate_agent_eval_yaml(report),
            ReportFormat::JUnit => self.generate_agent_eval_junit(report),
            ReportFormat::Console => self.generate_agent_eval_console(report),
        }
    }

    /// Generate Markdown report for agent evaluation
    fn generate_agent_eval_markdown(&self, report: &AgentEvalReport) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "# Agent Evaluation Report: {}", report.suite_name).unwrap();
        writeln!(output).unwrap();

        // Metadata
        let status_emoji = if report.all_passed() {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        };
        writeln!(
            output,
            "**Date**: {} | **Status**: {} ({}/{})",
            &report.generated_at[..10],
            status_emoji,
            report.metrics.passed,
            report.metrics.total_cases
        )
        .unwrap();
        writeln!(output).unwrap();

        // Summary table
        writeln!(output, "## Summary").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "| Metric | Value |").unwrap();
        writeln!(output, "|--------|-------|").unwrap();
        writeln!(output, "| Total Cases | {} |", report.metrics.total_cases).unwrap();
        writeln!(output, "| Passed | {} |", report.metrics.passed).unwrap();
        writeln!(
            output,
            "| Pass Rate | {:.1}% |",
            report.metrics.pass_rate * 100.0
        )
        .unwrap();
        writeln!(
            output,
            "| Correctness Rate | {:.1}% |",
            report.metrics.correctness.rate * 100.0
        )
        .unwrap();
        writeln!(
            output,
            "| Avg Latency (P50) | {:.0}ms |",
            report.metrics.latency_stats.median_ms
        )
        .unwrap();
        writeln!(
            output,
            "| Avg Latency (P95) | {:.0}ms |",
            report.metrics.latency_stats.p95_ms
        )
        .unwrap();
        writeln!(
            output,
            "| Total Cost | ${:.4} |",
            report.metrics.cost_stats.total_cost_usd
        )
        .unwrap();
        writeln!(
            output,
            "| Avg Cost/Case | ${:.4} |",
            report.metrics.cost_stats.avg_cost_per_case_usd
        )
        .unwrap();
        writeln!(output).unwrap();

        // Tool accuracy
        writeln!(output, "## Tool Accuracy").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "| Metric | Value |").unwrap();
        writeln!(output, "|--------|-------|").unwrap();
        writeln!(
            output,
            "| Precision | {:.2} |",
            report.metrics.tool_usage.avg_precision
        )
        .unwrap();
        writeln!(
            output,
            "| Recall | {:.2} |",
            report.metrics.tool_usage.avg_recall
        )
        .unwrap();
        writeln!(
            output,
            "| F1 Score | {:.2} |",
            report.metrics.tool_usage.avg_f1_score
        )
        .unwrap();
        writeln!(output).unwrap();

        // Quality scores (if available)
        if let Some(ref quality) = report.metrics.quality {
            writeln!(output, "## Quality Assessment").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "| Metric | Value |").unwrap();
            writeln!(output, "|--------|-------|").unwrap();
            writeln!(
                output,
                "| Overall Score | {:.2} |",
                quality.overall_avg_score
            )
            .unwrap();
            for (criterion, score) in &quality.avg_scores_by_criterion {
                writeln!(output, "| {} | {:.2} |", criterion, score).unwrap();
            }
            writeln!(output).unwrap();
        }

        // Failed tests
        let failed = report.failed_results();
        if !failed.is_empty() {
            writeln!(
                output,
                "## Failed Tests ({}/{})",
                failed.len(),
                report.metrics.total_cases
            )
            .unwrap();
            writeln!(output).unwrap();
            for result in failed {
                writeln!(
                    output,
                    "### ❌ {} ({})",
                    result.test_case_name, result.test_case_id
                )
                .unwrap();
                if let Some(ref reason) = result.failure_reason {
                    writeln!(output, "**Reason**: {}", reason).unwrap();
                }
                writeln!(
                    output,
                    "- **Correctness**: {}",
                    if result.correctness.matches {
                        "✅"
                    } else {
                        "❌"
                    }
                )
                .unwrap();
                writeln!(
                    output,
                    "- **Tool F1**: {:.2}",
                    result.tool_accuracy.f1_score
                )
                .unwrap();
                writeln!(output, "- **Latency**: {:.0}ms", result.latency.total_ms).unwrap();
                writeln!(output, "- **Cost**: ${:.4}", result.cost.total_cost_usd).unwrap();
                writeln!(output).unwrap();
            }
        }

        output
    }

    /// Generate HTML report for agent evaluation
    fn generate_agent_eval_html(&self, report: &AgentEvalReport) -> String {
        let status_color = if report.all_passed() {
            "#22c55e"
        } else {
            "#ef4444"
        };
        let status_text = if report.all_passed() {
            "PASSED"
        } else {
            "FAILED"
        };

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Agent Evaluation Report - {suite_name}</title>
    <style>
        body {{ font-family: system-ui, -apple-system, sans-serif; margin: 40px; background: #f9fafb; }}
        .header {{ background: white; padding: 24px; border-radius: 8px; margin-bottom: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .status {{ display: inline-block; padding: 6px 12px; border-radius: 6px; color: white; background: {status_color}; font-weight: 600; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 24px; }}
        .metric-card {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .metric-label {{ font-size: 14px; color: #6b7280; margin-bottom: 8px; }}
        .metric-value {{ font-size: 28px; font-weight: 700; color: #111827; }}
        .section {{ background: white; padding: 24px; border-radius: 8px; margin-bottom: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .section-title {{ font-size: 20px; font-weight: 600; margin-bottom: 16px; color: #111827; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #e5e7eb; }}
        th {{ background: #f9fafb; font-weight: 600; color: #374151; }}
        .failed-test {{ border-left: 4px solid #ef4444; padding-left: 16px; margin-bottom: 16px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Agent Evaluation Report: {suite_name}</h1>
        <p><strong>Date:</strong> {date} | <strong>Status:</strong> <span class="status">{status_text}</span></p>
        <p><strong>Pass Rate:</strong> {pass_rate:.1}% ({passed}/{total} tests)</p>
    </div>

    <div class="metrics">
        <div class="metric-card">
            <div class="metric-label">Correctness Rate</div>
            <div class="metric-value">{correctness_rate:.1}%</div>
        </div>
        <div class="metric-card">
            <div class="metric-label">Avg Latency (P95)</div>
            <div class="metric-value">{latency_p95:.0}ms</div>
        </div>
        <div class="metric-card">
            <div class="metric-label">Total Cost</div>
            <div class="metric-value">${total_cost:.4}</div>
        </div>
        <div class="metric-card">
            <div class="metric-label">Tool F1 Score</div>
            <div class="metric-value">{tool_f1:.2}</div>
        </div>
    </div>

    <div class="section">
        <div class="section-title">Test Results</div>
        <table>
            <thead>
                <tr>
                    <th>Test ID</th>
                    <th>Name</th>
                    <th>Status</th>
                    <th>Latency</th>
                    <th>Cost</th>
                    <th>Tool F1</th>
                </tr>
            </thead>
            <tbody>
                {rows}
            </tbody>
        </table>
    </div>
</body>
</html>"#,
            suite_name = report.suite_name,
            date = &report.generated_at[..10],
            status_text = status_text,
            status_color = status_color,
            pass_rate = report.metrics.pass_rate * 100.0,
            passed = report.metrics.passed,
            total = report.metrics.total_cases,
            correctness_rate = report.metrics.correctness.rate * 100.0,
            latency_p95 = report.metrics.latency_stats.p95_ms,
            total_cost = report.metrics.cost_stats.total_cost_usd,
            tool_f1 = report.metrics.tool_usage.avg_f1_score,
            rows = report.results.iter().map(|r| {
                let status_icon = if r.passed { "✅" } else { "❌" };
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:.0}ms</td><td>${:.4}</td><td>{:.2}</td></tr>",
                    r.test_case_id, r.test_case_name, status_icon,
                    r.latency.total_ms, r.cost.total_cost_usd, r.tool_accuracy.f1_score
                )
            }).collect::<Vec<_>>().join("\n                ")
        )
    }

    /// Generate JSON report for agent evaluation
    fn generate_agent_eval_json(&self, report: &AgentEvalReport) -> String {
        serde_json::to_string_pretty(report)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize report: {}\"}}", e))
    }

    /// Generate YAML report for agent evaluation
    fn generate_agent_eval_yaml(&self, report: &AgentEvalReport) -> String {
        serde_yaml::to_string(report)
            .unwrap_or_else(|e| format!("error: Failed to serialize report: {}", e))
    }

    /// Generate JUnit XML report for agent evaluation
    fn generate_agent_eval_junit(&self, report: &AgentEvalReport) -> String {
        let mut output = String::new();

        // XML header
        writeln!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>").unwrap();

        // Testsuite
        let total_time = report
            .results
            .iter()
            .map(|r| r.latency.total_ms)
            .sum::<f64>()
            / 1000.0;
        writeln!(
            output,
            "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"0\" time=\"{:.3}\" timestamp=\"{}\">",
            escape_xml(&report.suite_name),
            report.metrics.total_cases,
            report.metrics.total_cases - report.metrics.passed,
            total_time,
            report.generated_at
        )
        .unwrap();

        // Test cases
        for result in &report.results {
            let time = result.latency.total_ms / 1000.0;
            writeln!(
                output,
                "  <testcase name=\"{}\" classname=\"agent_eval\" time=\"{:.3}\">",
                escape_xml(&result.test_case_name),
                time
            )
            .unwrap();

            if !result.passed {
                if let Some(ref reason) = result.failure_reason {
                    writeln!(
                        output,
                        "    <failure message=\"{}\" type=\"AgentEvalFailure\">",
                        escape_xml(reason)
                    )
                    .unwrap();
                    writeln!(output, "Test Case: {}", escape_xml(&result.test_case_id)).unwrap();
                    writeln!(output, "Correctness: {}", result.correctness.matches).unwrap();
                    writeln!(output, "Tool F1: {:.2}", result.tool_accuracy.f1_score).unwrap();
                    writeln!(output, "Latency: {:.0}ms", result.latency.total_ms).unwrap();
                    writeln!(output, "Cost: ${:.4}", result.cost.total_cost_usd).unwrap();
                    writeln!(output, "    </failure>").unwrap();
                }
            }

            writeln!(output, "  </testcase>").unwrap();
        }

        writeln!(output, "</testsuite>").unwrap();
        output
    }

    /// Generate console report for agent evaluation
    fn generate_agent_eval_console(&self, report: &AgentEvalReport) -> String {
        const GREEN: &str = "\x1b[32m";
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const CYAN: &str = "\x1b[36m";
        const BOLD: &str = "\x1b[1m";
        const DIM: &str = "\x1b[2m";
        const RESET: &str = "\x1b[0m";

        let mut output = String::new();

        // Header
        writeln!(
            output,
            "{}════════════════════════════════════════{}",
            CYAN, RESET
        )
        .unwrap();
        writeln!(output, "{}{}Agent Evaluation Report{}", BOLD, CYAN, RESET).unwrap();
        writeln!(
            output,
            "{}════════════════════════════════════════{}",
            CYAN, RESET
        )
        .unwrap();
        writeln!(output).unwrap();

        // Status
        let (status_color, status_text) = if report.all_passed() {
            (GREEN, "✓ PASSED")
        } else {
            (RED, "✗ FAILED")
        };
        writeln!(
            output,
            "{}{}{} {}/{} tests passed ({:.1}%){}",
            BOLD,
            status_color,
            status_text,
            report.metrics.passed,
            report.metrics.total_cases,
            report.metrics.pass_rate * 100.0,
            RESET
        )
        .unwrap();
        writeln!(output).unwrap();

        // Metrics
        writeln!(output, "{}Metrics:{}", BOLD, RESET).unwrap();
        writeln!(
            output,
            "  {}Correctness Rate:{}    {:.1}%",
            DIM,
            RESET,
            report.metrics.correctness.rate * 100.0
        )
        .unwrap();
        writeln!(
            output,
            "  {}Latency (P50/P95):{} {:.0}ms / {:.0}ms",
            DIM, RESET, report.metrics.latency_stats.median_ms, report.metrics.latency_stats.p95_ms
        )
        .unwrap();
        writeln!(
            output,
            "  {}Total Cost:{}        ${:.4}",
            DIM, RESET, report.metrics.cost_stats.total_cost_usd
        )
        .unwrap();
        writeln!(
            output,
            "  {}Tool F1 Score:{}     {:.2}",
            DIM, RESET, report.metrics.tool_usage.avg_f1_score
        )
        .unwrap();
        writeln!(output).unwrap();

        // Failed tests
        let failed = report.failed_results();
        if !failed.is_empty() {
            writeln!(output, "{}Failed Tests:{}", YELLOW, RESET).unwrap();
            for result in failed.iter().take(5) {
                writeln!(
                    output,
                    "  {}{}✗{} {}",
                    RED, BOLD, RESET, result.test_case_name
                )
                .unwrap();
                if let Some(ref reason) = result.failure_reason {
                    writeln!(output, "    {}→ {}{}", DIM, reason, RESET).unwrap();
                }
            }
            if failed.len() > 5 {
                writeln!(
                    output,
                    "  {}... and {} more{}",
                    DIM,
                    failed.len() - 5,
                    RESET
                )
                .unwrap();
            }
        }

        writeln!(output).unwrap();
        writeln!(
            output,
            "{}────────────────────────────────────────{}",
            DIM, RESET
        )
        .unwrap();

        output
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::TestMeta;

    #[test]
    fn test_report_creation() {
        let results = vec![
            TestResult::passed(TestMeta::new("test_one"), 100),
            TestResult::passed(TestMeta::new("test_two"), 200),
            TestResult::failed(TestMeta::new("test_three"), 50, "assertion failed"),
        ];

        let report = TestReport::new("MyTestSuite", results);

        assert_eq!(report.suite_name, "MyTestSuite");
        assert_eq!(report.summary.total, 3);
        assert_eq!(report.summary.passed, 2);
        assert_eq!(report.summary.failed, 1);
        assert!(!report.summary.all_passed());
    }

    #[test]
    fn test_markdown_generation() {
        let results = vec![TestResult::passed(TestMeta::new("test_example"), 100)];
        let report = TestReport::new("ExampleSuite", results);
        let reporter = Reporter::markdown();
        let markdown = reporter.generate(&report);

        assert!(markdown.contains("# Test Report: ExampleSuite"));
        assert!(markdown.contains("✅ PASSED"));
        assert!(markdown.contains("test_example"));
    }

    #[test]
    fn test_json_generation() {
        let results = vec![TestResult::passed(TestMeta::new("test_json"), 50)];
        let report = TestReport::new("JsonSuite", results);
        let reporter = Reporter::json();
        let json = reporter.generate(&report);

        assert!(json.contains("\"suite_name\": \"JsonSuite\""));
        assert!(json.contains("\"test_json\""));
    }

    #[test]
    fn test_junit_generation() {
        let results = vec![
            TestResult::passed(TestMeta::new("test_pass"), 100),
            TestResult::failed(TestMeta::new("test_fail"), 50, "oops"),
        ];
        let report = TestReport::new("JUnitSuite", results);
        let reporter = Reporter::junit();
        let xml = reporter.generate(&report);

        assert!(xml.contains("<testsuite"));
        assert!(xml.contains("tests=\"2\""));
        assert!(xml.contains("failures=\"1\""));
        assert!(xml.contains("<failure"));
    }

    #[test]
    fn test_junit_enhanced_attributes() {
        // Test with file_path and line_number
        let mut meta1 = TestMeta::new("test_with_location");
        meta1.file_path = Some("src/lib.rs".to_string());
        meta1.line_number = Some(42);
        meta1.full_name = "my_module::test_with_location".to_string();

        let mut meta2 = TestMeta::new("test_error");
        meta2.file_path = Some("src/error.rs".to_string());
        meta2.line_number = Some(100);
        meta2.full_name = "my_module::test_error".to_string();

        let results = vec![
            TestResult::passed(meta1, 150),
            TestResult {
                meta: meta2,
                status: TestStatus::Error,
                duration_ms: 25,
                error: Some("Runtime error".to_string()),
                stack_trace: Some("stack trace".to_string()),
                profile_metrics: None,
                started_at: chrono::Utc::now().to_rfc3339(),
            },
        ];

        let report = TestReport::new("EnhancedSuite", results);
        let reporter = Reporter::junit();
        let xml = reporter.generate(&report);

        // Verify timestamp attribute on testsuite
        assert!(xml.contains("timestamp=\""));

        // Verify file and line attributes on testcase
        assert!(xml.contains("file=\"src/lib.rs\""));
        assert!(xml.contains("line=\"42\""));
        assert!(xml.contains("file=\"src/error.rs\""));
        assert!(xml.contains("line=\"100\""));

        // Verify type attribute on error element
        assert!(xml.contains("<error type=\"Error\""));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500B");
        assert_eq!(format_bytes(1024), "1.00KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00GB");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_agent_eval_report_creation() {
        use crate::agent_eval::result::*;
        use crate::benchmark::BenchmarkStats;
        use chrono::Utc;

        let results = vec![AgentEvalResult {
            test_case_id: "test-001".to_string(),
            test_case_name: "Test 1".to_string(),
            passed: true,
            actual_output: "output".to_string(),
            correctness: CorrectnessResult::passed(MatchType::Exact),
            tool_accuracy: ToolAccuracyResult {
                precision: 1.0,
                recall: 1.0,
                f1_score: 1.0,
                missing_tools: Vec::new(),
                unexpected_tools: Vec::new(),
            },
            quality_scores: None,
            latency: LatencyMetrics {
                total_ms: 100.0,
                within_budget: true,
                budget_ms: None,
            },
            cost: CostMetrics {
                total_cost_usd: 0.001,
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                within_budget: true,
                budget_usd: None,
                model: "gpt-4o-mini".to_string(),
            },
            timestamp: Utc::now(),
            failure_reason: None,
        }];

        let mut by_match_type = std::collections::HashMap::new();
        by_match_type.insert("Exact".to_string(), 1);

        let metrics = AgentEvalMetrics {
            total_cases: 1,
            passed: 1,
            pass_rate: 1.0,
            correctness: CorrectnessMetrics {
                total: 1,
                correct: 1,
                rate: 1.0,
                by_match_type,
            },
            tool_usage: ToolUsageMetrics {
                avg_precision: 1.0,
                avg_recall: 1.0,
                avg_f1_score: 1.0,
                total_tools_called: 0,
                total_expected_tools: 0,
            },
            quality: None,
            latency_stats: BenchmarkStats::default(),
            cost_stats: CostStats {
                total_cost_usd: 0.001,
                avg_cost_per_case_usd: 0.001,
                total_tokens: 15,
                total_prompt_tokens: 10,
                total_completion_tokens: 5,
                min_cost_usd: 0.001,
                max_cost_usd: 0.001,
            },
        };

        let report = AgentEvalReport::new("TestSuite", results, metrics);

        assert_eq!(report.suite_name, "TestSuite");
        assert_eq!(report.results.len(), 1);
        assert!(report.all_passed());
    }

    #[test]
    fn test_agent_eval_markdown_generation() {
        use crate::agent_eval::result::*;
        use crate::benchmark::BenchmarkStats;
        use chrono::Utc;

        let results = vec![AgentEvalResult {
            test_case_id: "test-001".to_string(),
            test_case_name: "Capital question".to_string(),
            passed: true,
            actual_output: "Paris".to_string(),
            correctness: CorrectnessResult::passed(MatchType::Regex),
            tool_accuracy: ToolAccuracyResult {
                precision: 1.0,
                recall: 1.0,
                f1_score: 1.0,
                missing_tools: Vec::new(),
                unexpected_tools: Vec::new(),
            },
            quality_scores: None,
            latency: LatencyMetrics {
                total_ms: 1000.0,
                within_budget: true,
                budget_ms: Some(2000.0),
            },
            cost: CostMetrics {
                total_cost_usd: 0.0015,
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                within_budget: true,
                budget_usd: Some(0.01),
                model: "gpt-4o-mini".to_string(),
            },
            timestamp: Utc::now(),
            failure_reason: None,
        }];

        let mut by_match_type = std::collections::HashMap::new();
        by_match_type.insert("Regex".to_string(), 1);

        let metrics = AgentEvalMetrics {
            total_cases: 1,
            passed: 1,
            pass_rate: 1.0,
            correctness: CorrectnessMetrics {
                total: 1,
                correct: 1,
                rate: 1.0,
                by_match_type,
            },
            tool_usage: ToolUsageMetrics {
                avg_precision: 1.0,
                avg_recall: 1.0,
                avg_f1_score: 1.0,
                total_tools_called: 0,
                total_expected_tools: 0,
            },
            quality: None,
            latency_stats: BenchmarkStats::default(),
            cost_stats: CostStats {
                total_cost_usd: 0.0015,
                avg_cost_per_case_usd: 0.0015,
                total_tokens: 150,
                total_prompt_tokens: 100,
                total_completion_tokens: 50,
                min_cost_usd: 0.0015,
                max_cost_usd: 0.0015,
            },
        };

        let report = AgentEvalReport::new("AgentSuite", results, metrics);
        let reporter = Reporter::markdown();
        let output = reporter.generate_agent_eval(&report);

        assert!(output.contains("# Agent Evaluation Report: AgentSuite"));
        assert!(output.contains("✅ PASSED"));
        assert!(output.contains("Pass Rate"));
        assert!(output.contains("100.0%"));
        assert!(output.contains("Correctness Rate"));
        assert!(output.contains("Tool Accuracy"));
    }

    #[test]
    fn test_agent_eval_json_generation() {
        use crate::agent_eval::result::*;
        use crate::benchmark::BenchmarkStats;

        let results = vec![];
        let metrics = AgentEvalMetrics {
            total_cases: 0,
            passed: 0,
            pass_rate: 0.0,
            correctness: CorrectnessMetrics {
                total: 0,
                correct: 0,
                rate: 0.0,
                by_match_type: std::collections::HashMap::new(),
            },
            tool_usage: ToolUsageMetrics {
                avg_precision: 0.0,
                avg_recall: 0.0,
                avg_f1_score: 0.0,
                total_tools_called: 0,
                total_expected_tools: 0,
            },
            quality: None,
            latency_stats: BenchmarkStats::default(),
            cost_stats: CostStats {
                total_cost_usd: 0.0,
                avg_cost_per_case_usd: 0.0,
                total_tokens: 0,
                total_prompt_tokens: 0,
                total_completion_tokens: 0,
                min_cost_usd: 0.0,
                max_cost_usd: 0.0,
            },
        };

        let report = AgentEvalReport::new("JSONSuite", results, metrics);
        let reporter = Reporter::json();
        let output = reporter.generate_agent_eval(&report);

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["suite_name"], "JSONSuite");
        assert_eq!(parsed["metrics"]["total_cases"], 0);
    }

    #[test]
    fn test_agent_eval_junit_generation() {
        use crate::agent_eval::result::*;
        use crate::benchmark::BenchmarkStats;
        use chrono::Utc;

        let results = vec![
            AgentEvalResult {
                test_case_id: "test-001".to_string(),
                test_case_name: "Test Pass".to_string(),
                passed: true,
                actual_output: "output".to_string(),
                correctness: CorrectnessResult::passed(MatchType::Exact),
                tool_accuracy: ToolAccuracyResult {
                    precision: 1.0,
                    recall: 1.0,
                    f1_score: 1.0,
                    missing_tools: Vec::new(),
                    unexpected_tools: Vec::new(),
                },
                quality_scores: None,
                latency: LatencyMetrics {
                    total_ms: 100.0,
                    within_budget: true,
                    budget_ms: None,
                },
                cost: CostMetrics {
                    total_cost_usd: 0.001,
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                    within_budget: true,
                    budget_usd: None,
                    model: "gpt-4o-mini".to_string(),
                },
                timestamp: Utc::now(),
                failure_reason: None,
            },
            AgentEvalResult {
                test_case_id: "test-002".to_string(),
                test_case_name: "Test Fail".to_string(),
                passed: false,
                actual_output: "wrong".to_string(),
                correctness: CorrectnessResult::failed("mismatch".to_string()),
                tool_accuracy: ToolAccuracyResult {
                    precision: 0.5,
                    recall: 0.5,
                    f1_score: 0.5,
                    missing_tools: Vec::new(),
                    unexpected_tools: Vec::new(),
                },
                quality_scores: None,
                latency: LatencyMetrics {
                    total_ms: 200.0,
                    within_budget: true,
                    budget_ms: None,
                },
                cost: CostMetrics {
                    total_cost_usd: 0.002,
                    prompt_tokens: 20,
                    completion_tokens: 10,
                    total_tokens: 30,
                    within_budget: true,
                    budget_usd: None,
                    model: "gpt-4o-mini".to_string(),
                },
                timestamp: Utc::now(),
                failure_reason: Some("Correctness check failed".to_string()),
            },
        ];

        let mut by_match_type = std::collections::HashMap::new();
        by_match_type.insert("Exact".to_string(), 1);

        let metrics = AgentEvalMetrics {
            total_cases: 2,
            passed: 1,
            pass_rate: 0.5,
            correctness: CorrectnessMetrics {
                total: 2,
                correct: 1,
                rate: 0.5,
                by_match_type,
            },
            tool_usage: ToolUsageMetrics {
                avg_precision: 0.75,
                avg_recall: 0.75,
                avg_f1_score: 0.75,
                total_tools_called: 0,
                total_expected_tools: 0,
            },
            quality: None,
            latency_stats: BenchmarkStats::default(),
            cost_stats: CostStats {
                total_cost_usd: 0.003,
                avg_cost_per_case_usd: 0.0015,
                total_tokens: 45,
                total_prompt_tokens: 30,
                total_completion_tokens: 15,
                min_cost_usd: 0.001,
                max_cost_usd: 0.002,
            },
        };

        let report = AgentEvalReport::new("JUnitSuite", results, metrics);
        let reporter = Reporter::junit();
        let output = reporter.generate_agent_eval(&report);

        assert!(output.contains("<testsuite"));
        assert!(output.contains("tests=\"2\""));
        assert!(output.contains("failures=\"1\""));
        assert!(output.contains("<failure"));
        assert!(output.contains("Correctness check failed"));
    }
}
// CODEGEN-END
