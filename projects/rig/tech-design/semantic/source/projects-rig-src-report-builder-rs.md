---
id: projects-rig-src-report-builder-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/report/builder.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/report/builder.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReportBuilder` | projects/rig/src/report/builder.rs | struct | pub | 18 |  |
| `add_criterion` | projects/rig/src/report/builder.rs | function | pub | 54 | add_criterion(&mut self, c: impl Into<String>) -> &mut Self |
| `add_finding` | projects/rig/src/report/builder.rs | function | pub | 44 | add_finding(&mut self, f: Finding) -> &mut Self |
| `add_findings` | projects/rig/src/report/builder.rs | function | pub | 49 | add_findings(&mut self, fs: impl IntoIterator<Item = Finding>) -> &mut Self |
| `add_missing` | projects/rig/src/report/builder.rs | function | pub | 59 | add_missing(&mut self, m: impl Into<String>) -> &mut Self |
| `agent_prompt` | projects/rig/src/report/builder.rs | function | pub | 75 | agent_prompt(&mut self, p: impl Into<String>) -> &mut Self |
| `finalize` | projects/rig/src/report/builder.rs | function | pub | 80 | finalize(self) -> RigReport |
| `new` | projects/rig/src/report/builder.rs | function | pub | 31 | new(verb: impl Into<String>, target: impl Into<String>) -> Self |
| `persist` | projects/rig/src/report/builder.rs | function | pub | 156 | persist(report: &RigReport, dir: &std::path::Path) |
| `scenarios_mut` | projects/rig/src/report/builder.rs | function | pub | 64 | scenarios_mut(&mut self) -> &mut ScenarioCounts |
| `tool_error` | projects/rig/src/report/builder.rs | function | pub | 70 | tool_error(&mut self, code: u8, message: impl Into<String>) -> &mut Self |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Report assembly — the SOLE site deriving `status` / `clean` / `exit_code`.
//!
//! Worst-wins fold: `ToolError > Regression > Findings > Clean`. Info-only
//! finding sets stay Clean (locating where time goes is success, not
//! failure); any Low+ finding moves to Findings; a `PinRegression` finding
//! forces Regression regardless of its severity histogram position.

use super::envelope::{
    Completion, EnvBlock, OverallStatus, RigReport, ScenarioCounts, Summary, SCHEMA_VERSION,
};
use super::finding::{Finding, Kind, Severity};

const SUMMARY_SAMPLE_MAX: usize = 8;

pub struct ReportBuilder {
    verb: String,
    target: String,
    findings: Vec<Finding>,
    scenarios: ScenarioCounts,
    criteria: Vec<String>,
    missing: Vec<String>,
    tool_error: Option<(u8, String)>,
    agent_prompt: Option<String>,
}

impl ReportBuilder {
    pub fn new(verb: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            verb: verb.into(),
            target: target.into(),
            findings: Vec::new(),
            scenarios: ScenarioCounts::default(),
            criteria: Vec::new(),
            missing: Vec::new(),
            tool_error: None,
            agent_prompt: None,
        }
    }

    pub fn add_finding(&mut self, f: Finding) -> &mut Self {
        self.findings.push(f);
        self
    }

    pub fn add_findings(&mut self, fs: impl IntoIterator<Item = Finding>) -> &mut Self {
        self.findings.extend(fs);
        self
    }

    pub fn add_criterion(&mut self, c: impl Into<String>) -> &mut Self {
        self.criteria.push(c.into());
        self
    }

    pub fn add_missing(&mut self, m: impl Into<String>) -> &mut Self {
        self.missing.push(m.into());
        self
    }

    pub fn scenarios_mut(&mut self) -> &mut ScenarioCounts {
        &mut self.scenarios
    }

    /// Force a tool-level error (3 usage / 4 missing tool / 5 io). Outranks
    /// every finding-derived status.
    pub fn tool_error(&mut self, code: u8, message: impl Into<String>) -> &mut Self {
        self.tool_error = Some((code, message.into()));
        self
    }

    pub fn agent_prompt(&mut self, p: impl Into<String>) -> &mut Self {
        self.agent_prompt = Some(p.into());
        self
    }

    pub fn finalize(self) -> RigReport {
        // Deterministic order: severity rank, then id.
        let mut findings = self.findings;
        findings.sort_by(|a, b| {
            a.severity
                .rank()
                .cmp(&b.severity.rank())
                .then_with(|| a.id.cmp(&b.id))
        });

        let mut summary = Summary::default();
        for f in &findings {
            match f.severity {
                Severity::Critical => summary.critical += 1,
                Severity::High => summary.high += 1,
                Severity::Medium => summary.medium += 1,
                Severity::Low => summary.low += 1,
                Severity::Info => summary.info += 1,
            }
        }
        summary.total = findings.len() as u32;
        summary.sample = findings
            .iter()
            .take(SUMMARY_SAMPLE_MAX)
            .map(|f| f.id.clone())
            .collect();
        summary.truncated = false;

        let has_regression = findings.iter().any(|f| f.kind == Kind::PinRegression);
        let has_actionable = findings.iter().any(|f| f.severity != Severity::Info);

        let status = match &self.tool_error {
            Some((code, _)) => OverallStatus::ToolError { code: *code },
            None if has_regression => OverallStatus::Regression,
            None if has_actionable => OverallStatus::Findings,
            None => OverallStatus::Clean,
        };

        let clean = status.is_clean();
        let agent_prompt = self.agent_prompt.unwrap_or_else(|| match &self.tool_error {
            Some((_, msg)) => format!("rig {} could not run: {msg}", self.verb),
            None if clean => format!(
                "rig {} found no issues for `{}`. Nothing to do.",
                self.verb, self.target
            ),
            None => format!(
                "rig {} surfaced {} finding(s) for `{}`. Inspect `findings[]` (sorted worst-first) and run each finding's `invoke.command`.",
                self.verb, summary.total, self.target
            ),
        });

        RigReport {
            schema_version: SCHEMA_VERSION.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            verb: self.verb,
            target: self.target,
            status,
            clean,
            exit_code: status.exit_code(),
            summary,
            scenarios: self.scenarios,
            findings,
            environment: EnvBlock::current(),
            completion: Completion {
                clean,
                criteria: self.criteria,
                missing: self.missing,
            },
            agent_prompt,
        }
    }
}

/// Persist the report under `.rig/last-report.json` relative to `dir`
/// (best-effort; failures are non-fatal).
pub fn persist(report: &RigReport, dir: &std::path::Path) {
    let rig_dir = dir.join(".rig");
    if std::fs::create_dir_all(&rig_dir).is_err() {
        return;
    }
    if let Ok(json) = serde_json::to_string_pretty(report) {
        let _ = std::fs::write(rig_dir.join("last-report.json"), json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::finding::{finding_id, Invoke};

    fn finding(kind: Kind, severity: Severity, subject: &str) -> Finding {
        Finding {
            id: finding_id(kind, subject),
            severity,
            kind,
            title: subject.to_string(),
            detail: String::new(),
            remediation: String::new(),
            invoke: Invoke::command("rig run"),
            evidence: serde_json::json!({}),
        }
    }

    #[test]
    fn empty_report_is_clean_exit_0() {
        let r = ReportBuilder::new("run", ".").finalize();
        assert!(r.clean);
        assert_eq!(r.exit_code, 0);
        assert_eq!(r.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn info_only_findings_stay_clean() {
        let mut b = ReportBuilder::new("run", ".");
        b.add_finding(finding(Kind::PinMissingBaseline, Severity::Info, "p"));
        let r = b.finalize();
        assert!(r.clean);
        assert_eq!(r.exit_code, 0);
        assert_eq!(r.summary.info, 1);
    }

    #[test]
    fn actionable_finding_yields_exit_1() {
        let mut b = ReportBuilder::new("run", ".");
        b.add_finding(finding(Kind::StepFailure, Severity::High, "s"));
        let r = b.finalize();
        assert_eq!(r.exit_code, 1);
        assert!(!r.clean);
    }

    #[test]
    fn pin_regression_yields_exit_2_over_findings() {
        let mut b = ReportBuilder::new("run", ".");
        b.add_finding(finding(Kind::StepFailure, Severity::High, "s"));
        b.add_finding(finding(Kind::PinRegression, Severity::High, "p"));
        let r = b.finalize();
        assert_eq!(r.exit_code, 2);
    }

    #[test]
    fn tool_error_outranks_everything() {
        let mut b = ReportBuilder::new("run", ".");
        b.add_finding(finding(Kind::PinRegression, Severity::High, "p"));
        b.tool_error(3, "no scenarios");
        let r = b.finalize();
        assert_eq!(r.exit_code, 3);
    }

    #[test]
    fn findings_sorted_worst_first_deterministically() {
        let mut b = ReportBuilder::new("run", ".");
        b.add_finding(finding(Kind::StepFailure, Severity::Info, "b"));
        b.add_finding(finding(Kind::StepFailure, Severity::High, "a"));
        b.add_finding(finding(Kind::StepFailure, Severity::Info, "a"));
        let r = b.finalize();
        assert_eq!(r.findings[0].severity, Severity::High);
        assert!(r.findings[1].id <= r.findings[2].id);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/report/builder.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/report/builder.rs` captured during rig
      standardization onto the codegen ladder.
```
