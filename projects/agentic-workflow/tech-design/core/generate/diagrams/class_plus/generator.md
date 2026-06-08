---
id: sdd-class_plus-generator-output
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ClassPlusOutput

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ClassPlusGenerator` | projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs | struct | pub | 20 |  |
| `ClassPlusOutput` | projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs | struct | pub | 13 |  |
| `generate` | projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs | function | pub | 28 | generate(         &self,         diagram: &ClassDiagramDef,         validation: ClassValidationResult,     ) -> Result<ClassPlusOutput, String> |
| `generate_mermaid` | projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs | function | pub | 66 | generate_mermaid(&self, diagram: &ClassDiagramDef) -> Result<String, String> |
| `new` | projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs | function | pub | 24 | new() -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ClassPlusOutput:
    type: object
    required: [frontmatter, diagram, validation, combined]
    description: Output of the Mermaid Plus generator.
    properties:
      frontmatter: { type: string }
      diagram: { type: string }
      validation:
        type: object
        x-rust-type: ClassValidationResult
      combined: { type: string }
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs -->
````rust
//! Class+ generator

use super::schema::{
    ClassDef, ClassDiagramDef, ClassStereotype, RelationshipDef, RelationshipType, Visibility,
};
use super::validator::ClassValidationResult;

/// Output of the Mermaid Plus generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/generator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClassPlusOutput {
    pub frontmatter: String,
    pub diagram: String,
    pub validation: ClassValidationResult,
    pub combined: String,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/generator.md#source
pub struct ClassPlusGenerator;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/generator.md#source
impl ClassPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        diagram: &ClassDiagramDef,
        validation: ClassValidationResult,
    ) -> Result<ClassPlusOutput, String> {
        let frontmatter = self.generate_frontmatter(diagram)?;
        let mermaid = self.generate_mermaid(diagram)?;

        // Combine into Mermaid+ format (frontmatter inside code block per Mermaid spec)
        let mut combined = String::new();
        combined.push_str("```mermaid\n");
        combined.push_str("---\n");
        combined.push_str(&frontmatter);
        combined.push_str("---\n");
        combined.push_str(&mermaid);
        combined.push_str("```\n");

        if !validation.warnings.is_empty() {
            combined.push_str("\n<!-- Validation Warnings:\n");
            for w in &validation.warnings {
                combined.push_str(&format!("  - {}: {} (at {})\n", w.code, w.message, w.path));
            }
            combined.push_str("-->\n");
        }

        Ok(ClassPlusOutput {
            frontmatter,
            diagram: mermaid,
            validation,
            combined,
        })
    }

    fn generate_frontmatter(&self, diagram: &ClassDiagramDef) -> Result<String, String> {
        let yaml = serde_yaml::to_string(diagram).map_err(|e| format!("YAML error: {}", e))?;
        Ok(yaml.strip_prefix("---\n").unwrap_or(&yaml).to_string())
    }

    pub fn generate_mermaid(&self, diagram: &ClassDiagramDef) -> Result<String, String> {
        let mut mermaid = String::new();
        mermaid.push_str("classDiagram\n");

        // Collect namespace classes
        let ns_classes: std::collections::HashSet<String> = diagram
            .namespaces
            .iter()
            .flat_map(|ns| ns.classes.iter().cloned())
            .collect();

        // Generate namespaces
        for ns in &diagram.namespaces {
            mermaid.push_str(&format!("    namespace {} {{\n", ns.name));
            for class_name in &ns.classes {
                if let Some(class_def) = diagram.classes.get(class_name) {
                    self.format_class(&mut mermaid, class_name, class_def, "        ")?;
                }
            }
            mermaid.push_str("    }\n");
        }

        // Generate standalone classes (sorted)
        let mut standalone: Vec<_> = diagram
            .classes
            .iter()
            .filter(|(name, _)| !ns_classes.contains(*name))
            .collect();
        standalone.sort_by(|a, b| a.0.cmp(b.0));

        for (class_name, class_def) in standalone {
            self.format_class(&mut mermaid, class_name, class_def, "    ")?;
        }

        // Generate relationships
        for rel in &diagram.relationships {
            mermaid.push_str(&format!("    {}\n", self.format_relationship(rel)?));
        }

        Ok(mermaid)
    }

    fn format_class(
        &self,
        mermaid: &mut String,
        name: &str,
        class_def: &ClassDef,
        indent: &str,
    ) -> Result<(), String> {
        mermaid.push_str(&format!("{}class {} {{\n", indent, name));

        // Stereotype
        if let Some(ref stereotype) = class_def.stereotype {
            let st = match stereotype {
                ClassStereotype::Interface => "<<interface>>",
                ClassStereotype::Abstract => "<<abstract>>",
                ClassStereotype::Enumeration => "<<enumeration>>",
                ClassStereotype::Service => "<<service>>",
                ClassStereotype::Entity => "<<entity>>",
                ClassStereotype::ValueObject => "<<valueObject>>",
                ClassStereotype::Aggregate => "<<aggregate>>",
            };
            mermaid.push_str(&format!("{}    {}\n", indent, st));
        }

        // Attributes
        for attr in &class_def.attributes {
            let vis = match attr.visibility {
                Visibility::Public => "+",
                Visibility::Private => "-",
                Visibility::Protected => "#",
                Visibility::Package => "~",
            };
            let static_marker = if attr.is_static { "$" } else { "" };
            mermaid.push_str(&format!(
                "{}    {}{}{} {}\n",
                indent, vis, static_marker, attr.attr_type, attr.name
            ));
        }

        // Methods
        for method in &class_def.methods {
            let vis = match method.visibility {
                Visibility::Public => "+",
                Visibility::Private => "-",
                Visibility::Protected => "#",
                Visibility::Package => "~",
            };
            let static_marker = if method.is_static { "$" } else { "" };
            let abstract_marker = if method.is_abstract { "*" } else { "" };
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| format!("{} {}", p.param_type, p.name))
                .collect();
            let return_type = method.return_type.as_deref().unwrap_or("void");
            mermaid.push_str(&format!(
                "{}    {}{}{}{}({}) {}\n",
                indent,
                vis,
                static_marker,
                abstract_marker,
                method.name,
                params.join(", "),
                return_type
            ));
        }

        mermaid.push_str(&format!("{}}}\n", indent));
        Ok(())
    }

    fn format_relationship(&self, rel: &RelationshipDef) -> Result<String, String> {
        let arrow = match rel.rel_type {
            RelationshipType::Inheritance => "<|--",
            RelationshipType::Composition => "*--",
            RelationshipType::Aggregation => "o--",
            RelationshipType::Association => "-->",
            RelationshipType::Dependency => "..>",
            RelationshipType::Realization => "..|>",
        };
        let mut result = format!("{} {} {}", rel.from, arrow, rel.to);
        if let Some(ref label) = rel.label {
            result = format!("{} : {}", result, label);
        }
        Ok(result)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/generator.md#source
impl Default for ClassPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::ClassValidator;
    use super::*;
    use serde_json::json;

    fn parse_diagram(json: serde_json::Value) -> ClassDiagramDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_class_diagram() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "classes": {
                "Animal": {
                    "stereotype": "abstract",
                    "attributes": [{ "name": "name", "type": "String", "visibility": "private" }],
                    "methods": [{ "name": "speak", "return_type": "void", "is_abstract": true }]
                },
                "Dog": {}
            },
            "relationships": [
                { "from": "Dog", "to": "Animal", "type": "inheritance" }
            ]
        }));

        let validation = ClassValidator::new().validate(&diagram);
        let output = ClassPlusGenerator::new()
            .generate(&diagram, validation)
            .unwrap();

        assert!(output.diagram.contains("classDiagram"));
        assert!(output.diagram.contains("class Animal"));
        assert!(output.diagram.contains("<<abstract>>"));
        assert!(output.diagram.contains("Dog <|-- Animal"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/class_plus/generator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Class+ generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
