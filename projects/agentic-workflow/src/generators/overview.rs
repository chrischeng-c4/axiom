//! Overview section generator.
//!
//! Produces a markdown prose skeleton for the `## Overview` section.
//! No code fence — pure markdown text.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/overview_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/overview_source.md#source
impl Generator for OverviewGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Overview
    }

    fn generate(&self, args: &GeneratorArgs) -> String {
        let context_hint = match &args.sdd_id {
            Some(id) => format!("\nChange: `{}`\n", id),
            None => String::new(),
        };

        let refs_hint = if args.sdd_refs.is_empty() {
            String::new()
        } else {
            format!("\nRelated: {}\n", args.sdd_refs.join(", "))
        };

        format!(
            "{}{}\
[Describe what this spec covers and why it exists. \
Include the scope, motivation, and high-level approach. \
Minimum 50 characters.]",
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
        assert_eq!(OverviewGenerator {}.section_type(), SectionType::Overview);
    }

    #[test]
    fn test_generate_no_args() {
        let args = GeneratorArgs::new(SectionType::Overview);
        let output = OverviewGenerator {}.generate(&args);
        assert!(output.contains("[Describe what this spec covers"));
    }

    #[test]
    fn test_generate_with_sdd_id() {
        let args = GeneratorArgs::new(SectionType::Overview).with_sdd_id("my-feature");
        let output = OverviewGenerator {}.generate(&args);
        assert!(output.contains("my-feature"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Overview)
            .with_sdd_refs(vec!["api-spec".to_string(), "data-model".to_string()]);
        let output = OverviewGenerator {}.generate(&args);
        assert!(output.contains("api-spec"));
        assert!(output.contains("data-model"));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Overview);
        let output = OverviewGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: overview lang: markdown -->"));
        assert!(output.contains("[Describe what this spec covers"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/overview_types.md#schema
// CODEGEN-BEGIN
/// OverviewGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/overview_types.md#schema
pub struct OverviewGenerator {}
// CODEGEN-END
