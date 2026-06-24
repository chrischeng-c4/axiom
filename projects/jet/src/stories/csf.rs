// HANDWRITE-BEGIN gap="missing-generator:logic:5570b214" tracker="pending-tracker" reason="CSF parser: given a story file source, extract the default export (meta: component ref, title, args, argTypes) and named exports (stories: name, args, render) using the existing extract_imports/tree-sitter surface."
//! Component Story Format (CSF) parser.
//!
//! Given the source of a `*.stories.@(ts|tsx|js|jsx)` file, this module
//! extracts the normalized CSF structure that the manager (B2) and controls
//! (B3) consume:
//!
//! - the **default export** is the *meta* (`component`, `title`, `args`,
//!   `argTypes`),
//! - every **named export** is a *story* (`name`, merged `args`, whether it
//!   carries a custom `render`).
//!
//! Parsing is tree-sitter based (the TSX grammar covers ts/tsx/js/jsx). We use
//! [`crate::bundler::imports::extract_imports`] to confirm a default export
//! exists, then walk the AST ourselves to read the object-literal field values
//! — `extract_imports` only reports export *kinds*, not the literal contents.
//!
//! Supported CSF shapes:
//! - `const meta = { ... }; export default meta;` (CSF3, the common case),
//! - `export default { ... };` (object inlined in the default export),
//! - `export const Primary = { args: { ... } };` named stories,
//! - object literals wrapped in `satisfies Meta<...>` / `as const` /
//!   a type-annotation (`: Story`) — the wrappers are transparently unwrapped.
//!
//! Deferred (TODO, tracked for a later iteration):
//! - the legacy CSF2 `Template.bind({})` + `Story.args = {}` mutation shape,
//! - `export { Primary } from './elsewhere'` re-exported stories,
//! - spread args (`args: { ...base, label: 'x' }`) — the spread is ignored.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use tree_sitter::{Node, Parser};

use crate::bundler::imports::{extract_imports, ExportKind};

/// A single field value read from a CSF object literal.
///
/// We deliberately keep this lossy-but-parseable rather than a full JS value
/// model: B3 (controls) only needs to render and round-trip these, and a
/// string-or-raw representation is enough to reconstruct an editable control
/// without re-implementing a JS evaluator.
#[derive(Debug, Clone, PartialEq)]
pub enum CsfValue {
    /// A string literal (quotes stripped): `'Hi'` -> `Hi`.
    Str(String),
    /// A boolean literal.
    Bool(bool),
    /// A numeric literal, kept as its source text (`42`, `1.5`, `0xff`).
    Number(String),
    /// `null` / `undefined`.
    Null,
    /// A nested object literal (e.g. `argTypes`, nested `args`).
    Object(BTreeMap<String, CsfValue>),
    /// Anything we do not destructure (identifiers, arrays, calls, JSX,
    /// arrow functions): kept as the raw source slice so callers can still
    /// display / round-trip it.
    Raw(String),
}

/// Parsed meta (the default export of a story file).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CsfMeta {
    /// `component:` field, as raw source (usually a component identifier).
    pub component: Option<String>,
    /// `title:` field (the sidebar path, `Components/Button`).
    pub title: Option<String>,
    /// `args:` object — default args applied to every story in the file.
    pub args: BTreeMap<String, CsfValue>,
    /// `argTypes:` object — control metadata for B3.
    pub arg_types: BTreeMap<String, CsfValue>,
}

/// Parsed named-export story.
#[derive(Debug, Clone, PartialEq)]
pub struct CsfStory {
    /// The export identifier (`Primary`, `Disabled`).
    pub export_name: String,
    /// Story-level `args:` object (merged over meta args by the index).
    pub args: BTreeMap<String, CsfValue>,
    /// Whether the story declares its own `render:` function.
    pub has_render: bool,
}

/// The full parse of one story file.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStoryFile {
    pub meta: CsfMeta,
    pub stories: Vec<CsfStory>,
}

