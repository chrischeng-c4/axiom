---
id: sdd-generate-state
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# State Diagram

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/state.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CompositeState` | projects/agentic-workflow/src/generate/diagrams/state.rs | struct | pub | 63 |  |
| `StateDef` | projects/agentic-workflow/src/generate/diagrams/state.rs | struct | pub | 34 |  |
| `StateInput` | projects/agentic-workflow/src/generate/diagrams/state.rs | struct | pub | 88 |  |
| `StateNote` | projects/agentic-workflow/src/generate/diagrams/state.rs | struct | pub | 75 |  |
| `StateTransition` | projects/agentic-workflow/src/generate/diagrams/state.rs | struct | pub | 50 |  |
| `StateType` | projects/agentic-workflow/src/generate/diagrams/state.rs | enum | pub | 15 |  |
| `generate_state_diagram` | projects/agentic-workflow/src/generate/diagrams/state.rs | function | pub | 106 | generate_state_diagram(input: &StateInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  StateType:
    type: string
    enum: [Normal, Start, End, Choice, Fork, Join]
    description: State node kind.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Normal, is_default: true, doc: "Standard state node." }
        - { name: Start,                 doc: "Start state pseudo-node." }
        - { name: End,                   doc: "End state pseudo-node." }
        - { name: Choice,                doc: "Branch / choice node." }
        - { name: Fork,                  doc: "Parallel fork node." }
        - { name: Join,                  doc: "Parallel join node." }

  StateDef:
    type: object
    required: [id, label, state_type]
    description: State node definition.
    properties:
      id:
        type: string
        description: "State identifier."
      label:
        type: string
        description: "Display label."
      state_type:
        $ref: "#/definitions/StateType"
        description: "State kind. JSON key is 'type' (Rust reserved word); defaults to Normal."
        x-serde-rename: "type"
        x-serde-default: true
      description:
        type: string
        description: "Long description shown via 'state \"...\" as id'."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  StateTransition:
    type: object
    required: [from, to]
    description: State transition edge.
    properties:
      from:
        type: string
        description: "Source state id."
      to:
        type: string
        description: "Target state id."
      label:
        type: string
        description: "Edge label."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  CompositeState:
    type: object
    required: [id, label, substates]
    description: Composite (nested) state grouping.
    properties:
      id:
        type: string
        description: "Composite state id."
      label:
        type: string
        description: "Display label."
      substates:
        type: array
        items: { type: string }
        description: "Child state ids contained in this composite."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  StateNote:
    type: object
    required: [state, text]
    description: Sticky note attached to a state.
    properties:
      state:
        type: string
        description: "State id this note is anchored to."
      text:
        type: string
        description: "Note text content."
      position:
        type: string
        description: "Note position (e.g. 'left' / 'right'). Defaults to 'right' at render time."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  StateInput:
    type: object
    required: [states, transitions, composite_states, notes]
    description: Input for state-diagram generation.
    properties:
      states:
        type: array
        items:
          $ref: "#/definitions/StateDef"
        description: "All state nodes in the diagram."
      transitions:
        type: array
        items:
          $ref: "#/definitions/StateTransition"
        description: "Transition edges."
        x-serde-default: true
      composite_states:
        type: array
        items:
          $ref: "#/definitions/CompositeState"
        description: "Composite (nested) groupings."
        x-serde-default: true
      notes:
        type: array
        items:
          $ref: "#/definitions/StateNote"
        description: "Sticky notes."
        x-serde-default: true
      direction:
        type: string
        description: "Diagram direction (TB / LR / etc.). Defaults to 'TB'."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/state.rs -->
```rust
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/state.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete state diagram module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] All six types verified against source: `StateType` derive list (including `Default`), `serde_rename_all: lowercase`, `is_default: true` on `Normal`, `StateDef.state_type` combined `x-serde-rename`+`x-serde-default`, Vec/optional field defaults across all structs — all accurate.
- [changes] Two-entry split (codegen / hand-written) for `state.rs` matches the sdd-generate-erd pattern; all six replaced symbols enumerated; hand-written boundary (`generate_state_diagram` fn + tests) clearly delimited.
- [overview] Both notable patterns (combined rename+default, is_default variant) called out explicitly with codegen implications — sufficient for implementation.
