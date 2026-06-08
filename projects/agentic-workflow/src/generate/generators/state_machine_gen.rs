// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_preamble.md#source
// CODEGEN-BEGIN
//! State machine code generator
//!
//! Generates Python Enum + transition function from a [`StateMachineDef`]
//! (state-machine section type):
//!
//! | Output file                 | Description                                    |
//! |-----------------------------|------------------------------------------------|
//! | `{machine_id}_states.py`    | Python Enum class + `transition()` function    |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::StateMachinePlus`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::SpecIR;

// ---------------------------------------------------------------------------
// StateMachineGenerator
// ---------------------------------------------------------------------------
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/state_machine_gen.md#schema
// CODEGEN-BEGIN
/// State machine generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/state_machine_gen.md#schema
pub struct StateMachineGenerator;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_runtime.md#source
impl StateMachineGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_runtime.md#source
impl Default for StateMachineGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SpecIRGenerator impl
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_runtime.md#source
impl SpecIRGenerator for StateMachineGenerator {
    fn can_generate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::StateMachinePlus { .. })
    }

    fn template_dir(&self) -> &'static str {
        "state_machine"
    }

    fn generate_from_ir(
        &self,
        spec: &SpecIR,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError> {
        let sm_def = match spec {
            SpecIR::StateMachinePlus { def, .. } => def,
            _ => {
                return Err(GeneratorError::SchemaError(
                    "StateMachineGenerator: expected SpecIR::StateMachinePlus variant".into(),
                ))
            }
        };

        let mut manifest = Manifest::new();

        let file_name = format!("{}_states.py", sm_def.id.replace('-', "_"));
        let output_path = settings.output_dir.join(&file_name);

        if output_path.exists() {
            match settings.overwrite_policy {
                OverwritePolicy::Error => {
                    return Err(GeneratorError::OverwriteNotAllowed(output_path));
                }
                OverwritePolicy::Skip => {
                    manifest.add(GeneratedFile::skipped(output_path));
                    return Ok(manifest);
                }
                OverwritePolicy::Overwrite => {}
            }
        }

        let template_name = format!("{}/states.py.j2", self.template_dir());
        let content = if engine.has_template(&template_name) {
            engine.render(&template_name, &sm_def).map_err(|e| {
                GeneratorError::TemplateRenderError {
                    template: template_name.clone(),
                    message: e.to_string(),
                }
            })?
        } else {
            generate_state_machine_python(sm_def)
        };

        manifest.add(GeneratedFile::written(output_path, &content));
        Ok(manifest)
    }
}

// ---------------------------------------------------------------------------
// Inline generator
// ---------------------------------------------------------------------------

/// Convert a machine ID like "my-workflow" to PascalCase "MyWorkflow".
fn to_pascal_case(s: &str) -> String {
    use heck::ToPascalCase;
    s.to_pascal_case()
}

/// Convert a state name to UPPER_SNAKE_CASE for the Enum member.
fn to_upper_snake(s: &str) -> String {
    use heck::ToShoutySnakeCase;
    s.to_shouty_snake_case()
}

/// Convert a state name to snake_case for the Enum value string.
fn to_snake(s: &str) -> String {
    use heck::ToSnakeCase;
    s.to_snake_case()
}

