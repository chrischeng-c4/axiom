// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#source
// CODEGEN-BEGIN
//! Class Diagram Generation
//!
//! Generates Mermaid class diagrams for data structures and domain models.

use crate::generate::{GenerateError, Result};

use serde::{Deserialize, Serialize};

/// Visibility modifier.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Public visibility (`+`).
    #[default]
    Public,
    /// Private visibility (`-`).
    Private,
    /// Protected visibility (`#`).
    Protected,
    /// Package-private visibility (`~`).
    Package,
}

/// Class stereotype.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stereotype {
    Interface,
    Abstract,
    Enumeration,
    Service,
}

/// One class attribute.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAttribute {
    /// Attribute name.
    pub name: String,
    /// Attribute type. JSON key 'type' (Rust reserved word).
    #[serde(rename = "type")]
    pub attr_type: String,
    /// Visibility (defaults to Public).
    #[serde(default)]
    pub visibility: Visibility,
    /// Marks the attribute as static (`$`).
    #[serde(default)]
    pub static_attr: bool,
}

/// One class method.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMethod {
    /// Method name.
    pub name: String,
    /// Parameter list.
    #[serde(default)]
    pub parameters: Vec<MethodParam>,
    /// Optional return type.
    #[serde(default)]
    pub return_type: Option<String>,
    /// Visibility (defaults to Public).
    #[serde(default)]
    pub visibility: Visibility,
    /// Marks the method as static (`$`).
    #[serde(default)]
    pub static_method: bool,
    /// Marks the method as abstract (`*`).
    #[serde(default)]
    pub abstract_method: bool,
}

/// One method parameter.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodParam {
    /// Parameter name.
    pub name: String,
    /// Parameter type. JSON key 'type' (Rust reserved word).
    #[serde(rename = "type")]
    pub param_type: String,
}

/// One class definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDef {
    /// Class name.
    pub name: String,
    /// Optional class stereotype.
    #[serde(default)]
    pub stereotype: Option<Stereotype>,
    /// Class attributes.
    #[serde(default)]
    pub attributes: Vec<ClassAttribute>,
    /// Class methods.
    #[serde(default)]
    pub methods: Vec<ClassMethod>,
}

/// Class-relationship kind.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipType {
    Inheritance,
    Composition,
    Aggregation,
    Association,
    Dependency,
    Realization,
}

/// One relationship edge between two classes.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassRelationship {
    /// Source class name.
    pub from: String,
    /// Target class name.
    pub to: String,
    /// Edge kind. JSON key 'type' (Rust reserved word).
    #[serde(rename = "type")]
    pub rel_type: RelationshipType,
    /// Optional edge label.
    #[serde(default)]
    pub label: Option<String>,
    /// Optional source-side multiplicity (e.g. '1..*').
    #[serde(default)]
    pub multiplicity_from: Option<String>,
    /// Optional target-side multiplicity (e.g. '0..1').
    #[serde(default)]
    pub multiplicity_to: Option<String>,
}

/// A namespace / package grouping a set of class names.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    /// Namespace name.
    pub name: String,
    /// Class names contained in this namespace.
    pub classes: Vec<String>,
}

/// Input for class-diagram generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInput {
    /// All class definitions (need at least one at runtime).
    pub classes: Vec<ClassDef>,
    /// Class relationships.
    #[serde(default)]
    pub relationships: Vec<ClassRelationship>,
    /// Namespace groupings.
    #[serde(default)]
    pub namespaces: Vec<Namespace>,
}
/// Generate a Mermaid class diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/class.md#source
pub fn generate_class_diagram(input: &ClassInput) -> Result<String> {
    if input.classes.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one class required".to_string(),
        ));
    }

    let mut mermaid = String::new();
    mermaid.push_str("classDiagram\n");

    // Generate namespaces
    for ns in &input.namespaces {
        mermaid.push_str(&format!("    namespace {} {{\n", ns.name));
        for class_name in &ns.classes {
            if let Some(class) = input.classes.iter().find(|c| &c.name == class_name) {
                mermaid.push_str(&format_class(class, "        ")?);
            }
        }
        mermaid.push_str("    }\n");
    }

    // Generate standalone classes
    let ns_classes: std::collections::HashSet<_> = input
        .namespaces
        .iter()
        .flat_map(|ns| ns.classes.iter())
        .collect();

    for class in &input.classes {
        if !ns_classes.contains(&class.name) {
            mermaid.push_str(&format_class(class, "    ")?);
        }
    }

    // Generate relationships
    for rel in &input.relationships {
        mermaid.push_str(&format!("    {}\n", format_relationship(rel)?));
    }

    Ok(mermaid)
}

