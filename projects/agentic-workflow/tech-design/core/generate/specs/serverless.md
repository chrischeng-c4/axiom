---
id: sdd-generate-specs-serverless
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Serverless Workflow Spec Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/specs/serverless.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DataCondition` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 71 |  |
| `DefaultCondition` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 84 |  |
| `FunctionType` | projects/agentic-workflow/src/generate/specs/serverless.rs | enum | pub | 16 |  |
| `OnError` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 92 |  |
| `ParallelBranch` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 107 |  |
| `ServerlessWorkflowInput` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 173 |  |
| `StateType` | projects/agentic-workflow/src/generate/specs/serverless.rs | enum | pub | 28 |  |
| `WorkflowAction` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 56 |  |
| `WorkflowError` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 162 |  |
| `WorkflowFunction` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 42 |  |
| `WorkflowState` | projects/agentic-workflow/src/generate/specs/serverless.rs | struct | pub | 118 |  |
| `generate_serverless_workflow` | projects/agentic-workflow/src/generate/specs/serverless.rs | function | pub | 197 | generate_serverless_workflow(input: &ServerlessWorkflowInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FunctionType:
    type: string
    enum: [Rest, Rpc, Graphql, Expression, Custom]
    description: Function type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: lowercase

  StateType:
    type: string
    enum: [Operation, Switch, Sleep, Parallel, Foreach, Inject, Event, Callback]
    description: State type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: lowercase

  WorkflowFunction:
    type: object
    required: [name, func_type, operation]
    description: Function definition.
    properties:
      name:
        type: string
        description: "Function name."
      func_type:
        type: string
        x-rust-type: "FunctionType"
        x-serde-rename: "type"
        description: "Function type."
      operation:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Operation reference."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  WorkflowAction:
    type: object
    required: [name, function_ref, arguments]
    description: Action definition.
    properties:
      name:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Action name."
      function_ref:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Reference to a defined function."
      arguments:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
        description: "Function arguments."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DataCondition:
    type: object
    required: [condition, transition, name]
    description: Data condition.
    properties:
      condition:
        type: string
        description: "Condition expression."
      transition:
        type: string
        description: "Target state on match."
      name:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Optional condition name."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DefaultCondition:
    type: object
    required: [transition]
    description: Default condition.
    properties:
      transition:
        type: string
        description: "Target state when no other conditions match."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  OnError:
    type: object
    required: [error_ref, transition, end]
    description: Error handler.
    properties:
      error_ref:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Reference to a defined error."
      transition:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Target state on error."
      end:
        type: boolean
        x-rust-type: "Option<bool>"
        x-serde-default: true
        description: "Terminate workflow on error."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ParallelBranch:
    type: object
    required: [name, actions]
    description: Parallel branch.
    properties:
      name:
        type: string
        description: "Branch name."
      actions:
        type: array
        items: { $ref: "#/definitions/WorkflowAction" }
        x-rust-type: "Vec<WorkflowAction>"
        x-serde-default: true
        description: "Actions in this branch."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  WorkflowState:
    type: object
    required: [name, state_type, actions, transition, end, data_conditions, default_condition, duration, branches, input_collection, iteration_param, data, on_errors]
    description: Workflow state.
    properties:
      name:
        type: string
        description: "State name."
      state_type:
        type: string
        x-rust-type: "StateType"
        x-serde-rename: "type"
        description: "State type."
      actions:
        type: array
        items: { $ref: "#/definitions/WorkflowAction" }
        x-rust-type: "Vec<WorkflowAction>"
        x-serde-default: true
        description: "Actions in this state."
      transition:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Target state on transition."
      end:
        type: boolean
        x-rust-type: "Option<bool>"
        x-serde-default: true
        description: "Terminate workflow on this state."
      data_conditions:
        type: array
        items: { $ref: "#/definitions/DataCondition" }
        x-rust-type: "Vec<DataCondition>"
        x-serde-default: true
        description: "Data conditions for switch states."
      default_condition:
        type: object
        x-rust-type: "Option<DefaultCondition>"
        x-serde-default: true
        description: "Default condition."
      duration:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Sleep duration (for sleep states)."
      branches:
        type: array
        items: { $ref: "#/definitions/ParallelBranch" }
        x-rust-type: "Vec<ParallelBranch>"
        x-serde-default: true
        description: "Branches for parallel states."
      input_collection:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Input collection for foreach states."
      iteration_param:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Iteration parameter name for foreach states."
      data:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
        description: "Data injected into the state context."
      on_errors:
        type: array
        items: { $ref: "#/definitions/OnError" }
        x-rust-type: "Vec<OnError>"
        x-serde-default: true
        description: "Error handlers for this state."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  WorkflowError:
    type: object
    required: [name, code]
    description: Error definition.
    properties:
      name:
        type: string
        description: "Error name."
      code:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Error code."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ServerlessWorkflowInput:
    type: object
    required: [id, name, start, states, version, description, functions, errors]
    description: Input for Serverless Workflow generation.
    properties:
      id:
        type: string
        description: "Workflow identifier."
      name:
        type: string
        description: "Workflow display name."
      start:
        type: string
        description: "Initial state name."
      states:
        type: array
        items: { $ref: "#/definitions/WorkflowState" }
        x-rust-type: "Vec<WorkflowState>"
        description: "Workflow states."
      version:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Workflow version."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Workflow description."
      functions:
        type: array
        items: { $ref: "#/definitions/WorkflowFunction" }
        x-rust-type: "Vec<WorkflowFunction>"
        x-serde-default: true
        description: "Function definitions."
      errors:
        type: array
        items: { $ref: "#/definitions/WorkflowError" }
        x-rust-type: "Vec<WorkflowError>"
        x-serde-default: true
        description: "Error definitions."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/specs/serverless.rs -->
```rust
//! Serverless Workflow 0.8 Specification Generation
//!
//! Generates Serverless Workflow 0.8 specifications for workflow orchestration.

