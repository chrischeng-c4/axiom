//! Dependency section generator.
//!
//! Produces a Mermaid Plus classDiagram skeleton with classes
//! and dependency relationships. Used for type hierarchy and
//! composition models.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/dependency_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/dependency_source.md#source
impl Generator for DependencyGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::Dependency
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
classDiagram\n\
    class ComponentA {{\n\
        +method_a()\n\
        +method_b()\n\
    }}\n\
    class ComponentB {{\n\
        +method_c()\n\
    }}\n\
    ComponentA --> ComponentB : depends\n\
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
            DependencyGenerator {}.section_type(),
            SectionType::Dependency
        );
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::Dependency).with_sdd_id("my-deps");
        let output = DependencyGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-deps"));
        assert!(output.contains("classDiagram"));
        assert!(output.contains("depends"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::Dependency)
            .with_sdd_id("my-deps")
            .with_sdd_refs(vec!["arch-spec".to_string()]);
        let output = DependencyGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"arch-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::Dependency);
        let output = DependencyGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: dependency lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/dependency_types.md#schema
// CODEGEN-BEGIN
/// DependencyGenerator unit struct (registered in generators/mod.rs).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/dependency_types.md#schema
pub struct DependencyGenerator {}
// CODEGEN-END