fn format_class(class: &ClassDef, indent: &str) -> Result<String> {
    let mut output = String::new();

    // Class declaration with stereotype
    if let Some(ref stereotype) = class.stereotype {
        let st = match stereotype {
            Stereotype::Interface => "<<interface>>",
            Stereotype::Abstract => "<<abstract>>",
            Stereotype::Enumeration => "<<enumeration>>",
            Stereotype::Service => "<<service>>",
        };
        output.push_str(&format!("{}class {} {{\n", indent, class.name));
        output.push_str(&format!("{}    {}\n", indent, st));
    } else {
        output.push_str(&format!("{}class {} {{\n", indent, class.name));
    }

    // Attributes
    for attr in &class.attributes {
        let vis = match attr.visibility {
            Visibility::Public => "+",
            Visibility::Private => "-",
            Visibility::Protected => "#",
            Visibility::Package => "~",
        };
        let static_marker = if attr.static_attr { "$" } else { "" };
        output.push_str(&format!(
            "{}    {}{}{} {}\n",
            indent, vis, static_marker, attr.attr_type, attr.name
        ));
    }

    // Methods
    for method in &class.methods {
        let vis = match method.visibility {
            Visibility::Public => "+",
            Visibility::Private => "-",
            Visibility::Protected => "#",
            Visibility::Package => "~",
        };
        let static_marker = if method.static_method { "$" } else { "" };
        let abstract_marker = if method.abstract_method { "*" } else { "" };
        let params: Vec<String> = method
            .parameters
            .iter()
            .map(|p| format!("{} {}", p.param_type, p.name))
            .collect();
        let return_type = method.return_type.as_deref().unwrap_or("void");
        output.push_str(&format!(
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

    output.push_str(&format!("{}}}\n", indent));
    Ok(output)
}

fn format_relationship(rel: &ClassRelationship) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_class_diagram() {
        let input = ClassInput {
            classes: vec![
                ClassDef {
                    name: "Animal".to_string(),
                    stereotype: Some(Stereotype::Abstract),
                    attributes: vec![ClassAttribute {
                        name: "name".to_string(),
                        attr_type: "String".to_string(),
                        visibility: Visibility::Private,
                        static_attr: false,
                    }],
                    methods: vec![ClassMethod {
                        name: "speak".to_string(),
                        parameters: vec![],
                        return_type: Some("void".to_string()),
                        visibility: Visibility::Public,
                        static_method: false,
                        abstract_method: true,
                    }],
                },
                ClassDef {
                    name: "Dog".to_string(),
                    stereotype: None,
                    attributes: vec![],
                    methods: vec![],
                },
            ],
            relationships: vec![ClassRelationship {
                from: "Dog".to_string(),
                to: "Animal".to_string(),
                rel_type: RelationshipType::Inheritance,
                label: None,
                multiplicity_from: None,
                multiplicity_to: None,
            }],
            namespaces: vec![],
        };

        let result = generate_class_diagram(&input).unwrap();
        assert!(result.contains("classDiagram"));
        assert!(result.contains("class Animal"));
        assert!(result.contains("<<abstract>>"));
        assert!(result.contains("Dog <|-- Animal"));
    }
}

// CODEGEN-END
