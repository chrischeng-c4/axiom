//! Block+ definition schema

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Block diagram definition (Mermaid block-beta).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDef {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default = "default_columns")]
    pub columns: u32,
    pub blocks: Vec<BlockNodeDef>,
    #[serde(default)]
    pub edges: Vec<BlockEdgeDef>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Edge between blocks.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockEdgeDef {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub style: BlockEdgeStyle,
}

/// Edge style.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlockEdgeStyle {
    /// Solid arrow (default).
    #[default]
    Arrow,
    /// Thick arrow.
    Thick,
    /// Dotted arrow.
    Dotted,
}

/// Block node definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockNodeDef {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub shape: BlockShape,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default)]
    pub children: Vec<BlockNodeDef>,
    #[serde(default)]
    pub child_columns: Option<u32>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Block shape types.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlockShape {
    /// Default rectangular.
    #[default]
    Default,
    /// Round.
    Round,
    /// Stadium.
    Stadium,
    /// Diamond.
    Diamond,
    /// Cylinder.
    Cylinder,
    /// Hexagon.
    Hexagon,
    /// Circle.
    Circle,
    /// Subroutine.
    Subroutine,
}
// CODEGEN-END
fn default_columns() -> u32 {
    1
}

fn default_width() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_block_diagram() {
        let json = json!({
            "id": "spec-deps",
            "title": "Spec Dependencies",
            "columns": 3,
            "blocks": [
                { "id": "auth", "label": "Auth Spec", "shape": "round", "width": 2 },
                { "id": "db", "label": "DB Schema", "shape": "cylinder" },
                { "id": "api", "label": "API Layer", "shape": "stadium" }
            ],
            "edges": [
                { "from": "auth", "to": "db", "label": "depends on" },
                { "from": "api", "to": "auth", "style": "thick" }
            ]
        });

        let diagram: BlockDef = serde_json::from_value(json).unwrap();
        assert_eq!(diagram.columns, 3);
        assert_eq!(diagram.blocks.len(), 3);
        assert_eq!(diagram.edges.len(), 2);
        assert_eq!(diagram.blocks[0].width, 2);
        assert_eq!(diagram.blocks[1].shape, BlockShape::Cylinder);
    }

    #[test]
    fn test_parse_nested_blocks() {
        let json = json!({
            "id": "nested",
            "columns": 2,
            "blocks": [
                {
                    "id": "group1",
                    "label": "Group 1",
                    "child_columns": 2,
                    "children": [
                        { "id": "a", "label": "A" },
                        { "id": "b", "label": "B" }
                    ]
                },
                { "id": "c", "label": "C" }
            ]
        });

        let diagram: BlockDef = serde_json::from_value(json).unwrap();
        assert_eq!(diagram.blocks[0].children.len(), 2);
    }

    #[test]
    fn test_defaults() {
        let json = json!({
            "id": "minimal",
            "blocks": [{ "id": "a", "label": "A" }]
        });

        let diagram: BlockDef = serde_json::from_value(json).unwrap();
        assert_eq!(diagram.columns, 1);
        assert_eq!(diagram.blocks[0].width, 1);
        assert_eq!(diagram.blocks[0].shape, BlockShape::Default);
    }
}
