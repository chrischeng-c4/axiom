// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#rust-source-unit
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::evidence::ExternalEvidence;

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
    pub evidence_count: u32,
    pub evidence_failed: u32,
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
        evidence: &[ExternalEvidence],
    ) -> Self {
        let mut summary = Self {
            files_scanned: files_scanned as u32,
            diagnostics_scanned: diagnostics_scanned as u32,
            security_findings: findings.len() as u32,
            evidence_count: evidence.len() as u32,
            evidence_failed: evidence.iter().filter(|item| !item.clean).count() as u32,
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
    pub isolated_runner: String,
    pub dynamic_journeys: String,
    pub resource_evidence: String,
    pub benchmark_budget: String,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
impl Default for IntegrationMap {
    fn default() -> Self {
        Self {
            static_engine: "compass".to_string(),
            isolated_runner: "vat".to_string(),
            dynamic_journeys: "rig".to_string(),
            resource_evidence: "meter".to_string(),
            benchmark_budget: "legacy arena (optional)".to_string(),
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<ExternalEvidence>,
    pub completion: Completion,
    pub integrations: IntegrationMap,
    pub agent_prompt: String,
}

/// @spec projects/guard/tech-design/semantic/source/projects-guard-src-report-rs.md#source
impl GuardReport {
    pub fn from_scan(
        target: impl Into<String>,
        files_scanned: usize,
        diagnostics_scanned: usize,
        findings: Vec<Finding>,
    ) -> Self {
        Self::from_scan_with_evidence(
            target,
            "guard-baseline-static/1",
            files_scanned,
            diagnostics_scanned,
            findings,
            Vec::new(),
        )
    }

    pub fn from_scan_with_evidence(
        target: impl Into<String>,
        policy_profile: impl Into<String>,
        files_scanned: usize,
        diagnostics_scanned: usize,
        mut findings: Vec<Finding>,
        evidence: Vec<ExternalEvidence>,
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
        let summary =
            Summary::from_findings(files_scanned, diagnostics_scanned, &findings, &evidence);
        let missing = missing_integrations(&evidence);
        let completion = Completion {
            clean: status.is_clean(),
            criteria: vec![
                "compass security diagnostics were scanned".to_string(),
                "findings were normalized into guard.report/1".to_string(),
                "vat/rig/meter evidence adapters are available; arena evidence is legacy optional"
                    .to_string(),
            ],
            missing,
        };
        let agent_prompt = if status.is_clean() {
            if completion.missing.is_empty() {
                "guard scan is clean for static and dynamic security evidence".to_string()
            } else {
                "guard scan is clean for the configured security evidence".to_string()
            }
        } else {
            format!(
                "guard found {} security finding(s); inspect summary.sample, findings, and evidence",
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
            evidence,
            completion,
            integrations: IntegrationMap::default(),
            agent_prompt,
        }
    }

    pub fn stub(verb: &str, prompt: impl Into<String>) -> Self {
        let status = OverallStatus::Clean;
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            verb: verb.to_string(),
            target: "-".to_string(),
            policy_profile: "guard-baseline-static/1".to_string(),
            status,
            exit_code: status.exit_code(),
            summary: Summary::default(),
            findings: Vec::new(),
            evidence: Vec::new(),
            completion: Completion {
                clean: true,
                criteria: vec!["offline self-description emitted".to_string()],
                missing: Vec::new(),
            },
            integrations: IntegrationMap::default(),
            agent_prompt: prompt.into(),
        }
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
            evidence: Vec::new(),
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

fn missing_integrations(evidence: &[ExternalEvidence]) -> Vec<String> {
    let has_tool = |tool: &str| evidence.iter().any(|item| item.tool == tool);
    let mut missing = Vec::new();
    if !has_tool("vat") {
        missing.push("vat isolated security runner evidence is not configured".to_string());
    }
    if !has_tool("rig") {
        missing.push("rig exploit/e2e journey evidence is not configured".to_string());
    }
    if !has_tool("meter") {
        missing.push("meter DoS/resource evidence is not configured".to_string());
    }
    missing
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
