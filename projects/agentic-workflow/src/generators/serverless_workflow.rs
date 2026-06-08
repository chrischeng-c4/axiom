//! Serverless Workflow section generator.
//!
//! Produces a Serverless Workflow 0.8 YAML skeleton with states,
//! transitions, actions, and `x-sdd` metadata injection.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/serverless_workflow_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/serverless_workflow_source.md#source
impl Generator for ServerlessWorkflowGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::ServerlessWorkflow
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let sdd_id = args.sdd_id.as_deref().unwrap_or("TODO");
        let refs_yaml = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            let refs: Vec<String> = args
                .sdd_refs
                .iter()
                .map(|r| format!("    - \"{}\"", r))
                .collect();
            format!("\n    refs:\n{}", refs.join("\n"))
        };

        format!(
            "```yaml\n\
id: \"[workflow-id]\"\n\
version: \"1.0\"\n\
specVersion: \"0.8\"\n\
name: \"[Workflow Name]\"\n\
x-sdd:\n\
  id: \"{sdd_id}\"{refs_yaml}\n\
start: InitState\n\
states:\n\
  - name: InitState\n\
    type: operation\n\
    actions:\n\
      - functionRef: initFunction\n\
    transition: ProcessState\n\
  - name: ProcessState\n\
    type: operation\n\
    actions:\n\
      - functionRef: processFunction\n\
    transition: DecisionState\n\
  - name: DecisionState\n\
    type: switch\n\
    dataConditions:\n\
      - condition: \"${{ .success == true }}\"\n\
        transition: EndState\n\
    defaultCondition:\n\
      transition: ErrorState\n\
  - name: EndState\n\
    type: end\n\
  - name: ErrorState\n\
    type: end\n\
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
            ServerlessWorkflowGenerator {}.section_type(),
            SectionType::ServerlessWorkflow
        );
    }

    #[test]
    fn test_generate_contains_workflow() {
        let args = GeneratorArgs::new(SectionType::ServerlessWorkflow).with_sdd_id("my-wf");
        let output = ServerlessWorkflowGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("specVersion: \"0.8\""));
        assert!(output.contains("x-sdd:"));
        assert!(output.contains("id: \"my-wf\""));
        assert!(output.contains("states:"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::ServerlessWorkflow)
            .with_sdd_id("my-wf")
            .with_sdd_refs(vec!["logic-spec".to_string()]);
        let output = ServerlessWorkflowGenerator {}.generate(&args);
        assert!(output.contains("\"logic-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::ServerlessWorkflow);
        let output = ServerlessWorkflowGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: serverless_workflow lang: yaml -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/serverless_workflow_types.md#schema
// CODEGEN-BEGIN
/// ServerlessWorkflowGenerator unit struct (registered in generators/mod.rs).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/serverless_workflow_types.md#schema
pub struct ServerlessWorkflowGenerator {}
// CODEGEN-END
