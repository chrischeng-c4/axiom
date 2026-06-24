// HANDWRITE-BEGIN gap="missing-generator:logic:962dae38" tracker="pending-tracker" reason="Tree-sitter walk of a component file: locate the component's props type (interface or type alias referenced by the component's first param) and return an ordered list of (prop name, type text, optional flag)."
//! Component prop-type extraction for `jet stories` controls (B3).
//!
//! Given the source of a component file and the component's name, this module
//! finds the component's props type and returns one [`PropDef`] per declared
//! prop, preserving source order. The controls layer ([`super::controls`]) maps
//! each [`PropDef`] to an editable control widget.
//!
//! ## How the props type is found
//!
//! We locate the component declaration by name and read its first parameter's
//! type annotation, which names the props type. The common React shapes are:
//!
//! - `function Button(props: ButtonProps) { ... }`
//! - `const Button = (props: ButtonProps) => ...`
//! - `const Button: React.FC<ButtonProps> = (props) => ...`
//!   (the props type is the *type argument* of `React.FC<...>` / `FC<...>`)
//! - `function Button(props: { primary: boolean }) { ... }`
//!   (an inline object type — read directly)
//!
//! The named props type (`ButtonProps`) is then resolved against an `interface`
//! declaration or a `type` alias **in the same file**, both of which reduce to a
//! TS object type (`{ name: type; ... }`) whose members we read.
//!
//! ## Deferred (TODO(#175 follow-up))
//!
//! These shapes are recognized-but-skipped rather than mis-parsed; each returns
//! an empty/partial result gracefully instead of erroring:
//! - generic props types (`ButtonProps<T>`),
//! - props types imported from another file (cross-file resolution),
//! - intersection / union props types (`A & B`, `A | B`),
//! - mapped / conditional / utility types (`Partial<...>`, `Pick<...>`).

use tree_sitter::{Node, Parser};

/// One declared prop of a component's props type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropDef {
    /// The prop name (`primary`, `label`).
    pub name: String,
    /// The prop's TS type, as source text (`boolean`, `string`, `"sm" | "lg"`).
    pub type_text: String,
    /// Whether the prop is optional (declared with `?`).
    pub optional: bool,
}

/// Extract the ordered prop definitions of `component_name` from a component
/// source file.
///
/// Returns an empty vector (never an error) when the component, its props type,
/// or that type's definition can't be found — controls degrade to "no props"
/// rather than failing the whole preview.
pub fn extract_props(component_source: &str, component_name: &str) -> Vec<PropDef> {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
        .is_err()
    {
        return Vec::new();
    }
    let Some(tree) = parser.parse(component_source, None) else {
        return Vec::new();
    };
    let root = tree.root_node();

    // 1. Find the component's first-parameter props type annotation.
    let Some(props_type) = find_component_props_type(component_source, root, component_name) else {
        return Vec::new();
    };

    // 2. Resolve that type to an object type and read its members.
    match props_type {
        PropsType::Inline(obj) => read_object_type_members(component_source, obj),
        PropsType::Named(name) => resolve_named_props_type(component_source, root, &name),
    }
}

/// The props type of a component: either an inline object type node, or the
/// name of a type to resolve elsewhere in the file.
enum PropsType<'a> {
    Inline(Node<'a>),
    Named(String),
}

