//! Unit-test section generator.
//!
//! Produces a Mermaid Plus requirementDiagram skeleton with
//! requirements, test scenarios, and verification links.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

/// Generator for `<!-- type: unit-test lang: mermaid -->` sections.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/test_plan_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_plan_source.md#source
impl Generator for TestPlanGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::UnitTest
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
requirementDiagram\n\
\n\
    requirement R1 {{\n\
        id: 1\n\
        text: [Requirement description]\n\
        risk: medium\n\
        verifymethod: test\n\
    }}\n\
\n\
    functionalRequirement R2 {{\n\
        id: 2\n\
        text: [Requirement description]\n\
        risk: low\n\
        verifymethod: test\n\
    }}\n\
\n\
    element TestCase1 {{\n\
        type: test\n\
        docref: [test file path]\n\
    }}\n\
\n\
    TestCase1 - verifies -> R1\n\
    TestCase1 - verifies -> R2\n\
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
        assert_eq!(TestPlanGenerator {}.section_type(), SectionType::UnitTest);
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::UnitTest).with_sdd_id("my-plan");
        let output = TestPlanGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-plan"));
        assert!(output.contains("requirementDiagram"));
        assert!(output.contains("verifies"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::UnitTest)
            .with_sdd_id("my-plan")
            .with_sdd_refs(vec!["req-spec".to_string()]);
        let output = TestPlanGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"req-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::UnitTest);
        let output = TestPlanGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: unit-test lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/test_plan_types.md#schema
// CODEGEN-BEGIN
/// TestPlanGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/test_plan_types.md#schema
pub struct TestPlanGenerator {}
// CODEGEN-END
