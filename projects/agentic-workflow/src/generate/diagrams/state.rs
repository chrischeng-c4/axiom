// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#source
// CODEGEN-BEGIN
//! State Diagram Generation
//!
//! Generates Mermaid state diagrams for state machines and workflow modeling.

use crate::generate::{GenerateError, Result};

use serde::{Deserialize, Serialize};

/// State node kind.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
    /// Standard state node.
    #[default]
    Normal,
    /// Start state pseudo-node.
    Start,
    /// End state pseudo-node.
    End,
    /// Branch / choice node.
    Choice,
    /// Parallel fork node.
    Fork,
    /// Parallel join node.
    Join,
}

/// State node definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDef {
    /// State identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// State kind. JSON key is 'type' (Rust reserved word); defaults to Normal.
    #[serde(rename = "type", default)]
    pub state_type: StateType,
    /// Long description shown via 'state "..." as id'.
    #[serde(default)]
    pub description: Option<String>,
}

/// State transition edge.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Source state id.
    pub from: String,
    /// Target state id.
    pub to: String,
    /// Edge label.
    #[serde(default)]
    pub label: Option<String>,
}

/// Composite (nested) state grouping.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeState {
    /// Composite state id.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Child state ids contained in this composite.
    pub substates: Vec<String>,
}

/// Sticky note attached to a state.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNote {
    /// State id this note is anchored to.
    pub state: String,
    /// Note text content.
    pub text: String,
    /// Note position (e.g. 'left' / 'right'). Defaults to 'right' at render time.
    #[serde(default)]
    pub position: Option<String>,
}

/// Input for state-diagram generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateInput {
    /// All state nodes in the diagram.
    pub states: Vec<StateDef>,
    /// Transition edges.
    #[serde(default)]
    pub transitions: Vec<StateTransition>,
    /// Composite (nested) groupings.
    #[serde(default)]
    pub composite_states: Vec<CompositeState>,
    /// Sticky notes.
    #[serde(default)]
    pub notes: Vec<StateNote>,
    /// Diagram direction (TB / LR / etc.). Defaults to 'TB'.
    #[serde(default)]
    pub direction: Option<String>,
}
/// Generate a Mermaid state diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state.md#source
pub fn generate_state_diagram(input: &StateInput) -> Result<String> {
    if input.states.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one state required".to_string(),
        ));
    }

    let mut mermaid = String::new();

    let direction = input.direction.as_deref().unwrap_or("TB");
    mermaid.push_str(&format!("stateDiagram-v2\n    direction {}\n", direction));

    // Find start and end states
    let start_state = input
        .states
        .iter()
        .find(|s| matches!(s.state_type, StateType::Start));
    let end_states: Vec<_> = input
        .states
        .iter()
        .filter(|s| matches!(s.state_type, StateType::End))
        .collect();

    // Generate initial transition
    if let Some(start) = start_state {
        mermaid.push_str(&format!("    [*] --> {}\n", start.id));
    }

    // Generate composite states
    for composite in &input.composite_states {
        mermaid.push_str(&format!(
            "    state \"{}\" as {} {{\n",
            composite.label, composite.id
        ));
        for substate_id in &composite.substates {
            if let Some(state) = input.states.iter().find(|s| &s.id == substate_id) {
                mermaid.push_str(&format!("        {}: {}\n", state.id, state.label));
            }
        }
        mermaid.push_str("    }\n");
    }

    // Generate standalone states
    let composite_states: std::collections::HashSet<_> = input
        .composite_states
        .iter()
        .flat_map(|c| c.substates.iter())
        .collect();

    for state in &input.states {
        if !composite_states.contains(&state.id)
            && !matches!(state.state_type, StateType::Start | StateType::End)
        {
            if let Some(ref desc) = state.description {
                mermaid.push_str(&format!("    state \"{}\" as {}\n", desc, state.id));
            }
            mermaid.push_str(&format!("    {}: {}\n", state.id, state.label));
        }
    }

    // Generate transitions
    for transition in &input.transitions {
        if let Some(ref label) = transition.label {
            mermaid.push_str(&format!(
                "    {} --> {}: {}\n",
                transition.from, transition.to, label
            ));
        } else {
            mermaid.push_str(&format!("    {} --> {}\n", transition.from, transition.to));
        }
    }

    // Generate end transitions
    for end in end_states {
        mermaid.push_str(&format!("    {} --> [*]\n", end.id));
    }

    // Generate notes
    for note in &input.notes {
        let pos = note.position.as_deref().unwrap_or("right");
        mermaid.push_str(&format!(
            "    note {} of {}: {}\n",
            pos, note.state, note.text
        ));
    }

    Ok(mermaid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_state_diagram() {
        let input = StateInput {
            states: vec![
                StateDef {
                    id: "idle".to_string(),
                    label: "Idle".to_string(),
                    state_type: StateType::Start,
                    description: None,
                },
                StateDef {
                    id: "running".to_string(),
                    label: "Running".to_string(),
                    state_type: StateType::Normal,
                    description: None,
                },
                StateDef {
                    id: "done".to_string(),
                    label: "Done".to_string(),
                    state_type: StateType::End,
                    description: None,
                },
            ],
            transitions: vec![
                StateTransition {
                    from: "idle".to_string(),
                    to: "running".to_string(),
                    label: Some("start".to_string()),
                },
                StateTransition {
                    from: "running".to_string(),
                    to: "done".to_string(),
                    label: Some("complete".to_string()),
                },
            ],
            composite_states: vec![],
            notes: vec![],
            direction: None,
        };

        let result = generate_state_diagram(&input).unwrap();
        assert!(result.contains("stateDiagram-v2"));
        assert!(result.contains("[*] --> idle"));
        assert!(result.contains("idle --> running: start"));
        assert!(result.contains("done --> [*]"));
    }
}

// CODEGEN-END