fn generate_state_machine_python(def: &crate::generate::diagrams::StateMachineDef) -> String {
    let resource_name = to_pascal_case(&def.id);

    // Collect all state names (sorted for deterministic output)
    let mut state_names: Vec<&String> = def.states.keys().collect();
    state_names.sort();

    // Build enum members
    let enum_members: Vec<String> = state_names
        .iter()
        .map(|name| format!("    {} = \"{}\"", to_upper_snake(name), to_snake(name)))
        .collect();

    // Build transition match arms by iterating states and their `on` handlers
    let mut match_arms: Vec<String> = Vec::new();
    let mut sorted_states: Vec<(&String, &crate::generate::diagrams::StateNodeDef)> =
        def.states.iter().collect();
    sorted_states.sort_by_key(|(k, _)| *k);

    for (state_name, state_def) in &sorted_states {
        if let Some(on_map) = &state_def.on {
            let mut events: Vec<(&String, &crate::generate::diagrams::TransitionInput)> =
                on_map.iter().collect();
            events.sort_by_key(|(k, _)| *k);

            for (event, transition) in events {
                let target = match transition {
                    crate::generate::diagrams::TransitionInput::Simple(t) => Some(t.clone()),
                    crate::generate::diagrams::TransitionInput::Detailed(d) => d.target.clone(),
                    crate::generate::diagrams::TransitionInput::Conditional(conds) => {
                        conds.first().and_then(|c| c.target.clone())
                    }
                };

                if let Some(target_state) = target {
                    match_arms.push(format!(
                        "        case ({class}State.{from_upper}, \"{event}\"):\n            return {class}State.{to_upper}",
                        class = resource_name,
                        from_upper = to_upper_snake(state_name),
                        event = event,
                        to_upper = to_upper_snake(&target_state),
                    ));
                }
            }
        }
    }

    let match_body = if match_arms.is_empty() {
        format!(
            "        case _:\n            raise ValueError(f\"Invalid transition: {{current}} + {{event}}\")"
        )
    } else {
        let mut body = match_arms.join("\n");
        body.push_str(&format!(
            "\n        case _:\n            raise ValueError(f\"Invalid transition: {{current}} + {{event}}\")"
        ));
        body
    };

    format!(
        r#"# Generated by sdd
from enum import Enum


class {class}State(str, Enum):
{members}


def transition(current: {class}State, event: str) -> {class}State:
    match (current, event):
{match_body}
"#,
        class = resource_name,
        members = enum_members.join("\n"),
        match_body = match_body,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::diagrams::{StateMachineDef, StateNodeDef, TransitionInput};
    use crate::generate::spec_ir::{SpecIR, SpecMetadata};
    use std::collections::HashMap;

    fn toggle_spec() -> SpecIR {
        let mut states = HashMap::new();
        let mut off_on = HashMap::new();
        off_on.insert(
            "TOGGLE".to_string(),
            TransitionInput::Simple("on".to_string()),
        );
        states.insert(
            "off".to_string(),
            StateNodeDef {
                on: Some(off_on),
                ..Default::default()
            },
        );
        let mut on_on = HashMap::new();
        on_on.insert(
            "TOGGLE".to_string(),
            TransitionInput::Simple("off".to_string()),
        );
        states.insert(
            "on".to_string(),
            StateNodeDef {
                on: Some(on_on),
                ..Default::default()
            },
        );

        SpecIR::StateMachinePlus {
            def: StateMachineDef {
                id: "toggle".to_string(),
                initial: "off".to_string(),
                states,
                guards: HashMap::new(),
                actions: HashMap::new(),
                description: None,
            },
            metadata: SpecMetadata::default(),
        }
    }

    #[test]
    fn test_can_generate_state_machine() {
        let gen = StateMachineGenerator::new();
        assert!(gen.can_generate(&toggle_spec()));
    }

    #[test]
    fn test_cannot_generate_non_state_machine() {
        use crate::generate::schema::JsonSchema;
        let gen = StateMachineGenerator::new();
        let api_spec = SpecIR::Api {
            schema: JsonSchema::default(),
            metadata: SpecMetadata::default(),
        };
        assert!(!gen.can_generate(&api_spec));
    }

    #[test]
    fn test_generate_produces_one_file() {
        let spec = toggle_spec();
        let settings = GeneratorSettings {
            output_dir: std::path::PathBuf::from("/tmp/test_sm_gen"),
            ..Default::default()
        };
        let engine = crate::generate::engine::TemplateEngine::empty();
        let gen = StateMachineGenerator::new();
        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();

        assert_eq!(manifest.files.len(), 1);
        let name = manifest
            .files
            .keys()
            .next()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(name, "toggle_states.py");
    }

    #[test]
    fn test_generated_python_content() {
        let def = match &toggle_spec() {
            SpecIR::StateMachinePlus { def, .. } => def.clone(),
            _ => unreachable!(),
        };
        let content = generate_state_machine_python(&def);

        assert!(content.contains("from enum import Enum"));
        assert!(content.contains("class ToggleState(str, Enum):"));
        assert!(content.contains("OFF = \"off\""));
        assert!(content.contains("ON = \"on\""));
        assert!(
            content.contains("def transition(current: ToggleState, event: str) -> ToggleState:")
        );
        assert!(content.contains("match (current, event):"));
        assert!(content.contains("ToggleState.OFF, \"TOGGLE\""));
        assert!(content.contains("ToggleState.ON"));
        assert!(content.contains("raise ValueError"));
    }

    #[test]
    fn test_generated_python_empty_transitions() {
        let mut states = HashMap::new();
        states.insert("idle".to_string(), StateNodeDef::default());

        let def = StateMachineDef {
            id: "empty-machine".to_string(),
            initial: "idle".to_string(),
            states,
            guards: HashMap::new(),
            actions: HashMap::new(),
            description: None,
        };
        let content = generate_state_machine_python(&def);

        assert!(content.contains("class EmptyMachineState(str, Enum):"));
        assert!(content.contains("IDLE = \"idle\""));
        assert!(content.contains("raise ValueError"));
    }
}
// CODEGEN-END
