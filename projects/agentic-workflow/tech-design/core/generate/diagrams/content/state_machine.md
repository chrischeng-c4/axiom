---
id: sdd-content-state-machine
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# StateMachineContent

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `StateKind` | projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs | enum | pub | 18 |  |
| `StateMachineContent` | projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs | struct | pub | 61 |  |
| `StateNode` | projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs | struct | pub | 35 |  |
| `Transition` | projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs | struct | pub | 47 |  |
| `terminal_ids` | projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs | function | pub | 98 | terminal_ids(&self) -> Vec<&str> |
| `transient_ids` | projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs | function | pub | 107 | transient_ids(&self) -> Vec<&str> |
| `transitions_from` | projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs | function | pub | 116 | transitions_from(&self, state_id: &str) -> Vec<&Transition> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  StateKind:
    type: string
    enum: [Initial, Normal, Terminal, Transient, Choice]
    description: Kind of a state in a state machine diagram.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Initial,                       doc: "Initial pseudo-state." }
        - { name: Normal,    is_default: true,   doc: "Normal state (default)." }
        - { name: Terminal,                      doc: "Terminal end state." }
        - { name: Transient,                     doc: "Transient state." }
        - { name: Choice,                        doc: "Choice pseudo-state." }

  StateNode:
    type: object
    required: [kind]
    description: A single state node in a state machine.
    properties:
      kind:
        $ref: "#/definitions/StateKind"
        x-serde-default: true
      label:
        type: string
      description:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Transition:
    type: object
    required: [from, to]
    description: A transition (edge) between states.
    properties:
      from:
        type: string
      to:
        type: string
      event:
        type: string
      guard:
        type: string
      label:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  StateMachineContent:
    type: object
    required: [id, initial, nodes, edges, classifications, emit_next_fn]
    description: Content type for `state-machine` section (stateDiagram-v2).
    properties:
      id:
        type: string
      initial:
        type: string
      nodes:
        type: object
        x-rust-type: "HashMap<String, StateNode>"
        x-serde-default: true
      edges:
        type: array
        items:
          $ref: "#/definitions/Transition"
        x-serde-default: true
      title:
        type: string
      type_name:
        type: string
        description: "Override for the generated Rust enum name."
        x-serde-default: true
      classifications:
        type: object
        x-rust-type: "std::collections::BTreeMap<String, Vec<String>>"
        description: "Named classifications mapped to is_<name>() methods."
        x-serde-default: true
      emit_next_fn:
        type: boolean
        description: "Emit fn next(&self, event) -> Option<Self> skeleton."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs -->
````rust
//! StateMachineContent — per-diagram Content type for state-machine (stateDiagram-v2).
//!
//! Replaces the XState-based schema in `state_plus/schema.rs` (design decision D8).
//! Content is parsed from Mermaid Plus YAML frontmatter in spec files.

// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#source

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Kind of a state in a state machine diagram.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StateKind {
    /// Initial pseudo-state.
    Initial,
    /// Normal state (default).
    #[default]
    Normal,
    /// Terminal end state.
    Terminal,
    /// Transient state.
    Transient,
    /// Choice pseudo-state.
    Choice,
}

/// A single state node in a state machine.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNode {
    #[serde(default)]
    pub kind: StateKind,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// A transition (edge) between states.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub guard: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

/// Content type for `state-machine` section (stateDiagram-v2).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineContent {
    pub id: String,
    pub initial: String,
    #[serde(default)]
    pub nodes: HashMap<String, StateNode>,
    #[serde(default)]
    pub edges: Vec<Transition>,
    #[serde(default)]
    pub title: Option<String>,
    /// Override for the generated Rust enum name.
    #[serde(default)]
    pub type_name: Option<String>,
    /// Named classifications mapped to is_<name>() methods.
    #[serde(default)]
    pub classifications: std::collections::BTreeMap<String, Vec<String>>,
    /// Emit fn next(&self, event) -> Option<Self> skeleton.
    #[serde(default)]
    pub emit_next_fn: bool,
}

/// Content type for `state-machine` section (stateDiagram-v2).
///
/// Parsed from Mermaid Plus YAML frontmatter:
/// ```yaml
/// id: my-sm
/// initial: idle
/// nodes:
///   idle: { kind: normal }
///   done: { kind: terminal }
/// edges:
///   - from: idle
///     to: done
///     event: complete
/// ```
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#source
impl StateMachineContent {
    /// Return all terminal state IDs.
    pub fn terminal_ids(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.kind == StateKind::Terminal)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Return all transient/choice state IDs.
    pub fn transient_ids(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|(_, n)| matches!(n.kind, StateKind::Transient | StateKind::Choice))
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Return outgoing transitions from a given state.
    pub fn transitions_from(&self, state_id: &str) -> Vec<&Transition> {
        self.edges.iter().filter(|t| t.from == state_id).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sm() -> StateMachineContent {
        let mut nodes = HashMap::new();
        nodes.insert(
            "idle".to_string(),
            StateNode {
                kind: StateKind::Normal,
                label: None,
                description: None,
            },
        );
        nodes.insert(
            "done".to_string(),
            StateNode {
                kind: StateKind::Terminal,
                label: None,
                description: None,
            },
        );
        nodes.insert(
            "processing".to_string(),
            StateNode {
                kind: StateKind::Transient,
                label: None,
                description: None,
            },
        );

        StateMachineContent {
            id: "my-sm".to_string(),
            initial: "idle".to_string(),
            nodes,
            edges: vec![
                Transition {
                    from: "idle".to_string(),
                    to: "processing".to_string(),
                    event: Some("start".to_string()),
                    guard: None,
                    label: None,
                },
                Transition {
                    from: "processing".to_string(),
                    to: "done".to_string(),
                    event: None,
                    guard: None,
                    label: None,
                },
            ],
            title: None,
            type_name: None,
            classifications: Default::default(),
            emit_next_fn: false,
        }
    }

    // @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#source
    #[test]
    fn test_terminal_ids() {
        let sm = make_sm();
        let terminals = sm.terminal_ids();
        assert!(terminals.contains(&"done"), "done should be terminal");
        assert!(!terminals.contains(&"idle"), "idle should not be terminal");
    }

    // @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#source
    #[test]
    fn test_transient_ids() {
        let sm = make_sm();
        let transients = sm.transient_ids();
        assert!(
            transients.contains(&"processing"),
            "processing should be transient"
        );
    }

    #[test]
    fn test_transitions_from() {
        let sm = make_sm();
        let from_idle = sm.transitions_from("idle");
        assert_eq!(from_idle.len(), 1, "idle should have 1 outgoing transition");
        assert_eq!(from_idle[0].to, "processing");
    }

    // @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/state_machine.md#source
    #[test]
    fn test_deserialize_from_yaml() {
        let yaml = r#"
id: my-sm
initial: idle
nodes:
  idle:
    kind: normal
  done:
    kind: terminal
edges:
  - from: idle
    to: done
    event: complete
"#;
        let sm: StateMachineContent = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(sm.id, "my-sm");
        assert_eq!(sm.initial, "idle");
        assert_eq!(sm.nodes.len(), 2);
        assert_eq!(sm.edges.len(), 1);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/content/state_machine.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete state-machine content module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 4-type scope; sibling pattern.
- [schema] HashMap + BTreeMap via x-rust-type; bare bool in required.
- [changes] codegen + hand-written split correct.