use crate::generate::{GenerateError, Result};
use serde_json::{json, Value};

use serde::{Deserialize, Serialize};

/// Function type.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionType {
    Rest,
    Rpc,
    Graphql,
    Expression,
    Custom,
}

/// State type.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateType {
    Operation,
    Switch,
    Sleep,
    Parallel,
    Foreach,
    Inject,
    Event,
    Callback,
}

/// Function definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFunction {
    /// Function name.
    pub name: String,
    /// Function type.
    #[serde(rename = "type")]
    pub func_type: FunctionType,
    /// Operation reference.
    #[serde(default)]
    pub operation: Option<String>,
}

/// Action definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAction {
    /// Action name.
    #[serde(default)]
    pub name: Option<String>,
    /// Reference to a defined function.
    #[serde(default)]
    pub function_ref: Option<String>,
    /// Function arguments.
    #[serde(default)]
    pub arguments: Option<Value>,
}

/// Data condition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCondition {
    /// Condition expression.
    pub condition: String,
    /// Target state on match.
    pub transition: String,
    /// Optional condition name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Default condition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultCondition {
    /// Target state when no other conditions match.
    pub transition: String,
}

/// Error handler.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnError {
    /// Reference to a defined error.
    #[serde(default)]
    pub error_ref: Option<String>,
    /// Target state on error.
    #[serde(default)]
    pub transition: Option<String>,
    /// Terminate workflow on error.
    #[serde(default)]
    pub end: Option<bool>,
}

/// Parallel branch.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelBranch {
    /// Branch name.
    pub name: String,
    /// Actions in this branch.
    #[serde(default)]
    pub actions: Vec<WorkflowAction>,
}

/// Workflow state.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// State name.
    pub name: String,
    /// State type.
    #[serde(rename = "type")]
    pub state_type: StateType,
    /// Actions in this state.
    #[serde(default)]
    pub actions: Vec<WorkflowAction>,
    /// Target state on transition.
    #[serde(default)]
    pub transition: Option<String>,
    /// Terminate workflow on this state.
    #[serde(default)]
    pub end: Option<bool>,
    /// Data conditions for switch states.
    #[serde(default)]
    pub data_conditions: Vec<DataCondition>,
    /// Default condition.
    #[serde(default)]
    pub default_condition: Option<DefaultCondition>,
    /// Sleep duration (for sleep states).
    #[serde(default)]
    pub duration: Option<String>,
    /// Branches for parallel states.
    #[serde(default)]
    pub branches: Vec<ParallelBranch>,
    /// Input collection for foreach states.
    #[serde(default)]
    pub input_collection: Option<String>,
    /// Iteration parameter name for foreach states.
    #[serde(default)]
    pub iteration_param: Option<String>,
    /// Data injected into the state context.
    #[serde(default)]
    pub data: Option<Value>,
    /// Error handlers for this state.
    #[serde(default)]
    pub on_errors: Vec<OnError>,
}

