// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Reporters: `TermReporter` (stdout) + `JsonReporter` (.jet/test-results.json).
//!
//! ## Result schema (`jet.test.result.v1`)
//!
//! The JSON written to `.jet/test-results.json` is keyed by
//! [`SCHEMA_VERSION`] so downstream tools can detect drift. The schema is
//! additive — new optional fields may appear without bumping the version;
//! breaking changes bump the major. See parent #2594 result/evidence epic.

use crate::test_runner::config::{Reporter, RunnerConfig};
use crate::test_runner::coverage::CoverageSummary;
use crate::test_runner::discovery::SpecFile;
use crate::test_runner::wire::WorkerEvent;
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

/// Stable schema tag for the `jet test` result envelope. Embedded in the
/// JSON reporter output so consumers can detect schema drift.
// @spec #2604 — stabilize jet test result schema and text output
pub const SCHEMA_VERSION: &str = "jet.test.result.v1";

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Passed,
    Failed,
    Skipped,
    TimedOut,
    Crashed,
}

/// Source-file location parsed out of an error stack — `file:line:col`.
/// `None` when the worker did not attach a stack we could parse.
// @spec #2610 — attach source locations and rerun commands to failures
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Error details stored in `TestReport` for HTML reporter consumption.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
#[derive(Debug, Clone, Serialize)]
pub struct TestError {
    pub message: String,
    /// Full stack trace string from the worker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// Structured diff from an `expect()` failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    /// First user-frame location parsed out of `stack`, if any. Always points
    /// at the failing spec file (frames inside the @jet/test runtime are
    /// stripped).
    // @spec #2610
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl SourceLocation {
    /// Parse a V8/Node stack frame string and extract the first user-visible
    /// `file:line:col`. Frames pointing at the in-tree `@jet/test` runtime
    /// (paths containing `/node_modules/@jet/`) and synthetic boot wrappers
    /// (`__jet_boot.mjs`, `__jet_spec.mjs`) are skipped so callers get the
    /// spec line that actually failed.
    ///
    /// Returns `None` if no frame matches the expected `at fn (path:line:col)`
    /// shape.
    // @spec #2610
    pub fn parse_from_stack(stack: &str) -> Option<Self> {
        for raw_line in stack.lines() {
            let line = raw_line.trim_start();
            if !line.starts_with("at ") {
                continue;
            }
            // Two shapes:
            //   at fn (file:line:col)
            //   at file:line:col
            let after_at = &line[3..];
            let frame_target = if let Some(open) = after_at.rfind('(') {
                let close = after_at.rfind(')')?;
                &after_at[open + 1..close]
            } else {
                after_at
            };

            if frame_target.contains("/node_modules/@jet/")
                || frame_target.contains("__jet_boot.mjs")
                || frame_target.contains("__jet_spec.mjs")
                || frame_target.contains("/node_modules/@playwright/")
            {
                continue;
            }

            // Strip file:// prefix if present.
            let frame_target = frame_target.strip_prefix("file://").unwrap_or(frame_target);

            // Walk from the end to find ":line:col" (col optional).
            let (path, line_no, col_no) = parse_path_line_col(frame_target)?;
            return Some(SourceLocation {
                file: PathBuf::from(path),
                line: line_no,
                column: col_no,
            });
        }
        None
    }
}

fn parse_path_line_col(s: &str) -> Option<(&str, u32, Option<u32>)> {
    let last_colon = s.rfind(':')?;
    let (head, tail_a) = s.split_at(last_colon);
    let tail_a = &tail_a[1..]; // skip the colon
    let num_a = tail_a.parse::<u32>().ok()?;
    // If the head has another :number suffix, treat num_a as column.
    if let Some(second_colon) = head.rfind(':') {
        let (path, tail_b) = head.split_at(second_colon);
        let tail_b = &tail_b[1..];
        if let Ok(line) = tail_b.parse::<u32>() {
            return Some((path, line, Some(num_a)));
        }
    }
    // Otherwise num_a is the line, no column.
    Some((head, num_a, None))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct TestReport {
    pub file: PathBuf,
    pub suite: Vec<String>,
    pub name: String,
    pub outcome: Outcome,
    pub duration_ms: u64,
    /// Structured error info (message + optional stack + optional diff).
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R3
    pub error: Option<TestError>,
    /// Path to the trace archive for this test, if trace capture was enabled.
    /// Used by the HTML reporter to embed per-test deep-link trace view URLs.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R4
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_path: Option<PathBuf>,
    /// 1-indexed shard number when `--shard=i/N` is active. `null` in serial runs.
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R8
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_index: Option<u32>,
    /// Total shard count N when `--shard=i/N` is active. `null` in serial runs.
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R8
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_total: Option<u32>,
    /// Absolute paths to artifacts captured on test failure — screenshots
    /// today, video/trace later. Empty for passing tests.
    // @spec .aw/tech-design/projects/jet/logic/auto-artifacts.md#A1
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<PathBuf>,
}

/// Aggregated summary emitted at the end of a `run` invocation.
///
/// JSON serialization carries `schema_version` as the first field so
/// downstream tools can route by envelope tag.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    /// Schema tag — see [`SCHEMA_VERSION`].
    // @spec #2604
    pub schema_version: &'static str,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
    pub reports: Vec<TestReport>,
    /// Aggregated coverage summary. `None` when coverage was disabled or
    /// no producer attached a summary to this run.
    // @spec #2714
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<CoverageSummary>,
}

#[cfg(test)]
mod source_location_tests {
    use super::*;

