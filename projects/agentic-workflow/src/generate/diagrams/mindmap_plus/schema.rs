//! Mindmap+ definition schema

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/schema.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Mindmap definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindmapDef {
    /// Diagram identifier.
    pub id: String,
    /// Root node.
    pub root: MindmapNodeDef,
    /// Diagram description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Mindmap node definition (recursive).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindmapNodeDef {
    /// Node label.
    pub label: String,
    /// Node shape; absent key defaults to Square.
    #[serde(default)]
    pub shape: MindmapShapePlus,
    /// Icon (emoji or text).
    #[serde(default)]
    pub icon: Option<String>,
    /// Child nodes.
    #[serde(default)]
    pub children: Vec<MindmapNodeDef>,
    /// Description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Mermaid-Plus mindmap node shape. Default is Square so an absent
/// JSON shape key deserialises to a sensible visual.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/schema.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MindmapShapePlus {
    /// Square node (default).
    #[default]
    Square,
    /// Rounded square node.
    Rounded,
    /// Circular node.
    Circle,
    /// Bang (callout) node.
    Bang,
    /// Cloud node.
    Cloud,
    /// Hexagonal node.
    Hexagon,
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_mindmap() {
        let json = json!({
            "id": "project-structure",
            "root": {
                "label": "Project",
                "shape": "square",
                "children": [
                    {
                        "label": "Frontend",
                        "shape": "rounded",
                        "icon": "🎨",
                        "children": [
                            { "label": "React", "shape": "circle" },
                            { "label": "TypeScript" }
                        ]
                    },
                    {
                        "label": "Backend",
                        "shape": "rounded",
                        "icon": "⚙️",
                        "children": [
                            { "label": "Rust" },
                            { "label": "PostgreSQL" }
                        ]
                    }
                ]
            }
        });

        let mindmap: MindmapDef = serde_json::from_value(json).unwrap();
        assert_eq!(mindmap.root.label, "Project");
        assert_eq!(mindmap.root.children.len(), 2);
        assert_eq!(mindmap.root.children[0].children.len(), 2);
    }

    #[test]
    fn test_simple_mindmap() {
        let json = json!({
            "id": "simple",
            "root": {
                "label": "Root",
                "children": [
                    { "label": "Child 1" },
                    { "label": "Child 2" }
                ]
            }
        });

        let mindmap: MindmapDef = serde_json::from_value(json).unwrap();
        assert_eq!(mindmap.root.children.len(), 2);
    }
}
