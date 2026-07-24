//! Syntax-only lowering for Python-authored UI tech designs.
//!
//! The author writes concise, ordinary Python declarations decorated with
//! `@page` and `@component`. This service never imports or executes that
//! module: it parses the restricted authoring syntax with tree-sitter and
//! lowers it into the existing wireframe/component/design-token IR structs.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-python-ui-component-lowering.md#logic

use anyhow::{bail, Context, Result};
use serde::Serialize;
use tree_sitter::{Node, Parser};

use crate::generate::spec_ir::{
    AttributeDef, ComponentSpec, DesignTokenEntry, DesignTokenSpec, EventDef, PropDef, SlotDef,
    WireframeNode, WireframeSpec,
};

/// Fully lowered UI sections from one Python TD module.
#[derive(Debug, Clone, Serialize)]
pub struct PythonUiTd {
    pub wireframe: WireframeSpec,
    pub components: Vec<ComponentSpec>,
    pub design_tokens: Option<DesignTokenSpec>,
}

/// Parse a restricted Python UI TD source file without importing it.
///
/// Supported authoring forms are deliberately small:
///
/// ```python
/// @component("Short summary")
/// def TaskRow(todo: Todo, on_toggle: Event[TodoId]): ...
///
/// @page
/// def TodoPage(todos: list[Todo]):
///     return AppShell(main=TaskList(todos=todos, item=TaskRow()))
///
/// token("color.brand", "#b9f2dc", "color")
/// ```
///
/// Exactly one `@page` function is required. `Event[T]` parameters lower to
/// custom events, `Slot[T]` parameters lower to named slots, and all other
/// typed parameters become reflected component attributes. Token calls are
/// optional and intentionally use positional literals to keep the accepted
/// syntax deterministic.
pub fn lower_python_ui_td(source: &str) -> Result<PythonUiTd> {
    let tree = parse(source)?;
    let root = tree.root_node();
    let mut page = None;
    let mut components = Vec::new();
    let mut tokens = Vec::new();

    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor) {
        match node.kind() {
            "decorated_definition" => {
                let (decorators, definition) = decorated_definition(node, source)?;
                let Some(definition) = definition else {
                    continue;
                };
                let name = function_name(definition, source)?;
                if decorators.iter().any(|decorator| decorator.name == "page") {
                    if page.is_some() {
                        bail!("Python UI TD diagnostic [duplicate-page]: declare exactly one @page function per module");
                    }
                    page = Some(lower_page(definition, &name, source)?);
                }
                if let Some(decorator) = decorators
                    .iter()
                    .find(|decorator| decorator.name == "component")
                {
                    components.push(lower_component(
                        definition,
                        &name,
                        decorator.summary.as_deref(),
                        source,
                    )?);
                }
            }
            "expression_statement" => {
                if let Some(token) = lower_token(node, source)? {
                    tokens.push(token);
                }
            }
            _ => {}
        }
    }

    let Some(wireframe) = page else {
        bail!("Python UI TD diagnostic [missing-page]: declare one @page function that returns a component tree");
    };
    components.sort_by(|left, right| left.tag_name.cmp(&right.tag_name));
    tokens.sort_by(|left, right| left.path.cmp(&right.path));
    if tokens.windows(2).any(|pair| pair[0].path == pair[1].path) {
        bail!("Python UI TD diagnostic [duplicate-token]: token paths must be unique");
    }
    Ok(PythonUiTd {
        wireframe,
        components,
        design_tokens: (!tokens.is_empty()).then_some(DesignTokenSpec {
            name: "ui".to_string(),
            tokens,
        }),
    })
}

#[derive(Debug)]
struct Decorator {
    name: String,
    summary: Option<String>,
}

fn parse(source: &str) -> Result<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into())?;
    let tree = parser
        .parse(source, None)
        .context("tree-sitter returned no Python syntax tree")?;
    if tree.root_node().has_error() {
        bail!("Python UI TD diagnostic [syntax-error]: repair the Python syntax before lowering UI components");
    }
    Ok(tree)
}

fn decorated_definition<'a>(
    node: Node<'a>,
    source: &'a str,
) -> Result<(Vec<Decorator>, Option<Node<'a>>)> {
    let mut decorators = Vec::new();
    let mut definition = None;
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "decorator" => decorators.push(parse_decorator(text(child, source))?),
            "function_definition" | "async_function_definition" => definition = Some(child),
            _ => {}
        }
    }
    Ok((decorators, definition))
}

