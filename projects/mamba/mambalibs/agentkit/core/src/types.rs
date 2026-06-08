use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Message role in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool call request from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: serde_json::Value,
    pub error: Option<String>,
}

/// Message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,

    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Role within a governed agent team (#1545). Distinct from the chat-level
/// `Role` enum above, which models message authorship in a single
/// conversation. `AgentRole` models which functional seat in a multi-agent
/// SDD pipeline a given agent fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRole {
    PM,
    Designer,
    Dev,
    Data,
    QA,
    Release,
}

/// A governed agent team — an ordered set of roles that collaborate on one
/// SDD workflow run. Used by the cue backend to invoke `cclab-agent-mamba`
/// for end-to-end Prompt-to-Governed-App execution (#1545).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentTeam {
    pub roles: Vec<AgentRole>,
}

/// Admin review-ticket payload emitted when an agent run touches a
/// governance-flagged surface (deployment tests, SaaS API usage, costly
/// resources, PII access, high-risk capabilities). Carries the eight fields
/// required by #1545 acceptance criterion #3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTicket {
    pub workspace: String,
    pub app: String,
    pub resource: String,
    pub risk: String,
    pub environment: String,
    pub data_scope: String,
    pub output: String,
    pub rationale: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_with_tool_calls() {
        let msg = Message::assistant("I'll read the file").with_tool_calls(vec![ToolCall {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "/tmp/test.txt"}),
        }]);
        assert!(msg.tool_calls.is_some());
        assert_eq!(msg.tool_calls.unwrap().len(), 1);
    }
}
