---
id: sdd-generate-mindmap
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Mindmap Diagram

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/mindmap.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MindmapInput` | projects/agentic-workflow/src/generate/diagrams/mindmap.rs | struct | pub | 65 |  |
| `MindmapNode` | projects/agentic-workflow/src/generate/diagrams/mindmap.rs | struct | pub | 49 |  |
| `MindmapRoot` | projects/agentic-workflow/src/generate/diagrams/mindmap.rs | struct | pub | 35 |  |
| `MindmapShape` | projects/agentic-workflow/src/generate/diagrams/mindmap.rs | enum | pub | 16 |  |
| `generate_mindmap` | projects/agentic-workflow/src/generate/diagrams/mindmap.rs | function | pub | 74 | generate_mindmap(input: &MindmapInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  MindmapShape:
    type: string
    enum: [Square, Rounded, Circle, Bang, Cloud, Hexagon]
    description: Mindmap node shape.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Square,  is_default: true, doc: "Default rectangular shape." }
        - { name: Rounded,                   doc: "Rounded rectangle." }
        - { name: Circle,                    doc: "Circle." }
        - { name: Bang,                      doc: "Explosion / bang." }
        - { name: Cloud,                     doc: "Cloud." }
        - { name: Hexagon,                   doc: "Hexagon." }

  MindmapRoot:
    type: object
    required: [label, shape]
    description: Root node of the mindmap.
    properties:
      label:
        type: string
        description: "Root node label."
      shape:
        $ref: "#/definitions/MindmapShape"
        description: "Root node shape (defaults to Square)."
        x-serde-default: true
      icon:
        type: string
        description: "Optional icon (emoji or text) prepended to the label."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  MindmapNode:
    type: object
    required: [parent, label, shape]
    description: Child node in the mindmap (anchored to a parent label).
    properties:
      parent:
        type: string
        description: "Parent node label this child attaches to."
      label:
        type: string
        description: "Node label."
      shape:
        $ref: "#/definitions/MindmapShape"
        description: "Node shape (defaults to Square)."
        x-serde-default: true
      icon:
        type: string
        description: "Optional icon (emoji or text)."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  MindmapInput:
    type: object
    required: [root, nodes]
    description: Input for mindmap generation.
    properties:
      root:
        $ref: "#/definitions/MindmapRoot"
        description: "Root node of the mindmap."
      nodes:
        type: array
        items:
          $ref: "#/definitions/MindmapNode"
        description: "Child nodes (parent links by label)."
        x-serde-default: true
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/mindmap.rs -->
```rust
//! Mindmap Diagram Generation
//!
//! Generates Mermaid mindmap diagrams for concept organization and feature breakdown.

use crate::generate::Result;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Mindmap node shape.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MindmapShape {
    /// Default rectangular shape.
    #[default]
    Square,
    /// Rounded rectangle.
    Rounded,
    /// Circle.
    Circle,
    /// Explosion / bang.
    Bang,
    /// Cloud.
    Cloud,
    /// Hexagon.
    Hexagon,
}

/// Root node of the mindmap.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindmapRoot {
    /// Root node label.
    pub label: String,
    /// Root node shape (defaults to Square).
    #[serde(default)]
    pub shape: MindmapShape,
    /// Optional icon (emoji or text) prepended to the label.
    #[serde(default)]
    pub icon: Option<String>,
}

/// Child node in the mindmap (anchored to a parent label).
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindmapNode {
    /// Parent node label this child attaches to.
    pub parent: String,
    /// Node label.
    pub label: String,
    /// Node shape (defaults to Square).
    #[serde(default)]
    pub shape: MindmapShape,
    /// Optional icon (emoji or text).
    #[serde(default)]
    pub icon: Option<String>,
}

/// Input for mindmap generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindmapInput {
    /// Root node of the mindmap.
    pub root: MindmapRoot,
    /// Child nodes (parent links by label).
    #[serde(default)]
    pub nodes: Vec<MindmapNode>,
}
/// Generate a Mermaid mindmap diagram
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap.md#source
pub fn generate_mindmap(input: &MindmapInput) -> Result<String> {
    // Build a tree structure
    let mut tree: HashMap<String, Vec<&MindmapNode>> = HashMap::new();

    for node in &input.nodes {
        tree.entry(node.parent.clone()).or_default().push(node);
    }

    // Generate Mermaid mindmap
    let mut mermaid = String::new();
    mermaid.push_str("mindmap\n");

    // Format root
    mermaid.push_str(&format!(
        "  {}\n",
        format_node(
            &input.root.label,
            &input.root.shape,
            input.root.icon.as_deref()
        )?
    ));

    // Recursively generate child nodes
    generate_children(&mut mermaid, &input.root.label, &tree, 2)?;

    Ok(mermaid)
}