fn parse_decorator(value: &str) -> Result<Decorator> {
    let value = value.trim().trim_start_matches('@');
    let name = value
        .split(['(', ' ', '\t'])
        .next()
        .unwrap_or_default()
        .to_string();
    if name.is_empty() {
        bail!("Python UI TD diagnostic [invalid-decorator]: decorator name is required");
    }
    let summary = value
        .strip_prefix("component(")
        .and_then(|tail| tail.strip_suffix(')'))
        .map(unquote)
        .transpose()?;
    Ok(Decorator { name, summary })
}

fn lower_page(node: Node<'_>, name: &str, source: &str) -> Result<WireframeSpec> {
    let body = node
        .child_by_field_name("body")
        .context("Python UI TD diagnostic [page-body]: @page needs a return component tree")?;
    let return_call = find_return_call(body)
        .context("Python UI TD diagnostic [page-return]: @page must return a component call")?;
    Ok(WireframeSpec {
        name: name.to_string(),
        component_type: "page".to_string(),
        props: parameters(node, source)?.into_iter().map(to_prop).collect(),
        layout: vec![lower_call_tree(return_call, source)?],
    })
}

fn lower_component(
    node: Node<'_>,
    name: &str,
    summary: Option<&str>,
    source: &str,
) -> Result<ComponentSpec> {
    let mut attributes = Vec::new();
    let mut slots = Vec::new();
    let mut events = Vec::new();
    for parameter in parameters(node, source)? {
        if let Some(detail_type) = generic_argument(&parameter.type_name, "Event") {
            events.push(EventDef {
                name: kebab(
                    parameter
                        .name
                        .strip_prefix("on_")
                        .unwrap_or(&parameter.name),
                ),
                detail_type: Some(to_target_type(detail_type)),
                description: None,
            });
        } else if generic_argument(&parameter.type_name, "Slot").is_some() {
            slots.push(SlotDef {
                name: kebab(&parameter.name),
                description: None,
            });
        } else {
            attributes.push(AttributeDef {
                name: kebab(&parameter.name),
                attr_type: to_target_type(&parameter.type_name),
                required: parameter.required,
                description: None,
            });
        }
    }
    Ok(ComponentSpec {
        tag_name: kebab(name),
        summary: summary
            .unwrap_or("Python-authored UI component")
            .to_string(),
        attributes,
        slots,
        events,
    })
}

#[derive(Debug)]
struct Parameter {
    name: String,
    type_name: String,
    required: bool,
}

fn parameters(node: Node<'_>, source: &str) -> Result<Vec<Parameter>> {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return Ok(Vec::new());
    };
    let raw = text(parameters, source)
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')');
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    raw.split(',')
        .map(|item| {
            let item = item.trim();
            let (declaration, default) = match item.split_once('=') {
                Some((left, right)) => (left.trim(), Some(right.trim())),
                None => (item, None),
            };
            let (name, type_name) = declaration.split_once(':').ok_or_else(|| {
                anyhow::anyhow!("Python UI TD diagnostic [missing-type]: parameter `{declaration}` must use a type annotation")
            })?;
            let name = name.trim().trim_start_matches('*');
            if !python_identifier(name) {
                bail!("Python UI TD diagnostic [invalid-parameter]: `{name}` is not a supported parameter name");
            }
            Ok(Parameter {
                name: name.to_string(),
                type_name: type_name.trim().to_string(),
                required: default.is_none(),
            })
        })
        .collect()
}

fn to_prop(parameter: Parameter) -> PropDef {
    PropDef {
        name: parameter.name,
        prop_type: to_target_type(&parameter.type_name),
        required: parameter.required,
        default_value: None,
        description: None,
    }
}

fn find_return_call<'a>(node: Node<'a>) -> Option<Node<'a>> {
    if node.kind() == "return_statement" {
        let mut cursor = node.walk();
        return node
            .named_children(&mut cursor)
            .find(|child| child.kind() == "call");
    }
    let mut cursor = node.walk();
    let found = node.named_children(&mut cursor).find_map(find_return_call);
    found
}

fn lower_call_tree(call: Node<'_>, source: &str) -> Result<WireframeNode> {
    let function = call.child_by_field_name("function").context(
        "Python UI TD diagnostic [unsupported-call]: component call needs a function name",
    )?;
    let kind = text(function, source).to_string();
    if !pascal_identifier(&kind) {
        bail!("Python UI TD diagnostic [unsupported-call]: `{kind}` must be a PascalCase component name");
    }
    let mut children = Vec::new();
    if let Some(arguments) = call.child_by_field_name("arguments") {
        let mut cursor = arguments.walk();
        for argument in arguments.named_children(&mut cursor) {
            let value = if argument.kind() == "keyword_argument" {
                argument.child_by_field_name("value")
            } else {
                Some(argument)
            };
            if let Some(value) = value.filter(|node| node.kind() == "call") {
                children.push(lower_call_tree(value, source)?);
            }
        }
    }
    Ok(WireframeNode {
        kind,
        label: None,
        children,
    })
}

