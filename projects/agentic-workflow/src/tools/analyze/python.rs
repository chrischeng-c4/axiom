// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/analyze/python.md#source
// CODEGEN-BEGIN
//! Python source code analysis using tree-sitter

use super::{AnalysisResult, ClassInfo, FieldInfo, FunctionInfo, ParamInfo};
use crate::Result;

/// Analyze Python source code
/// @spec projects/agentic-workflow/tech-design/core/tools/analyze/python.md#source
pub fn analyze(source: &str) -> Result<AnalysisResult> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into())?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse Python source"))?;

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

/// Recursively extract Python nodes
fn extract_nodes(
    node: &tree_sitter::Node,
    source: &str,
    functions: &mut Vec<FunctionInfo>,
    classes: &mut Vec<ClassInfo>,
    patterns: &mut Vec<String>,
) {
    match node.kind() {
        "function_definition" | "async_function_definition" => {
            if let Some(func) = extract_function(node, source) {
                for dec in &func.decorators {
                    if dec.contains("route")
                        || dec.contains("get")
                        || dec.contains("post")
                        || dec.contains("put")
                        || dec.contains("delete")
                        || dec.contains("api")
                    {
                        if !patterns.contains(&"http-api".to_string()) {
                            patterns.push("http-api".to_string());
                        }
                    }
                    if dec.contains("event") || dec.contains("handler") || dec.contains("subscribe")
                    {
                        if !patterns.contains(&"event-driven".to_string()) {
                            patterns.push("event-driven".to_string());
                        }
                    }
                }
                functions.push(func);
            }
            return;
        }
        "class_definition" => {
            if let Some(class) = extract_class(node, source) {
                for base in &class.bases {
                    if base.contains("BaseModel")
                        || base.contains("Schema")
                        || base.contains("Model")
                        || base.contains("DataClass")
                    {
                        if !patterns.contains(&"data-model".to_string()) {
                            patterns.push("data-model".to_string());
                        }
                    }
                }
                classes.push(class);
            }
            return;
        }
        "decorated_definition" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_nodes(&child, source, functions, classes, patterns);
            }
            return;
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_nodes(&child, source, functions, classes, patterns);
    }
}

/// Extract function info from Python AST node
fn extract_function(node: &tree_sitter::Node, source: &str) -> Option<FunctionInfo> {
    let (func_node, decorator_node) = if node.kind() == "decorated_definition" {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        let func = children
            .iter()
            .find(|c| c.kind() == "function_definition" || c.kind() == "async_function_definition")
            .copied()?;
        (func, Some(*node))
    } else {
        (*node, None)
    };

    let name_node = func_node.child_by_field_name("name")?;
    let name = node_text(&name_node, source).to_string();

    let func_text = node_text(&func_node, source);
    let is_async = func_node.kind() == "async_function_definition"
        || func_text.trim_start().starts_with("async ");

    let mut params = Vec::new();
    if let Some(params_node) = func_node.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let param_name = node_text(&child, source);
                    if param_name != "self" && param_name != "cls" {
                        params.push(ParamInfo {
                            name: param_name.to_string(),
                            type_annotation: None,
                        });
                    }
                }
                "typed_parameter" | "typed_default_parameter" => {
                    if let Some(pname) = child.child_by_field_name("name") {
                        let param_name = node_text(&pname, source);
                        if param_name != "self" && param_name != "cls" {
                            let type_ann = child
                                .child_by_field_name("type")
                                .map(|t| node_text(&t, source).to_string());
                            params.push(ParamInfo {
                                name: param_name.to_string(),
                                type_annotation: type_ann,
                            });
                        }
                    }
                }
                "default_parameter" => {
                    if let Some(pname) = child.child_by_field_name("name") {
                        let param_name = node_text(&pname, source);
                        params.push(ParamInfo {
                            name: param_name.to_string(),
                            type_annotation: None,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    let return_type = func_node
        .child_by_field_name("return_type")
        .map(|n| node_text(&n, source).to_string());

    let mut decorators = Vec::new();
    if let Some(dec_node) = decorator_node {
        let mut cursor = dec_node.walk();
        for child in dec_node.children(&mut cursor) {
            if child.kind() == "decorator" {
                decorators.push(node_text(&child, source).to_string());
            }
        }
    } else {
        let mut prev = node.prev_sibling();
        while let Some(sibling) = prev {
            if sibling.kind() == "decorator" {
                decorators.push(node_text(&sibling, source).to_string());
            } else {
                break;
            }
            prev = sibling.prev_sibling();
        }
    }

    let doc = extract_docstring(&func_node, source);

    Some(FunctionInfo {
        name,
        params,
        return_type,
        decorators,
        is_async,
        doc,
    })
}

/// Extract class info from Python AST node
fn extract_class(node: &tree_sitter::Node, source: &str) -> Option<ClassInfo> {
    let name_node = node.child_by_field_name("name")?;
    let name = node_text(&name_node, source).to_string();

    let mut bases = Vec::new();
    if let Some(args) = node.child_by_field_name("superclasses") {
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "attribute" {
                bases.push(node_text(&child, source).to_string());
            }
        }
    }

    let mut fields = Vec::new();
    let mut methods = Vec::new();

    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            match child.kind() {
                "function_definition" | "async_function_definition" => {
                    if let Some(func) = extract_function(&child, source) {
                        methods.push(func);
                    }
                }
                "expression_statement" => {
                    if let Some(assign) = child.child(0) {
                        if assign.kind() == "assignment" {
                            if let Some(left) = assign.child_by_field_name("left") {
                                if left.kind() == "identifier" {
                                    let field_name = node_text(&left, source).to_string();
                                    let type_ann = assign
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
                }
                _ => {}
            }
        }
    }

    let doc = extract_docstring(node, source);

    Some(ClassInfo {
        name,
        bases,
        fields,
        methods,
        doc,
    })
}

/// Extract docstring from Python function or class
fn extract_docstring(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let body = node.child_by_field_name("body")?;
    let mut cursor = body.walk();
    let first_child = body.children(&mut cursor).next()?;

    if first_child.kind() == "expression_statement" {
        if let Some(expr) = first_child.child(0) {
            if expr.kind() == "string" {
                let text = node_text(&expr, source);
                let doc = text
                    .trim_start_matches("\"\"\"")
                    .trim_start_matches("'''")
                    .trim_end_matches("\"\"\"")
                    .trim_end_matches("'''")
                    .trim();
                return Some(doc.to_string());
            }
        }
    }
    None
}

fn node_text<'a>(node: &tree_sitter::Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

// CODEGEN-END
