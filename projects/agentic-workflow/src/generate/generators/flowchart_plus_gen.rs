// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen_preamble.md#source
// CODEGEN-BEGIN
//! Flowchart+ code generator
//!
//! Generates Python function skeletons from a [`FlowchartDef`]
//! (flowchart/logic section type) with YAML metadata:
//!
//! | Output file                 | Description                                    |
//! |-----------------------------|------------------------------------------------|
//! | `{flowchart_id}_flow.py`    | Python function skeletons with `@sdd:implement` markers |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::FlowchartPlus`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::diagrams::{
    FlowchartDef, FlowchartEdgeDef as EdgeDef, FlowchartNodeDef as NodeDef, NodeShape, SemanticType,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::SpecIR;

// ---------------------------------------------------------------------------
// FlowchartPlusGenerator
// ---------------------------------------------------------------------------
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen.md#schema
// CODEGEN-BEGIN
/// FlowchartPlus generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen.md#schema
pub struct FlowchartPlusGenerator;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen_runtime.md#source
impl FlowchartPlusGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen_runtime.md#source
impl Default for FlowchartPlusGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SpecIRGenerator impl
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen_runtime.md#source
impl SpecIRGenerator for FlowchartPlusGenerator {
    fn can_generate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::FlowchartPlus { .. })
    }

    fn template_dir(&self) -> &'static str {
        "flowchart_plus"
    }

    fn generate_from_ir(
        &self,
        spec: &SpecIR,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError> {
        let fc_def = match spec {
            SpecIR::FlowchartPlus { def, .. } => def,
            _ => {
                return Err(GeneratorError::SchemaError(
                    "FlowchartPlusGenerator: expected SpecIR::FlowchartPlus variant".into(),
                ))
            }
        };

        let mut manifest = Manifest::new();

        let file_name = format!("{}_flow.py", fc_def.id.replace('-', "_"));
        let output_path = settings.output_dir.join(&file_name);

        if output_path.exists() {
            match settings.overwrite_policy {
                OverwritePolicy::Error => {
                    return Err(GeneratorError::OverwriteNotAllowed(output_path));
                }
                OverwritePolicy::Skip => {
                    manifest.add(GeneratedFile::skipped(output_path));
                    return Ok(manifest);
                }
                OverwritePolicy::Overwrite => {}
            }
        }

        let template_name = format!("{}/flow.py.j2", self.template_dir());
        let content = if engine.has_template(&template_name) {
            engine.render(&template_name, &fc_def).map_err(|e| {
                GeneratorError::TemplateRenderError {
                    template: template_name.clone(),
                    message: e.to_string(),
                }
            })?
        } else {
            generate_flowchart_python(fc_def)
        };

        manifest.add(GeneratedFile::written(output_path, &content));
        Ok(manifest)
    }
}

// ---------------------------------------------------------------------------
// Inline generator
// ---------------------------------------------------------------------------

fn to_snake(s: &str) -> String {
    use heck::ToSnakeCase;
    s.to_snake_case()
}

/// Determine the function signature from a node's semantic type.
fn node_signature(node_id: &str, node: &NodeDef) -> (String, String, String) {
    let fn_name = to_snake(node_id);

    match &node.semantic {
        Some(SemanticType::Validation { input, .. }) => {
            (fn_name, format!("{}: Any", input), "bool".into())
        }
        Some(SemanticType::Condition { expression }) => {
            let param = expression.split('.').next().unwrap_or("ctx");
            (fn_name, format!("{}: Any", param), "bool".into())
        }
        Some(SemanticType::DbQuery { output, .. }) => {
            let ret = output.as_deref().unwrap_or("Any");
            (fn_name, "db: Any".into(), ret.to_string())
        }
        Some(SemanticType::DbMutation { .. }) => {
            (fn_name, "db: Any, data: Any".into(), "None".into())
        }
        Some(SemanticType::ApiCall { output, .. }) => {
            let ret = output.as_deref().unwrap_or("Any");
            (fn_name, "client: Any".into(), ret.to_string())
        }
        Some(SemanticType::Transform { input, output, .. }) => {
            (fn_name, format!("{}: Any", input), output.clone())
        }
        Some(SemanticType::Start) | Some(SemanticType::End { .. }) => {
            // Skip start/end nodes - they don't become functions
            (fn_name, String::new(), String::new())
        }
        _ => {
            // Default: generic function
            (fn_name, "request: Any".into(), "Any".into())
        }
    }
}