    #[test]
    fn parses_paren_frame_with_line_and_col() {
        let stack = "Error: bad\n    at Object.<anonymous> (/tmp/specs/a.spec.ts:12:34)";
        let loc = SourceLocation::parse_from_stack(stack).expect("loc");
        assert_eq!(loc.file, PathBuf::from("/tmp/specs/a.spec.ts"));
        assert_eq!(loc.line, 12);
        assert_eq!(loc.column, Some(34));
    }

    #[test]
    fn parses_bare_frame_with_line_only() {
        let stack = "Error: bad\n    at /tmp/specs/a.spec.ts:7";
        let loc = SourceLocation::parse_from_stack(stack).expect("loc");
        assert_eq!(loc.line, 7);
        assert_eq!(loc.column, None);
    }

    #[test]
    fn skips_runtime_and_boot_frames() {
        let stack = "Error: bad\n\
            at expect (/tmp/work/node_modules/@jet/test/index.mjs:123:4)\n\
            at runTest (/tmp/work/__jet_boot.mjs:55:6)\n\
            at userFn (/tmp/specs/a.spec.ts:7:10)";
        let loc = SourceLocation::parse_from_stack(stack).expect("loc");
        assert_eq!(loc.file, PathBuf::from("/tmp/specs/a.spec.ts"));
        assert_eq!(loc.line, 7);
        assert_eq!(loc.column, Some(10));
    }

    #[test]
    fn strips_file_url_prefix() {
        let stack = "Error: bad\n    at fn (file:///tmp/specs/a.spec.ts:9:1)";
        let loc = SourceLocation::parse_from_stack(stack).expect("loc");
        assert_eq!(loc.file, PathBuf::from("/tmp/specs/a.spec.ts"));
        assert_eq!(loc.line, 9);
        assert_eq!(loc.column, Some(1));
    }

    #[test]
    fn returns_none_when_no_user_frame() {
        let stack = "Error: bad\n\
            at expect (/tmp/work/node_modules/@jet/test/index.mjs:1:1)\n\
            at runtime (/tmp/work/node_modules/@playwright/test/index.mjs:9:9)";
        assert!(SourceLocation::parse_from_stack(stack).is_none());
    }

