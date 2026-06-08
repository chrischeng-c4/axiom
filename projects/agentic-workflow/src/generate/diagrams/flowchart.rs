// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#source
// CODEGEN-BEGIN
//! Flowchart Diagram Generation
//!
//! Generates Mermaid flowchart diagrams for algorithms, business logic, and decision trees.
//! Supports optional semantic fields for spec-to-code generation.

use crate::generate::{GenerateError, Result};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Flow direction (Mermaid `flowchart <dir>` keyword).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum FlowDirection {
    /// Left to Right.
    #[serde(rename = "LR")]
    LR,
    /// Right to Left.
    #[serde(rename = "RL")]
    RL,
    /// Top to Bottom (default).
    #[default]
    #[serde(rename = "TB")]
    TB,
    /// Bottom to Top.
    #[serde(rename = "BT")]
    BT,
}

/// Flowchart node shape.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeShape {
    /// Plain rectangle (default).
    #[default]
    Rectangle,
    /// Rounded rectangle.
    Rounded,
    /// Stadium.
    Stadium,
    /// Subroutine box.
    Subroutine,
    /// Cylinder.
    Cylinder,
    /// Circle.
    Circle,
    /// Diamond / decision.
    Diamond,
    /// Hexagon.
    Hexagon,
    /// Parallelogram (input/output).
    Parallelogram,
    /// Trapezoid.
    Trapezoid,
}

/// Flowchart edge style.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EdgeStyle {
    /// Solid arrow (default).
    #[default]
    Arrow,
    /// Thick arrow.
    Thick,
    /// Dotted arrow.
    Dotted,
}

/// Node semantic kind for spec-to-code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticType {
    Start,
    End,
    Validation,
    Condition,
    DbQuery,
    DbMutation,
    ApiCall,
    Transform,
    Assign,
    Return,
    RaiseError,
}

/// Database operation kind.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DbOperation {
    Select,
    Insert,
    Update,
    Delete,
    Upsert,
}

/// HTTP method.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Input parameter specification.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputSpec {
    /// Parameter name.
    #[serde(default)]
    pub name: Option<String>,
    /// Type annotation. JSON key 'type'.
    #[serde(rename = "type", default)]
    pub type_annotation: Option<String>,
    /// Source expression for the parameter.
    #[serde(default)]
    pub source: Option<String>,
}

/// Output specification.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputSpec {
    /// Output name.
    #[serde(default)]
    pub name: Option<String>,
    /// Type annotation. JSON key 'type'.
    #[serde(rename = "type", default)]
    pub type_annotation: Option<String>,
}

/// Error specification.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorSpec {
    /// Error code.
    #[serde(default)]
    pub code: Option<i32>,
    /// Error message.
    #[serde(default)]
    pub message: Option<String>,
}

/// Node semantic metadata for code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeSemantic {
    /// Semantic kind. JSON key 'type'.
    #[serde(rename = "type", default)]
    pub semantic_type: Option<SemanticType>,
    /// Input parameter.
    #[serde(default)]
    pub input: Option<InputSpec>,
    /// Output specification.
    #[serde(default)]
    pub output: Option<OutputSpec>,
    /// Error specification.
    #[serde(default)]
    pub error: Option<ErrorSpec>,
    /// Database operation kind.
    #[serde(default)]
    pub operation: Option<DbOperation>,
    /// Database table name.
    #[serde(default)]
    pub table: Option<String>,
    /// Database filter expression.
    #[serde(default)]
    pub filter: Option<String>,
    /// API URL.
    #[serde(default)]
    pub url: Option<String>,
    /// HTTP method.
    #[serde(default)]
    pub method: Option<HttpMethod>,
    /// Code pattern hint.
    #[serde(default)]
    pub code_pattern: Option<String>,
}

/// Edge semantic metadata.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EdgeSemantic {
    /// Edge condition expression.
    #[serde(default)]
    pub condition: Option<String>,
    /// Whether this edge marks the error path.
    #[serde(default)]
    pub is_error_path: Option<bool>,
}

/// Flowchart node.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowchartNode {
    /// Unique node identifier.
    pub id: String,
    /// Node display text.
    pub label: String,
    /// Node shape (defaults to Rectangle).
    #[serde(default)]
    pub shape: NodeShape,
    /// Optional semantic metadata.
    #[serde(default)]
    pub semantic: Option<NodeSemantic>,
}

