//! State machine section generator.
//!
//! Produces a Mermaid Plus stateDiagram-v2 skeleton with states
//! and transitions. Used for finite state machine lifecycle models.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/state_machine_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/state_machine_source.md#source
impl Generator for StateMachineGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::StateMachine
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let sdd_id = args.sdd_id.as_deref().unwrap_or("TODO");
        let refs_yaml = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            let refs: Vec<String> = args
                .sdd_refs
                .iter()
                .map(|r| format!("  - $ref: \"{}\"", r))
                .collect();
            format!("\nrefs:\n{}", refs.join("\n"))
        };

        format!(
            "```mermaid\n\
---\n\
id: {sdd_id}{refs_yaml}\n\
---\n\
stateDiagram-v2\n\
    [*] --> Idle\n\
    Idle --> Processing : start\n\
    Processing --> Done : complete\n\
    Processing --> Error : fail\n\
    Error --> Idle : reset\n\
    Done --> [*]\n\
```"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::{Generator, GeneratorArgs};
    use crate::models::spec_rules::SectionType;

    #[test]
    fn test_section_type() {
        assert_eq!(
            StateMachineGenerator {}.section_type(),
            SectionType::StateMachine
        );
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::StateMachine).with_sdd_id("my-fsm");
        let output = StateMachineGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-fsm"));
        assert!(output.contains("stateDiagram-v2"));
        assert!(output.contains("[*]"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::StateMachine)
            .with_sdd_id("my-fsm")
            .with_sdd_refs(vec!["logic-spec".to_string()]);
        let output = StateMachineGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"logic-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::StateMachine);
        let output = StateMachineGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: state-machine lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/state_machine_types.md#schema
// CODEGEN-BEGIN
/// StateMachineGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/state_machine_types.md#schema
pub struct StateMachineGenerator {}
// CODEGEN-END
