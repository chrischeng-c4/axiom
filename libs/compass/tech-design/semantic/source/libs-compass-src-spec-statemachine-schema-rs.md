---
id: libs-compass-src-spec-statemachine-schema-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/spec/statemachine/schema.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/spec/statemachine/schema.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/spec/statemachine/schema.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `StateMachineDef` | libs/compass/src/spec/statemachine/schema.rs | struct | pub | 11 | pub struct StateMachineDef { |
| `StateNodeDef` | libs/compass/src/spec/statemachine/schema.rs | struct | pub | 36 | pub struct StateNodeDef { |
| `StateType` | libs/compass/src/spec/statemachine/schema.rs | enum | pub | 69 | pub enum StateType { |
| `TransitionInput` | libs/compass/src/spec/statemachine/schema.rs | enum | pub | 80 | pub enum TransitionInput { |
| `TransitionDetail` | libs/compass/src/spec/statemachine/schema.rs | struct | pub | 93 | pub struct TransitionDetail { |
| `ActionRef` | libs/compass/src/spec/statemachine/schema.rs | enum | pub | 114 | pub enum ActionRef { |
| `to_vec` | libs/compass/src/spec/statemachine/schema.rs | function | pub | 120 | pub fn to_vec(&self) -> Vec<String> { |
| `GuardDef` | libs/compass/src/spec/statemachine/schema.rs | struct | pub | 130 | pub struct GuardDef { |
| `ActionDef` | libs/compass/src/spec/statemachine/schema.rs | struct | pub | 141 | pub struct ActionDef { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! State machine definition schema
//!
//! JSON schema for LLM to generate state machine definitions.
//! Designed for easy validation and Mermaid conversion.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State machine definition (input from LLM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineDef {
    /// Machine identifier (required, alphanumeric + hyphen/underscore)
    pub id: String,

    /// Initial state ID (must exist in states)
    pub initial: String,

    /// State definitions keyed by state ID
    pub states: HashMap<String, StateNodeDef>,

    /// Guard condition definitions (optional)
    #[serde(default)]
    pub guards: HashMap<String, GuardDef>,

    /// Action definitions (optional)
    #[serde(default)]
    pub actions: HashMap<String, ActionDef>,

    /// Machine description (optional)
    #[serde(default)]
    pub description: Option<String>,
}

/// State node definition
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateNodeDef {
    /// State type: atomic, compound, parallel, final
    #[serde(rename = "type", default)]
    pub node_type: Option<StateType>,

    /// Initial substate (for compound states)
    #[serde(default)]
    pub initial: Option<String>,

    /// Child states (for compound/parallel states)
    #[serde(default)]
    pub states: Option<HashMap<String, StateNodeDef>>,

    /// Event handlers: event -> transition
    #[serde(default)]
    pub on: Option<HashMap<String, TransitionInput>>,

    /// Entry actions
    #[serde(default)]
    pub entry: Option<ActionRef>,

    /// Exit actions
    #[serde(default)]
    pub exit: Option<ActionRef>,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
}

/// State node types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
    #[default]
    Atomic,
    Compound,
    Parallel,
    Final,
}

/// Transition definition (flexible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransitionInput {
    /// Simple: just target state name
    Simple(String),

    /// Detailed: target + guard + actions
    Detailed(TransitionDetail),

    /// Conditional: multiple transitions with guards
    Conditional(Vec<TransitionDetail>),
}

/// Detailed transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDetail {
    /// Target state ID
    #[serde(default)]
    pub target: Option<String>,

    /// Guard condition name (must be defined in guards)
    #[serde(default)]
    pub guard: Option<String>,

    /// Actions to execute (must be defined in actions)
    #[serde(default)]
    pub actions: Option<ActionRef>,

    /// Transition description
    #[serde(default)]
    pub description: Option<String>,
}

/// Action reference - single or multiple
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionRef {
    Single(String),
    Multiple(Vec<String>),
}

impl ActionRef {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            ActionRef::Single(s) => vec![s.clone()],
            ActionRef::Multiple(v) => v.clone(),
        }
    }
}

/// Guard condition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardDef {
    /// Guard description or expression
    pub condition: String,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
}

/// Action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    /// Action description or implementation hint
    pub effect: String,

    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/spec/statemachine/schema.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/spec/statemachine/schema.rs` captured during libs codegen standardization.
```
