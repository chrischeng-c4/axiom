//! Flowchart section generator.
//!
//! Produces a Mermaid Plus flowchart skeleton with nodes, edges,
//! and conditions. Used for decision logic and data flow diagrams.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/flowchart_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/flowchart_source.md#source
impl Generator for FlowchartGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Logic
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
flowchart TD\n\
    A[Start] --> B{{Decision?}}\n\
    B -->|Yes| C[Action A]\n\
    B -->|No| D[Action B]\n\
    C --> E[End]\n\
    D --> E\n\
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
        assert_eq!(FlowchartGenerator {}.section_type(), SectionType::Logic);
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::Logic).with_sdd_id("my-flow");
        let output = FlowchartGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-flow"));
        assert!(output.contains("flowchart TD"));
        assert!(output.contains("Decision?"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Logic)
            .with_sdd_id("my-flow")
            .with_sdd_refs(vec!["spec-a".to_string()]);
        let output = FlowchartGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"spec-a\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Logic);
        let output = FlowchartGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: logic lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/flowchart_types.md#schema
// CODEGEN-BEGIN
/// FlowchartGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/flowchart_types.md#schema
pub struct FlowchartGenerator {}
// CODEGEN-END
