// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#source
// CODEGEN-BEGIN
//! InteractionContent — per-diagram Content type for interaction (sequenceDiagram).
//!
//! Replaces the XState-based schema in `sequence_plus/schema.rs` (design decision D8).
//! Content is parsed from Mermaid Plus YAML frontmatter in spec files.

// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#source

use serde::{Deserialize, Serialize};

/// Kind of actor in a sequence diagram.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActorKind {
    /// Actor (human / external).
    Actor,
    /// Participant (default).
    #[default]
    Participant,
    /// System component.
    System,
}

/// An actor (participant) in a sequence diagram.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    #[serde(default)]
    pub kind: ActorKind,
    #[serde(default)]
    pub label: Option<String>,
}

/// A message exchanged between actors. `async` is a Rust reserved word — codegen output requires a manual r# prefix.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub name: String,
    #[serde(rename = "async", default)]
    pub r#async: bool,
    #[serde(default)]
    pub returns: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

/// Content type for `interaction` section (sequenceDiagram).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContent {
    pub id: String,
    #[serde(default)]
    pub actors: Vec<Actor>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub title: Option<String>,
}

/// Content type for `interaction` section (sequenceDiagram).
///
/// Parsed from Mermaid Plus YAML frontmatter:
/// ```yaml
/// id: my-interaction
/// actors:
///   - id: Client
///     kind: actor
///   - id: Server
///     kind: system
/// messages:
///   - from: Client
///     to: Server
///     name: create_issue
///   - from: Server
///     to: Client
///     name: issue_created
///     returns: Issue
/// ```
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#source
impl InteractionContent {
    /// Return all messages sent from a given actor.
    pub fn messages_from(&self, actor_id: &str) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|m| m.from == actor_id)
            .collect()
    }

    /// Return all messages received by a given actor.
    pub fn messages_to(&self, actor_id: &str) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.to == actor_id).collect()
    }

    /// Return unique actor IDs involved in messages.
    pub fn actor_ids(&self) -> Vec<&str> {
        self.actors.iter().map(|a| a.as_str()).collect()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#source
impl Actor {
    fn as_str(&self) -> &str {
        &self.id
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#source
    #[test]
    fn test_deserialize_from_yaml() {
        let yaml = r#"
id: my-interaction
actors:
  - id: Client
    kind: actor
  - id: Server
    kind: system
messages:
  - from: Client
    to: Server
    name: create_issue
  - from: Server
    to: Client
    name: issue_created
    returns: Issue
"#;
        let interaction: InteractionContent = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(interaction.id, "my-interaction");
        assert_eq!(interaction.actors.len(), 2);
        assert_eq!(interaction.messages.len(), 2);
    }

    // @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/interaction.md#source
    #[test]
    fn test_messages_from() {
        let yaml = r#"
id: my-interaction
actors:
  - id: Client
messages:
  - from: Client
    to: Server
    name: create_issue
  - from: Server
    to: Client
    name: issue_created
"#;
        let interaction: InteractionContent = serde_yaml::from_str(yaml).unwrap();
        let from_client = interaction.messages_from("Client");
        assert_eq!(from_client.len(), 1);
        assert_eq!(from_client[0].name, "create_issue");
    }
}

// CODEGEN-END
