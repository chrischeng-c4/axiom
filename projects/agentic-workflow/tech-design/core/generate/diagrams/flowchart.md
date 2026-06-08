---
id: sdd-generate-flowchart
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Flowchart Diagram

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/flowchart.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DbOperation` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | enum | pub | 96 |  |
| `EdgeSemantic` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 194 |  |
| `EdgeStyle` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | enum | pub | 64 |  |
| `ErrorSpec` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 146 |  |
| `FlowDirection` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | enum | pub | 16 |  |
| `FlowchartEdge` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 222 |  |
| `FlowchartInput` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 253 |  |
| `FlowchartNode` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 206 |  |
| `HttpMethod` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | enum | pub | 108 |  |
| `InputSpec` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 119 |  |
| `NodeSemantic` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 158 |  |
| `NodeShape` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | enum | pub | 36 |  |
| `OutputSpec` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 134 |  |
| `SemanticType` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | enum | pub | 78 |  |
| `Subgraph` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | struct | pub | 241 |  |
| `generate_flowchart` | projects/agentic-workflow/src/generate/diagrams/flowchart.rs | function | pub | 268 | generate_flowchart(input: &FlowchartInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FlowDirection:
    type: string
    enum: [LR, RL, TB, BT]
    description: Flow direction (Mermaid `flowchart <dir>` keyword).
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      variants:
        - { name: LR,                   doc: "Left to Right." }
        - { name: RL,                   doc: "Right to Left." }
        - { name: TB, is_default: true, doc: "Top to Bottom (default)." }
        - { name: BT,                   doc: "Bottom to Top." }

  NodeShape:
    type: string
    enum:
      - Rectangle
      - Rounded
      - Stadium
      - Subroutine
      - Cylinder
      - Circle
      - Diamond
      - Hexagon
      - Parallelogram
      - Trapezoid
    description: Flowchart node shape.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Rectangle,    is_default: true, doc: "Plain rectangle (default)." }
        - { name: Rounded,                        doc: "Rounded rectangle." }
        - { name: Stadium,                        doc: "Stadium." }
        - { name: Subroutine,                     doc: "Subroutine box." }
        - { name: Cylinder,                       doc: "Cylinder." }
        - { name: Circle,                         doc: "Circle." }
        - { name: Diamond,                        doc: "Diamond / decision." }
        - { name: Hexagon,                        doc: "Hexagon." }
        - { name: Parallelogram,                  doc: "Parallelogram (input/output)." }
        - { name: Trapezoid,                      doc: "Trapezoid." }

  EdgeStyle:
    type: string
    enum: [Arrow, Thick, Dotted]
    description: Flowchart edge style.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Arrow,  is_default: true, doc: "Solid arrow (default)." }
        - { name: Thick,                    doc: "Thick arrow." }
        - { name: Dotted,                   doc: "Dotted arrow." }

  SemanticType:
    type: string
    enum:
      - Start
      - End
      - Validation
      - Condition
      - DbQuery
      - DbMutation
      - ApiCall
      - Transform
      - Assign
      - Return
      - RaiseError
    description: Node semantic kind for spec-to-code generation.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: snake_case

  DbOperation:
    type: string
    enum: [Select, Insert, Update, Delete, Upsert]
    description: Database operation kind.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: UPPERCASE

  HttpMethod:
    type: string
    enum: [Get, Post, Put, Patch, Delete]
    description: HTTP method.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: UPPERCASE

  InputSpec:
    type: object
    description: Input parameter specification.
    properties:
      name:
        type: string
        description: "Parameter name."
        x-serde-default: true
      type_annotation:
        type: string
        description: "Type annotation. JSON key 'type'."
        x-serde-rename: "type"
        x-serde-default: true
      source:
        type: string
        description: "Source expression for the parameter."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  OutputSpec:
    type: object
    description: Output specification.
    properties:
      name:
        type: string
        description: "Output name."
        x-serde-default: true
      type_annotation:
        type: string
        description: "Type annotation. JSON key 'type'."
        x-serde-rename: "type"
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  ErrorSpec:
    type: object
    description: Error specification.
    properties:
      code:
        type: integer
        x-rust-type: "Option<i32>"
        description: "Error code."
        x-serde-default: true
      message:
        type: string
        description: "Error message."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  NodeSemantic:
    type: object
    description: Node semantic metadata for code generation.
    properties:
      semantic_type:
        $ref: "#/definitions/SemanticType"
        description: "Semantic kind. JSON key 'type'."
        x-serde-rename: "type"
        x-serde-default: true
      input:
        $ref: "#/definitions/InputSpec"
        description: "Input parameter."
        x-serde-default: true
      output:
        $ref: "#/definitions/OutputSpec"
        description: "Output specification."
        x-serde-default: true
      error:
        $ref: "#/definitions/ErrorSpec"
        description: "Error specification."
        x-serde-default: true
      operation:
        $ref: "#/definitions/DbOperation"
        description: "Database operation kind."
        x-serde-default: true
      table:
        type: string
        description: "Database table name."
        x-serde-default: true
      filter:
        type: string
        description: "Database filter expression."
        x-serde-default: true
      url:
        type: string
        description: "API URL."
        x-serde-default: true
      method:
        $ref: "#/definitions/HttpMethod"
        description: "HTTP method."
        x-serde-default: true
      code_pattern:
        type: string
        description: "Code pattern hint."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  EdgeSemantic:
    type: object
    description: Edge semantic metadata.
    properties:
      condition:
        type: string
        description: "Edge condition expression."
        x-serde-default: true
      is_error_path:
        type: boolean
        x-rust-type: "Option<bool>"
        description: "Whether this edge marks the error path."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  FlowchartNode:
    type: object
    required: [id, label, shape]
    description: Flowchart node.
    properties:
      id:
        type: string
        description: "Unique node identifier."
      label:
        type: string
        description: "Node display text."
      shape:
        $ref: "#/definitions/NodeShape"
        description: "Node shape (defaults to Rectangle)."
        x-serde-default: true
      semantic:
        $ref: "#/definitions/NodeSemantic"
        description: "Optional semantic metadata."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  FlowchartEdge:
    type: object
    required: [from, to, style]
    description: Flowchart edge.
    properties:
      from:
        type: string
        description: "Source node id."
      to:
        type: string
        description: "Target node id."
      label:
        type: string
        description: "Edge label."
        x-serde-default: true
      style:
        $ref: "#/definitions/EdgeStyle"
        description: "Edge style (defaults to Arrow)."
        x-serde-default: true
      semantic:
        $ref: "#/definitions/EdgeSemantic"
        description: "Optional semantic metadata."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Subgraph:
    type: object
    required: [id, title, nodes]
    description: Subgraph definition.
    properties:
      id:
        type: string
        description: "Subgraph identifier."
      title:
        type: string
        description: "Subgraph title."
      nodes:
        type: array
        items: { type: string }
        description: "Node ids contained in this subgraph."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  FlowchartInput:
    type: object
    required: [direction, nodes, edges, subgraphs]
    description: Input for flowchart generation.
    properties:
      direction:
        $ref: "#/definitions/FlowDirection"
        description: "Flow direction (required, bare FlowDirection)."
      nodes:
        type: array
        items:
          $ref: "#/definitions/FlowchartNode"
        description: "All nodes (need at least one at runtime)."
      edges:
        type: array
        items:
          $ref: "#/definitions/FlowchartEdge"
        description: "Edges connecting nodes."
        x-serde-default: true
      subgraphs:
        type: array
        items:
          $ref: "#/definitions/Subgraph"
        description: "Optional subgraph groupings."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/flowchart.rs -->
```rust
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
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/flowchart.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Mermaid flowchart diagram module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Minor count inconsistency: the overview header says "Fourteen serde shapes" and "Structs (8)" but the schema defines 15 types (6 enums + 9 structs). The requirements text also says "14 types (6 enums + 8 structs)". The schema itself is complete and unambiguous with all 15 definitions present — this is a documentation nit only and does not affect implementation.
- [schema] All conventions are correctly applied: `FlowDirection` has no `serde_rename_all` with `TB` carrying `is_default: true`; `NodeShape`/`EdgeStyle` use `lowercase`; `SemanticType` uses `snake_case`; `DbOperation`/`HttpMethod` use `UPPERCASE`. Vec fields with `x-serde-default: true` are in `required:`. Bare `FlowDirection` and `nodes` Vec (no source-side `#[serde(default)]`) are in `required:` without `x-serde-default`. `ErrorSpec.code` and `EdgeSemantic.is_error_path` use `x-rust-type: "Option<...>"` in `required:` per convention. All correct.
- [changes] The `replaces` list has 15 entries matching the schema definitions; the description correctly identifies the manual `impl Default` deletion. Hand-written boundary is clearly delineated.
