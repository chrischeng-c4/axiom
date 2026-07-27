// SPEC-MANAGED: apps/agentic-workflow/tech-design/src/agentic_workflow/work_items/coordination_contract_schema.py
//! Strongly typed projection of the public AW coordination JSON Schemas.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const AW_COORDINATION_SCHEMA_VERSION: &str = "aw.coordination.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinationVersion {
    #[serde(rename = "aw.coordination.v1")]
    V1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskKind {
    #[serde(rename = "task")]
    Task,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskDocument {
    pub schema_version: CoordinationVersion,
    pub kind: TaskKind,
    pub task_id: String,
    pub workflow_root: String,
    pub dependencies: Vec<String>,
    pub required_gates: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispatchKind {
    #[serde(rename = "dispatch")]
    Dispatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AwAuthority {
    #[serde(rename = "aw")]
    Aw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchStatus {
    Active,
    Superseded,
    Interrupted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchDocument {
    pub schema_version: CoordinationVersion,
    pub kind: DispatchKind,
    pub task_id: String,
    pub dispatch_id: String,
    pub attempt: u64,
    pub assignee: String,
    pub authority: AwAuthority,
    pub status: DispatchStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    #[serde(rename = "message")]
    Message,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Heartbeat,
    Completion,
    Escalation,
    BlockedQuestion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageDocument {
    pub schema_version: CoordinationVersion,
    pub kind: MessageKind,
    pub event_id: String,
    pub task_id: String,
    pub dispatch_id: String,
    pub sequence: u64,
    pub sender: String,
    pub message_type: MessageType,
    pub evidence: Vec<String>,
    pub body: Map<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateKind {
    #[serde(rename = "gate")]
    Gate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateType {
    Evidence,
    Decision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Pending,
    Satisfied,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationAuthority {
    Aw,
    Human,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDocument {
    pub schema_version: CoordinationVersion,
    pub kind: GateKind,
    pub gate_id: String,
    pub task_id: String,
    pub gate_type: GateType,
    pub status: GateStatus,
    pub authority: CoordinationAuthority,
    pub evidence: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;

    fn round_trip<T>(fixture: &str) -> T
    where
        T: DeserializeOwned + Serialize + PartialEq + std::fmt::Debug,
    {
        let decoded: T = serde_json::from_str(fixture).expect("fixture decodes");
        let encoded = serde_json::to_string(&decoded).expect("fixture encodes");
        let reparsed: T = serde_json::from_str(&encoded).expect("round-trip decodes");
        assert_eq!(decoded, reparsed);
        decoded
    }

    /// @spec #2586
    #[test]
    fn coordination_documents_round_trip_and_reject_unknown_versions() {
        round_trip::<TaskDocument>(
            r#"{"schema_version":"aw.coordination.v1","kind":"task","task_id":"task:2586","workflow_root":"change:#2586","dependencies":[],"required_gates":["gate:contract"]}"#,
        );
        round_trip::<DispatchDocument>(
            r#"{"schema_version":"aw.coordination.v1","kind":"dispatch","task_id":"task:2586","dispatch_id":"dispatch:2586:1","attempt":1,"assignee":"agent:worker","authority":"aw","status":"active"}"#,
        );
        round_trip::<MessageDocument>(
            r#"{"schema_version":"aw.coordination.v1","kind":"message","event_id":"event:2586:1","task_id":"task:2586","dispatch_id":"dispatch:2586:1","sequence":1,"sender":"agent:worker","message_type":"heartbeat","evidence":[],"body":{}}"#,
        );
        round_trip::<GateDocument>(
            r#"{"schema_version":"aw.coordination.v1","kind":"gate","gate_id":"gate:contract","task_id":"task:2586","gate_type":"evidence","status":"pending","authority":"aw","evidence":[]}"#,
        );

        let unknown_version = r#"{"schema_version":"aw.coordination.v999","kind":"task","task_id":"task:2586","workflow_root":"change:#2586","dependencies":[],"required_gates":[]}"#;
        let error = serde_json::from_str::<TaskDocument>(unknown_version)
            .expect_err("unknown protocol versions fail closed");
        assert!(error.to_string().contains("unknown variant"));
    }

    /// @spec #2586
    #[test]
    fn coordination_documents_reject_unknown_fields_and_wrong_kinds() {
        let unknown_field = r#"{"schema_version":"aw.coordination.v1","kind":"gate","gate_id":"gate:contract","task_id":"task:2586","gate_type":"evidence","status":"pending","authority":"aw","evidence":[],"client_state":true}"#;
        assert!(serde_json::from_str::<GateDocument>(unknown_field).is_err());

        let wrong_kind = r#"{"schema_version":"aw.coordination.v1","kind":"message","task_id":"task:2586","dispatch_id":"dispatch:2586:1","attempt":1,"assignee":"agent:worker","authority":"aw","status":"active"}"#;
        assert!(serde_json::from_str::<DispatchDocument>(wrong_kind).is_err());
    }
}