/// Error definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    /// Error name.
    pub name: String,
    /// Error code.
    #[serde(default)]
    pub code: Option<String>,
}

/// Input for Serverless Workflow generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerlessWorkflowInput {
    /// Workflow identifier.
    pub id: String,
    /// Workflow display name.
    pub name: String,
    /// Initial state name.
    pub start: String,
    /// Workflow states.
    pub states: Vec<WorkflowState>,
    /// Workflow version.
    #[serde(default)]
    pub version: Option<String>,
    /// Workflow description.
    #[serde(default)]
    pub description: Option<String>,
    /// Function definitions.
    #[serde(default)]
    pub functions: Vec<WorkflowFunction>,
    /// Error definitions.
    #[serde(default)]
    pub errors: Vec<WorkflowError>,
}
/// Generate a Serverless Workflow 0.8 specification
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/serverless.md#source
pub fn generate_serverless_workflow(input: &ServerlessWorkflowInput) -> Result<String> {
    if input.states.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one state required".to_string(),
        ));
    }

    let version = input.version.as_deref().unwrap_or("1.0.0");

    let mut workflow = json!({
        "id": input.id,
        "version": version,
        "specVersion": "0.8",
        "name": input.name,
        "start": input.start
    });

    if let Some(ref desc) = input.description {
        workflow["description"] = json!(desc);
    }

    // Functions
    if !input.functions.is_empty() {
        let functions: Vec<Value> = input
            .functions
            .iter()
            .map(|f| {
                let type_str = match f.func_type {
                    FunctionType::Rest => "rest",
                    FunctionType::Rpc => "rpc",
                    FunctionType::Graphql => "graphql",
                    FunctionType::Expression => "expression",
                    FunctionType::Custom => "custom",
                };
                let mut func_obj = json!({
                    "name": f.name,
                    "type": type_str
                });
                if let Some(ref op) = f.operation {
                    func_obj["operation"] = json!(op);
                }
                func_obj
            })
            .collect();
        workflow["functions"] = json!(functions);
    }

    // Errors
    if !input.errors.is_empty() {
        let errors: Vec<Value> = input
            .errors
            .iter()
            .map(|e| {
                let mut err_obj = json!({ "name": e.name });
                if let Some(ref code) = e.code {
                    err_obj["code"] = json!(code);
                }
                err_obj
            })
            .collect();
        workflow["errors"] = json!(errors);
    }

    // States
    let states: Vec<Value> = input
        .states
        .iter()
        .map(|s| {
            let type_str = match s.state_type {
                StateType::Operation => "operation",
                StateType::Switch => "switch",
                StateType::Sleep => "sleep",
                StateType::Parallel => "parallel",
                StateType::Foreach => "foreach",
                StateType::Inject => "inject",
                StateType::Event => "event",
                StateType::Callback => "callback",
            };

            let mut state_obj = json!({
                "name": s.name,
                "type": type_str
            });

            // Actions (for operation state)
            if !s.actions.is_empty() {
                let actions: Vec<Value> = s
                    .actions
                    .iter()
                    .map(|a| {
                        let mut action_obj = json!({});
                        if let Some(ref name) = a.name {
                            action_obj["name"] = json!(name);
                        }
                        if let Some(ref func_ref) = a.function_ref {
                            action_obj["functionRef"] = json!(func_ref);
                        }
                        if let Some(ref args) = a.arguments {
                            action_obj["arguments"] = args.clone();
                        }
                        action_obj
                    })
                    .collect();
                state_obj["actions"] = json!(actions);
            }

            // Data conditions (for switch state)
            if !s.data_conditions.is_empty() {
                let conditions: Vec<Value> = s
                    .data_conditions
                    .iter()
                    .map(|c| {
                        let mut cond_obj = json!({
                            "condition": c.condition,
                            "transition": c.transition
                        });
                        if let Some(ref name) = c.name {
                            cond_obj["name"] = json!(name);
                        }
                        cond_obj
                    })
                    .collect();
                state_obj["dataConditions"] = json!(conditions);
            }

            if let Some(ref default) = s.default_condition {
                state_obj["defaultCondition"] = json!({ "transition": default.transition });
            }

            // Transition or end
            if let Some(ref transition) = s.transition {
                state_obj["transition"] = json!(transition);
            }
            if let Some(end) = s.end {
                state_obj["end"] = json!(end);
            }

            // Sleep duration
            if let Some(ref duration) = s.duration {
                state_obj["duration"] = json!(duration);
            }

            // Parallel branches
            if !s.branches.is_empty() {
                let branches: Vec<Value> = s
                    .branches
                    .iter()
                    .map(|b| {
                        json!({
                            "name": b.name,
                            "actions": b.actions.iter().map(|a| {
                                let mut action_obj = json!({});
                                if let Some(ref func_ref) = a.function_ref {
                                    action_obj["functionRef"] = json!(func_ref);
                                }
                                action_obj
                            }).collect::<Vec<_>>()
                        })
                    })
                    .collect();
                state_obj["branches"] = json!(branches);
            }

            // Foreach
            if let Some(ref input_coll) = s.input_collection {
                state_obj["inputCollection"] = json!(input_coll);
            }
            if let Some(ref iter_param) = s.iteration_param {
                state_obj["iterationParam"] = json!(iter_param);
            }

            // Inject data
            if let Some(ref data) = s.data {
                state_obj["data"] = data.clone();
            }

            // Error handlers
            if !s.on_errors.is_empty() {
                let errors: Vec<Value> = s
                    .on_errors
                    .iter()
                    .map(|e| {
                        let mut err_obj = json!({});
                        if let Some(ref err_ref) = e.error_ref {
                            err_obj["errorRef"] = json!(err_ref);
                        }
                        if let Some(ref transition) = e.transition {
                            err_obj["transition"] = json!(transition);
                        }
                        if let Some(end) = e.end {
                            err_obj["end"] = json!(end);
                        }
                        err_obj
                    })
                    .collect();
                state_obj["onErrors"] = json!(errors);
            }

            state_obj
        })
        .collect();

    workflow["states"] = json!(states);

    serde_yaml::to_string(&workflow).map_err(|e| GenerateError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_workflow() {
        let input = ServerlessWorkflowInput {
            id: "greeting".to_string(),
            name: "Greeting Workflow".to_string(),
            start: "greet".to_string(),
            version: Some("1.0.0".to_string()),
            description: Some("A simple greeting workflow".to_string()),
            functions: vec![WorkflowFunction {
                name: "greetFunction".to_string(),
                func_type: FunctionType::Rest,
                operation: Some("http://example.com/greet".to_string()),
            }],
            errors: vec![],
            states: vec![WorkflowState {
                name: "greet".to_string(),
                state_type: StateType::Operation,
                actions: vec![WorkflowAction {
                    name: None,
                    function_ref: Some("greetFunction".to_string()),
                    arguments: None,
                }],
                transition: None,
                end: Some(true),
                data_conditions: vec![],
                default_condition: None,
                duration: None,
                branches: vec![],
                input_collection: None,
                iteration_param: None,
                data: None,
                on_errors: vec![],
            }],
        };

        let result = generate_serverless_workflow(&input).unwrap();
        assert!(result.contains("specVersion: '0.8'"));
        assert!(result.contains("greeting"));
        assert!(result.contains("greet"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/specs/serverless.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Serverless Workflow specification generation module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Eleven serde shapes; mix of structs/enums; Vec + Options + foreign types.
- [schema] All well-formed; x-serde-rename for `type`-keyed fields; Option<bool>/Option<Value> via x-rust-type.
- [changes] All eleven in `replaces`.
