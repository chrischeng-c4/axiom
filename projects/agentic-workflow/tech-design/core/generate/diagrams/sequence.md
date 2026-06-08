---
id: sdd-generate-sequence
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Sequence Diagram

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/sequence.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AltBlock` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | struct | pub | 108 |  |
| `LoopBlock` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | struct | pub | 96 |  |
| `Message` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | struct | pub | 55 |  |
| `MessageType` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | enum | pub | 27 |  |
| `Participant` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | struct | pub | 42 |  |
| `ParticipantType` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | enum | pub | 15 |  |
| `SequenceInput` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | struct | pub | 126 |  |
| `SequenceNote` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | struct | pub | 76 |  |
| `generate_sequence` | projects/agentic-workflow/src/generate/diagrams/sequence.rs | function | pub | 143 | generate_sequence(input: &SequenceInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ParticipantType:
    type: string
    enum: [Participant, Actor]
    description: Sequence participant kind.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Participant, is_default: true, doc: "Standard participant box." }
        - { name: Actor,                         doc: "Stick-figure actor." }

  MessageType:
    type: string
    enum: [Solid, Dotted, SolidOpen, DottedOpen]
    description: Sequence-message arrow style.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      serde_rename_all: snake_case
      variants:
        - { name: Solid,      is_default: true, doc: "Filled-arrowhead solid line (default)." }
        - { name: Dotted,                       doc: "Filled-arrowhead dotted line." }
        - { name: SolidOpen,                    doc: "Open-arrowhead solid line." }
        - { name: DottedOpen,                   doc: "Open-arrowhead dotted line." }

  Participant:
    type: object
    required: [id, label, participant_type]
    description: One participant in the sequence diagram.
    properties:
      id:
        type: string
        description: "Participant identifier."
      label:
        type: string
        description: "Display label."
      participant_type:
        $ref: "#/definitions/ParticipantType"
        description: "Participant kind. JSON key 'type'; defaults to Participant."
        x-serde-rename: "type"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Message:
    type: object
    required: [from, to, text, message_type, activate, deactivate]
    description: One arrow between two participants.
    properties:
      from:
        type: string
        description: "Source participant id."
      to:
        type: string
        description: "Target participant id."
      text:
        type: string
        description: "Message text."
      message_type:
        $ref: "#/definitions/MessageType"
        description: "Arrow style. JSON key 'type'; defaults to Solid."
        x-serde-rename: "type"
        x-serde-default: true
      activate:
        type: boolean
        description: "Activate the target lifeline (`+`)."
        x-serde-default: true
      deactivate:
        type: boolean
        description: "Deactivate the target lifeline (`-`)."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SequenceNote:
    type: object
    required: [text, participants, after_message]
    description: Note attached to participant(s).
    properties:
      text:
        type: string
        description: "Note text."
      participant:
        type: string
        description: "Single anchor participant id."
        x-serde-default: true
      participants:
        type: array
        items: { type: string }
        x-rust-type: "Option<Vec<String>>"
        description: "Multiple anchor participant ids (overlay 'over' notes)."
        x-serde-default: true
      position:
        type: string
        description: "Position keyword: 'left of' / 'right of' / 'over'."
        x-serde-default: true
      after_message:
        type: integer
        x-rust-type: "Option<usize>"
        description: "Insert this note after the message at this index."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  LoopBlock:
    type: object
    required: [label, start_message, end_message]
    description: Message-index range bounding a `loop ... end` block.
    properties:
      label:
        type: string
        description: "Loop label."
      start_message:
        type: integer
        x-rust-type: usize
        description: "First-message index inside the loop."
      end_message:
        type: integer
        x-rust-type: usize
        description: "Last-message index inside the loop."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  AltBlock:
    type: object
    required: [condition, start_message, end_message, else_end_message]
    description: Message-index range bounding an `alt ... else ... end` block.
    properties:
      condition:
        type: string
        description: "Branch condition label."
      start_message:
        type: integer
        x-rust-type: usize
        description: "First-message index inside the alt branch."
      end_message:
        type: integer
        x-rust-type: usize
        description: "Last-message index inside the alt branch."
      else_condition:
        type: string
        description: "Optional else-branch condition label."
        x-serde-default: true
      else_end_message:
        type: integer
        x-rust-type: "Option<usize>"
        description: "Optional last-message index inside the else branch."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SequenceInput:
    type: object
    required: [participants, messages, notes, loops, alts]
    description: Input for sequence-diagram generation.
    properties:
      participants:
        type: array
        items:
          $ref: "#/definitions/Participant"
        description: "All participants (need at least 2 at runtime)."
      messages:
        type: array
        items:
          $ref: "#/definitions/Message"
        description: "Ordered messages between participants."
      notes:
        type: array
        items:
          $ref: "#/definitions/SequenceNote"
        description: "Notes anchored to messages."
        x-serde-default: true
      loops:
        type: array
        items:
          $ref: "#/definitions/LoopBlock"
        description: "Loop-block ranges over message indices."
        x-serde-default: true
      alts:
        type: array
        items:
          $ref: "#/definitions/AltBlock"
        description: "Alt-block ranges over message indices."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/sequence.rs -->
```rust
//! Sequence Diagram Generation
//!
//! Generates Mermaid sequence diagrams for API interactions and service flows.

