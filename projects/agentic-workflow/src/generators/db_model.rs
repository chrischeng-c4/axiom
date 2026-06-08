//! DB model section generator.
//!
//! Produces a Mermaid Plus erDiagram skeleton with entities, fields,
//! and relations. Used for database and entity-relationship models.

use super::{Generator, GeneratorArgs};
use crate::models::spec_rules::SectionType;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/db_model_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/db_model_source.md#source
impl Generator for DbModelGenerator {
    fn section_type(&self) -> SectionType {
        SectionType::DbModel
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
erDiagram\n\
    ENTITY_A {{\n\
        string id PK\n\
        string name\n\
        datetime created_at\n\
    }}\n\
    ENTITY_B {{\n\
        string id PK\n\
        string entity_a_id FK\n\
        string value\n\
    }}\n\
    ENTITY_A ||--o{{ ENTITY_B : has\n\
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
        assert_eq!(DbModelGenerator {}.section_type(), SectionType::DbModel);
    }

    #[test]
    fn test_generate_contains_mermaid_plus() {
        let args = GeneratorArgs::new(SectionType::DbModel).with_sdd_id("my-model");
        let output = DbModelGenerator {}.generate(&args);
        assert!(output.contains("```mermaid"));
        assert!(output.contains("id: my-model"));
        assert!(output.contains("erDiagram"));
        assert!(output.contains("ENTITY_A"));
    }

    #[test]
    fn test_generate_with_refs() {
        let args = GeneratorArgs::new(SectionType::DbModel)
            .with_sdd_id("my-model")
            .with_sdd_refs(vec!["data-spec".to_string()]);
        let output = DbModelGenerator {}.generate(&args);
        assert!(output.contains("$ref: \"data-spec\""));
    }

    #[test]
    fn test_with_annotation() {
        let args = GeneratorArgs::new(SectionType::DbModel);
        let output = DbModelGenerator {}.generate_with_annotation(&args);
        assert!(output.starts_with("<!-- type: db-model lang: mermaid -->"));
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/db_model_types.md#schema
// CODEGEN-BEGIN
/// DbModelGenerator unit struct (registered in generators/mod.rs).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/db_model_types.md#schema
pub struct DbModelGenerator {}
// CODEGEN-END
