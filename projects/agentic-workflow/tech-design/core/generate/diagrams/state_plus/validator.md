---
id: sdd-generate-state-plus-validator
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# State Plus Validator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Severity` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | enum | pub | 18 |  |
| `StateMachineValidator` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | struct | pub | 51 |  |
| `ValidationError` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | struct | pub | 38 |  |
| `ValidationResult` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | struct | pub | 26 |  |
| `new` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | function | pub | 90 | new() -> Self |
| `ok` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | function | pub | 57 | ok() -> Self |
| `strict` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | function | pub | 94 | strict(mut self) -> Self |
| `validate` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | function | pub | 100 | validate(&self, machine: &StateMachineDef) -> ValidationResult |
| `with_error` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | function | pub | 65 | with_error(mut self, error: ValidationError) -> Self |
| `with_warning` | projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs | function | pub | 71 | with_warning(mut self, warning: ValidationError) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  Severity:
    type: string
    enum: [Error, Warning]
    description: Error severity.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, PartialEq]
      serde_rename_all: lowercase

  ValidationResult:
    type: object
    required: [valid, errors, warnings]
    description: Validation result.
    properties:
      valid:
        type: boolean
        description: "Whether validation passed (no errors)."
      errors:
        type: array
        items: { $ref: "#/definitions/ValidationError" }
        x-rust-type: "Vec<ValidationError>"
        description: "Validation errors."
      warnings:
        type: array
        items: { $ref: "#/definitions/ValidationError" }
        x-rust-type: "Vec<ValidationError>"
        description: "Validation warnings."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Default]

  ValidationError:
    type: object
    required: [code, message, path, severity]
    description: Validation error/warning.
    properties:
      code:
        type: string
        description: "Error code."
      message:
        type: string
        description: "Human-readable message."
      path:
        type: string
        description: "JSON pointer path."
      severity:
        type: string
        x-rust-type: "Severity"
        description: "Severity level."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]

  StateMachineValidator:
    type: object
    required: [strict]
    description: State machine validator.
    properties:
      strict:
        type: boolean
        x-rust-visibility: private
        description: "Strict mode: treats warnings as errors for certain checks."
    x-rust-struct:
      derive: []
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs -->
```rust
//! State machine semantic validator
//!
//! Validates state machine definitions for:
//! - Structural correctness (initial exists, targets valid)
//! - Semantic correctness (reachability, guard/action references)

use super::schema::{StateMachineDef, StateNodeDef, StateType, TransitionInput};
use std::collections::{HashMap, HashSet};

use serde::Serialize;

/// Error severity.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/validator.md#schema
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

/// Validation result.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/validator.md#schema
#[derive(Debug, Clone, Serialize, Default)]
pub struct ValidationResult {
    /// Whether validation passed (no errors).
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<ValidationError>,
    /// Validation warnings.
    pub warnings: Vec<ValidationError>,
}

/// Validation error/warning.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/validator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    /// Error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// JSON pointer path.
    pub path: String,
    /// Severity level.
    pub severity: Severity,
}

/// State machine validator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/validator.md#schema
pub struct StateMachineValidator {
    /// Strict mode: treats warnings as errors for certain checks.
    strict: bool,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/validator.md#source
impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: ValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: ValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// State info for validation - includes node reference and path
#[derive(Debug, Clone)]
struct StateInfo<'a> {
    /// The state node definition
    node: &'a StateNodeDef,
    /// Path-qualified ID (e.g., "parent.child")
    path_id: String,
    /// Simple ID (last component)
    simple_id: String,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/validator.md#source
impl StateMachineValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Validate a state machine definition
    pub fn validate(&self, machine: &StateMachineDef) -> ValidationResult {
        let mut result = ValidationResult::ok();

        // Build flat map with path-qualified IDs
        let states = self.collect_states(&machine.states, "");

        // 1. Check initial state exists
        if !states.iter().any(|s| s.simple_id == machine.initial) {
            result = result.with_error(ValidationError {
                code: "MISSING_INITIAL_STATE".to_string(),
                message: format!("Initial state '{}' not found in states", machine.initial),
                path: "initial".to_string(),
                severity: Severity::Error,
            });
        }

        // 2. Validate all transitions
        for state_info in &states {
            if let Some(ref on) = state_info.node.on {
                for (event, transition) in on {
                    self.validate_transition(
                        &state_info.path_id,
                        event,
                        transition,
                        &states,
                        machine,
                        &mut result,
                    );
                }
            }

            // 3. Validate compound states have initial
            if let Some(StateType::Compound) = state_info.node.node_type {
                if state_info.node.initial.is_none() && state_info.node.states.is_some() {
                    result = result.with_warning(ValidationError {
                        code: "MISSING_COMPOUND_INITIAL".to_string(),
                        message: format!(
                            "Compound state '{}' has substates but no initial",
                            state_info.path_id
                        ),
                        path: format!("states.{}", state_info.path_id),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        // 4. Check for unreachable states (warning)
        let reachable = self.find_reachable(&machine.initial, &states);
        for state_info in &states {
            if !reachable.contains(&state_info.path_id) && state_info.path_id != machine.initial {
                // Check if it's a substate of a reachable parent
                let parent_reachable = state_info
                    .path_id
                    .rsplit_once('.')
                    .is_some_and(|(parent, _)| reachable.contains(parent));

                if !parent_reachable {
                    result = result.with_warning(ValidationError {
                        code: "UNREACHABLE_STATE".to_string(),
                        message: format!("State '{}' may be unreachable", state_info.path_id),
                        path: format!("states.{}", state_info.path_id),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        // 5. Validate guard references
        for state_info in &states {
            if let Some(ref on) = state_info.node.on {
                for (_, transition) in on {
                    self.validate_guard_refs(transition, machine, &state_info.path_id, &mut result);
                }
            }
        }

        // 6. Validate action references
        for state_info in &states {
            if let Some(ref on) = state_info.node.on {
                for (_, transition) in on {
                    self.validate_action_refs(
                        transition,
                        machine,
                        &state_info.path_id,
                        &mut result,
                    );
                }
            }
            // Check entry/exit actions
            if let Some(ref entry) = state_info.node.entry {
                for action in entry.to_vec() {
                    if !machine.actions.contains_key(&action) {
                        result = result.with_warning(ValidationError {
                            code: "UNDEFINED_ACTION".to_string(),
                            message: format!("Entry action '{}' not defined in actions", action),
                            path: format!("states.{}.entry", state_info.path_id),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            if let Some(ref exit) = state_info.node.exit {
                for action in exit.to_vec() {
                    if !machine.actions.contains_key(&action) {
                        result = result.with_warning(ValidationError {
                            code: "UNDEFINED_ACTION".to_string(),
                            message: format!("Exit action '{}' not defined in actions", action),
                            path: format!("states.{}.exit", state_info.path_id),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        // 7. In strict mode, convert certain warnings to errors
        if self.strict {
            let strict_codes = ["MISSING_COMPOUND_INITIAL"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));

            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = Severity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }

    /// Collect all states with path-qualified IDs
    fn collect_states<'a>(
        &self,
        states: &'a HashMap<String, StateNodeDef>,
        prefix: &str,
    ) -> Vec<StateInfo<'a>> {
        let mut result = Vec::new();

        for (id, node) in states {
            let path_id = if prefix.is_empty() {
                id.clone()
            } else {
                format!("{}.{}", prefix, id)
            };

            result.push(StateInfo {
                node,
                path_id: path_id.clone(),
                simple_id: id.clone(),
            });

            // Recursively collect substates
            if let Some(ref substates) = node.states {
                result.extend(self.collect_states(substates, &path_id));
            }
        }

        result
    }

    /// Validate a transition
    fn validate_transition(
        &self,
        from_path: &str,
        event: &str,
        transition: &TransitionInput,
        all_states: &[StateInfo],
        _machine: &StateMachineDef,
        result: &mut ValidationResult,
    ) {
        match transition {
            TransitionInput::Simple(target) => {
                self.validate_target(from_path, event, target, all_states, result);
            }
            TransitionInput::Detailed(detail) => {
                if let Some(ref target) = detail.target {
                    self.validate_target(from_path, event, target, all_states, result);
                }
            }
            TransitionInput::Conditional(conditions) => {
                for detail in conditions {
                    if let Some(ref target) = detail.target {
                        self.validate_target(from_path, event, target, all_states, result);
                    }
                }
            }
        }
    }

    /// Validate a transition target exists
    fn validate_target(
        &self,
        from_path: &str,
        event: &str,
        target: &str,
        all_states: &[StateInfo],
        result: &mut ValidationResult,
    ) {
        // Check if target exists (either as simple ID or path-qualified)
        let target_exists = all_states
            .iter()
            .any(|s| s.simple_id == target || s.path_id == target);

        // Also check if target is relative to current state's parent
        let parent_relative_exists = if let Some((parent, _)) = from_path.rsplit_once('.') {
            let full_path = format!("{}.{}", parent, target);
            all_states.iter().any(|s| s.path_id == full_path)
        } else {
            false
        };

        if !target_exists && !parent_relative_exists {
            *result = std::mem::take(result).with_error(ValidationError {
                code: "INVALID_TRANSITION_TARGET".to_string(),
                message: format!(
                    "Transition target '{}' not found (from '{}' on '{}')",
                    target, from_path, event
                ),
                path: format!("states.{}.on.{}", from_path, event),
                severity: Severity::Error,
            });
        }
    }

    /// Find all reachable states from initial
    fn find_reachable(&self, initial: &str, all_states: &[StateInfo]) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut queue = vec![initial.to_string()];

        while let Some(current) = queue.pop() {
            if reachable.contains(&current) {
                continue;
            }
            reachable.insert(current.clone());

            // Find the state info
            if let Some(state_info) = all_states
                .iter()
                .find(|s| s.simple_id == current || s.path_id == current)
            {
                if let Some(ref on) = state_info.node.on {
                    for (_, transition) in on {
                        for target in self.get_targets(transition) {
                            if !reachable.contains(&target) {
                                queue.push(target);
                            }
                        }
                    }
                }

                // Substates of compound states are reachable if parent is reachable
                if let Some(ref substates) = state_info.node.states {
                    for substate_id in substates.keys() {
                        let sub_path = format!("{}.{}", state_info.path_id, substate_id);
                        if !reachable.contains(&sub_path) {
                            queue.push(sub_path);
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Get all target states from a transition
    fn get_targets(&self, transition: &TransitionInput) -> Vec<String> {
        match transition {
            TransitionInput::Simple(target) => vec![target.clone()],
            TransitionInput::Detailed(detail) => detail.target.clone().map_or(vec![], |t| vec![t]),
            TransitionInput::Conditional(conditions) => {
                conditions.iter().filter_map(|d| d.target.clone()).collect()
            }
        }
    }

    /// Validate guard references
    fn validate_guard_refs(
        &self,
        transition: &TransitionInput,
        machine: &StateMachineDef,
        state_path: &str,
        result: &mut ValidationResult,
    ) {
        let guards = match transition {
            TransitionInput::Simple(_) => vec![],
            TransitionInput::Detailed(d) => d.guard.clone().map_or(vec![], |g| vec![g]),
            TransitionInput::Conditional(cs) => cs.iter().filter_map(|d| d.guard.clone()).collect(),
        };

        for guard in guards {
            if !machine.guards.contains_key(&guard) {
                *result = std::mem::take(result).with_warning(ValidationError {
                    code: "UNDEFINED_GUARD".to_string(),
                    message: format!("Guard '{}' not defined in guards", guard),
                    path: format!("states.{}", state_path),
                    severity: Severity::Warning,
                });
            }
        }
    }

    /// Validate action references
    fn validate_action_refs(
        &self,
        transition: &TransitionInput,
        machine: &StateMachineDef,
        state_path: &str,
        result: &mut ValidationResult,
    ) {
        let actions = match transition {
            TransitionInput::Simple(_) => vec![],
            TransitionInput::Detailed(d) => d.actions.as_ref().map_or(vec![], |a| a.to_vec()),
            TransitionInput::Conditional(cs) => cs
                .iter()
                .flat_map(|d| d.actions.as_ref().map_or(vec![], |a| a.to_vec()))
                .collect(),
        };

        for action in actions {
            if !machine.actions.contains_key(&action) {
                *result = std::mem::take(result).with_warning(ValidationError {
                    code: "UNDEFINED_ACTION".to_string(),
                    message: format!("Action '{}' not defined in actions", action),
                    path: format!("states.{}", state_path),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/validator.md#source
impl Default for StateMachineValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_machine(json: serde_json::Value) -> StateMachineDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_simple_machine() {
        let machine = parse_machine(json!({
            "id": "toggle",
            "initial": "off",
            "states": {
                "off": { "on": { "TOGGLE": "on" } },
                "on": { "on": { "TOGGLE": "off" } }
            }
        }));

        let result = StateMachineValidator::new().validate(&machine);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_initial() {
        let machine = parse_machine(json!({
            "id": "test",
            "initial": "nonexistent",
            "states": {
                "a": {}
            }
        }));

        let result = StateMachineValidator::new().validate(&machine);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == "MISSING_INITIAL_STATE"));
    }

    #[test]
    fn test_invalid_target() {
        let machine = parse_machine(json!({
            "id": "test",
            "initial": "a",
            "states": {
                "a": { "on": { "GO": "nonexistent" } }
            }
        }));

        let result = StateMachineValidator::new().validate(&machine);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == "INVALID_TRANSITION_TARGET"));
    }

    #[test]
    fn test_undefined_guard_warning() {
        let machine = parse_machine(json!({
            "id": "test",
            "initial": "a",
            "states": {
                "a": {
                    "on": {
                        "GO": { "target": "b", "guard": "undefined_guard" }
                    }
                },
                "b": {}
            }
        }));

        let result = StateMachineValidator::new().validate(&machine);
        assert!(result.valid); // Warnings don't invalidate
        assert!(result.warnings.iter().any(|w| w.code == "UNDEFINED_GUARD"));
    }

    #[test]
    fn test_nested_state_validation() {
        let machine = parse_machine(json!({
            "id": "workflow",
            "initial": "draft",
            "states": {
                "draft": { "on": { "SUBMIT": "review" } },
                "review": {
                    "type": "compound",
                    "initial": "pending",
                    "states": {
                        "pending": { "on": { "APPROVE": "approved" } },
                        "approved": { "type": "final" }
                    }
                }
            }
        }));

        let result = StateMachineValidator::new().validate(&machine);
        assert!(result.valid);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/state_plus/validator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete State+ validator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Four public types: result, error, severity, validator.
- [schema] All well-formed; private bool field on validator.
- [changes] Standard split with all four in `replaces`; private StateInfo preserved.
