---
id: sdd-generate-erd
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ERD Diagram

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/erd.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Cardinality` | projects/agentic-workflow/src/generate/diagrams/erd.rs | enum | pub | 61 |  |
| `Entity` | projects/agentic-workflow/src/generate/diagrams/erd.rs | struct | pub | 49 |  |
| `EntityAttribute` | projects/agentic-workflow/src/generate/diagrams/erd.rs | struct | pub | 29 |  |
| `ErdInput` | projects/agentic-workflow/src/generate/diagrams/erd.rs | struct | pub | 92 |  |
| `ErdRelationship` | projects/agentic-workflow/src/generate/diagrams/erd.rs | struct | pub | 75 |  |
| `KeyType` | projects/agentic-workflow/src/generate/diagrams/erd.rs | enum | pub | 14 |  |
| `generate_erd` | projects/agentic-workflow/src/generate/diagrams/erd.rs | function | pub | 101 | generate_erd(input: &ErdInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  KeyType:
    type: string
    enum: [PK, FK, UK]
    description: Attribute key type.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      variants:
        - { name: PK, doc: "Primary key." }
        - { name: FK, doc: "Foreign key." }
        - { name: UK, doc: "Unique key." }

  EntityAttribute:
    type: object
    required: [name, attr_type, nullable]
    description: Entity attribute.
    properties:
      name:
        type: string
        description: "Attribute name."
      attr_type:
        type: string
        x-serde-rename: "type"
        description: "Data type (mapped to JSON key 'type' — Rust reserved word)."
      key:
        $ref: "#/definitions/KeyType"
        description: "Key type."
        x-serde-default: true
      nullable:
        type: boolean
        description: "Is nullable."
        x-serde-default: true
      comment:
        type: string
        description: "Comment/description."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Entity:
    type: object
    required: [name, attributes]
    description: Entity definition.
    properties:
      name:
        type: string
        description: "Entity name."
      attributes:
        type: array
        items:
          $ref: "#/definitions/EntityAttribute"
        description: "Entity attributes."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Cardinality:
    type: string
    enum:
      - OneToOne
      - OneToMany
      - ManyToOne
      - ManyToMany
      - OneOrMoreToOne
      - OneToOneOrMore
      - ZeroOrOneToOne
      - OneToZeroOrOne
    description: Relationship cardinality between two entities.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: kebab-case

  ErdRelationship:
    type: object
    required: [from, to, cardinality, identifying]
    description: Entity relationship.
    properties:
      from:
        type: string
        description: "Source entity."
      to:
        type: string
        description: "Target entity."
      cardinality:
        $ref: "#/definitions/Cardinality"
      label:
        type: string
        description: "Relationship label."
        x-serde-default: true
      identifying:
        type: boolean
        description: "Is identifying relationship."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ErdInput:
    type: object
    required: [entities, relationships]
    description: Input for ERD generation.
    properties:
      entities:
        type: array
        items:
          $ref: "#/definitions/Entity"
        description: "Entity definitions."
      relationships:
        type: array
        items:
          $ref: "#/definitions/ErdRelationship"
        description: "Relationships between entities."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/erd.rs -->
```rust
//! Entity Relationship Diagram Generation
//!
//! Generates Mermaid ER diagrams for database design and data modeling.

use crate::generate::{GenerateError, Result};

use serde::{Deserialize, Serialize};

/// Attribute key type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    /// Primary key.
    #[serde(rename = "PK")]
    PK,
    /// Foreign key.
    #[serde(rename = "FK")]
    FK,
    /// Unique key.
    #[serde(rename = "UK")]
    UK,
}

/// Entity attribute.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttribute {
    /// Attribute name.
    pub name: String,
    /// Data type (mapped to JSON key 'type' — Rust reserved word).
    #[serde(rename = "type")]
    pub attr_type: String,
    /// Key type.
    #[serde(default)]
    pub key: Option<KeyType>,
    /// Is nullable.
    #[serde(default)]
    pub nullable: bool,
    /// Comment/description.
    #[serde(default)]
    pub comment: Option<String>,
}

/// Entity definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity name.
    pub name: String,
    /// Entity attributes.
    #[serde(default)]
    pub attributes: Vec<EntityAttribute>,
}

/// Relationship cardinality between two entities.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
    OneOrMoreToOne,
    OneToOneOrMore,
    ZeroOrOneToOne,
    OneToZeroOrOne,
}

/// Entity relationship.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErdRelationship {
    /// Source entity.
    pub from: String,
    /// Target entity.
    pub to: String,
    pub cardinality: Cardinality,
    /// Relationship label.
    #[serde(default)]
    pub label: Option<String>,
    /// Is identifying relationship.
    #[serde(default)]
    pub identifying: bool,
}

