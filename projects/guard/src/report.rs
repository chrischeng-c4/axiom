// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#rust-source-unit
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::baseline::Baseline;

pub const SCHEMA_VERSION: &str = "guard.report/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
impl Severity {
    pub fn rank(self) -> u8 {
        match self {
            Severity::Critical => 0,
            Severity::High => 1,
            Severity::Medium => 2,
            Severity::Low => 3,
            Severity::Info => 4,
        }
    }

    pub fn is_actionable(self) -> bool {
        !matches!(self, Severity::Info)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub enum OverallStatus {
    Clean,
    Findings,
    ToolError { code: u8 },
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
impl OverallStatus {
    pub fn exit_code(self) -> i32 {
        match self {
            OverallStatus::Clean => 0,
            OverallStatus::Findings => 1,
            OverallStatus::ToolError { code } => code as i32,
        }
    }

    pub fn is_clean(self) -> bool {
        matches!(self, OverallStatus::Clean)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub struct Location {
    pub path: String,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub rule: String,
    pub title: String,
    pub detail: String,
    pub remediation: String,
    pub location: Location,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub struct Summary {
    pub files_scanned: u32,
    pub diagnostics_scanned: u32,
    pub security_findings: u32,
    /// Findings absent from the accepted baseline (the gate count). Equals
    /// `security_findings` until a baseline is applied.
    pub new_findings: u32,
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub info: u32,
    pub sample: Vec<String>,
    pub truncated: bool,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
impl Summary {
    pub fn from_findings(
        files_scanned: usize,
        diagnostics_scanned: usize,
        findings: &[Finding],
    ) -> Self {
        let mut summary = Self {
            files_scanned: files_scanned as u32,
            diagnostics_scanned: diagnostics_scanned as u32,
            security_findings: findings.len() as u32,
            new_findings: findings.len() as u32,
            truncated: false,
            ..Self::default()
        };
        for finding in findings {
            match finding.severity {
                Severity::Critical => summary.critical += 1,
                Severity::High => summary.high += 1,
                Severity::Medium => summary.medium += 1,
                Severity::Low => summary.low += 1,
                Severity::Info => summary.info += 1,
            }
        }
        summary.sample = findings.iter().take(8).map(|f| f.id.clone()).collect();
        summary
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub struct Completion {
    pub clean: bool,
    pub criteria: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub struct IntegrationMap {
    pub static_engine: String,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
impl Default for IntegrationMap {
    fn default() -> Self {
        Self {
            static_engine: "compass".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub struct GuardReport {
    pub schema_version: String,
    pub tool_version: String,
    pub verb: String,
    pub target: String,
    pub policy_profile: String,
    pub status: OverallStatus,
    pub exit_code: i32,
    pub summary: Summary,
    pub findings: Vec<Finding>,
    pub completion: Completion,
    pub integrations: IntegrationMap,
    pub agent_prompt: String,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
impl GuardReport {
    pub fn from_scan(
        target: impl Into<String>,
        policy_profile: impl Into<String>,
        files_scanned: usize,
        diagnostics_scanned: usize,
        mut findings: Vec<Finding>,
    ) -> Self {
        findings.sort_by(|a, b| {
            a.severity
                .rank()
                .cmp(&b.severity.rank())
                .then_with(|| a.id.cmp(&b.id))
        });
        let actionable = findings.iter().any(|f| f.severity.is_actionable());
        let status = if actionable {
            OverallStatus::Findings
        } else {
            OverallStatus::Clean
        };
        let summary = Summary::from_findings(files_scanned, diagnostics_scanned, &findings);
        let completion = Completion {
            clean: status.is_clean(),
            criteria: vec![
                "compass security diagnostics were scanned".to_string(),
                "findings were normalized into guard.report/1".to_string(),
            ],
            missing: Vec::new(),
        };
        let agent_prompt = if status.is_clean() {
            "guard scan is clean for static security findings".to_string()
        } else {
            format!(
                "guard found {} security finding(s); inspect summary.sample and findings",
                summary.security_findings
            )
        };
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            verb: "scan".to_string(),
            target: target.into(),
            policy_profile: policy_profile.into(),
            exit_code: status.exit_code(),
            status,
            summary,
            findings,
            completion,
            integrations: IntegrationMap::default(),
            agent_prompt,
        }
    }

    /// Re-gate against an accepted baseline: only findings ABSENT from the
    /// baseline count toward the exit code. Known/accepted findings stay in the
    /// report for visibility but no longer turn the gate red. No-op on tool
    /// errors.
    pub fn apply_baseline(&mut self, baseline: &Baseline) {
        if matches!(self.status, OverallStatus::ToolError { .. }) {
            return;
        }
        let new_actionable = self
            .findings
            .iter()
            .filter(|f| f.severity.is_actionable() && !baseline.contains(&f.id))
            .count() as u32;
        self.summary.new_findings = new_actionable;
        self.status = if new_actionable > 0 {
            OverallStatus::Findings
        } else {
            OverallStatus::Clean
        };
        self.exit_code = self.status.exit_code();
        self.completion.clean = self.status.is_clean();
        self.agent_prompt = if new_actionable > 0 {
            format!(
                "guard found {new_actionable} new security finding(s) absent from the accepted baseline; inspect summary.sample and findings"
            )
        } else if self.summary.security_findings > 0 {
            format!(
                "guard scan: {} finding(s), all present in the accepted baseline",
                self.summary.security_findings
            )
        } else {
            "guard scan is clean for static security findings".to_string()
        };
    }

    pub fn tool_error(
        verb: &str,
        target: impl Into<String>,
        code: u8,
        message: impl Into<String>,
    ) -> Self {
        let status = OverallStatus::ToolError { code };
        let message = message.into();
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            verb: verb.to_string(),
            target: target.into(),
            policy_profile: "guard-baseline-static/1".to_string(),
            status,
            exit_code: status.exit_code(),
            summary: Summary::default(),
            findings: Vec::new(),
            completion: Completion {
                clean: false,
                criteria: Vec::new(),
                missing: vec![message.clone()],
            },
            integrations: IntegrationMap::default(),
            agent_prompt: format!("guard {verb} could not run: {message}"),
        }
    }

    pub fn persist(&self, dir: &Path) {
        let report_dir = dir.join(".guard");
        if std::fs::create_dir_all(&report_dir).is_err() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(report_dir.join("last-report.json"), json);
        }
    }

    pub fn read_last(dir: &Path) -> anyhow::Result<Self> {
        let path = dir.join(".guard/last-report.json");
        let text = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&text)?)
    }
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
pub fn finding_id(rule: &str, path: &str, line: u32) -> String {
    let subject = format!("{path}:{line}");
    let squashed: String = subject
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    format!("compass:{rule}:{squashed}")
}
// CODEGEN-END