/// Parse the CSF structure of a story file.
///
/// `is_tsx` selects the parser flavor; the TSX grammar is a strict superset of
/// JS/JSX/TS so we always parse with it (the bundler does the same), but the
/// flag is kept for API symmetry and future grammar specialization.
pub fn parse_csf(source: &str, _is_tsx: bool) -> Result<ParsedStoryFile> {
    // Confirm a default export exists before doing the (more expensive) walk;
    // a story file with no default export is not valid CSF.
    let imports = extract_imports(source, true)?;
    let has_default = imports
        .exports
        .iter()
        .any(|e| e.kind == ExportKind::Default);
    if !has_default {
        return Err(anyhow!(
            "no default export found (CSF requires `export default` meta)"
        ));
    }

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("tree-sitter failed to parse story file"))?;
    let root = tree.root_node();

    // First pass: collect every top-level `const NAME = {...}` so a default
    // export written as `export default meta` can resolve `meta` to its object.
    let mut top_level_consts: BTreeMap<String, Node> = BTreeMap::new();
    let mut default_object: Option<Node> = None;
    let mut named_stories: Vec<(String, Node)> = Vec::new();

    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        match child.kind() {
            "lexical_declaration" | "variable_declaration" => {
                for (name, value) in declarators(source, child) {
                    if let Some(obj) = unwrap_to_object(value) {
                        top_level_consts.insert(name, obj);
                    }
                }
            }
            "export_statement" => {
                if is_default_export(child) {
                    default_object = default_export_object(source, child, &top_level_consts);
                } else if let Some(decl) = first_child_of_kind(child, "lexical_declaration")
                    .or_else(|| first_child_of_kind(child, "variable_declaration"))
                {
                    // `export const Primary = {...}` — one or more named stories.
                    for (name, value) in declarators(source, decl) {
                        if let Some(obj) = unwrap_to_object(value) {
                            named_stories.push((name, obj));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let meta = match default_object {
        Some(obj) => parse_meta_object(source, obj),
        None => CsfMeta::default(),
    };

    let stories = named_stories
        .into_iter()
        .map(|(export_name, obj)| parse_story_object(source, &export_name, obj))
        .collect();

    Ok(ParsedStoryFile { meta, stories })
}

/// Read the meta object's `component` / `title` / `args` / `argTypes` fields.
fn parse_meta_object(source: &str, obj: Node) -> CsfMeta {
    let mut meta = CsfMeta::default();
    for (key, value) in object_pairs(source, obj) {
        match key.as_str() {
            "component" => meta.component = Some(node_text(value, source).to_string()),
            "title" => {
                meta.title = Some(match value_of(source, value) {
                    CsfValue::Str(s) => s,
                    _ => strip_quotes(node_text(value, source)),
                });
            }
            "args" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    meta.args = map;
                }
            }
            "argTypes" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    meta.arg_types = map;
                }
            }
            _ => {}
        }
    }
    meta
}

/// Read a named story's `args` object and detect a `render` field.
fn parse_story_object(source: &str, export_name: &str, obj: Node) -> CsfStory {
    let mut args = BTreeMap::new();
    let mut has_render = false;
    for (key, value) in object_pairs(source, obj) {
        match key.as_str() {
            "args" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    args = map;
                }
            }
            "render" => has_render = true,
            _ => {}
        }
    }
    CsfStory {
        export_name: export_name.to_string(),
        args,
        has_render,
    }
}

// ── AST helpers ──────────────────────────────────────────────────────────────

/// Yield `(identifier, value_node)` for each declarator of a `const`/`let`/`var`
/// declaration node.
fn declarators<'a>(source: &str, decl: Node<'a>) -> Vec<(String, Node<'a>)> {
    let mut out = Vec::new();
    let mut cursor = decl.walk();
    for child in decl.named_children(&mut cursor) {
        if child.kind() == "variable_declarator" {
            let name = first_child_of_kind(child, "identifier");
            let value = declarator_value(child);
            if let (Some(name), Some(value)) = (name, value) {
                out.push((node_text(name, source).to_string(), value));
            }
        }
    }
    out
}

/// The initializer node of a `variable_declarator` (skips identifier + type).
fn declarator_value(decl: Node) -> Option<Node> {
    let mut cursor = decl.walk();
    let mut last = None;
    for child in decl.named_children(&mut cursor) {
        match child.kind() {
            "identifier" | "type_annotation" => {}
            _ => last = Some(child),
        }
    }
    last
}

/// Unwrap `satisfies Meta<...>`, `as const`, and parenthesized expressions to
/// reach the underlying `object` literal, if any.
fn unwrap_to_object(node: Node) -> Option<Node> {
    match node.kind() {
        "object" => Some(node),
        "satisfies_expression" | "as_expression" | "parenthesized_expression" => {
            if let Some(obj) = first_child_of_kind(node, "object") {
                return Some(obj);
            }
            // recurse into the first named child in case of chained wrappers
            named_children(node).into_iter().find_map(unwrap_to_object)
        }
        _ => None,
    }
}

/// True when this `export_statement` is `export default ...`.
fn is_default_export(export_stmt: Node) -> bool {
    let mut cursor = export_stmt.walk();
    let children: Vec<Node> = export_stmt.children(&mut cursor).collect();
    children.iter().any(|c| c.kind() == "default")
}

