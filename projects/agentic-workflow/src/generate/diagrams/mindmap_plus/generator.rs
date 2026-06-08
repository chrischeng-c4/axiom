// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/generator.md#source
// CODEGEN-BEGIN
//! Mindmap+ generator

use super::schema::{MindmapDef, MindmapNodeDef, MindmapShapePlus};
use super::validator::MindmapValidationResult;

/// Output of the Mermaid Plus generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/generator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct MindmapPlusOutput {
    pub frontmatter: String,
    pub diagram: String,
    pub validation: MindmapValidationResult,
    pub combined: String,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/generator.md#source
pub struct MindmapPlusGenerator;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/generator.md#source
impl MindmapPlusGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        mindmap: &MindmapDef,
        validation: MindmapValidationResult,
    ) -> Result<MindmapPlusOutput, String> {
        let frontmatter = self.generate_frontmatter(mindmap)?;
        let mermaid = self.generate_mermaid(mindmap)?;

        // Combine into Mermaid+ format (frontmatter inside code block per Mermaid spec)
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

        Ok(MindmapPlusOutput {
            frontmatter,
            diagram: mermaid,
            validation,
            combined,
        })
    }

    fn generate_frontmatter(&self, mindmap: &MindmapDef) -> Result<String, String> {
        let yaml = serde_yaml::to_string(mindmap).map_err(|e| format!("YAML error: {}", e))?;
        Ok(yaml.strip_prefix("---\n").unwrap_or(&yaml).to_string())
    }

    pub fn generate_mermaid(&self, mindmap: &MindmapDef) -> Result<String, String> {
        let mut mermaid = String::new();
        mermaid.push_str("mindmap\n");

        // Generate root
        mermaid.push_str(&format!("  {}\n", self.format_node(&mindmap.root)?));

        // Generate children recursively
        self.generate_children(&mindmap.root.children, &mut mermaid, 2)?;

        Ok(mermaid)
    }

    fn generate_children(
        &self,
        children: &[MindmapNodeDef],
        mermaid: &mut String,
        indent_level: usize,
    ) -> Result<(), String> {
        let indent = "  ".repeat(indent_level);

        for child in children {
            mermaid.push_str(&format!("{}{}\n", indent, self.format_node(child)?));
            self.generate_children(&child.children, mermaid, indent_level + 1)?;
        }

        Ok(())
    }

    fn format_node(&self, node: &MindmapNodeDef) -> Result<String, String> {
        let display_label = if let Some(ref icon) = node.icon {
            format!("{} {}", icon, node.label)
        } else {
            node.label.clone()
        };

        let node_str = match node.shape {
            MindmapShapePlus::Square => format!("[{}]", display_label),
            MindmapShapePlus::Rounded => format!("({})", display_label),
            MindmapShapePlus::Circle => format!("(({}))", display_label),
            MindmapShapePlus::Bang => format!(")){}((", display_label),
            MindmapShapePlus::Cloud => format!("){}(", display_label),
            MindmapShapePlus::Hexagon => format!("{{{{{{{}}}}}}}", display_label),
        };

        Ok(node_str)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/generator.md#source
impl Default for MindmapPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator::MindmapValidator;
    use super::*;
    use serde_json::json;

    fn parse_mindmap(json: serde_json::Value) -> MindmapDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_generate_mindmap() {
        let mindmap = parse_mindmap(json!({
            "id": "test",
            "root": {
                "label": "Project",
                "shape": "square",
                "children": [
                    {
                        "label": "Frontend",
                        "shape": "rounded",
                        "children": [
                            { "label": "React", "shape": "circle" }
                        ]
                    },
                    { "label": "Backend", "shape": "rounded" }
                ]
            }
        }));

        let validation = MindmapValidator::new().validate(&mindmap);
        let output = MindmapPlusGenerator::new()
            .generate(&mindmap, validation)
            .unwrap();

        assert!(output.diagram.contains("mindmap"));
        assert!(output.diagram.contains("[Project]"));
        assert!(output.diagram.contains("(Frontend)"));
        assert!(output.diagram.contains("((React))"));
        assert!(output.diagram.contains("(Backend)"));
    }

    #[test]
    fn test_generate_with_icons() {
        let mindmap = parse_mindmap(json!({
            "id": "test",
            "root": {
                "label": "Features",
                "icon": "🎯",
                "shape": "rounded",
                "children": [
                    { "label": "Auth", "icon": "🔐" }
                ]
            }
        }));

        let validation = MindmapValidator::new().validate(&mindmap);
        let output = MindmapPlusGenerator::new()
            .generate(&mindmap, validation)
            .unwrap();

        assert!(output.diagram.contains("🎯 Features"));
        assert!(output.diagram.contains("🔐 Auth"));
    }
}

// CODEGEN-END