fn lower_token(node: Node<'_>, source: &str) -> Result<Option<DesignTokenEntry>> {
    let mut cursor = node.walk();
    let Some(call) = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "call")
    else {
        return Ok(None);
    };
    let Some(function) = call.child_by_field_name("function") else {
        return Ok(None);
    };
    if text(function, source) != "token" {
        return Ok(None);
    }
    let Some(arguments) = call.child_by_field_name("arguments") else {
        bail!("Python UI TD diagnostic [invalid-token]: token requires path, value, and type literals");
    };
    let mut argument_cursor = arguments.walk();
    let values = arguments
        .named_children(&mut argument_cursor)
        .map(|value| unquote(text(value, source)))
        .collect::<Result<Vec<_>>>()?;
    if values.len() != 3 || values.iter().any(|value| value.is_empty()) {
        bail!("Python UI TD diagnostic [invalid-token]: use token(\"path\", \"value\", \"type\")");
    }
    Ok(Some(DesignTokenEntry {
        path: values[0].clone(),
        value: values[1].clone(),
        token_type: values[2].clone(),
        description: None,
    }))
}

fn function_name(node: Node<'_>, source: &str) -> Result<String> {
    let name = node
        .child_by_field_name("name")
        .map(|node| text(node, source).to_string())
        .unwrap_or_default();
    if !pascal_identifier(&name) {
        bail!("Python UI TD diagnostic [invalid-component-name]: `{name}` must be PascalCase");
    }
    Ok(name)
}

fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

fn unquote(value: &str) -> Result<String> {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .map(str::to_string)
        .context("Python UI TD diagnostic [literal-required]: use a quoted string literal")
}

fn generic_argument<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    value
        .strip_prefix(&format!("{name}["))
        .and_then(|value| value.strip_suffix(']'))
        .map(str::trim)
}

fn to_target_type(value: &str) -> String {
    match value.trim() {
        "str" => "string".to_string(),
        "int" | "float" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "None" => "null".to_string(),
        value if value.starts_with("list[") && value.ends_with(']') => {
            format!("{}[]", to_target_type(&value[5..value.len() - 1]))
        }
        value => value.to_string(),
    }
}

fn kebab(value: &str) -> String {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_ascii_uppercase() && index > 0 {
            output.push('-');
        }
        output.push(character.to_ascii_lowercase());
    }
    output.replace('_', "-")
}

fn python_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().enumerate().all(|(index, byte)| {
            byte == b'_' || byte.is_ascii_alphabetic() || (index > 0 && byte.is_ascii_digit())
        })
}

fn pascal_identifier(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_uppercase) && python_identifier(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODO_UI: &str = include_str!("../../../../examples/todo-app/td/src/interface/todo_ui.py");

    #[test]
    fn lowers_todo_components_into_existing_ui_ir() {
        let ui = lower_python_ui_td(TODO_UI).unwrap();
        assert_eq!(ui.wireframe.name, "TodoPage");
        assert_eq!(ui.wireframe.layout[0].kind, "AppShell");
        assert!(ui.wireframe.layout[0]
            .children
            .iter()
            .any(|child| child.kind == "Stack"));
        assert_eq!(ui.components.len(), 5);
        let row = ui
            .components
            .iter()
            .find(|component| component.tag_name == "task-row")
            .unwrap();
        assert_eq!(row.attributes[0].attr_type, "Todo");
        assert_eq!(row.events[0].name, "toggle");
        assert_eq!(row.events[0].detail_type.as_deref(), Some("TodoId"));
        assert_eq!(ui.design_tokens.unwrap().tokens.len(), 4);
    }

    #[test]
    fn rejects_missing_page_and_untyped_component_parameters() {
        let missing_page = "@component\ndef TaskRow(todo: Todo): ...";
        assert!(lower_python_ui_td(missing_page)
            .unwrap_err()
            .to_string()
            .contains("missing-page"));
        let missing_type = "@page\ndef TodoPage(todos):\n    return TaskList()";
        assert!(lower_python_ui_td(missing_type)
            .unwrap_err()
            .to_string()
            .contains("missing-type"));
    }
}
