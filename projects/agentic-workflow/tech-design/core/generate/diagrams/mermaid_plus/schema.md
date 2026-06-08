---
id: sdd-generate-mermaid-plus-schema
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# State Plus Schema

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ActionDef` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 130 |  |
| `ActionRef` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | enum | pub | 45 |  |
| `BlockMigrationResult` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 246 |  |
| `BlockMigrationStatus` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | enum | pub | 263 |  |
| `GuardDef` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 119 |  |
| `MigrationAudit` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 273 |  |
| `MigrationMode` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | enum | pub | 284 |  |
| `MigrationReport` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 292 |  |
| `StateMachineDef` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 53 |  |
| `StateNodeDef` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 74 |  |
| `StateType` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | enum | pub | 16 |  |
| `TransitionDetail` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | struct | pub | 101 |  |
| `TransitionInput` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | enum | pub | 32 |  |
| `to_vec` | projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs | function | pub | 139 | to_vec(&self) -> Vec<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  StateType:
    type: string
    enum: [Atomic, Compound, Parallel, Final]
    description: State node types.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, PartialEq, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Atomic, is_default: true, doc: "Atomic state (default)." }
        - { name: Compound, doc: "Compound state with substates." }
        - { name: Parallel, doc: "Parallel state with concurrent regions." }
        - { name: Final, doc: "Final state." }

  TransitionInput:
    type: string
    enum: [Simple, Detailed, Conditional]
    description: Transition definition (flexible format).
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_untagged: true
      variants:
        - name: Simple
          kind: tuple
          doc: "Simple: just target state name."
          fields:
            - { rust_type: String }
        - name: Detailed
          kind: tuple
          doc: "Detailed: target + guard + actions."
          fields:
            - { rust_type: TransitionDetail }
        - name: Conditional
          kind: tuple
          doc: "Conditional: multiple transitions with guards."
          fields:
            - { rust_type: "Vec<TransitionDetail>" }

  ActionRef:
    type: string
    enum: [Single, Multiple]
    description: Action reference - single or multiple.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_untagged: true
      variants:
        - name: Single
          kind: tuple
          fields:
            - { rust_type: String }
        - name: Multiple
          kind: tuple
          fields:
            - { rust_type: "Vec<String>" }

  StateMachineDef:
    type: object
    required: [id, initial, states, guards, actions, description]
    description: State machine definition (input from LLM).
    properties:
      id:
        type: string
        description: "Machine identifier (required, alphanumeric + hyphen/underscore)."
      initial:
        type: string
        description: "Initial state ID (must exist in states)."
      states:
        type: object
        x-rust-type: "HashMap<String, StateNodeDef>"
        description: "State definitions keyed by state ID."
      guards:
        type: object
        x-rust-type: "HashMap<String, GuardDef>"
        x-serde-default: true
        description: "Guard condition definitions (optional)."
      actions:
        type: object
        x-rust-type: "HashMap<String, ActionDef>"
        x-serde-default: true
        description: "Action definitions (optional)."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Machine description (optional)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  StateNodeDef:
    type: object
    required: [node_type, initial, states, on, entry, exit, description]
    description: State node definition.
    properties:
      node_type:
        type: string
        x-rust-type: "Option<StateType>"
        x-serde-rename: "type"
        x-serde-default: true
        description: "State type: atomic, compound, parallel, final."
      initial:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Initial substate (for compound states)."
      states:
        type: object
        x-rust-type: "Option<HashMap<String, StateNodeDef>>"
        x-serde-default: true
        description: "Child states (for compound/parallel states)."
      on:
        type: object
        x-rust-type: "Option<HashMap<String, TransitionInput>>"
        x-serde-default: true
        description: "Event handlers: event -> transition."
      entry:
        type: object
        x-rust-type: "Option<ActionRef>"
        x-serde-default: true
        description: "Entry actions."
      exit:
        type: object
        x-rust-type: "Option<ActionRef>"
        x-serde-default: true
        description: "Exit actions."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Human-readable description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  TransitionDetail:
    type: object
    required: [target, guard, actions, description]
    description: Detailed transition definition.
    properties:
      target:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Target state ID."
      guard:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Guard condition name (must be defined in guards)."
      actions:
        type: object
        x-rust-type: "Option<ActionRef>"
        x-serde-default: true
        description: "Actions to execute (must be defined in actions)."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Transition description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  GuardDef:
    type: object
    required: [condition, description]
    description: Guard condition definition.
    properties:
      condition:
        type: string
        description: "Guard description or expression."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Human-readable description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ActionDef:
    type: object
    required: [effect, description]
    description: Action definition.
    properties:
      effect:
        type: string
        description: "Action description or implementation hint."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Human-readable description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs -->
```rust
//! State machine definition schema
//!
//! XState-compatible JSON schema for LLM to generate state machine definitions.
//! Designed for easy validation and Mermaid conversion.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// State node types.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
    /// Atomic state (default).
    #[default]
    Atomic,
    /// Compound state with substates.
    Compound,
    /// Parallel state with concurrent regions.
    Parallel,
    /// Final state.
    Final,
}

/// Transition definition (flexible format).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransitionInput {
    /// Simple: just target state name.
    Simple(String),
    /// Detailed: target + guard + actions.
    Detailed(TransitionDetail),
    /// Conditional: multiple transitions with guards.
    Conditional(Vec<TransitionDetail>),
}

/// Action reference - single or multiple.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionRef {
    Single(String),
    Multiple(Vec<String>),
}

