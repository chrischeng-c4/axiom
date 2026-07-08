---
id: libs-compass-src-spec-statemachine-mermaid-plus-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/spec/statemachine/mermaid_plus.rs`.
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

# Standardized libs/compass/src/spec/statemachine/mermaid_plus.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/spec/statemachine/mermaid_plus.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MermaidPlusOutput` | libs/compass/src/spec/statemachine/mermaid_plus.rs | struct | pub | 15 | pub struct MermaidPlusOutput { |
| `MermaidPlusGenerator` | libs/compass/src/spec/statemachine/mermaid_plus.rs | struct | pub | 27 | pub struct MermaidPlusGenerator; |
| `new` | libs/compass/src/spec/statemachine/mermaid_plus.rs | function | pub | 30 | pub fn new() -> Self { |
| `generate` | libs/compass/src/spec/statemachine/mermaid_plus.rs | function | pub | 35 | pub fn generate( |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Mermaid+ generator
//!
//! Generates Mermaid+ output from validated state machine definitions.
//! Mermaid+ = YAML frontmatter (structured definition) + Mermaid diagram

use super::schema::{
    ActionRef, StateMachineDef, StateNodeDef, StateType, TransitionDetail, TransitionInput,
};
use super::validator::ValidationResult;
use serde::Serialize;
use std::collections::HashMap;

/// Mermaid+ output structure
#[derive(Debug, Clone, Serialize)]
pub struct MermaidPlusOutput {
    /// YAML frontmatter content (without --- markers)
    pub frontmatter: String,
    /// Mermaid diagram content (without ```mermaid``` markers)
    pub diagram: String,
    /// Validation result
    pub validation: ValidationResult,
    /// Combined Mermaid+ format (ready to embed in markdown)
    pub combined: String,
}

/// Mermaid+ generator
pub struct MermaidPlusGenerator;

impl MermaidPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate Mermaid+ output from a state machine definition
    pub fn generate(
        &self,
        machine: &StateMachineDef,
        validation: ValidationResult,
    ) -> Result<MermaidPlusOutput, String> {
        // Generate YAML frontmatter
        let frontmatter = self.generate_frontmatter(machine)?;

        // Generate Mermaid diagram
        let diagram = self.generate_mermaid(machine)?;

        // Combine into Mermaid+ format (frontmatter inside code block per Mermaid spec)
        let mut combined = String::new();
        combined.push_str("```mermaid\n");
        combined.push_str("---\n");
        combined.push_str(&frontmatter);
        combined.push_str("---\n");
        combined.push_str(&diagram);
        combined.push_str("```\n");

        // Add validation warnings as HTML comments
        if !validation.warnings.is_empty() {
            combined.push_str("\n<!-- Validation Warnings:\n");
            for w in &validation.warnings {
                combined.push_str(&format!("  - {}: {} (at {})\n", w.code, w.message, w.path));
            }
            combined.push_str("-->\n");
        }

        Ok(MermaidPlusOutput {
            frontmatter,
            diagram,
            validation,
            combined,
        })
    }

    /// Generate YAML frontmatter from machine definition
    fn generate_frontmatter(&self, machine: &StateMachineDef) -> Result<String, String> {
        // Use serde_yaml but strip the leading "---\n" if present
        let yaml = serde_yaml::to_string(machine)
            .map_err(|e| format!("YAML serialization error: {}", e))?;

        // serde_yaml adds "---\n" at the start, strip it since we add our own
        let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml);

        Ok(yaml.to_string())
    }

    /// Generate Mermaid stateDiagram-v2 from machine definition
    fn generate_mermaid(&self, machine: &StateMachineDef) -> Result<String, String> {
        let mut mermaid = String::new();
        mermaid.push_str("stateDiagram-v2\n");

        // Add initial transition
        mermaid.push_str(&format!("    [*] --> {}\n", machine.initial));

        // Generate states and transitions
        self.generate_states(&machine.states, &mut mermaid, "    ")?;

        Ok(mermaid)
    }

    /// Generate Mermaid for states recursively
    fn generate_states(
        &self,
        states: &HashMap<String, StateNodeDef>,
        mermaid: &mut String,
        indent: &str,
    ) -> Result<(), String> {
        // Sort states for consistent output
        let mut state_ids: Vec<_> = states.keys().collect();
        state_ids.sort();

        for state_id in state_ids {
            let node = &states[state_id];
            let node_type = node.node_type.as_ref().unwrap_or(&StateType::Atomic);

            // Handle compound/parallel states
            if let Some(ref substates) = node.states {
                if *node_type == StateType::Parallel {
                    // Parallel state with region separators
                    if let Some(ref desc) = node.description {
                        mermaid
                            .push_str(&format!("{}state \"{}\" as {}\n", indent, desc, state_id));
                    }
                    mermaid.push_str(&format!("{}state {} {{\n", indent, state_id));

                    // Render each child as a separate region with -- separators
                    let mut substate_ids: Vec<_> = substates.keys().collect();
                    substate_ids.sort();

                    let child_indent = format!("{}    ", indent);
                    for (i, substate_id) in substate_ids.iter().enumerate() {
                        if i > 0 {
                            // Add region separator between parallel regions
                            mermaid.push_str(&format!("{}--\n", child_indent));
                        }
                        // Generate the substate inline (not recursive for parallel regions)
                        let subnode = &substates[*substate_id];
                        self.generate_single_state(substate_id, subnode, mermaid, &child_indent)?;
                    }
                } else {
                    // Compound state
                    if let Some(ref desc) = node.description {
                        mermaid
                            .push_str(&format!("{}state \"{}\" as {}\n", indent, desc, state_id));
                    }
                    mermaid.push_str(&format!("{}state {} {{\n", indent, state_id));

                    // Add initial for compound
                    if let Some(ref initial) = node.initial {
                        mermaid.push_str(&format!("{}    [*] --> {}\n", indent, initial));
                    }

                    self.generate_states(substates, mermaid, &format!("{}    ", indent))?;
                }
                mermaid.push_str(&format!("{}}}\n", indent));
            } else if *node_type == StateType::Final {
                // Final state - add transition to [*]
                mermaid.push_str(&format!("{}{} --> [*]\n", indent, state_id));
            } else {
                // Regular state with description
                if let Some(ref desc) = node.description {
                    mermaid.push_str(&format!("{}state \"{}\" as {}\n", indent, desc, state_id));
                }
            }

            // Generate transitions
            if let Some(ref on) = node.on {
                let mut events: Vec<_> = on.keys().collect();
                events.sort();

                for event in events {
                    let transition = &on[event];
                    self.generate_transition(state_id, event, transition, mermaid, indent)?;
                }
            }
        }

        Ok(())
    }

    /// Generate Mermaid for a single state (used for parallel regions)
    fn generate_single_state(
        &self,
        state_id: &str,
        node: &StateNodeDef,
        mermaid: &mut String,
        indent: &str,
    ) -> Result<(), String> {
        let node_type = node.node_type.as_ref().unwrap_or(&StateType::Atomic);

        // Handle nested compound/parallel states within parallel regions
        if let Some(ref substates) = node.states {
            if let Some(ref desc) = node.description {
                mermaid.push_str(&format!("{}state \"{}\" as {}\n", indent, desc, state_id));
            }
            mermaid.push_str(&format!("{}state {} {{\n", indent, state_id));

            // Check if this is a nested parallel state
            if *node_type == StateType::Parallel {
                // Render nested parallel with region separators
                let mut substate_ids: Vec<_> = substates.keys().collect();
                substate_ids.sort();

                let child_indent = format!("{}    ", indent);
                for (i, substate_id) in substate_ids.iter().enumerate() {
                    if i > 0 {
                        mermaid.push_str(&format!("{}--\n", child_indent));
                    }
                    let subnode = &substates[*substate_id];
                    self.generate_single_state(substate_id, subnode, mermaid, &child_indent)?;
                }
            } else {
                // Compound state - add initial and recurse
                if let Some(ref initial) = node.initial {
                    mermaid.push_str(&format!("{}    [*] --> {}\n", indent, initial));
                }
                self.generate_states(substates, mermaid, &format!("{}    ", indent))?;
            }

            mermaid.push_str(&format!("{}}}\n", indent));
        } else if *node_type == StateType::Final {
            mermaid.push_str(&format!("{}{} --> [*]\n", indent, state_id));
        } else {
            // Regular atomic state - always emit state declaration for visibility in parallel regions
            if let Some(ref desc) = node.description {
                mermaid.push_str(&format!("{}state \"{}\" as {}\n", indent, desc, state_id));
            } else {
                // Emit simple state declaration to ensure visibility
                mermaid.push_str(&format!("{}state {}\n", indent, state_id));
            }
        }

        // Generate transitions
        if let Some(ref on) = node.on {
            let mut events: Vec<_> = on.keys().collect();
            events.sort();

            for event in events {
                let transition = &on[event];
                self.generate_transition(state_id, event, transition, mermaid, indent)?;
            }
        }

        Ok(())
    }

    /// Generate Mermaid for a transition
    fn generate_transition(
        &self,
        from: &str,
        event: &str,
        transition: &TransitionInput,
        mermaid: &mut String,
        indent: &str,
    ) -> Result<(), String> {
        match transition {
            TransitionInput::Simple(target) => {
                mermaid.push_str(&format!("{}{} --> {}: {}\n", indent, from, target, event));
            }
            TransitionInput::Detailed(detail) => {
                self.generate_detailed_transition(from, event, detail, mermaid, indent)?;
            }
            TransitionInput::Conditional(conditions) => {
                for detail in conditions {
                    self.generate_detailed_transition(from, event, detail, mermaid, indent)?;
                }
            }
        }
        Ok(())
    }

    fn generate_detailed_transition(
        &self,
        from: &str,
        event: &str,
        detail: &TransitionDetail,
        mermaid: &mut String,
        indent: &str,
    ) -> Result<(), String> {
        // Determine target: use explicit target or self (internal transition)
        let target = detail.target.as_deref().unwrap_or(from);

        let mut label = event.to_string();

        // Add guard
        if let Some(ref guard) = detail.guard {
            label = format!("{} [{}]", label, guard);
        }

        // Add actions
        if let Some(ref actions) = detail.actions {
            let action_str = match actions {
                ActionRef::Single(a) => a.clone(),
                ActionRef::Multiple(list) => list.join(", "),
            };
            label = format!("{} / {}", label, action_str);
        }

        mermaid.push_str(&format!("{}{} --> {}: {}\n", indent, from, target, label));
        Ok(())
    }
}

impl Default for MermaidPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::StateMachineValidator;
    use super::*;
    use serde_json::json;

    fn parse_machine(json: serde_json::Value) -> StateMachineDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_simple_mermaid() {
        let machine = parse_machine(json!({
            "id": "toggle",
            "initial": "off",
            "states": {
                "off": { "on": { "TOGGLE": "on" } },
                "on": { "on": { "TOGGLE": "off" } }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        assert!(output.diagram.contains("stateDiagram-v2"));
        assert!(output.diagram.contains("[*] --> off"));
        assert!(output.diagram.contains("off --> on: TOGGLE"));
        assert!(output.diagram.contains("on --> off: TOGGLE"));
    }

    #[test]
    fn test_generate_with_guards() {
        let machine = parse_machine(json!({
            "id": "fetch",
            "initial": "idle",
            "states": {
                "idle": {
                    "on": {
                        "FETCH": { "target": "loading", "guard": "canFetch" }
                    }
                },
                "loading": { "on": { "SUCCESS": "done" } },
                "done": { "type": "final" }
            },
            "guards": {
                "canFetch": { "condition": "retries < 3" }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        assert!(output
            .diagram
            .contains("idle --> loading: FETCH [canFetch]"));
        assert!(output.diagram.contains("done --> [*]"));
    }

    #[test]
    fn test_generate_nested_states() {
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

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        assert!(output.diagram.contains("state review {"));
        assert!(output.diagram.contains("[*] --> pending"));
    }

    #[test]
    fn test_parallel_state_with_regions() {
        // Parallel states should have -- separators between regions
        let machine = parse_machine(json!({
            "id": "upload",
            "initial": "processing",
            "states": {
                "processing": {
                    "type": "parallel",
                    "states": {
                        "upload": {
                            "initial": "pending",
                            "type": "compound",
                            "states": {
                                "pending": { "on": { "COMPLETE": "done" } },
                                "done": { "type": "final" }
                            }
                        },
                        "thumbnail": {
                            "initial": "generating",
                            "type": "compound",
                            "states": {
                                "generating": { "on": { "COMPLETE": "done" } },
                                "done": { "type": "final" }
                            }
                        }
                    }
                }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        // Should have parallel state wrapper
        assert!(
            output.diagram.contains("state processing {"),
            "Should have parallel state wrapper. Diagram:\n{}",
            output.diagram
        );
        // Should have region separator
        assert!(
            output.diagram.contains("--"),
            "Should have region separator --. Diagram:\n{}",
            output.diagram
        );
        // Both regions should be rendered
        assert!(
            output.diagram.contains("state thumbnail {"),
            "Should have thumbnail region. Diagram:\n{}",
            output.diagram
        );
        assert!(
            output.diagram.contains("state upload {"),
            "Should have upload region. Diagram:\n{}",
            output.diagram
        );
    }

    #[test]
    fn test_internal_transition_self_loop() {
        // Internal transitions (no target) should render as self-transitions
        let machine = parse_machine(json!({
            "id": "counter",
            "initial": "counting",
            "states": {
                "counting": {
                    "on": {
                        "INCREMENT": { "actions": "increment" },
                        "RESET": "idle"
                    }
                },
                "idle": { "on": { "START": "counting" } }
            },
            "actions": {
                "increment": { "effect": "context.count += 1" }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        // Internal transition should render as self-loop
        assert!(
            output
                .diagram
                .contains("counting --> counting: INCREMENT / increment"),
            "Internal transition should render as self-loop. Diagram:\n{}",
            output.diagram
        );
        // Regular transition should still work
        assert!(output.diagram.contains("counting --> idle: RESET"));
    }

    #[test]
    fn test_internal_transition_with_guard() {
        // Internal transition with guard should also render as self-loop
        let machine = parse_machine(json!({
            "id": "validator",
            "initial": "editing",
            "states": {
                "editing": {
                    "on": {
                        "VALIDATE": { "guard": "hasInput", "actions": "runValidation" }
                    }
                }
            },
            "guards": { "hasInput": { "condition": "input.length > 0" } },
            "actions": { "runValidation": { "effect": "validate(context.input)" } }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        assert!(
            output
                .diagram
                .contains("editing --> editing: VALIDATE [hasInput] / runValidation"),
            "Internal transition with guard should render correctly. Diagram:\n{}",
            output.diagram
        );
    }

    #[test]
    fn test_mermaid_plus_format() {
        let machine = parse_machine(json!({
            "id": "simple",
            "initial": "a",
            "states": {
                "a": { "on": { "GO": "b" } },
                "b": { "type": "final" }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        // Check combined format (frontmatter inside code block)
        assert!(output.combined.starts_with("```mermaid\n---\n"));
        assert!(output.combined.contains("id: simple"));
        assert!(output.combined.contains("initial: a"));
        assert!(output.combined.contains("stateDiagram-v2"));
    }

    #[test]
    fn test_conditional_transitions_render() {
        // Conditional transitions should render multiple arrows with guards
        let machine = parse_machine(json!({
            "id": "conditional",
            "initial": "idle",
            "states": {
                "idle": {
                    "on": {
                        "SUBMIT": [
                            { "target": "success", "guard": "isValid" },
                            { "target": "error", "guard": "hasErrors" },
                            { "target": "pending" }
                        ]
                    }
                },
                "success": { "type": "final" },
                "error": {},
                "pending": {}
            },
            "guards": {
                "isValid": { "condition": "true" },
                "hasErrors": { "condition": "false" }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        // All conditional branches should be rendered
        assert!(
            output
                .diagram
                .contains("idle --> success: SUBMIT [isValid]"),
            "Should render guarded transition to success. Diagram:\n{}",
            output.diagram
        );
        assert!(
            output
                .diagram
                .contains("idle --> error: SUBMIT [hasErrors]"),
            "Should render guarded transition to error. Diagram:\n{}",
            output.diagram
        );
        assert!(
            output.diagram.contains("idle --> pending: SUBMIT"),
            "Should render default transition to pending. Diagram:\n{}",
            output.diagram
        );
    }

    #[test]
    fn test_deep_nested_state_rendering() {
        // Deep nesting should render correctly with proper indentation
        let machine = parse_machine(json!({
            "id": "deep",
            "initial": "l1",
            "states": {
                "l1": {
                    "type": "compound",
                    "initial": "l2",
                    "states": {
                        "l2": {
                            "type": "compound",
                            "initial": "l3",
                            "states": {
                                "l3": { "on": { "GO": "done" } },
                                "done": { "type": "final" }
                            }
                        }
                    }
                }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        // Should have nested state blocks
        assert!(
            output.diagram.contains("state l1 {"),
            "Should have l1 state block"
        );
        assert!(
            output.diagram.contains("state l2 {"),
            "Should have l2 state block"
        );
        // Initial transitions at each level
        assert!(
            output.diagram.contains("[*] --> l2"),
            "Should have l1 initial"
        );
        assert!(
            output.diagram.contains("[*] --> l3"),
            "Should have l2 initial"
        );
    }

    #[test]
    fn test_nested_parallel_states() {
        // Nested parallel states should also have region separators
        let machine = parse_machine(json!({
            "id": "nested_parallel",
            "initial": "outer",
            "states": {
                "outer": {
                    "type": "parallel",
                    "states": {
                        "region1": {
                            "type": "parallel",
                            "states": {
                                "sub1": {},
                                "sub2": {}
                            }
                        },
                        "region2": {
                            "type": "compound",
                            "initial": "a",
                            "states": {
                                "a": { "on": { "GO": "b" } },
                                "b": { "type": "final" }
                            }
                        }
                    }
                }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        // Outer parallel should have separators
        assert!(
            output.diagram.contains("state outer {"),
            "Should have outer parallel. Diagram:\n{}",
            output.diagram
        );
        // Nested parallel (region1) should also be rendered with separators
        assert!(
            output.diagram.contains("state region1 {"),
            "Should have nested parallel region1. Diagram:\n{}",
            output.diagram
        );
        // Should have multiple -- separators (outer and nested)
        let separator_count = output.diagram.matches("--").count();
        assert!(
            separator_count >= 2,
            "Should have at least 2 separators (outer + nested). Got {}. Diagram:\n{}",
            separator_count,
            output.diagram
        );
    }

    #[test]
    fn test_atomic_states_visible_in_parallel() {
        // Atomic states without descriptions should still be visible in parallel regions
        let machine = parse_machine(json!({
            "id": "parallel_atomic",
            "initial": "parallel",
            "states": {
                "parallel": {
                    "type": "parallel",
                    "states": {
                        "atomic1": {},
                        "atomic2": {},
                        "atomic3": { "description": "Third region" }
                    }
                }
            }
        }));

        let validation = StateMachineValidator::new().validate(&machine);
        let output = MermaidPlusGenerator::new()
            .generate(&machine, validation)
            .unwrap();

        // All atomic states should be visible (have state declarations)
        assert!(
            output.diagram.contains("state atomic1"),
            "atomic1 should be declared. Diagram:\n{}",
            output.diagram
        );
        assert!(
            output.diagram.contains("state atomic2"),
            "atomic2 should be declared. Diagram:\n{}",
            output.diagram
        );
        assert!(
            output.diagram.contains("state \"Third region\" as atomic3"),
            "atomic3 should have description. Diagram:\n{}",
            output.diagram
        );
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/spec/statemachine/mermaid_plus.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/spec/statemachine/mermaid_plus.rs` captured during libs codegen standardization.
```
