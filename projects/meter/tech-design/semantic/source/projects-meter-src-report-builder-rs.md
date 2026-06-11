---
id: projects-meter-src-report-builder-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/report/builder.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/report/builder.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReportBuilder` | projects/meter/src/report/builder.rs | struct | pub | 20 |  |
| `add_criterion` | projects/meter/src/report/builder.rs | function | pub | 92 | add_criterion(&mut self, criterion: impl Into<String>) -> &mut Self |
| `add_finding` | projects/meter/src/report/builder.rs | function | pub | 68 | add_finding(&mut self, finding: Finding) -> &mut Self |
| `add_findings` | projects/meter/src/report/builder.rs | function | pub | 74 | add_findings(&mut self, findings: impl IntoIterator<Item = Finding>) -> &mut Self |
| `add_missing` | projects/meter/src/report/builder.rs | function | pub | 101 | add_missing(&mut self, missing: impl Into<String>) -> &mut Self |
| `finalize` | projects/meter/src/report/builder.rs | function | pub | 254 | finalize(mut self) -> MeterReport |
| `forward_exit` | projects/meter/src/report/builder.rs | function | pub | 116 | forward_exit(&mut self, code: i32) -> &mut Self |
| `informational_findings_are_clean` | projects/meter/src/report/builder.rs | function | pub | 122 | informational_findings_are_clean(&mut self) -> &mut Self |
| `new` | projects/meter/src/report/builder.rs | function | pub | 51 | new(verb: impl Into<String>, target: impl Into<String>) -> Self |
| `requires_hitl` | projects/meter/src/report/builder.rs | function | pub | 128 | requires_hitl(&mut self, value: bool) -> &mut Self |
| `tool_error` | projects/meter/src/report/builder.rs | function | pub | 109 | tool_error(&mut self, code: i32, message: impl Into<String>) -> &mut Self |
| `with_environment` | projects/meter/src/report/builder.rs | function | pub | 86 | with_environment(&mut self, env: EnvBlock) -> &mut Self |
| `with_last_run` | projects/meter/src/report/builder.rs | function | pub | 80 | with_last_run(&mut self, record: RunnerRecord) -> &mut Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/report/builder.rs -->
````rust
//! [`ReportBuilder`] — the SINGLE assembly point for a [`MeterReport`].
//!
//! `finalize()` is the SOLE site that derives `status`, `clean`, `exit_code`,
//! `terminal`, `completion`, and `agent_prompt`. Every verb funnels through here
//! so the exit-code contract has exactly one source of truth.

use super::envelope::{
    Completion, EnvBlock, FindingsSummary, MeterReport, OverallStatus, RunnerRecord, SCHEMA_VERSION,
};
use super::finding::{Finding, Kind, Severity};

/// Default number of findings included inline in `summary.sample`.
const DEFAULT_SAMPLE_N: usize = 20;

/// Incrementally assemble a [`MeterReport`]; `finalize()` derives all
/// status-dependent fields.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-builder-rs.md#source
pub struct ReportBuilder {
    verb: String,
    target: String,
    findings: Vec<Finding>,
    last_run: Option<RunnerRecord>,
    environment: Option<EnvBlock>,
    /// Criteria the verb declares for a "clean" outcome.
    criteria: Vec<String>,
    /// Explicit `completion.missing` entries. When `Some`, `finalize()` emits
    /// these verbatim (independent of clean status) instead of deriving missing
    /// from `criteria`. The composite `meter run` verb uses this to list the
    /// sub-verbs it did NOT run, each with a human reason (e.g. "profile: no
    /// --profile-bin given"), so the agent sees coverage gaps even on a clean
    /// sweep.
    explicit_missing: Option<Vec<String>>,
    /// Explicit tool-error override: when set, finalize() emits this status
    /// verbatim instead of deriving from findings.
    tool_error: Option<(i32, String)>,
    /// Explicit forwarded exit code (the `test`/delegate verb yields the child
    /// code rather than imposing meter's own 0/1/2 verdicts).
    forced_exit: Option<i32>,
    /// Some verbs use Info findings as the successful payload, not as issues.
    /// `meter profile` is the canonical case: hotspots/boundary costs tell the
    /// agent where time goes, and only an explicit threshold breach should fail.
    info_findings_are_clean: bool,
    requires_hitl: bool,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-builder-rs.md#source
impl ReportBuilder {
    /// Start a builder for `verb` against `target`.
    pub fn new(verb: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            verb: verb.into(),
            target: target.into(),
            findings: Vec::new(),
            last_run: None,
            environment: None,
            criteria: Vec::new(),
            explicit_missing: None,
            tool_error: None,
            forced_exit: None,
            info_findings_are_clean: false,
            requires_hitl: false,
        }
    }

    /// Append one finding.
    pub fn add_finding(&mut self, finding: Finding) -> &mut Self {
        self.findings.push(finding);
        self
    }

    /// Append many findings.
    pub fn add_findings(&mut self, findings: impl IntoIterator<Item = Finding>) -> &mut Self {
        self.findings.extend(findings);
        self
    }

    /// Record the delegated child runner.
    pub fn with_last_run(&mut self, record: RunnerRecord) -> &mut Self {
        self.last_run = Some(record);
        self
    }

    /// Override the detected environment block (otherwise `EnvBlock::detect()`).
    pub fn with_environment(&mut self, env: EnvBlock) -> &mut Self {
        self.environment = Some(env);
        self
    }

    /// Declare a success criterion for this verb.
    pub fn add_criterion(&mut self, criterion: impl Into<String>) -> &mut Self {
        self.criteria.push(criterion.into());
        self
    }

    /// Record one explicit `completion.missing` entry (a coverage gap the verb
    /// could not close), e.g. a sub-verb the composite `meter run` did not run.
    /// Setting any explicit missing entry switches `completion.missing` to the
    /// explicit list verbatim — it is no longer derived from `criteria`.
    pub fn add_missing(&mut self, missing: impl Into<String>) -> &mut Self {
        self.explicit_missing
            .get_or_insert_with(Vec::new)
            .push(missing.into());
        self
    }

    /// Force a `ToolError` status with the given code/message.
    pub fn tool_error(&mut self, code: i32, message: impl Into<String>) -> &mut Self {
        self.tool_error = Some((code, message.into()));
        self
    }

    /// Force the process exit code to the given value, overriding the status's
    /// natural code. Used by `test` to FORWARD the child's exit code.
    pub fn forward_exit(&mut self, code: i32) -> &mut Self {
        self.forced_exit = Some(code);
        self
    }

    /// Treat an all-Info finding set as a clean successful payload.
    pub fn informational_findings_are_clean(&mut self) -> &mut Self {
        self.info_findings_are_clean = true;
        self
    }

    /// Mark the report as needing human intervention.
    pub fn requires_hitl(&mut self, value: bool) -> &mut Self {
        self.requires_hitl = value;
        self
    }

    /// Derive `OverallStatus` from the accumulated findings (worst-wins). This is
    /// the severity ladder Wave 7's `meter run` also relies on:
    /// `ToolError > Regression > Findings > Clean`.
    ///
    /// A forced tool error always takes precedence. Otherwise a `Regression`
    /// finding elevates the whole report to `OverallStatus::Regression` (exit 2)
    /// ONLY when its severity is medium-or-worse — i.e. an engine `Severe`
    /// (=> `High`) or `Moderate` (=> `Medium`) slowdown. A `Minor` regression
    /// (=> `Low`) is informational and stays a plain `Findings` (exit 1) along
    /// with every non-regression finding.
    fn derive_status(&self) -> OverallStatus {
        if let Some((code, message)) = &self.tool_error {
            return OverallStatus::ToolError {
                code: *code,
                message: message.clone(),
            };
        }
        // A medium-or-worse regression finding => Regression (exit 2) — the worst
        // non-error rung. Minor regressions (severity Low/Info) do NOT elevate.
        let severe_regressions = self
            .findings
            .iter()
            .filter(|f| {
                matches!(f.kind, Kind::Regression) && f.severity.rank() >= Severity::Medium.rank()
            })
            .count();
        if severe_regressions > 0 {
            return OverallStatus::Regression {
                count: severe_regressions,
            };
        }
        if !self.findings.is_empty()
            && !(self.info_findings_are_clean
                && self.findings.iter().all(|f| f.severity == Severity::Info))
        {
            return OverallStatus::Findings {
                count: self.findings.len(),
            };
        }
        OverallStatus::Clean
    }

    /// Tally findings into a [`FindingsSummary`] with a bounded inline sample.
    fn summarize(&self) -> FindingsSummary {
        let mut s = FindingsSummary {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            info: 0,
            total: self.findings.len(),
            truncated: false,
            sample: Vec::new(),
            payload_path: None,
        };
        for f in &self.findings {
            match f.severity {
                Severity::Critical => s.critical += 1,
                Severity::High => s.high += 1,
                Severity::Medium => s.medium += 1,
                Severity::Low => s.low += 1,
                Severity::Info => s.info += 1,
            }
        }
        s.sample = self
            .findings
            .iter()
            .take(DEFAULT_SAMPLE_N)
            .cloned()
            .collect();
        s.truncated = self.findings.len() > DEFAULT_SAMPLE_N;
        s
    }

    /// Sort findings: severity desc, then id asc — EXCEPT ranked performance
    /// findings, which the C1 contract pins to `self_ns` DESC. Within a severity
    /// tie, two hot spots or two boundary costs compare by `evidence.self_ns`
    /// desc (id asc as the final tiebreaker for determinism); any other pairing
    /// falls back to id asc.
    fn sort_findings(findings: &mut [Finding]) {
        findings.sort_by(|a, b| {
            b.severity.rank().cmp(&a.severity.rank()).then_with(|| {
                if is_ranked_performance_kind(a.kind) && a.kind == b.kind {
                    // self_ns desc; equal self_ns falls back to id asc.
                    finding_self_ns(b)
                        .cmp(&finding_self_ns(a))
                        .then_with(|| a.id.cmp(&b.id))
                } else {
                    a.id.cmp(&b.id)
                }
            })
        });
    }

    /// The agent next-action prompt for a derived status.
    fn agent_prompt(&self, status: &OverallStatus) -> String {
        match status {
            OverallStatus::Clean => format!(
                "meter {} found no issues for `{}`. Nothing to do; this result is terminal.",
                self.verb, self.target
            ),
            OverallStatus::Findings { count } => format!(
                "meter {} surfaced {} finding(s) for `{}`. Inspect `findings[]`, run each \
                 finding's `invoke.command`, then re-run to confirm.",
                self.verb, count, self.target
            ),
            OverallStatus::Regression { count } => format!(
                "meter {} detected {} performance regression(s) for `{}` (exit 2). Inspect \
                 `findings[]` of kind `regression` and address the slowdown before merging.",
                self.verb, count, self.target
            ),
            OverallStatus::ToolError { code, message } => format!(
                "meter {} could not complete for `{}` (tool error {}): {}. Resolve the tool/usage \
                 issue (see `environment.note`) and re-run.",
                self.verb, self.target, code, message
            ),
        }
    }

    /// Assemble the final [`MeterReport`]. SOLE site deriving status/clean/exit_code/
    /// terminal/completion/agent_prompt.
    pub fn finalize(mut self) -> MeterReport {
        Self::sort_findings(&mut self.findings);
        let status = self.derive_status();
        let summary = self.summarize();
        let environment = self.environment.take().unwrap_or_else(EnvBlock::detect);

        let natural_exit = status.exit_code();
        let exit_code = self.forced_exit.unwrap_or(natural_exit);
        let clean = status.is_clean();
        // Every derived status is terminal in this surface.
        let terminal = true;

        let agent_prompt = self.agent_prompt(&status);

        // `completion.missing`: when the verb set explicit missing entries (the
        // composite `meter run` lists the sub-verbs it did NOT run, with a human
        // reason for each), emit them verbatim regardless of clean status —
        // un-run coverage is a gap even on an otherwise-clean sweep. Otherwise
        // fall back to the default: nothing missing when clean, else the
        // unmet criteria.
        let missing = match &self.explicit_missing {
            Some(explicit) => explicit.clone(),
            None if clean => Vec::new(),
            None => self.criteria.clone(),
        };
        let completion = Completion {
            clean,
            criteria: self.criteria.clone(),
            missing,
        };

        MeterReport {
            schema_version: SCHEMA_VERSION.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            verb: self.verb,
            target: self.target,
            status,
            clean,
            exit_code,
            terminal,
            last_run: self.last_run,
            summary,
            findings: self.findings,
            environment,
            completion,
            agent_prompt,
            requires_hitl: self.requires_hitl,
        }
    }
}

fn is_ranked_performance_kind(kind: Kind) -> bool {
    matches!(kind, Kind::Hotspot | Kind::BoundaryCost)
}

/// Extract a ranked performance finding's `evidence.self_ns`; absent or
/// non-numeric => 0 (it sinks to the bottom of the kind group).
fn finding_self_ns(f: &Finding) -> u64 {
    f.evidence
        .get("self_ns")
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::finding::{Invoke, Kind};

    fn env() -> EnvBlock {
        EnvBlock {
            os: "macos".into(),
            arch: "aarch64".into(),
            nextest_present: false,
            sampler_backend: "macos-sample".into(),
            rustc_version: None,
            note: String::new(),
        }
    }

    fn finding(id: &str, sev: Severity, kind: Kind) -> Finding {
        Finding {
            id: id.into(),
            severity: sev,
            kind,
            title: "t".into(),
            detail: "d".into(),
            remediation: "r".into(),
            invoke: Invoke::command("cargo test"),
            evidence: serde_json::Value::Null,
            location: None,
        }
    }

    #[test]
    fn empty_builder_is_clean() {
        let mut b = ReportBuilder::new("test", "/tmp");
        b.with_environment(env());
        let r = b.finalize();
        assert!(r.clean);
        assert_eq!(r.exit_code, 0);
        assert!(r.terminal);
        assert_eq!(r.summary.total, 0);
    }

    #[test]
    fn findings_yield_exit_1() {
        let mut b = ReportBuilder::new("test", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("test_failure:a", Severity::High, Kind::TestFailure));
        let r = b.finalize();
        assert!(!r.clean);
        assert_eq!(r.exit_code, 1);
        assert!(matches!(r.status, OverallStatus::Findings { count: 1 }));
        assert_eq!(r.summary.high, 1);
    }

    #[test]
    fn regression_yields_exit_2() {
        let mut b = ReportBuilder::new("bench", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("regression:x", Severity::High, Kind::Regression));
        let r = b.finalize();
        assert_eq!(r.exit_code, 2);
        assert!(!r.clean);
        assert!(matches!(r.status, OverallStatus::Regression { count: 1 }));
    }

    #[test]
    fn medium_regression_yields_exit_2() {
        // A Moderate (=> Medium) regression is medium-or-worse and elevates.
        let mut b = ReportBuilder::new("bench", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("regression:y", Severity::Medium, Kind::Regression));
        let r = b.finalize();
        assert_eq!(r.exit_code, 2);
        assert!(!r.clean);
        assert!(matches!(r.status, OverallStatus::Regression { count: 1 }));
    }

    #[test]
    fn non_regression_finding_yields_exit_1_not_2() {
        // A plain finding (no regression) stays Findings/exit 1.
        let mut b = ReportBuilder::new("audit", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("rust_vuln:z", Severity::Critical, Kind::RustVuln));
        let r = b.finalize();
        assert_eq!(r.exit_code, 1);
        assert!(matches!(r.status, OverallStatus::Findings { count: 1 }));
    }

    #[test]
    fn regression_plus_lower_finding_still_exit_2_worst_wins() {
        // A Regression alongside a non-regression finding: Regression outranks
        // plain Findings (worst-wins ladder), so the report is exit 2.
        let mut b = ReportBuilder::new("run", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("regression:x", Severity::High, Kind::Regression));
        b.add_finding(finding("rust_warning:w", Severity::Low, Kind::RustWarning));
        let r = b.finalize();
        assert_eq!(r.exit_code, 2);
        // Only the medium-or-worse regression(s) are counted in the status.
        assert!(matches!(r.status, OverallStatus::Regression { count: 1 }));
        // But the full finding set (both) is retained.
        assert_eq!(r.findings.len(), 2);
    }

    #[test]
    fn minor_regression_does_not_elevate_to_exit_2() {
        // A Minor (=> Low) regression is informational only: it stays exit 1.
        let mut b = ReportBuilder::new("bench", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("regression:minor", Severity::Low, Kind::Regression));
        let r = b.finalize();
        assert_eq!(r.exit_code, 1);
        assert!(matches!(r.status, OverallStatus::Findings { count: 1 }));
    }

    #[test]
    fn opt_in_info_findings_can_be_clean_payloads() {
        let mut b = ReportBuilder::new("profile", "/tmp");
        b.with_environment(env());
        b.informational_findings_are_clean();
        b.add_finding(finding("boundary:x", Severity::Info, Kind::BoundaryCost));
        let r = b.finalize();
        assert!(r.clean);
        assert_eq!(r.exit_code, 0);
        assert!(matches!(r.status, OverallStatus::Clean));
        assert_eq!(r.summary.info, 1);
        assert!(r.completion.missing.is_empty());
    }

    #[test]
    fn opt_in_info_clean_does_not_hide_high_findings() {
        let mut b = ReportBuilder::new("profile", "/tmp");
        b.with_environment(env());
        b.informational_findings_are_clean();
        b.add_finding(finding("hotspot:x", Severity::High, Kind::Hotspot));
        let r = b.finalize();
        assert!(!r.clean);
        assert_eq!(r.exit_code, 1);
        assert!(matches!(r.status, OverallStatus::Findings { count: 1 }));
    }

    #[test]
    fn tool_error_takes_precedence() {
        let mut b = ReportBuilder::new("audit", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("rust_vuln:x", Severity::Critical, Kind::RustVuln));
        b.tool_error(4, "cargo-audit missing");
        let r = b.finalize();
        assert_eq!(r.exit_code, 4);
        assert!(matches!(r.status, OverallStatus::ToolError { code: 4, .. }));
    }

    #[test]
    fn forward_exit_overrides_natural_code() {
        let mut b = ReportBuilder::new("test", "/tmp");
        b.with_environment(env());
        // A delegated child failed (exit 101) but produced no parseable findings.
        b.forward_exit(101);
        let r = b.finalize();
        // status is Clean (no findings) but the forwarded exit wins.
        assert!(r.clean);
        assert_eq!(r.exit_code, 101);
    }

    #[test]
    fn findings_sorted_severity_desc_then_id() {
        let mut b = ReportBuilder::new("audit", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("rust_vuln:b", Severity::Low, Kind::RustVuln));
        b.add_finding(finding("rust_vuln:a", Severity::Critical, Kind::RustVuln));
        b.add_finding(finding("rust_vuln:c", Severity::Critical, Kind::RustVuln));
        let r = b.finalize();
        // Critical first, ties broken by id asc.
        assert_eq!(r.findings[0].id, "rust_vuln:a");
        assert_eq!(r.findings[1].id, "rust_vuln:c");
        assert_eq!(r.findings[2].id, "rust_vuln:b");
    }

    /// A hotspot finding with a given self_ns evidence value.
    fn hotspot(id: &str, self_ns: u64) -> Finding {
        Finding {
            id: id.into(),
            severity: Severity::Info,
            kind: Kind::Hotspot,
            title: "h".into(),
            detail: "d".into(),
            remediation: "r".into(),
            invoke: Invoke::command("meter profile"),
            evidence: serde_json::json!({ "self_ns": self_ns }),
            location: None,
        }
    }

    #[test]
    fn hotspot_findings_sort_by_self_ns_desc_not_id() {
        // Equal severity (Info) hot spots must rank by self_ns DESC, NOT id asc —
        // the C1 ranked-hot-spot contract. ids here are deliberately in the
        // opposite order to self_ns so an id-asc sort would fail.
        let mut b = ReportBuilder::new("profile", "/tmp");
        b.with_environment(env());
        b.add_finding(hotspot("hotspot:aaa_low", 10));
        b.add_finding(hotspot("hotspot:zzz_high", 1000));
        b.add_finding(hotspot("hotspot:mmm_mid", 100));
        let r = b.finalize();
        let ns: Vec<u64> = r
            .findings
            .iter()
            .map(|f| f.evidence["self_ns"].as_u64().unwrap())
            .collect();
        assert_eq!(ns, vec![1000, 100, 10], "hotspots must be self_ns desc");
        assert_eq!(r.findings[0].id, "hotspot:zzz_high");
    }

    #[test]
    fn boundary_cost_findings_sort_by_self_ns_desc_not_id() {
        let mut high = finding("boundary:aaa_high", Severity::Info, Kind::BoundaryCost);
        high.evidence = serde_json::json!({ "self_ns": 6_000_000u64 });
        let mut low = finding("boundary:zzz_low", Severity::Info, Kind::BoundaryCost);
        low.evidence = serde_json::json!({ "self_ns": 1_500_000u64 });

        let mut b = ReportBuilder::new("profile", "/tmp");
        b.with_environment(env());
        b.add_finding(low);
        b.add_finding(high);
        let r = b.finalize();
        assert_eq!(r.findings[0].id, "boundary:aaa_high");
        assert_eq!(r.findings[1].id, "boundary:zzz_low");
    }

    #[test]
    fn completion_missing_empty_when_clean() {
        let mut b = ReportBuilder::new("test", "/tmp");
        b.with_environment(env());
        b.add_criterion("no test failures");
        let r = b.finalize();
        assert!(r.completion.clean);
        assert!(r.completion.missing.is_empty());
        assert_eq!(r.completion.criteria.len(), 1);
    }
}

/// `meter run` composite worst-wins folding tests.
///
/// These prove the worst-wins folding the `meter run` sweep relies on, at the
/// builder seam (the SOLE exit-derivation site). The composite folds ALL
/// sub-verb findings into one builder; `finalize()` derives the worst-wins
/// status. They are `capture`-gated because the composite `meter run` verb (and the
/// only caller that folds delegated-test + regression findings together) lives
/// behind the `capture` feature — so the DEFAULT lib test count stays unchanged,
/// while `--features capture` exercises the §4 rule end-to-end at this seam.
#[cfg(all(test, feature = "capture"))]
mod run_fold_tests {
    use super::*;
    use crate::report::finding::{Invoke, Kind};

    fn env() -> EnvBlock {
        EnvBlock {
            os: "macos".into(),
            arch: "aarch64".into(),
            nextest_present: false,
            sampler_backend: "macos-sample".into(),
            rustc_version: None,
            note: String::new(),
        }
    }

    fn finding(id: &str, sev: Severity, kind: Kind) -> Finding {
        Finding {
            id: id.into(),
            severity: sev,
            kind,
            title: "t".into(),
            detail: "d".into(),
            remediation: "r".into(),
            invoke: Invoke::command("cargo test"),
            evidence: serde_json::Value::Null,
            location: None,
        }
    }

    #[test]
    fn run_fold_regression_plus_test_failure_is_regression_not_overridden() {
        // §4: {a Regression finding} + {a TestFailure finding} folded together
        // => Regression (exit 2). The test failure did NOT override the
        // regression even though both are recorded.
        let mut b = ReportBuilder::new("run", "/tmp");
        b.with_environment(env());
        b.add_finding(finding(
            "regression:bench_x",
            Severity::High,
            Kind::Regression,
        ));
        b.add_finding(finding(
            "test_failure:mod::t",
            Severity::High,
            Kind::TestFailure,
        ));
        let r = b.finalize();
        assert_eq!(r.exit_code, 2, "regression must win over a test failure");
        assert!(matches!(r.status, OverallStatus::Regression { count: 1 }));
        // Both findings are retained in the folded report.
        assert_eq!(r.findings.len(), 2);
    }

    #[test]
    fn run_fold_only_test_failure_is_exit_1() {
        // {only TestFailure} => Findings (exit 1).
        let mut b = ReportBuilder::new("run", "/tmp");
        b.with_environment(env());
        b.add_finding(finding(
            "test_failure:mod::t",
            Severity::High,
            Kind::TestFailure,
        ));
        let r = b.finalize();
        assert_eq!(r.exit_code, 1);
        assert!(matches!(r.status, OverallStatus::Findings { count: 1 }));
    }

    #[test]
    fn run_fold_tool_error_dominates_regression() {
        // {a sub ToolError + a Regression} => ToolError dominates (exit > 2) per
        // the ladder. A run forwards a sub-verb's HARD tool error (one that
        // prevents the verb from running at all) via the builder's `tool_error`
        // override; SOFT tool-unavailability is recorded as `completion.missing`
        // instead (see `run_fold_missing_is_kept_even_when_not_clean` and the
        // capture::run sweep tests), so the sweep continues.
        let mut b = ReportBuilder::new("run", "/tmp");
        b.with_environment(env());
        b.add_finding(finding(
            "regression:bench_x",
            Severity::High,
            Kind::Regression,
        ));
        b.tool_error(4, "a sub-verb tool errored");
        let r = b.finalize();
        assert!(r.exit_code > 2, "tool error must dominate a regression");
        assert!(matches!(r.status, OverallStatus::ToolError { code: 4, .. }));
    }

    #[test]
    fn run_fold_all_clean_lists_unrun_verbs_in_missing() {
        // {all clean} => exit 0, completion.clean == true, and completion.missing
        // lists the un-run verbs with a human reason each (the explicit-missing
        // path). The composite run sets these so an agent sees coverage gaps even
        // on a clean sweep.
        let mut b = ReportBuilder::new("run", "/tmp");
        b.with_environment(env());
        b.add_criterion("audit clean");
        b.add_criterion("test clean");
        b.add_missing("profile: no --profile-bin/--profile-example given");
        b.add_missing("bench: no --baseline given");
        b.add_missing("fuzz: no --fuzz-target/--fuzz-url given");
        let r = b.finalize();
        assert_eq!(r.exit_code, 0);
        assert!(r.clean);
        assert!(r.completion.clean);
        assert_eq!(r.completion.missing.len(), 3);
        assert!(r
            .completion
            .missing
            .iter()
            .any(|m| m.starts_with("profile:")));
        assert!(r.completion.missing.iter().any(|m| m.starts_with("bench:")));
        assert!(r.completion.missing.iter().any(|m| m.starts_with("fuzz:")));
    }

    #[test]
    fn run_fold_missing_is_kept_even_when_not_clean() {
        // Explicit missing entries survive a non-clean outcome too: an un-run
        // verb is still a coverage gap when other sub-verbs found issues.
        let mut b = ReportBuilder::new("run", "/tmp");
        b.with_environment(env());
        b.add_finding(finding("rust_vuln:x", Severity::High, Kind::RustVuln));
        b.add_missing("fuzz: no --fuzz-target/--fuzz-url given");
        let r = b.finalize();
        assert!(!r.clean);
        assert_eq!(
            r.completion.missing,
            vec!["fuzz: no --fuzz-target/--fuzz-url given"]
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/report/builder.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/report/builder.rs` captured during meter full-codegen standardization.
```
