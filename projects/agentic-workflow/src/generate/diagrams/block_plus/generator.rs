// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/generator.md#source
// CODEGEN-BEGIN
//! Block+ generator

use super::schema::{BlockDef, BlockEdgeStyle, BlockNodeDef, BlockShape};
use super::validator::BlockValidationResult;

/// Output of the Mermaid Plus block generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/generator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockPlusOutput {
    pub frontmatter: String,
    pub diagram: String,
    pub validation: BlockValidationResult,
    pub combined: String,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/generator.md#source
pub struct BlockPlusGenerator;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/generator.md#source
impl BlockPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        diagram: &BlockDef,
        validation: BlockValidationResult,
    ) -> Result<BlockPlusOutput, String> {
        let frontmatter = self.generate_frontmatter(diagram)?;
        let mermaid = self.generate_mermaid(diagram)?;

        let mut combined = String::new();
        combined.push_str("```mermaid\n");
        combined.push_str("---\n");
        combined.push_str(&frontmatter);
        combined.push_str("---\n");
        combined.push_str(&mermaid);
        combined.push_str("```\n");

        if !validation.warnings.is_empty() {
            combined.push_str("\n<!-- Validation Warnings:\n");
            for w in &validation.warnings {
                combined.push_str(&format!("  - {}: {} (at {})\n", w.code, w.message, w.path));
            }
            combined.push_str("-->\n");
        }

        Ok(BlockPlusOutput {
            frontmatter,
            diagram: mermaid,
            validation,
            combined,
        })
    }

    fn generate_frontmatter(&self, diagram: &BlockDef) -> Result<String, String> {
        let yaml = serde_yaml::to_string(diagram).map_err(|e| format!("YAML error: {}", e))?;
        Ok(yaml.strip_prefix("---\n").unwrap_or(&yaml).to_string())
    }

    pub fn generate_mermaid(&self, diagram: &BlockDef) -> Result<String, String> {
        let mut out = String::new();
        out.push_str("block-beta\n");
        out.push_str(&format!("    columns {}\n", diagram.columns));

        // Generate blocks
        self.generate_blocks(&diagram.blocks, &mut out, 1)?;

        // Generate edges
        for edge in &diagram.edges {
            let edge_str = match edge.style {
                BlockEdgeStyle::Arrow => "-->",
                BlockEdgeStyle::Thick => "==>",
                BlockEdgeStyle::Dotted => "-.->",
            };
            if let Some(label) = &edge.label {
                out.push_str(&format!(
                    "    {} {}|\"{}\"| {}\n",
                    edge.from, edge_str, label, edge.to
                ));
            } else {
                out.push_str(&format!("    {} {} {}\n", edge.from, edge_str, edge.to));
            }
        }

        Ok(out)
    }

    fn generate_blocks(
        &self,
        blocks: &[BlockNodeDef],
        out: &mut String,
        depth: usize,
    ) -> Result<(), String> {
        let indent = "    ".repeat(depth);
        for block in blocks {
            if !block.children.is_empty() {
                // Composite block
                let child_cols = block.children.len() as u32;
                let cols = block.child_columns.unwrap_or(child_cols);
                if block.width > 1 {
                    out.push_str(&format!("{}block:{}:{}\n", indent, block.id, block.width));
                } else {
                    out.push_str(&format!("{}block:{}\n", indent, block.id));
                }
                out.push_str(&format!("{}    columns {}\n", indent, cols));
                self.generate_blocks(&block.children, out, depth + 1)?;
                out.push_str(&format!("{}end\n", indent));
            } else {
                // Leaf block
                let shape_str = self.format_shape(&block.id, &block.label, &block.shape);
                if block.width > 1 {
                    out.push_str(&format!("{}{}:{}\n", indent, shape_str, block.width));
                } else {
                    out.push_str(&format!("{}{}\n", indent, shape_str));
                }
            }
        }
        Ok(())
    }

    fn format_shape(&self, id: &str, label: &str, shape: &BlockShape) -> String {
        match shape {
            BlockShape::Default => format!("{}[\"{}\"]", id, label),
            BlockShape::Round => format!("{}(\"{}\")", id, label),
            BlockShape::Stadium => format!("{}([\"{}\"])", id, label),
            BlockShape::Diamond => format!("{}{{\"{}\"}}", id, label),
            BlockShape::Cylinder => format!("{}[(\"{}\")]", id, label),
            BlockShape::Hexagon => format!("{}{{{{\"{}\"}}}}", id, label),
            BlockShape::Circle => format!("{}((\"{}\"))", id, label),
            BlockShape::Subroutine => format!("{}[[\"{}\"]]", id, label),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/generator.md#source
impl Default for BlockPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::BlockValidator;
    use super::*;
    use serde_json::json;

    fn parse_diagram(json: serde_json::Value) -> BlockDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_basic_block_diagram() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 3,
            "blocks": [
                { "id": "a", "label": "Auth Spec", "shape": "round", "width": 2 },
                { "id": "b", "label": "DB Schema", "shape": "cylinder" },
                { "id": "c", "label": "API Layer", "shape": "stadium" }
            ],
            "edges": [
                { "from": "a", "to": "b", "label": "depends on" },
                { "from": "c", "to": "a", "style": "thick" }
            ]
        }));

        let validation = BlockValidator::new().validate(&diagram);
        let output = BlockPlusGenerator::new()
            .generate(&diagram, validation)
            .unwrap();

        assert!(output.diagram.contains("block-beta"));
        assert!(output.diagram.contains("columns 3"));
        assert!(output.diagram.contains("a -->|\"depends on\"| b"));
        assert!(output.diagram.contains("c ==> a"));
    }

    #[test]
    fn test_generate_nested_blocks() {
        let diagram = parse_diagram(json!({
            "id": "nested",
            "columns": 2,
            "blocks": [
                {
                    "id": "group1",
                    "label": "Group",
                    "child_columns": 2,
                    "children": [
                        { "id": "a", "label": "A" },
                        { "id": "b", "label": "B" }
                    ]
                },
                { "id": "c", "label": "C" }
            ],
            "edges": [{ "from": "a", "to": "c" }]
        }));

        let validation = BlockValidator::new().validate(&diagram);
        let output = BlockPlusGenerator::new()
            .generate(&diagram, validation)
            .unwrap();

        assert!(output.diagram.contains("block:group1"));
        assert!(output.diagram.contains("columns 2"));
        assert!(output.diagram.contains("end"));
    }

    #[test]
    fn test_combined_format() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 1,
            "blocks": [{ "id": "a", "label": "A" }]
        }));

        let validation = BlockValidator::new().validate(&diagram);
        let output = BlockPlusGenerator::new()
            .generate(&diagram, validation)
            .unwrap();

        assert!(output.combined.starts_with("```mermaid\n---\n"));
        assert!(output.combined.contains("---\nblock-beta"));
        assert!(output.combined.ends_with("```\n"));
    }
}

// CODEGEN-END
