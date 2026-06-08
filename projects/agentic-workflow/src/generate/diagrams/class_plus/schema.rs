//! Class+ definition schema
//!
//! Structured class diagram definitions with OOP semantics.

use std::collections::HashMap;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Attribute definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDef {
    /// Attribute name.
    pub name: String,
    /// Attribute type.
    #[serde(rename = "type")]
    pub attr_type: String,
    /// Visibility.
    #[serde(default)]
    pub visibility: Visibility,
    /// Is static.
    #[serde(default)]
    pub is_static: bool,
    /// Default value.
    #[serde(default)]
    pub default_value: Option<String>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Class definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDef {
    /// Display name (optional, defaults to key).
    #[serde(default)]
    pub name: Option<String>,
    /// Class stereotype.
    #[serde(default)]
    pub stereotype: Option<ClassStereotype>,
    /// Attributes.
    #[serde(default)]
    pub attributes: Vec<AttributeDef>,
    /// Methods.
    #[serde(default)]
    pub methods: Vec<MethodDef>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Class diagram definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDiagramDef {
    /// Diagram identifier.
    pub id: String,
    /// Class definitions keyed by class name.
    pub classes: HashMap<String, ClassDef>,
    /// Relationships between classes.
    #[serde(default)]
    pub relationships: Vec<RelationshipDef>,
    /// Namespace/package groupings.
    #[serde(default)]
    pub namespaces: Vec<NamespaceDef>,
    /// Diagram description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Class stereotype.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClassStereotype {
    Interface,
    Abstract,
    Enumeration,
    Service,
    Entity,
    ValueObject,
    Aggregate,
}

/// Method definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDef {
    /// Method name.
    pub name: String,
    /// Parameters.
    #[serde(default)]
    pub parameters: Vec<ParameterDef>,
    /// Return type.
    #[serde(default)]
    pub return_type: Option<String>,
    /// Visibility.
    #[serde(default)]
    pub visibility: Visibility,
    /// Is static.
    #[serde(default)]
    pub is_static: bool,
    /// Is abstract.
    #[serde(default)]
    pub is_abstract: bool,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Namespace definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceDef {
    /// Namespace name.
    pub name: String,
    /// Classes in this namespace.
    pub classes: Vec<String>,
}

/// Parameter definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    #[serde(rename = "type")]
    pub param_type: String,
}

/// Relationship definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDef {
    /// Source class.
    pub from: String,
    /// Target class.
    pub to: String,
    /// Relationship type.
    #[serde(rename = "type")]
    pub rel_type: RelationshipType,
    /// Relationship label.
    #[serde(default)]
    pub label: Option<String>,
    /// Source multiplicity.
    #[serde(default)]
    pub from_multiplicity: Option<String>,
    /// Target multiplicity.
    #[serde(default)]
    pub to_multiplicity: Option<String>,
}

/// Relationship type between two classes.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipType {
    Inheritance,
    Composition,
    Aggregation,
    Association,
    Dependency,
    Realization,
}

/// Visibility modifier.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Public visibility (default).
    #[default]
    Public,
    /// Private visibility.
    Private,
    /// Protected visibility.
    Protected,
    /// Package-private visibility.
    Package,
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_class_diagram() {
        let json = json!({
            "id": "domain-model",
            "classes": {
                "User": {
                    "stereotype": "entity",
                    "attributes": [
                        { "name": "id", "type": "UUID", "visibility": "private" },
                        { "name": "email", "type": "String" }
                    ],
                    "methods": [
                        { "name": "validate", "return_type": "bool" }
                    ]
                },
                "Order": {
                    "attributes": [
                        { "name": "id", "type": "UUID" },
                        { "name": "total", "type": "Decimal" }
                    ]
                }
            },
            "relationships": [
                { "from": "Order", "to": "User", "type": "association", "label": "belongs to" }
            ]
        });

        let diagram: ClassDiagramDef = serde_json::from_value(json).unwrap();
        assert_eq!(diagram.classes.len(), 2);
        assert_eq!(diagram.relationships.len(), 1);
    }

    #[test]
    fn test_parse_with_namespaces() {
        let json = json!({
            "id": "layered",
            "classes": {
                "UserService": { "stereotype": "service" },
                "UserRepository": { "stereotype": "interface" },
                "User": { "stereotype": "entity" }
            },
            "namespaces": [
                { "name": "application", "classes": ["UserService"] },
                { "name": "domain", "classes": ["User", "UserRepository"] }
            ]
        });

        let diagram: ClassDiagramDef = serde_json::from_value(json).unwrap();
        assert_eq!(diagram.namespaces.len(), 2);
    }
}