use crate::generate::{GenerateError, Result};

use serde::{Deserialize, Serialize};

/// Sequence participant kind.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantType {
    /// Standard participant box.
    #[default]
    Participant,
    /// Stick-figure actor.
    Actor,
}

/// Sequence-message arrow style.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Filled-arrowhead solid line (default).
    #[default]
    Solid,
    /// Filled-arrowhead dotted line.
    Dotted,
    /// Open-arrowhead solid line.
    SolidOpen,
    /// Open-arrowhead dotted line.
    DottedOpen,
}

/// One participant in the sequence diagram.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Participant identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Participant kind. JSON key 'type'; defaults to Participant.
    #[serde(rename = "type", default)]
    pub participant_type: ParticipantType,
}

/// One arrow between two participants.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Source participant id.
    pub from: String,
    /// Target participant id.
    pub to: String,
    /// Message text.
    pub text: String,
    /// Arrow style. JSON key 'type'; defaults to Solid.
    #[serde(rename = "type", default)]
    pub message_type: MessageType,
    /// Activate the target lifeline (`+`).
    #[serde(default)]
    pub activate: bool,
    /// Deactivate the target lifeline (`-`).
    #[serde(default)]
    pub deactivate: bool,
}

/// Note attached to participant(s).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceNote {
    /// Note text.
    pub text: String,
    /// Single anchor participant id.
    #[serde(default)]
    pub participant: Option<String>,
    /// Multiple anchor participant ids (overlay 'over' notes).
    #[serde(default)]
    pub participants: Option<Vec<String>>,
    /// Position keyword: 'left of' / 'right of' / 'over'.
    #[serde(default)]
    pub position: Option<String>,
    /// Insert this note after the message at this index.
    #[serde(default)]
    pub after_message: Option<usize>,
}

/// Message-index range bounding a `loop ... end` block.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopBlock {
    /// Loop label.
    pub label: String,
    /// First-message index inside the loop.
    pub start_message: usize,
    /// Last-message index inside the loop.
    pub end_message: usize,
}

/// Message-index range bounding an `alt ... else ... end` block.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AltBlock {
    /// Branch condition label.
    pub condition: String,
    /// First-message index inside the alt branch.
    pub start_message: usize,
    /// Last-message index inside the alt branch.
    pub end_message: usize,
    /// Optional else-branch condition label.
    #[serde(default)]
    pub else_condition: Option<String>,
    /// Optional last-message index inside the else branch.
    #[serde(default)]
    pub else_end_message: Option<usize>,
}

