// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/schema.md#source
// CODEGEN-BEGIN
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

/// Whether the migration verb may write files (Issue A present) or only report (Issue A absent).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mermaid_plus/migrate.md#schema
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationMode {
    WriteMode,
    DryRunMode,
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
// CODEGEN-END