/// Find the props type of `component_name`, scanning top-level declarations for
/// a matching function/const and reading its props annotation.
fn find_component_props_type<'a>(
    source: &str,
    root: Node<'a>,
    component_name: &str,
) -> Option<PropsType<'a>> {
    for child in named_children(root) {
        match child.kind() {
            // `function Button(props: ButtonProps) { ... }`
            "function_declaration" => {
                if identifier_name(source, child).as_deref() == Some(component_name) {
                    if let Some(t) = props_type_from_params(source, child) {
                        return Some(t);
                    }
                }
            }
            // `const Button = (...) => ...` / `const Button: React.FC<...> = ...`
            "lexical_declaration" | "variable_declaration" => {
                if let Some(t) = props_type_from_declarators(source, child, component_name) {
                    return Some(t);
                }
            }
            // `export function Button(...)` / `export const Button = ...` /
            // `export default function Button(...)`.
            "export_statement" => {
                for inner in named_children(child) {
                    match inner.kind() {
                        "function_declaration" => {
                            if identifier_name(source, inner).as_deref() == Some(component_name) {
                                if let Some(t) = props_type_from_params(source, inner) {
                                    return Some(t);
                                }
                            }
                        }
                        "lexical_declaration" | "variable_declaration" => {
                            if let Some(t) =
                                props_type_from_declarators(source, inner, component_name)
                            {
                                return Some(t);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Read the props type from a `const NAME = ...` declaration matching
/// `component_name`.
fn props_type_from_declarators<'a>(
    source: &str,
    decl: Node<'a>,
    component_name: &str,
) -> Option<PropsType<'a>> {
    for declarator in named_children(decl) {
        if declarator.kind() != "variable_declarator" {
            continue;
        }
        let name = first_child_of_kind(declarator, "identifier")
            .map(|n| node_text(n, source).to_string());
        if name.as_deref() != Some(component_name) {
            continue;
        }

        // `const Button: React.FC<ButtonProps> = ...` — the props type is the
        // type argument of the variable's own type annotation.
        if let Some(type_ann) = first_child_of_kind(declarator, "type_annotation") {
            if let Some(arg) = fc_type_argument(source, type_ann) {
                return Some(arg);
            }
        }

        // Otherwise read the initializer's first parameter type:
        // `const Button = (props: ButtonProps) => ...`.
        for value in named_children(declarator) {
            match value.kind() {
                "arrow_function" | "function_expression" | "function" => {
                    if let Some(t) = props_type_from_params(source, value) {
                        return Some(t);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Extract the props type from a `React.FC<Props>` / `FC<Props>` type
/// annotation. The props type is the first type argument.
fn fc_type_argument<'a>(source: &str, type_annotation: Node<'a>) -> Option<PropsType<'a>> {
    // type_annotation -> generic_type { name, type_arguments }
    let generic = first_child_of_kind(type_annotation, "generic_type")?;
    let type_args = first_child_of_kind(generic, "type_arguments")?;
    // First named child of type_arguments is the first type argument.
    let first = named_children(type_args).into_iter().next()?;
    Some(props_type_from_type_node(source, first))
}

/// Read the props type from a callable node's parameter list (the first
/// parameter's type annotation).
fn props_type_from_params<'a>(source: &str, callable: Node<'a>) -> Option<PropsType<'a>> {
    let params = first_child_of_kind(callable, "formal_parameters")?;
    // The first required/optional parameter.
    let first_param = named_children(params)
        .into_iter()
        .find(|p| matches!(p.kind(), "required_parameter" | "optional_parameter"))?;
    let type_ann = first_child_of_kind(first_param, "type_annotation")?;
    // type_annotation's payload is its single non-trivial child.
    let type_node = named_children(type_ann).into_iter().next()?;
    Some(props_type_from_type_node(source, type_node))
}

/// Classify a type node as an inline object type or a named type reference.
fn props_type_from_type_node<'a>(source: &str, type_node: Node<'a>) -> PropsType<'a> {
    match type_node.kind() {
        // `{ primary: boolean; ... }`
        "object_type" => PropsType::Inline(type_node),
        // `ButtonProps`
        "type_identifier" => PropsType::Named(node_text(type_node, source).to_string()),
        // `Props.Whatever` or generic — TODO(#175 follow-up): cross-file /
        // generic / qualified props types are not resolved; treat the leading
        // identifier as a best-effort name so same-file matches still work.
        _ => PropsType::Named(node_text(type_node, source).to_string()),
    }
}

/// Resolve a named props type (`ButtonProps`) to its members by finding the
/// matching `interface` or `type` alias declaration in the same file.
fn resolve_named_props_type(source: &str, root: Node, type_name: &str) -> Vec<PropDef> {
    for child in named_children(root) {
        let decl = match child.kind() {
            "interface_declaration" | "type_alias_declaration" => child,
            "export_statement" => match named_children(child).into_iter().find(|n| {
                matches!(
                    n.kind(),
                    "interface_declaration" | "type_alias_declaration"
                )
            }) {
                Some(d) => d,
                None => continue,
            },
            _ => continue,
        };

        // The declared type name is the `type_identifier` child.
        let name = first_child_of_kind(decl, "type_identifier")
            .map(|n| node_text(n, source).to_string());
        if name.as_deref() != Some(type_name) {
            continue;
        }

        match decl.kind() {
            "interface_declaration" => {
                // interface ... { interface_body / object_type }
                if let Some(body) = first_child_of_kind(decl, "interface_body")
                    .or_else(|| first_child_of_kind(decl, "object_type"))
                {
                    return read_object_type_members(source, body);
                }
            }
            "type_alias_declaration" => {
                // type ButtonProps = { ... };  — read the object_type rhs.
                // TODO(#175 follow-up): intersections / unions / utility types
                // on the rhs are not destructured; only a direct object type is.
                if let Some(obj) = named_children(decl)
                    .into_iter()
                    .find(|n| n.kind() == "object_type")
                {
                    return read_object_type_members(source, obj);
                }
            }
            _ => {}
        }
    }
    Vec::new()
}

/// Read the `name: type` members of a TS object/interface body in source order.
///
/// Each `property_signature` contributes a [`PropDef`]; the `?` token marks the
/// prop optional. Index signatures, method signatures, and spreads are skipped.
fn read_object_type_members(source: &str, body: Node) -> Vec<PropDef> {
    let mut out = Vec::new();
    for member in named_children(body) {
        if member.kind() != "property_signature" {
            // TODO(#175 follow-up): method_signature / index_signature /
            // call_signature members are not surfaced as props.
            continue;
        }
        let Some(name_node) = first_child_of_kind(member, "property_identifier") else {
            continue;
        };
        let name = node_text(name_node, source).to_string();
        let optional = member_is_optional(member);
        let type_text = first_child_of_kind(member, "type_annotation")
            .and_then(|ann| named_children(ann).into_iter().next())
            .map(|t| node_text(t, source).trim().to_string())
            .unwrap_or_default();
        out.push(PropDef {
            name,
            type_text,
            optional,
        });
    }
    out
}

/// A property signature is optional when it contains a `?` token between the
/// name and the type annotation. tree-sitter exposes it as an anonymous `?`
/// child, so scan the member's raw children for it.
fn member_is_optional(member: Node) -> bool {
    let mut cursor = member.walk();
    let optional = member.children(&mut cursor).any(|c| c.kind() == "?");
    optional
}

// ── AST helpers (mirrors the small set used in csf.rs / frontend.rs) ──────────

fn named_children(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

fn first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    named_children(node).into_iter().find(|c| c.kind() == kind)
}

fn identifier_name(source: &str, node: Node) -> Option<String> {
    first_child_of_kind(node, "identifier").map(|n| node_text(n, source).to_string())
}

fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUTTON: &str = r#"
import React from 'react';

interface ButtonProps {
  primary: boolean;
  label: string;
  size: "sm" | "lg";
  count?: number;
}

export function Button(props: ButtonProps) {
  return null;
}
"#;

    #[test]
    fn extracts_interface_props_in_order() {
        let props = extract_props(BUTTON, "Button");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["primary", "label", "size", "count"]);
        assert_eq!(props[0].type_text, "boolean");
        assert_eq!(props[1].type_text, "string");
        assert_eq!(props[2].type_text, "\"sm\" | \"lg\"");
        assert_eq!(props[3].type_text, "number");
        assert!(!props[0].optional);
        assert!(props[3].optional, "count? is optional");
    }

    #[test]
    fn extracts_from_arrow_const() {
        let src = r#"
type CardProps = { title: string; elevated: boolean };
export const Card = (props: CardProps) => null;
"#;
        let props = extract_props(src, "Card");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["title", "elevated"]);
    }

    #[test]
    fn extracts_from_react_fc() {
        let src = r#"
import React from 'react';
interface BadgeProps { text: string; }
const Badge: React.FC<BadgeProps> = (props) => null;
"#;
        let props = extract_props(src, "Badge");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].name, "text");
        assert_eq!(props[0].type_text, "string");
    }

    #[test]
    fn extracts_inline_object_type() {
        let src = r#"
export function Tag(props: { name: string; closable?: boolean }) { return null; }
"#;
        let props = extract_props(src, "Tag");
        let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["name", "closable"]);
        assert!(props[1].optional);
    }

    #[test]
    fn unknown_component_yields_empty() {
        assert!(extract_props(BUTTON, "Missing").is_empty());
    }
}
// HANDWRITE-END
