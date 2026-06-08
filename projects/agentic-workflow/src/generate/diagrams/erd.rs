// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/erd.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