    #[test]
    fn returns_none_on_empty_or_unmatched() {
        assert!(SourceLocation::parse_from_stack("").is_none());
        assert!(SourceLocation::parse_from_stack("not a stack").is_none());
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl Default for Summary {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            reports: Vec::new(),
            coverage: None,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl Summary {
    /// Format a rerun hint for a single test report — the exact command a
    /// user can paste back into the shell to rerun just this failure.
    // @spec #2604
    pub fn rerun_hint(report: &TestReport, project_root: &std::path::Path) -> String {
        let rel = report
            .file
            .strip_prefix(project_root)
            .unwrap_or(&report.file);
        let suite_path = if report.suite.is_empty() {
            String::new()
        } else {
            format!("{} > ", report.suite.join(" > "))
        };
        format!(
            "jet test {} -g '{}{}'",
            rel.display(),
            suite_path,
            report.name
        )
    }
}

/// Combined reporter that fans out lifecycle events to every configured
/// sub-reporter. Each sub-reporter implements its own `Reporter` strategy.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub struct MultiReporter {
    term: bool,
    json: Option<PathBuf>,
    state: Mutex<ReporterState>,
    project_root: PathBuf,
}

#[derive(Default)]
struct ReporterState {
    started_at: Option<std::time::Instant>,
    specs_total: usize,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl MultiReporter {
    pub fn from_config(config: &RunnerConfig, project_root: PathBuf) -> Self {
        let term = config.reporters.contains(&Reporter::Term);
        let json = if config.reporters.contains(&Reporter::Json) {
            Some(project_root.join(".jet").join("test-results.json"))
        } else {
            None
        };
        Self {
            term,
            json,
            state: Mutex::new(ReporterState::default()),
            project_root,
        }
    }

    pub fn on_start(&self, specs: &[SpecFile]) -> Result<()> {
        let mut s = self.state.lock().expect("reporter state poisoned");
        s.started_at = Some(std::time::Instant::now());
        s.specs_total = specs.len();
        if self.term {
            println!("jet test: {} spec file(s) discovered", specs.len());
        }
        Ok(())
    }

    /// Forwards a worker event for live display (only test-end affects term
    /// output in v0 — we print a one-line status per test).
    pub fn on_event(&self, file: &SpecFile, event: &WorkerEvent) {
        if !self.term {
            return;
        }
        let rel = file.relative.display();
        match event {
            WorkerEvent::TestEnd {
                suite,
                name,
                outcome,
                duration_ms,
                error,
                ..
            } => {
                let (glyph, label) = match outcome {
                    crate::test_runner::wire::TestOutcome::Passed => ("  ok ", "PASS"),
                    crate::test_runner::wire::TestOutcome::Failed => (" FAIL", "FAIL"),
                    crate::test_runner::wire::TestOutcome::Skipped => (" skip", "SKIP"),
                    crate::test_runner::wire::TestOutcome::TimedOut => ("   T ", "TIMEOUT"),
                };
                let suite_path = if suite.is_empty() {
                    String::new()
                } else {
                    format!("{} > ", suite.join(" > "))
                };
                println!("{glyph} [{label}] {rel} :: {suite_path}{name} ({duration_ms}ms)");
                if let Some(err) = error {
                    let msg = match err {
                        crate::test_runner::wire::TestError { message, diff, .. } => {
                            (message, diff)
                        }
                    };
                    println!("       {}", msg.0);
                    if let Some(diff) = &msg.1 {
                        for line in diff.lines() {
                            println!("       {line}");
                        }
                    }
                }
            }
            WorkerEvent::Fatal { message } => {
                println!(" FATAL [{rel}] {message}");
            }
            WorkerEvent::Console { stream, message } => {
                // Only surface stderr during runs; stdout is noisy.
                if matches!(stream, crate::test_runner::wire::ConsoleStream::Stderr) {
                    println!("       [{rel}][stderr] {message}");
                }
            }
            _ => {}
        }
    }

    pub fn on_finish(&self, summary: &Summary) -> Result<()> {
        if self.term {
            let s = self.state.lock().expect("reporter state poisoned");
            let elapsed_ms = s
                .started_at
                .map(|t| t.elapsed().as_millis() as u64)
                .unwrap_or(0);
            println!();
            println!(
                "Tests: {} passed, {} failed, {} skipped  |  {}ms total",
                summary.passed, summary.failed, summary.skipped, elapsed_ms
            );
            if summary.failed > 0 {
                println!("Failing tests:");
                for r in summary.reports.iter().filter(|r| {
                    matches!(
                        r.outcome,
                        Outcome::Failed | Outcome::TimedOut | Outcome::Crashed
                    )
                }) {
                    let suite = if r.suite.is_empty() {
                        String::new()
                    } else {
                        format!("{} > ", r.suite.join(" > "))
                    };
                    println!(
                        "  - {} :: {}{}",
                        r.file
                            .strip_prefix(&self.project_root)
                            .unwrap_or(&r.file)
                            .display(),
                        suite,
                        r.name
                    );
                    // @spec #2604 — rerun hint: agent/human can paste this
                    // line to rerun just the failing case.
                    println!(
                        "      rerun: {}",
                        Summary::rerun_hint(r, &self.project_root)
                    );
                }
            }
        }

        if let Some(json_path) = &self.json {
            if let Some(parent) = json_path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create reporter output dir: {}", parent.display())
                })?;
            }
            let body = serde_json::to_string_pretty(summary)
                .context("Failed to serialise test summary")?;
            std::fs::write(json_path, body)
                .with_context(|| format!("Failed to write {}", json_path.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn json_reporter_writes_results_file() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.reporters = vec![Reporter::Json];
        let reporter = MultiReporter::from_config(&cfg, cfg.project_root.clone());

        let summary = Summary {
            schema_version: SCHEMA_VERSION,
            passed: 2,
            failed: 1,
            skipped: 0,
            duration_ms: 123,
            reports: vec![TestReport {
                file: tmp.path().join("x.spec.ts"),
                suite: vec![],
                name: "t".into(),
                outcome: Outcome::Passed,
                duration_ms: 5,
                error: None,
                trace_path: None,
                shard_index: None,
                shard_total: None,
                artifacts: Vec::new(),
            }],
            coverage: None,
        };
        reporter.on_finish(&summary).unwrap();
        let out_path = cfg.project_root.join(".jet/test-results.json");
        assert!(out_path.exists());
        let body = std::fs::read_to_string(&out_path).unwrap();
        assert!(body.contains("\"passed\""));
        assert!(body.contains("\"failed\": 1"));
        assert!(
            body.contains("\"schema_version\": \"jet.test.result.v1\""),
            "stable schema tag must be present: {body}"
        );
    }

    #[test]
    fn rerun_hint_includes_file_and_suite_qualified_name() {
        let report = TestReport {
            file: PathBuf::from("/abs/root/sub/x.spec.ts"),
            suite: vec!["Outer".into(), "Inner".into()],
            name: "renders".into(),
            outcome: Outcome::Failed,
            duration_ms: 1,
            error: None,
            trace_path: None,
            shard_index: None,
            shard_total: None,
            artifacts: Vec::new(),
        };
        let root = std::path::Path::new("/abs/root");
        let hint = Summary::rerun_hint(&report, root);
        assert_eq!(hint, "jet test sub/x.spec.ts -g 'Outer > Inner > renders'");
    }

    #[test]
    fn rerun_hint_omits_suite_separator_when_top_level() {
        let report = TestReport {
            file: PathBuf::from("/abs/root/x.spec.ts"),
            suite: vec![],
            name: "top".into(),
            outcome: Outcome::Failed,
            duration_ms: 1,
            error: None,
            trace_path: None,
            shard_index: None,
            shard_total: None,
            artifacts: Vec::new(),
        };
        let root = std::path::Path::new("/abs/root");
        let hint = Summary::rerun_hint(&report, root);
        assert_eq!(hint, "jet test x.spec.ts -g 'top'");
    }

    #[test]
    fn term_only_does_not_write_json() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.reporters = vec![Reporter::Term];
        let reporter = MultiReporter::from_config(&cfg, cfg.project_root.clone());
        reporter.on_finish(&Summary::default()).unwrap();
        assert!(!cfg.project_root.join(".jet/test-results.json").exists());
    }
}
// CODEGEN-END
