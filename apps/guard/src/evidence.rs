// SPEC-MANAGED: apps/guard/tech-design/src/evidence.py
// HANDWRITE-BEGIN gap="python-td-rust-body" tracker="#2823" reason="Guard evidence domain behavior remains native Rust"
// @spec WI #2931: executable Python TD parity baseline.
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::report::{Finding, Location, Severity};

#[derive(Debug, Clone)]
/// @spec apps/guard/tech-design/src/evidence.py
pub struct EvidenceCommand {
    pub tool: String,
    pub label: String,
    pub command: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
}

/// @spec apps/guard/tech-design/src/evidence.py
impl EvidenceCommand {
    pub fn argv(tool: impl Into<String>, label: impl Into<String>, command: Vec<String>) -> Self {
        Self {
            tool: tool.into(),
            label: label.into(),
            command,
            cwd: None,
            env: BTreeMap::new(),
        }
    }

    pub fn shell(
        tool: impl Into<String>,
        label: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        let command = command.into();
        Self::argv(
            tool,
            label,
            vec!["sh".to_string(), "-c".to_string(), command],
        )
    }

    pub fn display_command(&self) -> String {
        self.command.join(" ")
    }

    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// @spec apps/guard/tech-design/src/evidence.py
pub enum EvidenceStatus {
    Clean,
    Findings,
    ToolError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec apps/guard/tech-design/src/evidence.py
pub struct ExternalEvidence {
    pub tool: String,
    pub label: String,
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    pub status: EvidenceStatus,
    pub clean: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub finding_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stderr_tail: String,
}

/// @spec apps/guard/tech-design/src/evidence.py
impl ExternalEvidence {
    pub fn to_guard_finding(&self, target: &str) -> Option<Finding> {
        if self.clean {
            return None;
        }
        let rule = format!("{}-EVIDENCE", self.tool.to_ascii_uppercase());
        let command = self.command.join(" ");
        let severity = match self.status {
            EvidenceStatus::ToolError => Severity::High,
            EvidenceStatus::Findings => Severity::High,
            EvidenceStatus::Clean => Severity::Info,
        };
        Some(Finding {
            id: format!("evidence:{}:{}", squash(&self.tool), squash(&self.label)),
            severity,
            rule,
            title: format!("{} security evidence is not clean", self.tool),
            detail: format!(
                "`{command}` returned {:?} with {} finding(s)",
                self.exit_code, self.finding_count
            ),
            remediation: format!(
                "Inspect the {} report, fix the finding, then rerun `{command}`.",
                self.tool
            ),
            location: Location {
                path: target.to_string(),
                start_line: 0,
                start_col: 0,
                end_line: 0,
                end_col: 0,
            },
            evidence: serde_json::json!({
                "source": self.tool,
                "label": self.label,
                "command": self.command,
                "cwd": self.cwd,
                "env": self.env,
                "exit_code": self.exit_code,
                "report": self.report,
                "stderr_tail": self.stderr_tail,
            }),
        })
    }
}

/// @spec apps/guard/tech-design/src/evidence.py
pub fn run_evidence_commands(commands: &[EvidenceCommand]) -> Vec<ExternalEvidence> {
    commands.iter().map(run_one).collect()
}

fn run_one(command: &EvidenceCommand) -> ExternalEvidence {
    if command.command.is_empty() {
        return ExternalEvidence {
            tool: command.tool.clone(),
            label: command.label.clone(),
            command: Vec::new(),
            cwd: command.cwd.as_ref().map(|cwd| cwd.display().to_string()),
            env: command.env.clone(),
            status: EvidenceStatus::ToolError,
            clean: false,
            exit_code: None,
            finding_count: 0,
            report: None,
            stderr_tail: "empty evidence command".to_string(),
        };
    }

    let mut child = Command::new(&command.command[0]);
    child.args(&command.command[1..]);
    child.envs(&command.env);
    if let Some(cwd) = &command.cwd {
        child.current_dir(cwd);
    }
    let output = child.output();
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return ExternalEvidence {
                tool: command.tool.clone(),
                label: command.label.clone(),
                command: command.command.clone(),
                cwd: command.cwd.as_ref().map(|cwd| cwd.display().to_string()),
                env: command.env.clone(),
                status: EvidenceStatus::ToolError,
                clean: false,
                exit_code: None,
                finding_count: 0,
                report: None,
                stderr_tail: err.to_string(),
            };
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr_tail = tail_lossy(&output.stderr, 2000);
    let parsed = parse_json_payload(&stdout);
    let exit_code = output.status.code();
    let process_clean = output.status.success();
    let finding_count = parsed.as_ref().map(finding_count).unwrap_or(0);
    let report_clean = parsed.as_ref().is_some_and(report_clean);
    let clean = process_clean && report_clean && finding_count == 0;
    let status = if clean {
        EvidenceStatus::Clean
    } else if parsed.is_some() || exit_code.is_some() {
        EvidenceStatus::Findings
    } else {
        EvidenceStatus::ToolError
    };

    ExternalEvidence {
        tool: command.tool.clone(),
        label: command.label.clone(),
        command: command.command.clone(),
        cwd: command.cwd.as_ref().map(|cwd| cwd.display().to_string()),
        env: command.env.clone(),
        status,
        clean,
        exit_code,
        finding_count,
        report: parsed.as_ref().map(compact_report),
        stderr_tail,
    }
}

fn parse_json_payload(stdout: &str) -> Option<serde_json::Value> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(value) = serde_json::from_str(trimmed) {
        return Some(value);
    }
    trimmed
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line.trim()).ok())
}

