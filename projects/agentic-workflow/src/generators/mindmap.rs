//! Mindmap section generator.
//!
//! Produces a Mermaid Plus mindmap skeleton for hierarchical
//! decomposition. Used for feature scope, taxonomy, or structural
//! overviews.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/mindmap_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mindmap_source.md#source
impl Generator for MindmapGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Mindmap
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
mindmap\n\
  root((Root Topic))\n\
    Branch A\n\
      Leaf A1\n\
      Leaf A2\n\
    Branch B\n\
      Leaf B1\n\
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
        assert_eq!(MindmapGenerator {}.section_type(), SectionType::Mindmap);
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::Mindmap).with_sdd_id("my-map");
        let output = MindmapGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-map"));
        assert!(output.contains("mindmap"));
        assert!(output.contains("root"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Mindmap)
            .with_sdd_id("my-map")
            .with_sdd_refs(vec!["scope-spec".to_string()]);
        let output = MindmapGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"scope-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Mindmap);
        let output = MindmapGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: mindmap lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/mindmap_types.md#schema
// CODEGEN-BEGIN
/// MindmapGenerator unit struct.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/mindmap_types.md#schema
pub struct MindmapGenerator {}
// CODEGEN-END
