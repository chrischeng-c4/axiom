---
id: sdd-generate-state-plus-generator
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Mermaid Plus Generator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MermaidPlusGenerator` | projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs | struct | pub | 32 |  |
| `MermaidPlusOutput` | projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs | struct | pub | 19 |  |
| `generate` | projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs | function | pub | 40 | generate(         &self,         machine: &StateMachineDef,         validation: ValidationResult,     ) -> Result<MermaidPlusOutput, String> |
| `generate_mermaid` | projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs | function | pub | 90 | generate_mermaid(&self, machine: &StateMachineDef) -> Result<String, String> |
| `new` | projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs | function | pub | 35 | new() -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  MermaidPlusOutput:
    type: object
    required: [frontmatter, diagram, validation, combined]
    description: Mermaid+ generator output bundle.
    properties:
      frontmatter:
        type: string
        description: "YAML frontmatter content (without --- markers)."
      diagram:
        type: string
        description: "Mermaid diagram content (without ```mermaid``` markers)."
      validation:
        type: object
        x-rust-type: "ValidationResult"
        description: "Validation result."
      combined:
        type: string
        description: "Combined Mermaid+ format (ready to embed in markdown)."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]

  MermaidPlusGenerator:
    type: object
    required: []
    properties: {}
    description: Mermaid+ generator (unit struct).
    x-rust-struct:
      derive: []
      unit: true
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs -->
````rust
//! Mermaid+ generator
//!
//! Generates Mermaid+ output from validated state machine definitions.
//! Mermaid+ = YAML frontmatter (structured definition) + Mermaid diagram

use super::schema::{
    ActionRef, StateMachineDef, StateNodeDef, StateType, TransitionDetail, TransitionInput,
};
use super::validator::ValidationResult;
use std::collections::HashMap;

use serde::Serialize;

/// Mermaid+ generator output bundle.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/generator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct MermaidPlusOutput {
    /// YAML frontmatter content (without --- markers).
    pub frontmatter: String,
    /// Mermaid diagram content (without ```mermaid``` markers).
    pub diagram: String,
    /// Validation result.
    pub validation: ValidationResult,
    /// Combined Mermaid+ format (ready to embed in markdown).
    pub combined: String,
}

/// Mermaid+ generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/generator.md#schema
pub struct MermaidPlusGenerator;
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/generator.md#source
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
    pub fn generate_mermaid(&self, machine: &StateMachineDef) -> Result<String, String> {
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

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/state_plus/generator.md#source
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
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/state_plus/generator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete State+ generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Output struct + unit-struct generator; standard pattern.
- [schema] Both well-formed; foreign type ValidationResult via x-rust-type.
- [changes] Standard split.