/// Recursively generate child nodes
fn generate_children(
    mermaid: &mut String,
    parent_label: &str,
    tree: &HashMap<String, Vec<&MindmapNode>>,
    indent_level: usize,
) -> Result<()> {
    if let Some(children) = tree.get(parent_label) {
        for child in children {
            let indent = "  ".repeat(indent_level);
            mermaid.push_str(&format!(
                "{}{}\n",
                indent,
                format_node(&child.label, &child.shape, child.icon.as_deref())?
            ));

            // Recursively generate grandchildren
            generate_children(mermaid, &child.label, tree, indent_level + 1)?;
        }
    }

    Ok(())
}

/// Format a node based on its shape
fn format_node(label: &str, shape: &MindmapShape, icon: Option<&str>) -> Result<String> {
    let display_label = if let Some(ic) = icon {
        format!("{} {}", ic, label)
    } else {
        label.to_string()
    };

    let node_str = match shape {
        MindmapShape::Square => format!("[{}]", display_label),
        MindmapShape::Rounded => format!("({})", display_label),
        MindmapShape::Circle => format!("(({}))", display_label),
        MindmapShape::Bang => format!(")){}((", display_label),
        MindmapShape::Cloud => format!("){}(", display_label),
        MindmapShape::Hexagon => format!("{{{{{{{{{}}}}}}}}}", display_label),
    };

    Ok(node_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mindmap() {
        let input = MindmapInput {
            root: MindmapRoot {
                label: "Project".to_string(),
                shape: MindmapShape::Square,
                icon: None,
            },
            nodes: vec![
                MindmapNode {
                    parent: "Project".to_string(),
                    label: "Frontend".to_string(),
                    shape: MindmapShape::Rounded,
                    icon: None,
                },
                MindmapNode {
                    parent: "Project".to_string(),
                    label: "Backend".to_string(),
                    shape: MindmapShape::Rounded,
                    icon: None,
                },
                MindmapNode {
                    parent: "Frontend".to_string(),
                    label: "React".to_string(),
                    shape: MindmapShape::Circle,
                    icon: None,
                },
            ],
        };

        let result = generate_mindmap(&input).unwrap();
        assert!(result.contains("mindmap"));
        assert!(result.contains("[Project]"));
        assert!(result.contains("(Frontend)"));
        assert!(result.contains("(Backend)"));
        assert!(result.contains("((React))"));
    }

    #[test]
    fn test_mindmap_with_icons() {
        let input = MindmapInput {
            root: MindmapRoot {
                label: "Features".to_string(),
                shape: MindmapShape::Rounded,
                icon: Some("🎯".to_string()),
            },
            nodes: vec![MindmapNode {
                parent: "Features".to_string(),
                label: "Authentication".to_string(),
                shape: MindmapShape::Square,
                icon: Some("🔐".to_string()),
            }],
        };

        let result = generate_mindmap(&input).unwrap();
        assert!(result.contains("🎯 Features"));
        assert!(result.contains("🔐 Authentication"));
    }

    #[test]
    fn test_from_json() {
        let json = r#"{
            "root": {"label": "Test", "shape": "square"},
            "nodes": [{"parent": "Test", "label": "Child"}]
        }"#;

        let input: MindmapInput = serde_json::from_str(json).unwrap();
        let result = generate_mindmap(&input).unwrap();
        assert!(result.contains("[Test]"));
        assert!(result.contains("[Child]"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/mindmap.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete mindmap diagram generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [schema] All four type definitions match the source exactly: derives, `#[serde(rename_all = "lowercase")]` on `MindmapShape`, `#[default]` on `Square` encoded as `is_default: true`, `x-serde-default: true` on `shape` and `icon` fields in both `MindmapRoot` and `MindmapNode`, and `x-serde-default: true` on the `nodes` vec in `MindmapInput`.
- [changes] Two-entry changes split is correct: codegen entry lists all four replaced symbols; hand-written entry covers module docstring, `use` statements (incl. `HashMap`), `generate_mindmap`, `generate_children`, `format_node`, and test block.
- [overview] `is_default: true` pattern attribution to `sdd-generate-mindmap-plus-schema` is accurate; hand-written boundary is clearly delineated.
