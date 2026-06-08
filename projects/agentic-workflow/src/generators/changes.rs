//! Changes section generator.
//!
//! Produces a YAML file list skeleton for the `## Changes` section.
//! Format:
//! ```yaml
//! files:
//!   - path: <file_path>
//!     action: CREATE | MODIFY | DELETE
//!     desc: <description>
//! ```

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/changes_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/changes_source.md#source
impl Generator for ChangesGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Changes
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let context_comment = match &args.sdd_id {
            Some(id) => format!("# Change: {}\n", id),
            None => String::new(),
        };

        let refs_comment = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            format!("# Refs: {}\n", args.sdd_refs.join(", "))
        };

        format!(
            "```yaml\n\
{}{}files:\n\
  - path: <file_path>\n\
    action: CREATE  # CREATE | MODIFY | DELETE\n\
    desc: <description>\n\
  - path: <file_path_2>\n\
    action: MODIFY\n\
    desc: <description>\n\
```",
            context_comment, refs_comment
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
        assert_eq!(ChangesGenerator {}.section_type(), SectionType::Changes);
    }

    #[test]
    fn test_generate_contains_yaml_structure() {
        let args = GeneratorArgs::new(SectionType::Changes);
        let output = ChangesGenerator {}.generate(&args);
        assert!(output.contains("```yaml"));
        assert!(output.contains("files:"));
        assert!(output.contains("action: CREATE"));
        assert!(output.contains("path: <file_path>"));
        assert!(output.contains("```"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Changes).with_sdd_id("my-feature");
        let output = ChangesGenerator {}.generate(&args);
        assert!(output.contains("my-feature"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args =
            GeneratorArgs::new(SectionType::Changes).with_sdd_refs(vec!["api-spec".to_string()]);
        let output = ChangesGenerator {}.generate(&args);
        assert!(output.contains("api-spec"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Changes);
        let output = ChangesGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: changes lang: yaml -->"));
        assert!(output.contains("files:"));
    }

    #[test]
    fn test_yaml_has_all_three_actions_documented() {
        let args = GeneratorArgs::new(SectionType::Changes);
        let output = ChangesGenerator {}.generate(&args);
        // The template should document all three action values
        assert!(output.contains("CREATE"));
        assert!(output.contains("MODIFY"));
        assert!(output.contains("DELETE"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/changes_types.md#schema
// CODEGEN-BEGIN
/// ChangesGenerator unit struct (registered in generators/mod.rs).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/changes_types.md#schema
pub struct ChangesGenerator {}
// CODEGEN-END
