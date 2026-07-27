// SPEC-MANAGED: apps/agentic-workflow/tech-design/src/agentic_workflow/work_items/coordination_authority.py
//! AW-owned durable state and reconciliation for coordination clients.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::protocol::{
    CoordinationAuthority, DispatchDocument, DispatchStatus, GateDocument, GateStatus, GateType,
    MessageDocument, MessageType, TaskDocument,
};

const COORDINATION_STATE_SCHEMA: &str = "aw.coordination.state.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinationDecision {
    pub gate_id: String,
    pub choice: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinationState {
    pub schema_version: String,
    pub task: TaskDocument,
    pub dispatch: DispatchDocument,
    pub gates: Vec<GateDocument>,
    pub completion_advanced: bool,
    pub decision: Option<CoordinationDecision>,
    pub events: Vec<MessageDocument>,
    pub interrupt_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationOutcome {
    pub status: &'static str,
    pub reason: String,
    pub completion_advanced: bool,
    pub decision_advanced: bool,
    pub requires_hitl: bool,
}

impl ReconciliationOutcome {
    fn blocked(reason: impl Into<String>) -> Self {
        Self {
            status: "blocked",
            reason: reason.into(),
            completion_advanced: false,
            decision_advanced: false,
            requires_hitl: false,
        }
    }
}

/// @spec #2587
pub fn open_state(
    task: TaskDocument,
    dispatch: DispatchDocument,
    gates: Vec<GateDocument>,
) -> Result<CoordinationState> {
    if dispatch.task_id != task.task_id {
        anyhow::bail!("dispatch task identity does not match task");
    }
    if dispatch.status != DispatchStatus::Active {
        anyhow::bail!("initial dispatch must be active");
    }
    if gates.len() != task.required_gates.len() {
        anyhow::bail!("gate inventory must exactly match required gates");
    }
    let required: BTreeSet<&str> = task.required_gates.iter().map(String::as_str).collect();
    let actual: BTreeSet<&str> = gates.iter().map(|gate| gate.gate_id.as_str()).collect();
    if required != actual || actual.len() != gates.len() {
        anyhow::bail!("gate inventory must exactly match required gates");
    }
    for gate in &gates {
        if gate.task_id != task.task_id {
            anyhow::bail!("gate task identity does not match task");
        }
        if gate.status != GateStatus::Pending || !gate.evidence.is_empty() {
            anyhow::bail!("clients cannot pre-satisfy AW-owned gates");
        }
    }
    Ok(CoordinationState {
        schema_version: COORDINATION_STATE_SCHEMA.to_string(),
        task,
        dispatch,
        gates,
        completion_advanced: false,
        decision: None,
        events: Vec::new(),
        interrupt_reason: None,
    })
}

/// @spec #2587
pub fn satisfy_gate(state: &mut CoordinationState, gate_id: &str, evidence: &str) -> Result<()> {
    if evidence.is_empty() {
        anyhow::bail!("gate evidence must be non-empty");
    }
    let gate = state
        .gates
        .iter_mut()
        .find(|gate| gate.gate_id == gate_id)
        .with_context(|| format!("required gate `{gate_id}` does not exist"))?;
    if gate.gate_type != GateType::Evidence {
        anyhow::bail!("gate `{gate_id}` is not an evidence gate");
    }
    gate.status = GateStatus::Satisfied;
    gate.evidence = vec![evidence.to_string()];
    Ok(())
}

/// @spec #2587
pub fn interrupt_dispatch(state: &mut CoordinationState, reason: &str) -> Result<()> {
    if reason.is_empty() {
        anyhow::bail!("interrupt reason must be non-empty");
    }
    state.dispatch.status = DispatchStatus::Interrupted;
    state.interrupt_reason = Some(reason.to_string());
    Ok(())
}

/// @spec #2587
pub fn submit_event(
    state: &mut CoordinationState,
    event: MessageDocument,
) -> ReconciliationOutcome {
    if event.task_id != state.task.task_id {
        return ReconciliationOutcome::blocked("task identity does not match AW-owned state");
    }
    if state.dispatch.status != DispatchStatus::Active
        || event.dispatch_id != state.dispatch.dispatch_id
    {
        return ReconciliationOutcome::blocked("event does not target the active dispatch");
    }
    if event.message_type == MessageType::BlockedQuestion {
        state.events.push(event);
        return ReconciliationOutcome {
            status: "blocked",
            reason: "human decision is required".to_string(),
            completion_advanced: false,
            decision_advanced: false,
            requires_hitl: true,
        };
    }
    if event.message_type != MessageType::Completion {
        state.events.push(event);
        return ReconciliationOutcome {
            status: "recorded",
            reason: "coordination event recorded without lifecycle advancement".to_string(),
            completion_advanced: false,
            decision_advanced: false,
            requires_hitl: false,
        };
    }

    for gate_id in &state.task.required_gates {
        let Some(gate) = state.gates.iter().find(|gate| &gate.gate_id == gate_id) else {
            return ReconciliationOutcome::blocked(format!("required gate `{gate_id}` is missing"));
        };
        if gate.status != GateStatus::Satisfied {
            return ReconciliationOutcome::blocked(format!(
                "required gate `{gate_id}` is not satisfied"
            ));
        }
        if gate.evidence.is_empty() || !event.evidence.contains(gate_id) {
            return ReconciliationOutcome::blocked(format!(
                "required gate `{gate_id}` lacks cited evidence"
            ));
        }
    }
    state.events.push(event);
    state.completion_advanced = true;
    ReconciliationOutcome {
        status: "done",
        reason: "AW advanced completion from active dispatch evidence".to_string(),
        completion_advanced: true,
        decision_advanced: false,
        requires_hitl: false,
    }
}

/// @spec #2587
pub fn record_decision(
    state: &mut CoordinationState,
    gate_id: &str,
    choice: &str,
    evidence: &str,
) -> Result<ReconciliationOutcome> {
    if choice.is_empty() {
        anyhow::bail!("decision choice must be non-empty");
    }
    if evidence.is_empty() {
        anyhow::bail!("human decision evidence must be non-empty");
    }
    if state.dispatch.status != DispatchStatus::Active {
        return Ok(ReconciliationOutcome::blocked(
            "decision does not target the active dispatch",
        ));
    }
    let Some(gate) = state.gates.iter_mut().find(|gate| gate.gate_id == gate_id) else {
        return Ok(ReconciliationOutcome::blocked(format!(
            "decision gate `{gate_id}` does not exist"
        )));
    };
    if gate.gate_type != GateType::Decision || gate.authority != CoordinationAuthority::Human {
        return Ok(ReconciliationOutcome::blocked(
            "decision gate requires human authority",
        ));
    }
    gate.status = GateStatus::Satisfied;
    gate.evidence = vec![evidence.to_string()];
    state.decision = Some(CoordinationDecision {
        gate_id: gate_id.to_string(),
        choice: choice.to_string(),
        evidence: evidence.to_string(),
    });
    Ok(ReconciliationOutcome {
        status: "done",
        reason: "AW recorded human decision evidence".to_string(),
        completion_advanced: false,
        decision_advanced: true,
        requires_hitl: false,
    })
}

pub fn state_path(project_root: &Path, task_id: &str) -> PathBuf {
    let digest = Sha256::digest(task_id.as_bytes());
    crate::shared::workspace::workspace_runtime_path(project_root)
        .join("coordination")
        .join(format!("{digest:x}.json"))
}

pub fn save_state(project_root: &Path, state: &CoordinationState) -> Result<PathBuf> {
    let path = state_path(project_root, &state.task.task_id);
    let parent = path
        .parent()
        .context("coordination state path has no parent")?;
    fs::create_dir_all(parent)?;
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(state)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, &path)?;
    Ok(path)
}

