//! Frontend section generator.
//!
//! Produces a wireframe YAML DSL skeleton with page, layout,
//! sections, and `_sdd` metadata injection.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/frontend_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/frontend_source.md#source
impl Generator for FrontendGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Wireframe
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
_sdd:\n\
  id: \"{sdd_id}\"{refs_yaml}\n\
page: \"[Page Name]\"\n\
layout: flex-column\n\
sections:\n\
  - name: header\n\
    type: navbar\n\
    components:\n\
      - type: logo\n\
      - type: nav-links\n\
        items:\n\
          - label: \"[Link]\"\n\
            href: \"/path\"\n\
  - name: main\n\
    type: content\n\
    layout: grid-2col\n\
    components:\n\
      - type: card\n\
        title: \"[Title]\"\n\
        content: \"[Description]\"\n\
      - type: form\n\
        fields:\n\
          - name: \"[field]\"\n\
            type: text\n\
            required: true\n\
  - name: footer\n\
    type: footer\n\
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
        assert_eq!(FrontendGenerator {}.section_type(), SectionType::Wireframe);
    }

    #[test]
    fn test_generate_contains_wireframe() {
        let args = GeneratorArgs::new(SectionType::Wireframe).with_sdd_id("my-ui");
        let output = FrontendGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("_sdd:"));
        assert!(output.contains("id: \"my-ui\""));
        assert!(output.contains("page:"));
        assert!(output.contains("sections:"));
        assert!(output.contains("layout:"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Wireframe)
            .with_sdd_id("my-ui")
            .with_sdd_refs(vec!["api-spec".to_string()]);
        let output = FrontendGenerator {}.generate(&args);
        assert!(output.contains("\"api-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Wireframe);
        let output = FrontendGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: wireframe lang: yaml -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/frontend_types.md#schema
// CODEGEN-BEGIN
/// FrontendGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/frontend_types.md#schema
pub struct FrontendGenerator {}
// CODEGEN-END
