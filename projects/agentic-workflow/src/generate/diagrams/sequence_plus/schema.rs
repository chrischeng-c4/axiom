//! Sequence+ definition schema
//!
//! Structured sequence diagram definitions with loops, alt blocks, and notes.

use std::collections::HashMap;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Alt block type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AltBlockType {
    /// Alt block (default).
    #[default]
    Alt,
    /// Opt block.
    Opt,
    /// Par block (parallel).
    Par,
    /// Critical block.
    Critical,
    /// Break block.
    Break,
}

/// Alt/Opt block definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltDef {
    /// Block type.
    #[serde(rename = "type", default)]
    pub block_type: AltBlockType,
    /// Primary condition.
    pub condition: String,
    /// Start message index (0-based).
    pub start: usize,
    /// End message index for primary branch (0-based, inclusive).
    pub end: usize,
    /// Else branches (for alt blocks).
    #[serde(default)]
    pub else_branches: Vec<ElseBranch>,
}

/// Arrow type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArrowType {
    /// Solid arrow (default).
    #[default]
    Solid,
    /// Dotted arrow.
    Dotted,
    /// Solid arrow with open arrowhead.
    SolidOpen,
    /// Dotted arrow with open arrowhead.
    DottedOpen,
}

/// Else branch definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseBranch {
    /// Condition (empty for final else).
    #[serde(default)]
    pub condition: Option<String>,
    /// Start message index.
    pub start: usize,
    /// End message index.
    pub end: usize,
}

/// Loop block definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopDef {
    /// Loop condition/label.
    pub label: String,
    /// Start message index (0-based).
    pub start: usize,
    /// End message index (0-based, inclusive).
    pub end: usize,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Message definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDef {
    /// Source participant ID.
    pub from: String,
    /// Target participant ID.
    pub to: String,
    /// Message text.
    pub text: String,
    /// Arrow type.
    #[serde(rename = "type", default)]
    pub arrow_type: ArrowType,
    /// Activate target on this message.
    #[serde(default)]
    pub activate: bool,
    /// Deactivate source after this message.
    #[serde(default)]
    pub deactivate: bool,
    /// Message description (for documentation).
    #[serde(default)]
    pub description: Option<String>,
}

/// Note definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDef {
    /// Note text.
    pub text: String,
    /// Position.
    #[serde(default)]
    pub position: NotePosition,
    /// Participant ID(s) the note is attached to.
    #[serde(default)]
    pub participants: Vec<String>,
    /// After which message index (optional).
    #[serde(default)]
    pub after_message: Option<usize>,
}

/// Note position.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotePosition {
    /// Right of (default).
    #[default]
    RightOf,
    /// Left of.
    LeftOf,
    /// Over.
    Over,
}

/// Participant definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantDef {
    /// Display label.
    pub label: String,
    /// Participant type.
    #[serde(rename = "type", default)]
    pub participant_type: ParticipantType,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Participant type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantType {
    /// Standard participant (default).
    #[default]
    Participant,
    /// Actor (stick-figure) participant.
    Actor,
}

/// Sequence diagram definition (input from LLM).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceDef {
    /// Diagram identifier.
    pub id: String,
    /// Diagram title.
    #[serde(default)]
    pub title: Option<String>,
    /// Participant definitions keyed by participant ID.
    pub participants: HashMap<String, ParticipantDef>,
    /// Message sequence.
    pub messages: Vec<MessageDef>,
    /// Loop blocks.
    #[serde(default)]
    pub loops: Vec<LoopDef>,
    /// Alt/opt blocks.
    #[serde(default)]
    pub alts: Vec<AltDef>,
    /// Notes.
    #[serde(default)]
    pub notes: Vec<NoteDef>,
    /// Diagram description.
    #[serde(default)]
    pub description: Option<String>,
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_sequence() {
        let json = json!({
            "id": "api-call",
            "participants": {
                "client": { "label": "Client", "type": "actor" },
                "server": { "label": "API Server" }
            },
            "messages": [
                { "from": "client", "to": "server", "text": "GET /users" },
                { "from": "server", "to": "client", "text": "200 OK", "type": "dotted" }
            ]
        });

        let seq: SequenceDef = serde_json::from_value(json).unwrap();
        assert_eq!(seq.id, "api-call");
        assert_eq!(seq.participants.len(), 2);
        assert_eq!(seq.messages.len(), 2);
    }

    #[test]
    fn test_parse_with_activation() {
        let json = json!({
            "id": "auth-flow",
            "participants": {
                "client": { "label": "Client" },
                "auth": { "label": "Auth Service" },
                "db": { "label": "Database" }
            },
            "messages": [
                { "from": "client", "to": "auth", "text": "Login", "activate": true },
                { "from": "auth", "to": "db", "text": "Query user", "activate": true },
                { "from": "db", "to": "auth", "text": "User data", "type": "dotted", "deactivate": true },
                { "from": "auth", "to": "client", "text": "Token", "type": "dotted", "deactivate": true }
            ]
        });

        let seq: SequenceDef = serde_json::from_value(json).unwrap();
        assert!(seq.messages[0].activate);
        assert!(seq.messages[2].deactivate);
    }

    #[test]
    fn test_parse_with_loop() {
        let json = json!({
            "id": "retry-flow",
            "participants": {
                "client": { "label": "Client" },
                "server": { "label": "Server" }
            },
            "messages": [
                { "from": "client", "to": "server", "text": "Request" },
                { "from": "server", "to": "client", "text": "Response", "type": "dotted" }
            ],
            "loops": [
                { "label": "Retry 3 times", "start": 0, "end": 1 }
            ]
        });

        let seq: SequenceDef = serde_json::from_value(json).unwrap();
        assert_eq!(seq.loops.len(), 1);
        assert_eq!(seq.loops[0].label, "Retry 3 times");
    }

    #[test]
    fn test_parse_with_alt() {
        let json = json!({
            "id": "auth-result",
            "participants": {
                "client": { "label": "Client" },
                "auth": { "label": "Auth" }
            },
            "messages": [
                { "from": "client", "to": "auth", "text": "Login" },
                { "from": "auth", "to": "client", "text": "Success", "type": "dotted" },
                { "from": "auth", "to": "client", "text": "Failure", "type": "dotted" }
            ],
            "alts": [
                {
                    "type": "alt",
                    "condition": "Valid credentials",
                    "start": 1,
                    "end": 1,
                    "else_branches": [
                        { "condition": "Invalid credentials", "start": 2, "end": 2 }
                    ]
                }
            ]
        });

        let seq: SequenceDef = serde_json::from_value(json).unwrap();
        assert_eq!(seq.alts.len(), 1);
        assert_eq!(seq.alts[0].else_branches.len(), 1);
    }
}
