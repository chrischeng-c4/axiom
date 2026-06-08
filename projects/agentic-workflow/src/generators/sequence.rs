//! Sequence section generator.
//!
//! Produces a Mermaid Plus sequence diagram skeleton with participants
//! and messages. Used for interaction flows between actors/components.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/sequence_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/sequence_source.md#source
impl Generator for SequenceGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Interaction
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
sequenceDiagram\n\
    participant A as Actor A\n\
    participant B as Actor B\n\
    A->>B: Request\n\
    B-->>A: Response\n\
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
            SequenceGenerator {}.section_type(),
            SectionType::Interaction
        );
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::Interaction).with_sdd_id("my-seq");
        let output = SequenceGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-seq"));
        assert!(output.contains("sequenceDiagram"));
        assert!(output.contains("participant"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Interaction)
            .with_sdd_id("my-seq")
            .with_sdd_refs(vec!["api-spec".to_string()]);
        let output = SequenceGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"api-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Interaction);
        let output = SequenceGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: interaction lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/sequence_types.md#schema
// CODEGEN-BEGIN
/// SequenceGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/sequence_types.md#schema
pub struct SequenceGenerator {}
// CODEGEN-END
