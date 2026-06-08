// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/sequence.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