/// State machine definition (input from LLM).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineDef {
    /// Machine identifier (required, alphanumeric + hyphen/underscore).
    pub id: String,
    /// Initial state ID (must exist in states).
    pub initial: String,
    /// State definitions keyed by state ID.
    pub states: HashMap<String, StateNodeDef>,
    /// Guard condition definitions (optional).
    #[serde(default)]
    pub guards: HashMap<String, GuardDef>,
    /// Action definitions (optional).
    #[serde(default)]
    pub actions: HashMap<String, ActionDef>,
    /// Machine description (optional).
    #[serde(default)]
    pub description: Option<String>,
}

/// State node definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateNodeDef {
    /// State type: atomic, compound, parallel, final.
    #[serde(rename = "type", default)]
    pub node_type: Option<StateType>,
    /// Initial substate (for compound states).
    #[serde(default)]
    pub initial: Option<String>,
    /// Child states (for compound/parallel states).
    #[serde(default)]
    pub states: Option<HashMap<String, StateNodeDef>>,
    /// Event handlers: event -> transition.
    #[serde(default)]
    pub on: Option<HashMap<String, TransitionInput>>,
    /// Entry actions.
    #[serde(default)]
    pub entry: Option<ActionRef>,
    /// Exit actions.
    #[serde(default)]
    pub exit: Option<ActionRef>,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Detailed transition definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDetail {
    /// Target state ID.
    #[serde(default)]
    pub target: Option<String>,
    /// Guard condition name (must be defined in guards).
    #[serde(default)]
    pub guard: Option<String>,
    /// Actions to execute (must be defined in actions).
    #[serde(default)]
    pub actions: Option<ActionRef>,
    /// Transition description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Guard condition definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardDef {
    /// Guard description or expression.
    pub condition: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Action definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    /// Action description or implementation hint.
    pub effect: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#source
impl ActionRef {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            ActionRef::Single(s) => vec![s.clone()],
            ActionRef::Multiple(v) => v.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_machine() {
        let json = json!({
            "id": "toggle",
            "initial": "off",
            "states": {
                "off": {
                    "on": { "TOGGLE": "on" }
                },
                "on": {
                    "on": { "TOGGLE": "off" }
                }
            }
        });

        let machine: StateMachineDef = serde_json::from_value(json).unwrap();
        assert_eq!(machine.id, "toggle");
        assert_eq!(machine.initial, "off");
        assert_eq!(machine.states.len(), 2);
    }

    #[test]
    fn test_parse_with_guards_and_actions() {
        let json = json!({
            "id": "fetch",
            "initial": "idle",
            "states": {
                "idle": {
                    "on": {
                        "FETCH": {
                            "target": "loading",
                            "guard": "canFetch",
                            "actions": "startFetch"
                        }
                    }
                },
                "loading": {
                    "on": {
                        "SUCCESS": "success",
                        "FAILURE": "failure"
                    }
                },
                "success": { "type": "final" },
                "failure": {}
            },
            "guards": {
                "canFetch": { "condition": "retries < 3" }
            },
            "actions": {
                "startFetch": { "effect": "initiate API call" }
            }
        });

        let machine: StateMachineDef = serde_json::from_value(json).unwrap();
        assert_eq!(machine.guards.len(), 1);
        assert_eq!(machine.actions.len(), 1);
    }

    #[test]
    fn test_parse_nested_states() {
        let json = json!({
            "id": "workflow",
            "initial": "draft",
            "states": {
                "draft": {
                    "on": { "SUBMIT": "review" }
                },
                "review": {
                    "type": "compound",
                    "initial": "pending",
                    "states": {
                        "pending": {
                            "on": {
                                "APPROVE": "approved",
                                "REJECT": "rejected"
                            }
                        },
                        "approved": { "type": "final" },
                        "rejected": { "type": "final" }
                    }
                }
            }
        });

        let machine: StateMachineDef = serde_json::from_value(json).unwrap();
        let review = machine.states.get("review").unwrap();
        assert!(review.states.is_some());
        assert_eq!(review.states.as_ref().unwrap().len(), 3);
    }
}
/// Whether the migration verb may write files (Issue A present) or only report (Issue A absent).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#schema
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationMode {
    WriteMode,
    DryRunMode,
}

/// Outcome of migrating a single mermaid block.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#schema
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockMigrationStatus {
    Converted,
    AlreadyMigrated,
    SchemaExtensionNeeded,
    FlaggedForReview,
}

/// Audit record written into converted block YAML frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#schema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationAudit {
    /// ISO-8601 timestamp of the apply call.
    pub migrated_at: String,
    /// Version string of aw td migrate-mermaid at time of conversion.
    pub tool_version: String,
}

/// Result for a single block within a file migration run.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#schema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockMigrationResult {
    /// Relative path to the TD spec file containing this block.
    pub file_path: String,
    /// Zero-based index of the block within the file.
    pub block_index: i64,
    /// Outcome of the migration attempt.
    pub status: BlockMigrationStatus,
    /// Present when status is Converted.
    pub audit: Option<MigrationAudit>,
    /// Human-readable reason when status is FlaggedForReview or SchemaExtensionNeeded.
    pub flag_reason: Option<String>,
}

/// Aggregate report returned by a migration run.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#schema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationReport {
    /// Mode in which the run executed.
    pub mode: MigrationMode,
    /// Total legacy blocks discovered.
    pub total_blocks: i64,
    /// Blocks successfully converted (write mode only).
    pub converted: Option<u32>,
    /// Blocks skipped as schema-extension-needed.
    pub skipped: Option<u32>,
    /// Blocks flagged for human review.
    pub flagged: Option<u32>,
    /// Per-block outcome list.
    pub results: Vec<BlockMigrationResult>,
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/mermaid_plus/schema.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Mermaid+ schema module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Eight serde shapes including 2 untagged enums with tuple variants.
- [schema] All well-formed; serde_untagged + tuple variants for TransitionInput and ActionRef.
- [changes] All eight in `replaces`; impl ActionRef + tests + module-level items hand-written.