/// Flowchart edge.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowchartEdge {
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Edge label.
    #[serde(default)]
    pub label: Option<String>,
    /// Edge style (defaults to Arrow).
    #[serde(default)]
    pub style: EdgeStyle,
    /// Optional semantic metadata.
    #[serde(default)]
    pub semantic: Option<EdgeSemantic>,
}

/// Subgraph definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subgraph {
    /// Subgraph identifier.
    pub id: String,
    /// Subgraph title.
    pub title: String,
    /// Node ids contained in this subgraph.
    pub nodes: Vec<String>,
}

/// Input for flowchart generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowchartInput {
    /// Flow direction (required, bare FlowDirection).
    pub direction: FlowDirection,
    /// All nodes (need at least one at runtime).
    pub nodes: Vec<FlowchartNode>,
    /// Edges connecting nodes.
    #[serde(default)]
    pub edges: Vec<FlowchartEdge>,
    /// Optional subgraph groupings.
    #[serde(default)]
    pub subgraphs: Vec<Subgraph>,
}

/// Generate a Mermaid flowchart diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart.md#source
pub fn generate_flowchart(input: &FlowchartInput) -> Result<String> {
    if input.nodes.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one node is required".to_string(),
        ));
    }

    // Validate semantic fields
    for node in &input.nodes {
        if let Some(ref semantic) = node.semantic {
            validate_node_semantic(&node.id, semantic)?;
        }
    }

    let mut mermaid = String::new();
    let direction = match input.direction {
        FlowDirection::LR => "LR",
        FlowDirection::RL => "RL",
        FlowDirection::TB => "TB",
        FlowDirection::BT => "BT",
    };
    mermaid.push_str(&format!("flowchart {}\n", direction));

    // Collect subgraph node IDs
    let subgraph_node_ids: HashSet<String> = input
        .subgraphs
        .iter()
        .flat_map(|sg| sg.nodes.iter().cloned())
        .collect();

    // Generate subgraphs
    for subgraph in &input.subgraphs {
        mermaid.push_str(&format!(
            "    subgraph {}[\"{}\"]\n",
            subgraph.id, subgraph.title
        ));

        for node_id in &subgraph.nodes {
            if let Some(node) = input.nodes.iter().find(|n| &n.id == node_id) {
                mermaid.push_str(&format!(
                    "        {}\n",
                    format_node(&node.id, &node.label, &node.shape)?
                ));
            }
        }

        mermaid.push_str("    end\n");
    }

    // Generate standalone nodes
    for node in &input.nodes {
        if !subgraph_node_ids.contains(&node.id) {
            mermaid.push_str(&format!(
                "    {}\n",
                format_node(&node.id, &node.label, &node.shape)?
            ));
        }
    }

    // Generate edges
    for edge in &input.edges {
        mermaid.push_str(&format!(
            "    {}\n",
            format_edge(&edge.from, &edge.to, edge.label.as_deref(), &edge.style)?
        ));
    }

    Ok(mermaid)
}

/// Format a node based on its shape
fn format_node(id: &str, label: &str, shape: &NodeShape) -> Result<String> {
    let escaped_label = label.replace('"', "#quot;");

    let node_str = match shape {
        NodeShape::Rectangle => format!("{}[{}]", id, escaped_label),
        NodeShape::Rounded => format!("{}({})", id, escaped_label),
        NodeShape::Stadium => format!("{}([{}])", id, escaped_label),
        NodeShape::Subroutine => format!("{}[[{}]]", id, escaped_label),
        NodeShape::Cylinder => format!("{}[({})]", id, escaped_label),
        NodeShape::Circle => format!("{}(({}))", id, escaped_label),
        NodeShape::Diamond => format!("{}{{{}}} ", id, escaped_label),
        NodeShape::Hexagon => format!("{}{{{{{}}}}}", id, escaped_label),
        NodeShape::Parallelogram => format!("{}[/{}\\]", id, escaped_label),
        NodeShape::Trapezoid => format!("{}[\\{}/]", id, escaped_label),
    };

    Ok(node_str)
}