fn report_clean(value: &serde_json::Value) -> bool {
    if value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .is_none_or(str::is_empty)
    {
        return false;
    }
    let projected = value
        .get("clean")
        .and_then(|v| v.as_bool())
        .or_else(|| {
            value
                .get("completion")
                .and_then(|v| v.get("clean"))
                .and_then(|v| v.as_bool())
        })
        .or_else(|| {
            value
                .get("status")
                .and_then(|v| v.get("state"))
                .and_then(|v| v.as_str())
                .map(|state| state == "clean")
        })
        .or_else(|| value.get("ok").and_then(|v| v.as_bool()));
    let counts = finding_counts(value);
    let counts_are_consistent =
        !counts.is_empty() && counts.iter().all(|count| *count == counts[0]);
    projected == Some(true) && counts_are_consistent && counts[0] == 0
}

fn finding_count(value: &serde_json::Value) -> u32 {
    finding_counts(value).into_iter().max().unwrap_or(0)
}

fn finding_counts(value: &serde_json::Value) -> Vec<u32> {
    let mut counts = Vec::new();
    let summary_count = value
        .get("summary")
        .and_then(|v| v.get("security_findings"))
        .and_then(|v| v.as_u64())
        .or_else(|| {
            value
                .get("summary")
                .and_then(|v| v.get("total"))
                .and_then(|v| v.as_u64())
        });
    if let Some(count) = summary_count {
        counts.push(count as u32);
    }
    if let Some(findings) = value.get("findings").and_then(|v| v.as_array()) {
        counts.push(findings.len() as u32);
    }
    counts
}

fn compact_report(value: &serde_json::Value) -> serde_json::Value {
    let findings_preview = value
        .get("findings")
        .and_then(|v| v.as_array())
        .map(|items| serde_json::Value::Array(items.iter().take(4).cloned().collect::<Vec<_>>()));
    serde_json::json!({
        "schema_version": value.get("schema_version").cloned().unwrap_or(serde_json::Value::Null),
        "status": value.get("status").cloned().unwrap_or(serde_json::Value::Null),
        "clean": value.get("clean").cloned().unwrap_or(serde_json::Value::Null),
        "summary": value.get("summary").cloned().unwrap_or(serde_json::Value::Null),
        "completion": value.get("completion").cloned().unwrap_or(serde_json::Value::Null),
        "agent_prompt": value.get("agent_prompt").cloned().unwrap_or(serde_json::Value::Null),
        "ok": value.get("ok").cloned().unwrap_or(serde_json::Value::Null),
        "runner": value.get("runner").cloned().unwrap_or(serde_json::Value::Null),
        "runners": value.get("runners").cloned().unwrap_or(serde_json::Value::Null),
        "findings_preview": findings_preview.unwrap_or(serde_json::Value::Array(Vec::new())),
    })
}

fn tail_lossy(bytes: &[u8], max: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    if text.len() <= max {
        return text.to_string();
    }
    text[text.len() - max..].to_string()
}

fn squash(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_json_report_is_clean_evidence() {
        let command = EvidenceCommand::shell(
            "rig",
            "clean",
            "printf '%s' '{\"schema_version\":\"rig.report/1\",\"clean\":true,\"summary\":{\"total\":0}}'",
        );
        let evidence = run_evidence_commands(&[command]);
        assert_eq!(evidence.len(), 1);
        assert!(evidence[0].clean);
        assert_eq!(evidence[0].finding_count, 0);
    }

    #[test]
    fn nonclean_json_report_becomes_guard_finding() {
        let command = EvidenceCommand::shell(
            "rig",
            "attack",
            "printf '%s' '{\"schema_version\":\"rig.report/1\",\"clean\":false,\"summary\":{\"total\":2},\"findings\":[{\"id\":\"x\"}]}'",
        );
        let evidence = run_evidence_commands(&[command]);
        assert!(!evidence[0].clean);
        assert_eq!(evidence[0].finding_count, 2);
        let finding = evidence[0].to_guard_finding("demo").unwrap();
        assert_eq!(finding.rule, "RIG-EVIDENCE");
    }

    #[test]
    fn malformed_or_verdict_free_json_fails_closed() {
        for (label, command) in [
            ("empty", "true"),
            ("object", "printf '%s' '{}'"),
            (
                "findings-only",
                "printf '%s' '{\"schema_version\":\"rig.report/1\",\"findings\":[{\"id\":\"x\"}]}'",
            ),
            (
                "contradiction",
                "printf '%s' '{\"schema_version\":\"rig.report/1\",\"clean\":true,\"summary\":{\"total\":0},\"findings\":[{\"id\":\"x\"}]}'",
            ),
            (
                "countless-clean",
                "printf '%s' '{\"schema_version\":\"rig.report/1\",\"clean\":true}'",
            ),
        ] {
            let evidence =
                run_evidence_commands(&[EvidenceCommand::shell("rig", label, command)]);
            assert!(!evidence[0].clean, "{label} unexpectedly passed");
        }
    }
}
// HANDWRITE-END