/// Check if a node is a decision (diamond/condition) that routes to other nodes.
fn is_decision_node(node: &NodeDef) -> bool {
    matches!(node.shape, NodeShape::Diamond)
        || matches!(&node.semantic, Some(SemanticType::Condition { .. }))
}

/// Check if a node is a start/end node (not a real function).
fn is_terminal_node(node: &NodeDef) -> bool {
    matches!(
        &node.semantic,
        Some(SemanticType::Start) | Some(SemanticType::End { .. })
    )
}

fn generate_flowchart_python(def: &FlowchartDef) -> String {
    let mut output = String::from("# Generated by sdd\nfrom typing import Any\n\n");

    // Collect non-terminal nodes that will become functions
    let mut function_nodes: Vec<(&String, &NodeDef)> = def
        .nodes
        .iter()
        .filter(|(_, node)| !is_terminal_node(node))
        .collect();
    // Sort for determinism
    function_nodes.sort_by_key(|(id, _)| *id);

    // Build adjacency info for decision nodes
    let edges_from = |node_id: &str| -> Vec<&EdgeDef> {
        def.edges.iter().filter(|e| e.from == node_id).collect()
    };

    for (node_id, node) in &function_nodes {
        let (fn_name, params, return_type) = node_signature(node_id, node);

        if params.is_empty() && return_type.is_empty() {
            // Terminal node - skip
            continue;
        }

        let marker = to_snake(node_id);

        if is_decision_node(node) {
            // Decision node: generate an if/else skeleton based on outgoing edges
            let outgoing = edges_from(node_id);

            output.push_str(&format!(
                "def {}({}) -> {}:\n",
                fn_name, params, return_type
            ));

            if outgoing.len() >= 2 {
                let mut first = true;
                for edge in &outgoing {
                    let branch_label = edge
                        .label
                        .as_deref()
                        .or(edge.condition.as_deref())
                        .unwrap_or("condition");
                    let target_fn = to_snake(&edge.to);

                    if first {
                        output.push_str(&format!(
                            "    if {}:  # {}\n        # @sdd:implement {}\n        pass\n",
                            branch_label, branch_label, target_fn
                        ));
                        first = false;
                    } else {
                        output.push_str(&format!(
                            "    else:  # {}\n        # @sdd:implement {}\n        pass\n",
                            branch_label, target_fn
                        ));
                    }
                }
            } else {
                output.push_str(&format!("    # @sdd:implement {}\n    pass\n", marker));
            }
        } else {
            // Regular function node
            output.push_str(&format!(
                "def {}({}) -> {}:\n    # @sdd:implement {}\n    pass\n",
                fn_name, params, return_type, marker
            ));
        }

        output.push('\n');
    }

    output
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::diagrams::{
        FlowchartDef, FlowchartEdgeDef as EdgeDef, FlowchartNodeDef as NodeDef, NodeShape,
        SemanticType,
    };
    use crate::generate::spec_ir::{SpecIR, SpecMetadata};
    use indexmap::IndexMap;

    fn auth_flow_spec() -> SpecIR {
        let mut nodes = IndexMap::new();
        nodes.insert(
            "start".to_string(),
            NodeDef {
                label: "Start".into(),
                shape: NodeShape::Rounded,
                semantic: Some(SemanticType::Start),
                description: None,
                ..Default::default()
            },
        );
        nodes.insert(
            "validate_token".to_string(),
            NodeDef {
                label: "Validate Token".into(),
                shape: NodeShape::Rectangle,
                semantic: Some(SemanticType::Validation {
                    input: "req".into(),
                    rules: vec!["required: token".into()],
                    error_code: Some(401),
                    error_message: None,
                }),
                description: None,
                ..Default::default()
            },
        );
        nodes.insert(
            "check_auth".to_string(),
            NodeDef {
                label: "Check Auth".into(),
                shape: NodeShape::Diamond,
                semantic: Some(SemanticType::Condition {
                    expression: "is_valid".into(),
                }),
                description: None,
                ..Default::default()
            },
        );
        nodes.insert(
            "allow".to_string(),
            NodeDef {
                label: "Allow".into(),
                shape: NodeShape::Rectangle,
                semantic: None,
                description: None,
                ..Default::default()
            },
        );
        nodes.insert(
            "deny".to_string(),
            NodeDef {
                label: "Deny".into(),
                shape: NodeShape::Rectangle,
                semantic: None,
                description: None,
                ..Default::default()
            },
        );

        let edges = vec![
            EdgeDef {
                from: "start".into(),
                to: "validate_token".into(),
                label: None,
                style: Default::default(),
                condition: None,
                is_error_path: false,
            },
            EdgeDef {
                from: "validate_token".into(),
                to: "check_auth".into(),
                label: None,
                style: Default::default(),
                condition: None,
                is_error_path: false,
            },
            EdgeDef {
                from: "check_auth".into(),
                to: "allow".into(),
                label: Some("valid".into()),
                style: Default::default(),
                condition: None,
                is_error_path: false,
            },
            EdgeDef {
                from: "check_auth".into(),
                to: "deny".into(),
                label: Some("invalid".into()),
                style: Default::default(),
                condition: None,
                is_error_path: true,
            },
        ];

        SpecIR::FlowchartPlus {
            def: FlowchartDef {
                id: "auth-flow".into(),
                direction: Default::default(),
                nodes,
                edges,
                subgraphs: vec![],
                description: None,
            },
            metadata: SpecMetadata::default(),
        }
    }

    #[test]
    fn test_can_generate_flowchart() {
        let gen = FlowchartPlusGenerator::new();
        assert!(gen.can_generate(&auth_flow_spec()));
    }

    #[test]
    fn test_cannot_generate_non_flowchart() {
        use crate::generate::schema::JsonSchema;
        let gen = FlowchartPlusGenerator::new();
        let api_spec = SpecIR::Api {
            schema: JsonSchema::default(),
            metadata: SpecMetadata::default(),
        };
        assert!(!gen.can_generate(&api_spec));
    }

    #[test]
    fn test_generate_produces_one_file() {
        let spec = auth_flow_spec();
        let settings = GeneratorSettings {
            output_dir: std::path::PathBuf::from("/tmp/test_fc_gen"),
            ..Default::default()
        };
        let engine = crate::generate::engine::TemplateEngine::empty();
        let gen = FlowchartPlusGenerator::new();
        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();

        assert_eq!(manifest.files.len(), 1);
        let name = manifest
            .files
            .keys()
            .next()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(name, "auth_flow_flow.py");
    }

    #[test]
    fn test_generated_python_content() {
        let def = match &auth_flow_spec() {
            SpecIR::FlowchartPlus { def, .. } => def.clone(),
            _ => unreachable!(),
        };
        let content = generate_flowchart_python(&def);

        // Should have function skeletons
        assert!(content.contains("def validate_token("));
        assert!(content.contains("-> bool"));
        assert!(content.contains("# @sdd:implement"));
        // Decision node should have if/else
        assert!(content.contains("def check_auth("));
        assert!(content.contains("if "));
        assert!(content.contains("else:"));
        // Should not have a function for "start" (terminal node)
        assert!(!content.contains("def start("));
    }

    #[test]
    fn test_generated_python_simple_nodes() {
        let mut nodes = IndexMap::new();
        nodes.insert(
            "process_data".to_string(),
            NodeDef {
                label: "Process Data".into(),
                shape: NodeShape::Rectangle,
                semantic: Some(SemanticType::Transform {
                    input: "raw_data".into(),
                    output: "ProcessedData".into(),
                    expression: None,
                }),
                description: None,
                ..Default::default()
            },
        );

        let def = FlowchartDef {
            id: "simple-flow".into(),
            direction: Default::default(),
            nodes,
            edges: vec![],
            subgraphs: vec![],
            description: None,
        };
        let content = generate_flowchart_python(&def);

        assert!(content.contains("def process_data(raw_data: Any) -> ProcessedData:"));
        assert!(content.contains("# @sdd:implement process_data"));
    }
}
// CODEGEN-END