/// Input for sequence-diagram generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceInput {
    /// All participants (need at least 2 at runtime).
    pub participants: Vec<Participant>,
    /// Ordered messages between participants.
    pub messages: Vec<Message>,
    /// Notes anchored to messages.
    #[serde(default)]
    pub notes: Vec<SequenceNote>,
    /// Loop-block ranges over message indices.
    #[serde(default)]
    pub loops: Vec<LoopBlock>,
    /// Alt-block ranges over message indices.
    #[serde(default)]
    pub alts: Vec<AltBlock>,
}
/// Generate a Mermaid sequence diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#source
pub fn generate_sequence(input: &SequenceInput) -> Result<String> {
    if input.participants.len() < 2 {
        return Err(GenerateError::InvalidValue(
            "At least 2 participants required".to_string(),
        ));
    }

    let mut mermaid = String::new();
    mermaid.push_str("sequenceDiagram\n");

    // Generate participants
    for p in &input.participants {
        let type_str = match p.participant_type {
            ParticipantType::Actor => "actor",
            ParticipantType::Participant => "participant",
        };
        mermaid.push_str(&format!("    {} {} as {}\n", type_str, p.id, p.label));
    }

    // Generate messages with loops/alts
    for (i, msg) in input.messages.iter().enumerate() {
        // Check for loop start
        for loop_block in &input.loops {
            if loop_block.start_message == i {
                mermaid.push_str(&format!("    loop {}\n", loop_block.label));
            }
        }

        // Check for alt start
        for alt in &input.alts {
            if alt.start_message == i {
                mermaid.push_str(&format!("    alt {}\n", alt.condition));
            }
        }

        // Generate message
        let arrow = match msg.message_type {
            MessageType::Solid => "->>",
            MessageType::Dotted => "-->>",
            MessageType::SolidOpen => "->",
            MessageType::DottedOpen => "-->",
        };

        if msg.activate {
            mermaid.push_str(&format!(
                "    {}{}+{}: {}\n",
                msg.from, arrow, msg.to, msg.text
            ));
        } else if msg.deactivate {
            mermaid.push_str(&format!(
                "    {}{}-{}: {}\n",
                msg.from, arrow, msg.to, msg.text
            ));
        } else {
            mermaid.push_str(&format!(
                "    {}{}{}: {}\n",
                msg.from, arrow, msg.to, msg.text
            ));
        }

        // Check for notes after this message
        for note in &input.notes {
            if note.after_message == Some(i) {
                let pos = note.position.as_deref().unwrap_or("right of");
                if let Some(ref p) = note.participant {
                    mermaid.push_str(&format!("    Note {} {}: {}\n", pos, p, note.text));
                }
            }
        }

        // Check for alt else
        for alt in &input.alts {
            if alt.end_message == i && alt.else_condition.is_some() {
                mermaid.push_str(&format!(
                    "    else {}\n",
                    alt.else_condition.as_ref().unwrap()
                ));
            }
        }

        // Check for loop end
        for loop_block in &input.loops {
            if loop_block.end_message == i {
                mermaid.push_str("    end\n");
            }
        }

        // Check for alt end
        for alt in &input.alts {
            let end = alt.else_end_message.unwrap_or(alt.end_message);
            if end == i {
                mermaid.push_str("    end\n");
            }
        }
    }

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sequence() {
        let input = SequenceInput {
            participants: vec![
                Participant {
                    id: "A".to_string(),
                    label: "Alice".to_string(),
                    participant_type: ParticipantType::Actor,
                },
                Participant {
                    id: "B".to_string(),
                    label: "Bob".to_string(),
                    participant_type: ParticipantType::Participant,
                },
            ],
            messages: vec![
                Message {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    text: "Hello".to_string(),
                    message_type: MessageType::Solid,
                    activate: false,
                    deactivate: false,
                },
                Message {
                    from: "B".to_string(),
                    to: "A".to_string(),
                    text: "Hi".to_string(),
                    message_type: MessageType::Dotted,
                    activate: false,
                    deactivate: false,
                },
            ],
            notes: vec![],
            loops: vec![],
            alts: vec![],
        };

        let result = generate_sequence(&input).unwrap();
        assert!(result.contains("sequenceDiagram"));
        assert!(result.contains("actor A as Alice"));
        assert!(result.contains("A->>B: Hello"));
        assert!(result.contains("B-->>A: Hi"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/sequence.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Mermaid sequence diagram module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Minor prose inconsistency: opening sentence says "Seven serde shapes" but the bullet list and the changes description both correctly enumerate 8 types (ParticipantType, MessageType, Participant, Message, SequenceNote, LoopBlock, AltBlock, SequenceInput). No implementation ambiguity since all 8 are fully defined in schema and changes. Consider updating the count to "Eight" in a follow-up pass.
