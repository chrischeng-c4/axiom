---
id: sdd-mindmap_plus-generator-output
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# MindmapPlusOutput

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MindmapPlusGenerator` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs | struct | pub | 18 |  |
| `MindmapPlusOutput` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs | struct | pub | 11 |  |
| `generate` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs | function | pub | 26 | generate(         &self,         mindmap: &MindmapDef,         validation: MindmapValidationResult,     ) -> Result<MindmapPlusOutput, String> |
| `generate_mermaid` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs | function | pub | 64 | generate_mermaid(&self, mindmap: &MindmapDef) -> Result<String, String> |
| `new` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs | function | pub | 22 | new() -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  MindmapPlusOutput:
    type: object
    required: [frontmatter, diagram, validation, combined]
    description: Output of the Mermaid Plus generator.
    properties:
      frontmatter: { type: string }
      diagram: { type: string }
      validation:
        type: object
        x-rust-type: MindmapValidationResult
      combined: { type: string }
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs -->
````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/mindmap_plus/generator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Mindmap+ generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
