//! Doc section generator.
//!
//! Produces a markdown documentation skeleton for the `## Doc` section.
//! User-facing documentation describing what changed, how to use it,
//! and examples.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/doc_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/doc_source.md#source
impl Generator for DocGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Doc
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
### What Changed\n\
[Describe the change from a user perspective]\n\
\n\
### How to Use\n\
[Usage instructions with examples]\n\
\n\
### Examples\n\
[Concrete usage examples]",
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
        assert_eq!(DocGenerator {}.section_type(), SectionType::Doc);
    }

    #[test]
    fn test_generate_contains_structure() {
        let args = GeneratorArgs::new(SectionType::Doc);
        let output = DocGenerator {}.generate(&args);
        assert!(output.contains("### What Changed"));
        assert!(output.contains("### How to Use"));
        assert!(output.contains("### Examples"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Doc).with_sdd_id("my-feature");
        let output = DocGenerator {}.generate(&args);
        assert!(output.contains("sdd-id: my-feature"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Doc);
        let output = DocGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: doc lang: markdown -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/doc_types.md#schema
// CODEGEN-BEGIN
/// DocGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/doc_types.md#schema
pub struct DocGenerator {}
// CODEGEN-END
