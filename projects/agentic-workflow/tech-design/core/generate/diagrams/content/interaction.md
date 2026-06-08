---
id: sdd-content-interaction
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# InteractionContent

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/content/interaction.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Actor` | projects/agentic-workflow/src/generate/diagrams/content/interaction.rs | struct | pub | 29 |  |
| `ActorKind` | projects/agentic-workflow/src/generate/diagrams/content/interaction.rs | enum | pub | 16 |  |
| `InteractionContent` | projects/agentic-workflow/src/generate/diagrams/content/interaction.rs | struct | pub | 55 |  |
| `Message` | projects/agentic-workflow/src/generate/diagrams/content/interaction.rs | struct | pub | 40 |  |
| `actor_ids` | projects/agentic-workflow/src/generate/diagrams/content/interaction.rs | function | pub | 100 | actor_ids(&self) -> Vec<&str> |
| `messages_from` | projects/agentic-workflow/src/generate/diagrams/content/interaction.rs | function | pub | 87 | messages_from(&self, actor_id: &str) -> Vec<&Message> |
| `messages_to` | projects/agentic-workflow/src/generate/diagrams/content/interaction.rs | function | pub | 95 | messages_to(&self, actor_id: &str) -> Vec<&Message> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ActorKind:
    type: string
    enum: [Actor, Participant, System]
    description: Kind of actor in a sequence diagram.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Actor,                            doc: "Actor (human / external)." }
        - { name: Participant, is_default: true,    doc: "Participant (default)." }
        - { name: System,                           doc: "System component." }

  Actor:
    type: object
    required: [id, kind]
    description: An actor (participant) in a sequence diagram.
    properties:
      id:
        type: string
      kind:
        $ref: "#/definitions/ActorKind"
        x-serde-default: true
      label:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Message:
    type: object
    required: [from, to, name, async]
    description: A message exchanged between actors. `async` is a Rust reserved word — codegen output requires a manual r# prefix.
    properties:
      from:
        type: string
      to:
        type: string
      name:
        type: string
      async:
        type: boolean
        x-serde-default: true
      returns:
        type: string
      label:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  InteractionContent:
    type: object
    required: [id, actors, messages]
    description: Content type for `interaction` section (sequenceDiagram).
    properties:
      id:
        type: string
      actors:
        type: array
        items:
          $ref: "#/definitions/Actor"
        x-serde-default: true
      messages:
        type: array
        items:
          $ref: "#/definitions/Message"
        x-serde-default: true
      title:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/content/interaction.rs -->
````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/content/interaction.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete interaction content module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 4-type scope. ActorKind is_default migration explicit. r#async caveat called out.
- [schema] Patterns proven; Message field `async` has expected r# prefix needed post-gen.
- [changes] codegen + hand-written split correct.