/// Format an edge based on its style
fn format_edge(from: &str, to: &str, label: Option<&str>, style: &EdgeStyle) -> Result<String> {
    let edge_str = match (style, label) {
        (EdgeStyle::Arrow, Some(lbl)) => format!("{} -->|{}| {}", from, lbl, to),
        (EdgeStyle::Arrow, None) => format!("{} --> {}", from, to),
        (EdgeStyle::Thick, Some(lbl)) => format!("{} ==>|{}| {}", from, lbl, to),
        (EdgeStyle::Thick, None) => format!("{} ==> {}", from, to),
        (EdgeStyle::Dotted, Some(lbl)) => format!("{} -.->|{}| {}", from, lbl, to),
        (EdgeStyle::Dotted, None) => format!("{} -.-> {}", from, to),
    };

    Ok(edge_str)
}

/// Validate semantic type for a node
fn validate_node_semantic(node_id: &str, semantic: &NodeSemantic) -> Result<()> {
    if let Some(ref sem_type) = semantic.semantic_type {
        match sem_type {
            SemanticType::Validation | SemanticType::RaiseError => {
                if semantic.error.is_none() {
                    return Err(GenerateError::MissingField(format!(
                        "Node '{}' with semantic type '{:?}' requires 'error' field",
                        node_id, sem_type
                    )));
                }
            }
            SemanticType::DbQuery | SemanticType::DbMutation => {
                if semantic.table.is_none() {
                    return Err(GenerateError::MissingField(format!(
                        "Node '{}' with semantic type '{:?}' requires 'table' field",
                        node_id, sem_type
                    )));
                }
            }
            SemanticType::ApiCall => {
                if semantic.url.is_none() && semantic.method.is_none() {
                    return Err(GenerateError::MissingField(format!(
                        "Node '{}' with semantic type 'api_call' requires 'url' or 'method' field",
                        node_id
                    )));
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_flowchart() {
        let input = FlowchartInput {
            direction: FlowDirection::TB,
            nodes: vec![
                FlowchartNode {
                    id: "A".to_string(),
                    label: "Start".to_string(),
                    shape: NodeShape::Rounded,
                    semantic: None,
                },
                FlowchartNode {
                    id: "B".to_string(),
                    label: "Process".to_string(),
                    shape: NodeShape::Rectangle,
                    semantic: None,
                },
                FlowchartNode {
                    id: "C".to_string(),
                    label: "End".to_string(),
                    shape: NodeShape::Rounded,
                    semantic: None,
                },
            ],
            edges: vec![
                FlowchartEdge {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    label: None,
                    style: EdgeStyle::Arrow,
                    semantic: None,
                },
                FlowchartEdge {
                    from: "B".to_string(),
                    to: "C".to_string(),
                    label: None,
                    style: EdgeStyle::Arrow,
                    semantic: None,
                },
            ],
            subgraphs: vec![],
        };

        let result = generate_flowchart(&input).unwrap();
        assert!(result.contains("flowchart TB"));
        assert!(result.contains("A(Start)"));
        assert!(result.contains("B[Process]"));
        assert!(result.contains("C(End)"));
        assert!(result.contains("A --> B"));
        assert!(result.contains("B --> C"));
    }

    #[test]
    fn test_flowchart_with_subgraph() {
        let input = FlowchartInput {
            direction: FlowDirection::LR,
            nodes: vec![
                FlowchartNode {
                    id: "A".to_string(),
                    label: "Start".to_string(),
                    shape: NodeShape::Rounded,
                    semantic: None,
                },
                FlowchartNode {
                    id: "B".to_string(),
                    label: "Step 1".to_string(),
                    shape: NodeShape::Rectangle,
                    semantic: None,
                },
            ],
            edges: vec![],
            subgraphs: vec![Subgraph {
                id: "sg1".to_string(),
                title: "Process".to_string(),
                nodes: vec!["B".to_string()],
            }],
        };

        let result = generate_flowchart(&input).unwrap();
        assert!(result.contains("subgraph sg1[\"Process\"]"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_from_json() {
        let json = r#"{
            "direction": "LR",
            "nodes": [
                {"id": "A", "label": "Start", "shape": "rounded"},
                {"id": "B", "label": "End", "shape": "rounded"}
            ],
            "edges": [
                {"from": "A", "to": "B", "label": "next"}
            ]
        }"#;

        let input: FlowchartInput = serde_json::from_str(json).unwrap();
        let result = generate_flowchart(&input).unwrap();
        assert!(result.contains("flowchart LR"));
        assert!(result.contains("A -->|next| B"));
    }
}

// CODEGEN-END
