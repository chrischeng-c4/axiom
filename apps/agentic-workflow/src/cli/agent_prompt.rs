// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/specs/aw-agent-prompt-contract.md
// HANDWRITE-BEGIN typed-agent-prompt-contract
//! Typed lifecycle-to-agent projection for `aw.cli.v1`.
//!
//! The workflow engine supplies every value. This module validates and renders
//! that projection; it does not select transitions, execute expressions, or
//! decide completion.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const PROMPT_SCHEMA_VERSION: &str = "aw.prompt.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PromptTerminalLevel {
    Stage,
    Change,
    Root,
}

impl PromptTerminalLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stage => "stage",
            Self::Change => "change",
            Self::Root => "root",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PromptBlockerKind {
    Decision,
    Approval,
    Environment,
    RedGate,
    MissingEvidence,
}

impl PromptBlockerKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Approval => "approval",
            Self::Environment => "environment",
            Self::RedGate => "red_gate",
            Self::MissingEvidence => "missing_evidence",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PromptArtifact {
    pub(crate) kind: String,
    pub(crate) id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PromptScope {
    pub(crate) writable: Vec<String>,
    pub(crate) readonly: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PromptTransition {
    pub(crate) command: String,
    pub(crate) next_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PromptVerifier {
    pub(crate) command: String,
    pub(crate) predicate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PromptTerminal {
    pub(crate) level: PromptTerminalLevel,
    pub(crate) predicate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PromptBlocker {
    pub(crate) kind: PromptBlockerKind,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentPromptSpec {
    pub(crate) schema_version: String,
    pub(crate) state: String,
    pub(crate) artifact: PromptArtifact,
    pub(crate) scope: PromptScope,
    pub(crate) transition: PromptTransition,
    pub(crate) verifier: PromptVerifier,
    pub(crate) terminal: PromptTerminal,
    pub(crate) guards: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) blocker: Option<PromptBlocker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resume_command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) guidance: Vec<String>,
}

impl AgentPromptSpec {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.schema_version != PROMPT_SCHEMA_VERSION {
            return Err(format!("schema_version must be `{PROMPT_SCHEMA_VERSION}`"));
        }
        for (field, value) in [
            ("state", self.state.as_str()),
            ("artifact.kind", self.artifact.kind.as_str()),
            ("artifact.id", self.artifact.id.as_str()),
            ("terminal.predicate", self.terminal.predicate.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{field} must not be empty"));
            }
        }
        let writable = self
            .scope
            .writable
            .iter()
            .map(|path| path.trim())
            .filter(|path| !path.is_empty())
            .collect::<BTreeSet<_>>();
        let readonly = self
            .scope
            .readonly
            .iter()
            .map(|path| path.trim())
            .filter(|path| !path.is_empty())
            .collect::<BTreeSet<_>>();
        if let Some(path) = writable.intersection(&readonly).next() {
            return Err(format!(
                "scope path `{path}` cannot be both writable and readonly"
            ));
        }
        if self.transition.command.trim().is_empty()
            && self.terminal.predicate != "completion.workflow_complete == true"
        {
            return Err("non-root-terminal prompt requires transition.command".to_string());
        }
        if self.verifier.predicate.trim().is_empty() {
            return Err("verifier predicate must not be empty".to_string());
        }
        if self.verifier.command.trim().is_empty()
            && self.terminal.predicate != "completion.workflow_complete == true"
        {
            return Err("non-root-terminal prompt requires verifier.command".to_string());
        }
        if self.blocker.is_some()
            && self
                .resume_command
                .as_deref()
                .is_none_or(|command| command.trim().is_empty())
        {
            return Err("blocked prompt requires resume_command".to_string());
        }
        for expression in self
            .guards
            .iter()
            .chain(std::iter::once(&self.verifier.predicate))
            .chain(std::iter::once(&self.terminal.predicate))
        {
            if expression.contains('→') || expression.contains('⇒') {
                return Err("prompt expressions must use canonical ASCII operators".to_string());
            }
        }
        Ok(())
    }

    pub(crate) fn render(&self) -> Result<String, String> {
        self.validate()?;
        let writable = render_set(&self.scope.writable);
        let readonly = render_set(&self.scope.readonly);
        let mut lines = vec![
            format!("state := {}", self.state),
            format!("artifact := {}:{}", self.artifact.kind, self.artifact.id),
            format!("scope.writable := {writable}"),
            format!("scope.readonly := {readonly}"),
        ];
        if !self.transition.command.trim().is_empty() {
            lines.push(format!("{} -> {}", self.state, self.transition.next_state));
            lines.push(format!("next.command := `{}`", self.transition.command));
        }
        if !self.verifier.command.trim().is_empty() {
            lines.push(format!(
                "`{}` --gate-> {}",
                self.verifier.command, self.verifier.predicate
            ));
        }
        lines.push(format!(
            "terminal.{} --gate-> {}",
            self.terminal.level.as_str(),
            self.terminal.predicate
        ));
        lines.extend(self.guards.iter().map(|guard| format!("guard := {guard}")));
        if let Some(blocker) = &self.blocker {
            lines.push(format!(
                "blocker := {}: {}",
                blocker.kind.as_str(),
                blocker.reason
            ));
        }
        if let Some(resume) = &self.resume_command {
            lines.push(format!("resume := `{resume}`"));
        }
        lines.extend(
            self.guidance
                .iter()
                .map(|guidance| format!("guidance := {guidance}")),
        );
        Ok(lines.join("\n"))
    }
}

fn render_set(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    format!("{{{}}}", values.into_iter().collect::<Vec<_>>().join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> AgentPromptSpec {
        AgentPromptSpec {
            schema_version: PROMPT_SCHEMA_VERSION.to_string(),
            state: "td.authored".to_string(),
            artifact: PromptArtifact {
                kind: "td".to_string(),
                id: "#2440".to_string(),
            },
            scope: PromptScope {
                writable: vec!["tech-design/2440".to_string()],
                readonly: vec!["external-contracts/2440".to_string()],
            },
            transition: PromptTransition {
                command: "aw ec verify --stage td --wi 2440".to_string(),
                next_state: "ec_td_verifying".to_string(),
            },
            verifier: PromptVerifier {
                command: "aw ec verify --stage td --wi 2440".to_string(),
                predicate: "EC[TD].behavior == green".to_string(),
            },
            terminal: PromptTerminal {
                level: PromptTerminalLevel::Root,
                predicate: "completion.workflow_complete == true".to_string(),
            },
            guards: vec!["action == done != completion.workflow_complete".to_string()],
            blocker: None,
            resume_command: None,
            guidance: Vec::new(),
        }
    }

    #[test]
    fn typed_prompt_round_trips_and_renders_deterministically() {
        let contract = contract();
        let json = serde_json::to_string(&contract).unwrap();
        let decoded: AgentPromptSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, contract);
        assert_eq!(contract.render().unwrap(), decoded.render().unwrap());
        assert!(json.contains("\"schema_version\":\"aw.prompt.v1\""));
    }

    #[test]
    fn typed_prompt_rejects_overlapping_scope() {
        let mut contract = contract();
        contract.scope.readonly = contract.scope.writable.clone();
        assert!(contract.validate().unwrap_err().contains("both writable"));
    }

    #[test]
    fn typed_prompt_requires_blocker_resume() {
        let mut contract = contract();
        contract.blocker = Some(PromptBlocker {
            kind: PromptBlockerKind::Approval,
            reason: "independent EC approval required".to_string(),
        });
        assert!(contract.validate().unwrap_err().contains("resume_command"));
    }
}
// HANDWRITE-END typed-agent-prompt-contract
