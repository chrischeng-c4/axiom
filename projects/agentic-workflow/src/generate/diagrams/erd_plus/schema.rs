//! ERD+ definition schema

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Relationship cardinality between two entities.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Attribute definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERDAttributeDef {
    /// Attribute name.
    pub name: String,
    /// Data type (mapped to JSON key 'type' — Rust reserved word).
    #[serde(rename = "type")]
    pub data_type: String,
    /// Key type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<KeyType>,
    /// Is nullable.
    #[serde(default, skip_serializing_if = "is_false")]
    pub nullable: bool,
    /// Foreign key reference (entity.attribute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub references: Option<String>,
    /// Comment/description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// ERD definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERDDef {
    /// Diagram identifier.
    pub id: String,
    /// Entity definitions keyed by entity name.
    pub entities: indexmap::IndexMap<String, EntityDef>,
    /// Relationships between entities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<ERDRelationshipDef>,
    /// Diagram description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Relationship definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERDRelationshipDef {
    /// Source entity.
    pub from: String,
    /// Target entity.
    pub to: String,
    pub cardinality: Cardinality,
    /// Relationship label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Is identifying relationship.
    #[serde(default, skip_serializing_if = "is_false")]
    pub identifying: bool,
}

/// Entity definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDef {
    /// Display name (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Attributes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<ERDAttributeDef>,
    /// Description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Attribute key type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
// CODEGEN-END
fn is_false(v: &bool) -> bool {
    !v
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_erd() {
        let json = json!({
            "id": "ecommerce",
            "entities": {
                "User": {
                    "attributes": [
                        { "name": "id", "type": "UUID", "key": "PK" },
                        { "name": "email", "type": "VARCHAR(255)" }
                    ]
                },
                "Order": {
                    "attributes": [
                        { "name": "id", "type": "UUID", "key": "PK" },
                        { "name": "user_id", "type": "UUID", "key": "FK", "references": "User.id" }
                    ]
                }
            },
            "relationships": [
                { "from": "User", "to": "Order", "cardinality": "one-to-many", "label": "places" }
            ]
        });

        let erd: ERDDef = serde_json::from_value(json).unwrap();
        assert_eq!(erd.entities.len(), 2);
        assert_eq!(erd.relationships.len(), 1);
    }
}
