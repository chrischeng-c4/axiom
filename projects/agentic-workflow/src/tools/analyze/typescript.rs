// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/analyze/typescript.md#source
// CODEGEN-BEGIN
//! TypeScript/JavaScript source code analysis using tree-sitter

use super::{AnalysisResult, ClassInfo, FunctionInfo, ParamInfo};
use crate::Result;

/// Analyze TypeScript/JavaScript source code
/// @spec projects/agentic-workflow/tech-design/core/tools/analyze/typescript.md#source
pub fn analyze(source: &str) -> Result<AnalysisResult> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript source"))?;

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

/// Recursively extract TypeScript nodes
fn extract_nodes(
    node: &tree_sitter::Node,
    source: &str,
    functions: &mut Vec<FunctionInfo>,
    classes: &mut Vec<ClassInfo>,
    patterns: &mut Vec<String>,
) {
    match node.kind() {
        "function_declaration" | "arrow_function" | "method_definition" => {
            if let Some(func) = extract_function(node, source) {
                functions.push(func);
            }
        }
        "class_declaration" => {
            if let Some(class) = extract_class(node, source) {
                classes.push(class);
            }
        }
        "interface_declaration" | "type_alias_declaration" => {
            if let Some(class) = extract_interface(node, source) {
                if !patterns.contains(&"data-model".to_string()) {
                    patterns.push("data-model".to_string());
                }
                classes.push(class);
            }
        }
        "call_expression" => {
            let text = node_text(node, source);
            if text.contains(".get(")
                || text.contains(".post(")
                || text.contains(".put(")
                || text.contains(".delete(")
                || text.contains("router.")
                || text.contains("app.")
            {
                if !patterns.contains(&"http-api".to_string()) {
                    patterns.push("http-api".to_string());
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
        .map(|n| node_text(&n, source).to_string())
        .unwrap_or_else(|| "<anonymous>".to_string());

    let mut params = Vec::new();
    if let Some(params_node) = node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "required_parameter" || child.kind() == "optional_parameter" {
                if let Some(pattern) = child.child_by_field_name("pattern") {
                    let param_name = node_text(&pattern, source).to_string();
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

    let return_type = node
        .child_by_field_name("return_type")
        .map(|n| node_text(&n, source).to_string());

    let is_async = node.kind().contains("async") || {
        let text = node_text(node, source);
        text.starts_with("async ")
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

fn extract_class(node: &tree_sitter::Node, source: &str) -> Option<ClassInfo> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(&n, source).to_string())?;

    let mut bases = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "class_heritage" {
            let text = node_text(&child, source);
            bases.push(
                text.replace("extends ", "")
                    .replace("implements ", "")
                    .trim()
                    .to_string(),
            );
        }
    }

    Some(ClassInfo {
        name,
        bases,
        fields: Vec::new(),
        methods: Vec::new(),
        doc: None,
    })
}

fn extract_interface(node: &tree_sitter::Node, source: &str) -> Option<ClassInfo> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(&n, source).to_string())?;

    Some(ClassInfo {
        name,
        bases: Vec::new(),
        fields: Vec::new(),
        methods: Vec::new(),
        doc: None,
    })
}

fn node_text<'a>(node: &tree_sitter::Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

// CODEGEN-END
