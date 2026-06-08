//! Flowchart+ definition schema
//!
//! Structured flowchart definitions with semantic metadata for code generation.

use indexmap::IndexMap;
use std::collections::HashMap;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Database operation type.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DbOperation {
    Insert,
    Update,
    Delete,
    Upsert,
}

/// Edge definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDef {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Edge label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Edge style.
    #[serde(default, skip_serializing_if = "is_default_edge_style")]
    pub style: EdgeStyle,
    /// Semantic: condition expression (for conditional branches).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Semantic: is this an error path?
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_error_path: bool,
}

/// Edge style.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EdgeStyle {
    /// Arrow (default).
    #[default]
    Arrow,
    /// Thick line.
    Thick,
    /// Dotted line.
    Dotted,
}

/// Flow direction.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum FlowDirection {
    /// Top to bottom (default).
    #[default]
    TB,
    /// Bottom to top.
    BT,
    /// Left to right.
    LR,
    /// Right to left.
    RL,
}

/// Flowchart definition (input from LLM).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowchartDef {
    /// Diagram identifier.
    pub id: String,
    /// Flow direction.
    #[serde(default, skip_serializing_if = "is_default_direction")]
    pub direction: FlowDirection,
    /// Node definitions keyed by node ID.
    pub nodes: IndexMap<String, NodeDef>,
    /// Edge definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<EdgeDef>,
    /// Subgraph definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subgraphs: Vec<SubgraphDef>,
    /// Diagram description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// HTTP method.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Node definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeDef {
    /// Node display label.
    pub label: String,
    /// Node shape.
    #[serde(default, skip_serializing_if = "is_default_shape")]
    pub shape: NodeShape,
    /// Semantic type for code generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic: Option<SemanticType>,
    /// Node description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Primitive operation: when present the logic generator uses the named
    /// primitive's emit template rather than generic scaffolding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primitive: Option<PrimitiveKind>,
    /// Primitive input bindings: maps input field names to upstream
    /// variable names or literal values.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, serde_yaml::Value>,
    /// Rust variable name to which this node's output is bound.
    /// Downstream nodes reference it by this name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Node shape.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeShape {
    /// Rectangle (default).
    #[default]
    Rectangle,
    /// Rounded rectangle.
    Rounded,
    /// Stadium shape.
    Stadium,
    /// Subroutine shape.
    Subroutine,
    /// Cylinder (database) shape.
    Cylinder,
    /// Circle.
    Circle,
    /// Diamond (decision).
    Diamond,
    /// Hexagon.
    Hexagon,
    /// Parallelogram (input/output).
    Parallelogram,
    /// Trapezoid.
    Trapezoid,
}

/// Named primitive operations for flowchart YAML nodes.
/// Each variant maps to a single Rust emit template in PrimitiveRegistry.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveKind {
    ReadFile,
    WriteFile,
    AppendFile,
    PathExists,
    ParseJsonlStream,
    AppendLineAtomic,
    ParseJsonlStr,
    SerializeJsonlLine,
    RunSubprocess,
    ParseYaml,
    ParseJson,
    SerializeYaml,
    FormatTemplate,
    Now,
    TtyCheck,
    PrintStdout,
    Call,
}

/// Semantic type for code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SemanticType {
    /// Start node.
    Start,
    /// End/return node.
    End { output: Option<String> },
    /// Input validation.
    Validation {
        input: String,
        rules: Vec<String>,
        error_code: Option<i32>,
        error_message: Option<String>,
    },
    /// Condition/decision.
    Condition { expression: String },
    /// Database query (SELECT).
    DbQuery {
        table: String,
        filter: Option<String>,
        output: Option<String>,
    },
    /// Database mutation (INSERT/UPDATE/DELETE).
    DbMutation {
        operation: DbOperation,
        table: String,
        data: Option<String>,
    },
    /// External API call.
    ApiCall {
        method: HttpMethod,
        url: String,
        body: Option<String>,
        output: Option<String>,
    },
    /// Data transformation.
    Transform {
        input: String,
        output: String,
        expression: Option<String>,
    },
    /// Variable assignment.
    Assign { variable: String, value: String },
    /// Raise error.
    RaiseError { code: i32, message: String },
    /// Loop start.
    LoopStart { condition: String },
    /// Loop end.
    LoopEnd,
}

/// Subgraph definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubgraphDef {
    /// Subgraph identifier.
    pub id: String,
    /// Subgraph display label.
    pub label: String,
    /// Node IDs contained in this subgraph.
    pub nodes: Vec<String>,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}
// CODEGEN-END
fn is_default_direction(d: &FlowDirection) -> bool {
    *d == FlowDirection::TB
}

fn is_default_shape(s: &NodeShape) -> bool {
    *s == NodeShape::Rectangle
}

fn is_default_edge_style(s: &EdgeStyle) -> bool {
    *s == EdgeStyle::Arrow
}

fn is_false(b: &bool) -> bool {
    !b
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_flowchart() {
        let json = json!({
            "id": "login-flow",
            "direction": "TB",
            "nodes": {
                "start": { "label": "Start", "shape": "rounded" },
                "validate": { "label": "Validate Input", "shape": "rectangle" },
                "end": { "label": "End", "shape": "rounded" }
            },
            "edges": [
                { "from": "start", "to": "validate" },
                { "from": "validate", "to": "end" }
            ]
        });

        let flowchart: FlowchartDef = serde_json::from_value(json).unwrap();
        assert_eq!(flowchart.id, "login-flow");
        assert_eq!(flowchart.direction, FlowDirection::TB);
        assert_eq!(flowchart.nodes.len(), 3);
        assert_eq!(flowchart.edges.len(), 2);
    }

    #[test]
    fn test_parse_with_semantic() {
        let json = json!({
            "id": "api-flow",
            "nodes": {
                "start": {
                    "label": "Start",
                    "semantic": { "type": "start" }
                },
                "validate": {
                    "label": "Validate Request",
                    "semantic": {
                        "type": "validation",
                        "input": "request.body",
                        "rules": ["required: email", "format: email"],
                        "error_code": 400
                    }
                },
                "query_db": {
                    "label": "Query User",
                    "semantic": {
                        "type": "db_query",
                        "table": "users",
                        "filter": "email = $email",
                        "output": "user"
                    }
                }
            },
            "edges": []
        });

        let flowchart: FlowchartDef = serde_json::from_value(json).unwrap();
        assert_eq!(flowchart.nodes.len(), 3);

        let validate = flowchart.nodes.get("validate").unwrap();
        assert!(matches!(
            validate.semantic,
            Some(SemanticType::Validation { .. })
        ));
    }

    #[test]
    fn test_parse_with_subgraphs() {
        let json = json!({
            "id": "grouped-flow",
            "nodes": {
                "a": { "label": "A" },
                "b": { "label": "B" },
                "c": { "label": "C" }
            },
            "edges": [],
            "subgraphs": [
                { "id": "sg1", "label": "Group 1", "nodes": ["a", "b"] }
            ]
        });

        let flowchart: FlowchartDef = serde_json::from_value(json).unwrap();
        assert_eq!(flowchart.subgraphs.len(), 1);
        assert_eq!(flowchart.subgraphs[0].nodes.len(), 2);
    }
}