pub fn load_state(project_root: &Path, task_id: &str) -> Result<CoordinationState> {
    let path = state_path(project_root, task_id);
    let bytes =
        fs::read(&path).with_context(|| format!("coordination state not found for `{task_id}`"))?;
    let state: CoordinationState = serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid AW-owned coordination state: {}", path.display()))?;
    if state.schema_version != COORDINATION_STATE_SCHEMA || state.task.task_id != task_id {
        anyhow::bail!("AW-owned coordination state identity/version mismatch");
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordination::protocol::{
        AwAuthority, CoordinationVersion, DispatchKind, GateKind, MessageKind, TaskKind,
    };
    use serde_json::Map;

    fn state(required: &[(&str, GateType, CoordinationAuthority)]) -> CoordinationState {
        let task_id = "task:test".to_string();
        open_state(
            TaskDocument {
                schema_version: CoordinationVersion::V1,
                kind: TaskKind::Task,
                task_id: task_id.clone(),
                workflow_root: "change:#2587".to_string(),
                dependencies: vec![],
                required_gates: required
                    .iter()
                    .map(|(id, _, _)| (*id).to_string())
                    .collect(),
            },
            DispatchDocument {
                schema_version: CoordinationVersion::V1,
                kind: DispatchKind::Dispatch,
                task_id: task_id.clone(),
                dispatch_id: "dispatch:test:1".to_string(),
                attempt: 1,
                assignee: "agent:worker".to_string(),
                authority: AwAuthority::Aw,
                status: DispatchStatus::Active,
            },
            required
                .iter()
                .map(|(id, gate_type, authority)| GateDocument {
                    schema_version: CoordinationVersion::V1,
                    kind: GateKind::Gate,
                    gate_id: (*id).to_string(),
                    task_id: task_id.clone(),
                    gate_type: *gate_type,
                    status: GateStatus::Pending,
                    authority: *authority,
                    evidence: vec![],
                })
                .collect(),
        )
        .unwrap()
    }

    fn completion(evidence: &[&str]) -> MessageDocument {
        MessageDocument {
            schema_version: CoordinationVersion::V1,
            kind: MessageKind::Message,
            event_id: "event:test:1".to_string(),
            task_id: "task:test".to_string(),
            dispatch_id: "dispatch:test:1".to_string(),
            sequence: 1,
            sender: "agent:worker".to_string(),
            message_type: MessageType::Completion,
            evidence: evidence.iter().map(|value| (*value).to_string()).collect(),
            body: Map::new(),
        }
    }

    /// @spec #2587
    #[test]
    fn coordination_authority_requires_active_dispatch_and_all_gate_evidence() {
        let mut current = state(&[
            ("gate:test", GateType::Evidence, CoordinationAuthority::Aw),
            ("gate:lint", GateType::Evidence, CoordinationAuthority::Aw),
        ]);
        satisfy_gate(&mut current, "gate:test", "evidence:test").unwrap();
        assert!(!submit_event(&mut current, completion(&["gate:test"])).completion_advanced);
        satisfy_gate(&mut current, "gate:lint", "evidence:lint").unwrap();
        assert!(
            submit_event(&mut current, completion(&["gate:test", "gate:lint"])).completion_advanced
        );

        let mut interrupted =
            state(&[("gate:test", GateType::Evidence, CoordinationAuthority::Aw)]);
        satisfy_gate(&mut interrupted, "gate:test", "evidence:test").unwrap();
        interrupt_dispatch(&mut interrupted, "worker-lost").unwrap();
        assert!(!submit_event(&mut interrupted, completion(&["gate:test"])).completion_advanced);
    }

    /// @spec #2587
    #[test]
    fn coordination_authority_records_only_nonempty_human_decisions() {
        let mut current = state(&[(
            "gate:approval",
            GateType::Decision,
            CoordinationAuthority::Human,
        )]);
        assert!(record_decision(&mut current, "gate:approval", "", "human:42").is_err());
        assert!(current.decision.is_none());
        assert!(record_decision(&mut current, "gate:approval", "approved", "").is_err());
        assert!(current.decision.is_none());
        let outcome =
            record_decision(&mut current, "gate:approval", "approved", "human:42").unwrap();
        assert!(outcome.decision_advanced);
        assert_eq!(current.decision.unwrap().choice, "approved");
    }
}