/// Input for ERD generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErdInput {
    /// Entity definitions.
    pub entities: Vec<Entity>,
    /// Relationships between entities.
    #[serde(default)]
    pub relationships: Vec<ErdRelationship>,
}
/// Generate a Mermaid ER diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#source
pub fn generate_erd(input: &ErdInput) -> Result<String> {
    if input.entities.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one entity required".to_string(),
        ));
    }

    let mut mermaid = String::new();
    mermaid.push_str("erDiagram\n");

    // Generate entities
    for entity in &input.entities {
        mermaid.push_str(&format!("    {} {{\n", entity.name));
        for attr in &entity.attributes {
            let key_str = attr.key.as_ref().map_or("", |k| match k {
                KeyType::PK => " PK",
                KeyType::FK => " FK",
                KeyType::UK => " UK",
            });
            let comment = attr
                .comment
                .as_ref()
                .map_or(String::new(), |c| format!(" \"{}\"", c));
            mermaid.push_str(&format!(
                "        {}{} {}{}\n",
                attr.attr_type, key_str, attr.name, comment
            ));
        }
        mermaid.push_str("    }\n");
    }

    // Generate relationships
    for rel in &input.relationships {
        let (left, right) = cardinality_to_symbols(&rel.cardinality, rel.identifying);
        let label = rel.label.as_ref().map_or("", |l| l.as_str());
        mermaid.push_str(&format!(
            "    {} {}--{} {} : {}\n",
            rel.from, left, right, rel.to, label
        ));
    }

    Ok(mermaid)
}

fn cardinality_to_symbols(
    cardinality: &Cardinality,
    _identifying: bool,
) -> (&'static str, &'static str) {
    // Note: identifying relationships could use different symbols in a future enhancement
    match cardinality {
        Cardinality::OneToOne => ("||", "||"),
        Cardinality::OneToMany => ("||", "o{"),
        Cardinality::ManyToOne => ("}o", "||"),
        Cardinality::ManyToMany => ("}o", "o{"),
        Cardinality::OneOrMoreToOne => ("|{", "||"),
        Cardinality::OneToOneOrMore => ("||", "}|"),
        Cardinality::ZeroOrOneToOne => ("|o", "||"),
        Cardinality::OneToZeroOrOne => ("||", "o|"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_erd() {
        let input = ErdInput {
            entities: vec![
                Entity {
                    name: "User".to_string(),
                    attributes: vec![
                        EntityAttribute {
                            name: "id".to_string(),
                            attr_type: "int".to_string(),
                            key: Some(KeyType::PK),
                            nullable: false,
                            comment: None,
                        },
                        EntityAttribute {
                            name: "email".to_string(),
                            attr_type: "string".to_string(),
                            key: None,
                            nullable: false,
                            comment: None,
                        },
                    ],
                },
                Entity {
                    name: "Order".to_string(),
                    attributes: vec![
                        EntityAttribute {
                            name: "id".to_string(),
                            attr_type: "int".to_string(),
                            key: Some(KeyType::PK),
                            nullable: false,
                            comment: None,
                        },
                        EntityAttribute {
                            name: "user_id".to_string(),
                            attr_type: "int".to_string(),
                            key: Some(KeyType::FK),
                            nullable: false,
                            comment: None,
                        },
                    ],
                },
            ],
            relationships: vec![ErdRelationship {
                from: "User".to_string(),
                to: "Order".to_string(),
                cardinality: Cardinality::OneToMany,
                label: Some("places".to_string()),
                identifying: false,
            }],
        };

        let result = generate_erd(&input).unwrap();
        assert!(result.contains("erDiagram"));
        assert!(result.contains("User {"));
        assert!(result.contains("int PK id"));
        assert!(result.contains("Order {"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/erd.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete ERD module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] All six types correctly defined with required serde annotations: `x-serde-rename: "type"` on `EntityAttribute.attr_type`, `serde_rename_all: kebab-case` on `Cardinality`, bare-variant serialisation on `KeyType`, and `x-serde-default: true` on all Vec/Option fields — matches R2 exactly.
- [changes] Two-entry changes section correctly separates `impl_mode: codegen` (all 6 symbols) from `impl_mode: hand-written` (module docstring, use statements, `generate_erd`, `cardinality_to_symbols`, tests) — satisfies R4 and R5.
- [overview] Overview accurately enumerates all six types with their serde shapes and calls out the `x-serde-rename` mapping for the reserved-word field — clear and unambiguous.