/// Resolve the object literal of a default export, whether it inlines the
/// object (`export default {...}`) or references a `const meta` identifier.
fn default_export_object<'a>(
    source: &str,
    export_stmt: Node<'a>,
    consts: &BTreeMap<String, Node<'a>>,
) -> Option<Node<'a>> {
    let mut cursor = export_stmt.walk();
    for child in export_stmt.named_children(&mut cursor) {
        match child.kind() {
            "object" => return Some(child),
            "satisfies_expression" | "as_expression" | "parenthesized_expression" => {
                if let Some(obj) = unwrap_to_object(child) {
                    return Some(obj);
                }
            }
            "identifier" => {
                // `export default meta;`
                return consts.get(node_text(child, source)).copied();
            }
            _ => {}
        }
    }
    None
}

/// Iterate `(property_name, value_node)` of every `pair` in an `object` literal.
fn object_pairs<'a>(source: &str, obj: Node<'a>) -> Vec<(String, Node<'a>)> {
    let mut out = Vec::new();
    let mut cursor = obj.walk();
    for child in obj.named_children(&mut cursor) {
        if child.kind() == "pair" {
            let key = first_child_of_kind(child, "property_identifier")
                .or_else(|| first_child_of_kind(child, "string"));
            let value = pair_value(child);
            if let (Some(key), Some(value)) = (key, value) {
                let key_text = match key.kind() {
                    "string" => strip_quotes(node_text(key, source)),
                    _ => node_text(key, source).to_string(),
                };
                out.push((key_text, value));
            }
        }
    }
    out
}

/// The value node of a `pair` (the last named child: `[key, value]`).
fn pair_value(pair: Node) -> Option<Node> {
    named_children(pair).into_iter().last()
}

/// Convert a value node into a [`CsfValue`].
fn value_of(source: &str, node: Node) -> CsfValue {
    match node.kind() {
        "string" => CsfValue::Str(strip_quotes(node_text(node, source))),
        "true" => CsfValue::Bool(true),
        "false" => CsfValue::Bool(false),
        "null" | "undefined" => CsfValue::Null,
        "number" => CsfValue::Number(node_text(node, source).to_string()),
        "object" => {
            let mut map = BTreeMap::new();
            for (key, value) in object_pairs(source, node) {
                map.insert(key, value_of(source, value));
            }
            CsfValue::Object(map)
        }
        _ => CsfValue::Raw(node_text(node, source).to_string()),
    }
}

/// Collect the named children of a node into an owned Vec.
///
/// tree-sitter's `named_children` iterator borrows a `TreeCursor` that the
/// borrow checker treats as escaping if we return a value derived from it in
/// the same expression, so callers collect first.
fn named_children(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

fn first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    named_children(node).into_iter().find(|c| c.kind() == kind)
}

fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

fn strip_quotes(raw: &str) -> String {
    raw.trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const BUTTON: &str = r#"
import { Button } from './Button';
import type { Meta, StoryObj } from '@storybook/react';

const meta = {
  title: 'Components/Button',
  component: Button,
  args: { label: 'Hi', size: 'md' },
  argTypes: { size: { control: 'select' } },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: { primary: true, label: 'Click', count: 3 },
};

export const Disabled: Story = {
  args: { disabled: true },
  render: () => <Button />,
};
"#;

    #[test]
    fn parses_meta_and_named_stories() {
        let parsed = parse_csf(BUTTON, true).expect("parses");
        assert_eq!(parsed.meta.title.as_deref(), Some("Components/Button"));
        assert_eq!(parsed.meta.component.as_deref(), Some("Button"));
        assert_eq!(
            parsed.meta.args.get("label"),
            Some(&CsfValue::Str("Hi".into()))
        );
        assert!(parsed.meta.arg_types.contains_key("size"));

        assert_eq!(parsed.stories.len(), 2);
        let primary = &parsed.stories[0];
        assert_eq!(primary.export_name, "Primary");
        assert_eq!(primary.args.get("primary"), Some(&CsfValue::Bool(true)));
        assert_eq!(
            primary.args.get("count"),
            Some(&CsfValue::Number("3".into()))
        );
        assert!(!primary.has_render);

        let disabled = &parsed.stories[1];
        assert_eq!(disabled.export_name, "Disabled");
        assert!(disabled.has_render);
    }

    #[test]
    fn parses_inline_default_export_object() {
        let src = r#"
import { Card } from './Card';
export default {
  title: 'Forms/Card',
  component: Card,
} as const;

export const WithFooter = { args: { footer: true } };
"#;
        let parsed = parse_csf(src, true).expect("parses");
        assert_eq!(parsed.meta.title.as_deref(), Some("Forms/Card"));
        assert_eq!(parsed.meta.component.as_deref(), Some("Card"));
        assert_eq!(parsed.stories.len(), 1);
        assert_eq!(parsed.stories[0].export_name, "WithFooter");
    }

    #[test]
    fn missing_default_export_is_error() {
        let src = r#"
export const Orphan = { args: {} };
"#;
        assert!(parse_csf(src, true).is_err());
    }
}
// HANDWRITE-END
