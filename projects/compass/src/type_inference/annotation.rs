//! Type annotation parsing
//!
//! This module handles parsing Python type annotations from AST nodes.

use tree_sitter::Node;

use super::ty::{Param, ParamKind, Type};

/// Parse a type annotation from an AST node
pub fn parse_type_annotation(source: &str, node: &Node) -> Type {
    let text = node_text(source, node);

    match node.kind() {
        "identifier" | "type" => parse_simple_type(text),
        "subscript" => parse_generic_type(source, node),
        "binary_operator" => {
            // Union type: X | Y
            let left = node.child_by_field_name("left");
            let right = node.child_by_field_name("right");
            match (left, right) {
                (Some(l), Some(r)) => {
                    let left_ty = parse_type_annotation(source, &l);
                    let right_ty = parse_type_annotation(source, &r);
                    Type::union(vec![left_ty, right_ty])
                }
                _ => Type::Unknown,
            }
        }
        "none" => Type::None,
        _ => parse_simple_type(text),
    }
}

/// Parse a simple type name
pub fn parse_simple_type(name: &str) -> Type {
    match name {
        "int" => Type::Int,
        "float" => Type::Float,
        "str" => Type::Str,
        "bool" => Type::Bool,
        "bytes" => Type::Bytes,
        "None" => Type::None,
        "Any" => Type::Any,
        "object" => Type::Any,
        // PEP 673: Self type
        "Self" => Type::SelfType { class_name: None },
        // PEP 675: LiteralString
        "LiteralString" => Type::LiteralString,
        _ => Type::Instance {
            name: name.to_string(),
            module: None,
            type_args: vec![],
        },
    }
}

/// Parse a generic type like list[int], dict[str, int]
pub fn parse_generic_type(source: &str, node: &Node) -> Type {
    let base = node
        .child_by_field_name("value")
        .map(|n| node_text(source, &n))
        .unwrap_or("");

    let args = parse_type_args(source, node);

    match base {
        "list" | "List" => Type::list(args.first().cloned().unwrap_or(Type::Unknown)),
        "dict" | "Dict" => {
            let key = args.first().cloned().unwrap_or(Type::Unknown);
            let val = args.get(1).cloned().unwrap_or(Type::Unknown);
            Type::dict(key, val)
        }
        "set" | "Set" => Type::Set(Box::new(args.first().cloned().unwrap_or(Type::Unknown))),
        "tuple" | "Tuple" => Type::Tuple(args),
        "Optional" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::optional(inner)
        }
        "Union" => Type::union(args),
        "Callable" => {
            // Callable[[arg_types], return_type]
            if args.len() >= 2 {
                let ret = args.last().cloned().unwrap_or(Type::Unknown);
                let params: Vec<_> = args[..args.len() - 1]
                    .iter()
                    .enumerate()
                    .map(|(i, ty)| Param {
                        name: format!("_{}", i),
                        ty: ty.clone(),
                        has_default: false,
                        kind: ParamKind::Positional,
                    })
                    .collect();
                Type::Callable {
                    params,
                    ret: Box::new(ret),
                }
            } else {
                Type::Unknown
            }
        }
        "type" | "Type" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            if let Type::Instance { name, module, .. } = inner {
                Type::ClassType { name, module }
            } else {
                Type::Unknown
            }
        }
        // PEP 591: Final
        "Final" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::Final(Box::new(inner))
        }
        // PEP 593: Annotated
        "Annotated" => {
            if args.is_empty() {
                Type::Unknown
            } else {
                let inner = args[0].clone();
                // Metadata is simplified - store as strings
                let metadata: Vec<String> = args[1..].iter().map(|t| t.to_string()).collect();
                Type::Annotated {
                    inner: Box::new(inner),
                    metadata,
                }
            }
        }
        // PEP 612: Concatenate
        "Concatenate" => {
            if args.is_empty() {
                Type::Unknown
            } else {
                let param_spec = args.last().cloned().unwrap_or(Type::Unknown);
                let params = args[..args.len().saturating_sub(1)].to_vec();
                Type::Concatenate {
                    params,
                    param_spec: Box::new(param_spec),
                }
            }
        }
        // PEP 646: Unpack
        "Unpack" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::Unpack(Box::new(inner))
        }
        // PEP 647: TypeGuard
        "TypeGuard" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::TypeGuard(Box::new(inner))
        }
        // PEP 742: TypeIs
        "TypeIs" => {
            let inner = args.first().cloned().unwrap_or(Type::Unknown);
            Type::TypeIs(Box::new(inner))
        }
        _ => Type::Instance {
            name: base.to_string(),
            module: None,
            type_args: args,
        },
    }
}

/// Parse type arguments from a subscript node
pub fn parse_type_args(source: &str, node: &Node) -> Vec<Type> {
    let mut args = Vec::new();

    if let Some(subscript) = node.child_by_field_name("subscript") {
        match subscript.kind() {
            "tuple" | "expression_list" => {
                let mut cursor = subscript.walk();
                for child in subscript.children(&mut cursor) {
                    if child.kind() != "," && child.kind() != "(" && child.kind() != ")" {
                        args.push(parse_type_annotation(source, &child));
                    }
                }
            }
            _ => {
                args.push(parse_type_annotation(source, &subscript));
            }
        }
    }

    args
}

/// Get text of a node
fn node_text<'a>(source: &'a str, node: &Node) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}
