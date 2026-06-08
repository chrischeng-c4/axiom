// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/analyze/rust_lang.md#source
// CODEGEN-BEGIN
//! Rust source code analysis using tree-sitter

use super::{AnalysisResult, ClassInfo, FieldInfo, FunctionInfo, ParamInfo};
use crate::Result;

/// Analyze Rust source code
/// @spec projects/agentic-workflow/tech-design/core/tools/analyze/rust_lang.md#source
pub fn analyze(source: &str) -> Result<AnalysisResult> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse Rust source"))?;

    let mut functions = Vec::new();
    let mut classes = Vec::new();
    let mut patterns = Vec::new();

    let root = tree.root_node();
    extract_nodes(&root, source, &mut functions, &mut classes, &mut patterns);

    Ok(AnalysisResult {
        functions,
        classes,
        detected_patterns: patterns,
    })
}

/// Recursively extract Rust nodes
fn extract_nodes(
    node: &tree_sitter::Node,
    source: &str,
    functions: &mut Vec<FunctionInfo>,
    classes: &mut Vec<ClassInfo>,
    patterns: &mut Vec<String>,
) {
    match node.kind() {
        "function_item" => {
            if let Some(func) = extract_function(node, source) {
                functions.push(func);
            }
        }
        "struct_item" => {
            if let Some(class) = extract_struct(node, source) {
                classes.push(class);
            }
        }
        "impl_item" => {
            let text = node_text(node, source);
            if text.contains("impl Handler") || text.contains("impl Service") {
                if !patterns.contains(&"http-api".to_string()) {
                    patterns.push("http-api".to_string());
                }
            }
        }
        "attribute_item" => {
            let text = node_text(node, source);
            if text.contains("get(")
                || text.contains("post(")
                || text.contains("put(")
                || text.contains("delete(")
                || text.contains("route(")
            {
                if !patterns.contains(&"http-api".to_string()) {
                    patterns.push("http-api".to_string());
                }
            }
            if text.contains("derive")
                && (text.contains("Serialize") || text.contains("Deserialize"))
            {
                if !patterns.contains(&"data-model".to_string()) {
                    patterns.push("data-model".to_string());
                }
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_nodes(&child, source, functions, classes, patterns);
    }
}

fn extract_function(node: &tree_sitter::Node, source: &str) -> Option<FunctionInfo> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(&n, source).to_string())?;

    let mut params = Vec::new();
    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "parameter" {
                if let Some(pattern) = child.child_by_field_name("pattern") {
                    let param_name = node_text(&pattern, source).to_string();
                    if param_name != "self" && param_name != "&self" && param_name != "&mut self" {
                        let type_ann = child
                            .child_by_field_name("type")
                            .map(|t| node_text(&t, source).to_string());
                        params.push(ParamInfo {
                            name: param_name,
                            type_annotation: type_ann,
                        });
                    }
                }
            }
        }
    }

    let return_type = node
        .child_by_field_name("return_type")
        .map(|n| node_text(&n, source).to_string());

    let is_async = {
        let text = node_text(node, source);
        text.contains("async fn")
    };

    Some(FunctionInfo {
        name,
        params,
        return_type,
        decorators: Vec::new(),
        is_async,
        doc: None,
    })
}

fn extract_struct(node: &tree_sitter::Node, source: &str) -> Option<ClassInfo> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(&n, source).to_string())?;

    let mut fields = Vec::new();
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                if let Some(fname) = child.child_by_field_name("name") {
                    let field_name = node_text(&fname, source).to_string();
                    let type_ann = child
                        .child_by_field_name("type")
                        .map(|t| node_text(&t, source).to_string());
                    fields.push(FieldInfo {
                        name: field_name,
                        type_annotation: type_ann,
                    });
                }
            }
        }
    }

    Some(ClassInfo {
        name,
        bases: Vec::new(),
        fields,
        methods: Vec::new(),
        doc: None,
    })
}

fn node_text<'a>(node: &tree_sitter::Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

// CODEGEN-END
