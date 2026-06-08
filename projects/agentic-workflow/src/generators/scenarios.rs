//! Scenarios section generator.
//!
//! Produces a BDD acceptance scenario skeleton for the `## Scenarios` section.
//! Uses `### Scenario:` headings with GIVEN/WHEN/THEN structure.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

/// Generator for `<!-- type: scenarios lang: yaml -->` sections.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/scenarios_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/scenarios_source.md#source
impl Generator for ScenariosGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Scenarios
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let context_hint = match &args.sdd_id {
            Some(id) => format!("<!-- sdd-id: {} -->\n\n", id),
            None => String::new(),
        };

        let refs_hint = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            format!("<!-- sdd-refs: {} -->\n\n", args.sdd_refs.join(", "))
        };

        format!(
            "{}{}\
```yaml\n\
- id: S1\n\
  given: \"[Precondition — system state before action]\"\n\
  when: \"[Action or event that triggers the scenario]\"\n\
  then: \"[Expected observable outcome]\"\n\
\n\
- id: S2\n\
  given: \"[Error precondition]\"\n\
  when: \"[Invalid action or edge case]\"\n\
  then: \"[Expected error behavior]\"\n\
\n\
- id: S3\n\
  given: \"[Edge case precondition]\"\n\
  when: \"[Boundary action]\"\n\
  then: \"[Expected boundary behavior]\"\n\
  diagram_ref: \"[optional: section-anchor for related diagram]\"\n\
```",
            context_hint, refs_hint
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
        assert_eq!(ScenariosGenerator {}.section_type(), SectionType::Scenarios);
    }

    #[test]
    fn test_generate_contains_yaml_gwt_structure() {
        let args = GeneratorArgs::new(SectionType::Scenarios);
        let output = ScenariosGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("- id: S1"));
        assert!(output.contains("given:"));
        assert!(output.contains("when:"));
        assert!(output.contains("then:"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Scenarios).with_sdd_id("my-feature");
        let output = ScenariosGenerator {}.generate(&args);
        assert!(output.contains("sdd-id: my-feature"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Scenarios);
        let output = ScenariosGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: scenarios lang: yaml -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/scenarios_types.md#schema
// CODEGEN-BEGIN
/// ScenariosGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/scenarios_types.md#schema
pub struct ScenariosGenerator {}
// CODEGEN-END
