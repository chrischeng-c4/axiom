//! Requirements section generator.
//!
//! Produces a markdown skeleton for the `## Requirements` section.
//! Uses formal `### R1:` / `### R2:` headings with priority annotations.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

/// Generator for `<!-- type: requirements lang: mermaid -->` sections.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/requirements_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/requirements_source.md#source
impl Generator for RequirementsGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Requirements
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
### R1: [Requirement Title]\n\
[Description of what must be true]\n\
\n\
**Priority**: high\n\
\n\
### R2: [Requirement Title]\n\
[Description of what must be true]\n\
\n\
**Priority**: medium",
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
        assert_eq!(
            RequirementsGenerator {}.section_type(),
            SectionType::Requirements
        );
    }

    #[test]
    fn test_generate_contains_structure() {
        let args = GeneratorArgs::new(SectionType::Requirements);
        let output = RequirementsGenerator {}.generate(&args);
        assert!(output.contains("### R1:"));
        assert!(output.contains("### R2:"));
        assert!(output.contains("**Priority**:"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Requirements).with_sdd_id("my-feature");
        let output = RequirementsGenerator {}.generate(&args);
        assert!(output.contains("sdd-id: my-feature"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Requirements)
            .with_sdd_refs(vec!["api-spec".to_string()]);
        let output = RequirementsGenerator {}.generate(&args);
        assert!(output.contains("sdd-refs: api-spec"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Requirements);
        let output = RequirementsGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: requirements lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/requirements_types.md#schema
// CODEGEN-BEGIN
/// RequirementsGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/requirements_types.md#schema
pub struct RequirementsGenerator {}
// CODEGEN-END
