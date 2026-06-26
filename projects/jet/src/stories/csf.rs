// HANDWRITE-BEGIN gap="missing-generator:logic:5570b214" tracker="standardize-gap-projects-jet-src-stories-csf-rs" reason="CSF parser: given a story file source, extract the default export (meta: component ref, title, args, argTypes) and named exports (stories: name, args, render) using the existing extract_imports/tree-sitter surface."
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
//!   a type-annotation (`: Story`) — the wrappers are transparently unwrapped,
//! - the legacy CSF2 `const Primary = Template.bind({});` story shape, with
//!   later top-level `Primary.args = { ... };` / `Primary.storyName = '...'`
//!   mutations folded back onto the story,
//! - spread args (`args: { ...base, label: 'x' }`) where `base` is a statically
//!   known object in the same file (a `const base = { ... }` or another story's
//!   args) — the spread members are merged in, then explicit keys override,
//! - re-exported stories (`export { Primary } from './elsewhere'`,
//!   `export { A as B } from './elsewhere'`) are surfaced as
//!   [`ParsedStoryFile::re_exports`] for the caller to resolve (this parser is
//!   source-only and does not read sibling files).
//!
//! Deferred (TODO(#199 follow-up), graceful skip — never a crash):
//! - spread from an imported / dynamically computed base (unresolvable spreads
//!   keep the explicit keys and drop only the spread),
//! - computed / dynamic story names,
//! - the legacy `storiesOf(...)` imperative API.

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
    ///
    /// For a CSF2 `Template.bind({})` story this is `true`: the story renders
    /// through the bound template, so it carries its own render just like a
    /// CSF3 story with an explicit `render:` field.
    pub has_render: bool,
}

/// A re-exported story (`export { Primary } from './button.stories'`).
///
/// `parse_csf` is source-only and cannot read the sibling file, so it surfaces
/// these for the caller (`discover`) to resolve against the importing file.
#[derive(Debug, Clone, PartialEq)]
pub struct CsfReExport {
    /// The name this file exposes the story as (the `B` in `A as B`, else the
    /// plain name).
    pub exported_name: String,
    /// The name of the story in the source module (the `A` in `A as B`, else
    /// the plain name).
    pub local_name: String,
    /// The module specifier the story is re-exported from (`./button.stories`).
    pub relative_source: String,
}

/// The full parse of one story file.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedStoryFile {
    pub meta: CsfMeta,
    pub stories: Vec<CsfStory>,
    /// Re-exported stories pulled from sibling files; resolved by the caller.
    pub re_exports: Vec<CsfReExport>,
}

/// Parse the CSF structure of a story file.
///
/// `is_tsx` selects the parser flavor; the TSX grammar is a strict superset of
/// JS/JSX/TS so we always parse with it (the bundler does the same), but the
/// flag is kept for API symmetry and future grammar specialization.
pub fn parse_csf(source: &str, _is_tsx: bool) -> Result<ParsedStoryFile> {
    // Confirm the file is CSF: it must either declare a default export (the
    // meta) or re-export stories from a sibling (a barrel/aggregator file). A
    // file with neither is not a story file.
    let imports = extract_imports(source, true)?;
    let has_default = imports
        .exports
        .iter()
        .any(|e| e.kind == ExportKind::Default);
    let has_re_export = imports
        .exports
        .iter()
        .any(|e| e.kind == ExportKind::Named && e.source.is_some());
    if !has_default && !has_re_export {
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
    // export written as `export default meta` can resolve `meta` to its object,
    // and so spread args (`{ ...base }`) can resolve `base` to its members.
    let mut top_level_consts: BTreeMap<String, Node> = BTreeMap::new();
    let mut default_object: Option<Node> = None;
    // (export_name, object_node) for CSF3 `export const X = { ... }` stories.
    let mut named_stories: Vec<(String, Node)> = Vec::new();
    // CSF2 `const X = Template.bind({})` story identifiers (in source order).
    let mut bound_stories: Vec<String> = Vec::new();
    // Re-exported stories: `export { A as B } from './x'`.
    let mut re_exports: Vec<CsfReExport> = Vec::new();

    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        match child.kind() {
            "lexical_declaration" | "variable_declaration" => {
                for (name, value) in declarators(source, child) {
                    if let Some(obj) = unwrap_to_object(value) {
                        top_level_consts.insert(name, obj);
                    } else if is_bind_call(source, value) {
                        // CSF2 `const Primary = Template.bind({})`.
                        bound_stories.push(name);
                    }
                }
            }
            "export_statement" => {
                if is_default_export(child) {
                    default_object = default_export_object(source, child, &top_level_consts);
                } else if let Some(src) = re_export_source(source, child) {
                    // `export { Primary, A as B } from './elsewhere'`.
                    collect_re_exports(source, child, &src, &mut re_exports);
                } else if let Some(decl) = first_child_of_kind(child, "lexical_declaration")
                    .or_else(|| first_child_of_kind(child, "variable_declaration"))
                {
                    // `export const Primary = {...}` — one or more named stories,
                    // or `export const Primary = Template.bind({})` (CSF2 export
                    // form). Track which exported names are bound templates.
                    for (name, value) in declarators(source, decl) {
                        if let Some(obj) = unwrap_to_object(value) {
                            named_stories.push((name, obj));
                        } else if is_bind_call(source, value) {
                            bound_stories.push(name);
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

    // Second pass: collect top-level `X.args = {...}` / `X.storyName = '...'`
    // mutations so CSF2 bound stories can pick up their args, and so any story
    // can reference another story's args in a spread.
    let mutations = collect_story_mutations(source, root);

    // A resolver scope for spread args: top-level const objects keyed by name.
    let scope = SpreadScope {
        consts: &top_level_consts,
    };

    let mut stories: Vec<CsfStory> = named_stories
        .into_iter()
        .map(|(export_name, obj)| parse_story_object(source, &export_name, obj, &scope, &mutations))
        .collect();

    // CSF2 bound-template stories: render comes from the bound template, args
    // come from the later `X.args = {...}` mutation (if any).
    for name in bound_stories {
        // A name may be both a bound template and (mistakenly) re-declared as an
        // object; the object form already produced a story, so skip duplicates.
        if stories.iter().any(|s| s.export_name == name) {
            continue;
        }
        let args = mutations
            .get(&name)
            .map(|m| resolve_args(&m.args_pairs, source, &scope, &mutations))
            .unwrap_or_default();
        stories.push(CsfStory {
            export_name: name,
            args,
            // The render is supplied by the bound template.
            has_render: true,
        });
    }

    Ok(ParsedStoryFile {
        meta,
        stories,
        re_exports,
    })
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
///
/// `args` is resolved through [`resolve_args`] so a spread (`{ ...base, x }`)
/// is expanded against the file's static scope.
fn parse_story_object(
    source: &str,
    export_name: &str,
    obj: Node,
    scope: &SpreadScope,
    mutations: &BTreeMap<String, StoryMutation>,
) -> CsfStory {
    let mut args = BTreeMap::new();
    let mut has_render = false;
    for (key, value) in object_pairs(source, obj) {
        match key.as_str() {
            "args" => {
                if value.kind() == "object" {
                    args = resolve_object_args(value, source, scope, mutations);
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

// ── CSF2 mutations + spread resolution ────────────────────────────────────────

/// Static scope for resolving spread args within a single file: top-level
/// `const NAME = {object}` declarations keyed by identifier.
struct SpreadScope<'a> {
    consts: &'a BTreeMap<String, Node<'a>>,
}

/// A top-level `X.args = {...}` / `X.storyName = '...'` mutation, used by both
/// CSF2 bound stories and `...X.args` spreads.
struct StoryMutation<'a> {
    /// The `args =` RHS object's `pair`/`spread_element` nodes, in source order.
    args_pairs: Vec<Node<'a>>,
    /// `X.storyName = '...'` value, if assigned. (Surfaced for completeness;
    /// the story index keys off the export identifier today.)
    #[allow(dead_code)]
    story_name: Option<String>,
}

/// Walk the top level for `X.args = {...}` and `X.storyName = '...'`
/// assignment statements, grouping them by the mutated identifier `X`.
fn collect_story_mutations<'a>(source: &str, root: Node<'a>) -> BTreeMap<String, StoryMutation<'a>> {
    let mut out: BTreeMap<String, StoryMutation> = BTreeMap::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        if child.kind() != "expression_statement" {
            continue;
        }
        let Some(assign) = first_child_of_kind(child, "assignment_expression") else {
            continue;
        };
        // LHS must be `X.<prop>` (a member_expression with an identifier base).
        let kids = named_children(assign);
        let Some(lhs) = kids.first().copied() else {
            continue;
        };
        if lhs.kind() != "member_expression" {
            continue;
        }
        let lhs_kids = named_children(lhs);
        let (Some(base), Some(prop)) = (lhs_kids.first(), lhs_kids.get(1)) else {
            continue;
        };
        if base.kind() != "identifier" || prop.kind() != "property_identifier" {
            continue;
        }
        let story = node_text(*base, source).to_string();
        let prop_name = node_text(*prop, source);
        let Some(rhs) = kids.get(1).copied() else {
            continue;
        };

        let entry = out.entry(story).or_insert_with(|| StoryMutation {
            args_pairs: Vec::new(),
            story_name: None,
        });
        match prop_name {
            "args" => {
                if rhs.kind() == "object" {
                    entry.args_pairs = object_member_nodes(rhs);
                }
            }
            "storyName" => {
                if rhs.kind() == "string" {
                    entry.story_name = Some(strip_quotes(node_text(rhs, source)));
                }
            }
            _ => {}
        }
    }
    out
}

/// The `pair` / `spread_element` children of an `object` literal, in order.
fn object_member_nodes(obj: Node) -> Vec<Node> {
    named_children(obj)
        .into_iter()
        .filter(|c| matches!(c.kind(), "pair" | "spread_element"))
        .collect()
}

/// Resolve a story's `args` from an `object` literal node, expanding spreads.
fn resolve_object_args(
    obj: Node,
    source: &str,
    scope: &SpreadScope,
    mutations: &BTreeMap<String, StoryMutation>,
) -> BTreeMap<String, CsfValue> {
    resolve_args(&object_member_nodes(obj), source, scope, mutations)
}

/// Build an args map from an object's ordered member nodes, expanding any
/// `spread_element` against the static scope.
///
/// Spread semantics match JS object spread: earlier members are overwritten by
/// later ones, so an explicit key after a spread wins. An unresolvable spread
/// (imported / dynamic base) is skipped gracefully — the explicit keys remain.
fn resolve_args(
    members: &[Node],
    source: &str,
    scope: &SpreadScope,
    mutations: &BTreeMap<String, StoryMutation>,
) -> BTreeMap<String, CsfValue> {
    resolve_args_guarded(members, source, scope, mutations, 0)
}

/// Recursion-guarded inner resolver (spread bases may themselves spread).
fn resolve_args_guarded(
    members: &[Node],
    source: &str,
    scope: &SpreadScope,
    mutations: &BTreeMap<String, StoryMutation>,
    depth: usize,
) -> BTreeMap<String, CsfValue> {
    let mut out = BTreeMap::new();
    // Cheap cycle / runaway guard for self-referential spreads.
    if depth > 8 {
        return out;
    }
    for member in members {
        match member.kind() {
            "pair" => {
                if let Some((key, value)) = pair_kv(*member, source) {
                    out.insert(key, value_of(source, value));
                }
            }
            "spread_element" => {
                // `...base` -> resolve a statically-known object's members.
                if let Some(base) = spread_base_members(*member, source, scope, mutations) {
                    let resolved =
                        resolve_args_guarded(&base, source, scope, mutations, depth + 1);
                    for (k, v) in resolved {
                        out.insert(k, v);
                    }
                }
                // Unresolvable spread (imported / dynamic): TODO(#199 follow-up)
                // — skip it, keep the explicit keys.
            }
            _ => {}
        }
    }
    out
}

/// Resolve the member nodes a `spread_element` (`...X` or `...X.args`) refers
/// to, if the base is statically known in this file. Returns `None` for
/// anything dynamic / imported.
fn spread_base_members<'a>(
    spread: Node<'a>,
    source: &str,
    scope: &SpreadScope<'a>,
    mutations: &'a BTreeMap<String, StoryMutation<'a>>,
) -> Option<Vec<Node<'a>>> {
    let inner = named_children(spread).into_iter().next()?;
    match inner.kind() {
        // `...base` where `const base = { ... }` exists at the top level.
        "identifier" => {
            let name = node_text(inner, source);
            scope.consts.get(name).map(|obj| object_member_nodes(*obj))
        }
        // `...Primary.args` — reuse another CSF2 story's `X.args` mutation.
        "member_expression" => {
            let kids = named_children(inner);
            let (base, prop) = (kids.first()?, kids.get(1)?);
            if base.kind() == "identifier" && prop.kind() == "property_identifier" {
                let base_name = node_text(*base, source);
                let prop_name = node_text(*prop, source);
                if prop_name == "args" {
                    if let Some(m) = mutations.get(base_name) {
                        return Some(m.args_pairs.clone());
                    }
                }
            }
            None
        }
        _ => None,
    }
}

/// `(key, value_node)` of a `pair`, with the key string-normalized.
fn pair_kv<'a>(pair: Node<'a>, source: &str) -> Option<(String, Node<'a>)> {
    let key = first_child_of_kind(pair, "property_identifier")
        .or_else(|| first_child_of_kind(pair, "string"))?;
    let value = pair_value(pair)?;
    let key_text = match key.kind() {
        "string" => strip_quotes(node_text(key, source)),
        _ => node_text(key, source).to_string(),
    };
    Some((key_text, value))
}

/// True when `value` is a `Template.bind({...})`-shaped call expression: a call
/// on a `member_expression` whose property identifier is exactly `bind`.
fn is_bind_call(source: &str, value: Node) -> bool {
    // Unwrap `as`/`satisfies`/parens around the call (rare, but cheap).
    let Some(call) = unwrap_to_call(value) else {
        return false;
    };
    let Some(callee) = first_child_of_kind(call, "member_expression") else {
        return false;
    };
    named_children(callee)
        .into_iter()
        .filter(|c| c.kind() == "property_identifier")
        .any(|c| node_text(c, source) == "bind")
}

/// Unwrap `as`/`satisfies`/parenthesized wrappers to reach a `call_expression`.
fn unwrap_to_call(node: Node) -> Option<Node> {
    match node.kind() {
        "call_expression" => Some(node),
        "satisfies_expression" | "as_expression" | "parenthesized_expression" => {
            named_children(node).into_iter().find_map(unwrap_to_call)
        }
        _ => None,
    }
}

// ── re-export collection ───────────────────────────────────────────────────────

/// The module specifier of a re-exporting `export { ... } from '...'` statement,
/// or `None` if this is not a re-export.
fn re_export_source(source: &str, export_stmt: Node) -> Option<String> {
    // A re-export has both an `export_clause` and a trailing `string` source.
    let has_clause = first_child_of_kind(export_stmt, "export_clause").is_some();
    if !has_clause {
        return None;
    }
    let src = first_child_of_kind(export_stmt, "string")?;
    Some(strip_quotes(node_text(src, source)))
}

/// Collect each `export_specifier` of an `export { A, B as C } from '...'` into
/// a [`CsfReExport`].
fn collect_re_exports(
    source: &str,
    export_stmt: Node,
    relative_source: &str,
    out: &mut Vec<CsfReExport>,
) {
    let Some(clause) = first_child_of_kind(export_stmt, "export_clause") else {
        return;
    };
    for spec in named_children(clause) {
        if spec.kind() != "export_specifier" {
            continue;
        }
        let idents: Vec<Node> = named_children(spec)
            .into_iter()
            .filter(|c| c.kind() == "identifier")
            .collect();
        let (local_name, exported_name) = match idents.as_slice() {
            // `A as B`: local = A, exported = B.
            [a, b] => (
                node_text(*a, source).to_string(),
                node_text(*b, source).to_string(),
            ),
            // `A`: local = exported = A.
            [a] => {
                let n = node_text(*a, source).to_string();
                (n.clone(), n)
            }
            _ => continue,
        };
        out.push(CsfReExport {
            exported_name,
            local_name,
            relative_source: relative_source.to_string(),
        });
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

    #[test]
    fn csf2_template_bind_with_args_and_story_name() {
        let src = r#"
import { Toggle } from './Toggle';
export default { title: 'Legacy/Toggle', component: Toggle };

const Template = (args) => <Toggle {...args} />;
const Primary = Template.bind({});
Primary.args = { label: "Hi", on: true };
Primary.storyName = "The Primary";

export const Secondary = Template.bind({});
Secondary.args = { label: "Lo" };

export { Primary };
"#;
        let parsed = parse_csf(src, true).expect("parses");
        // Both bound templates surface as stories.
        let names: Vec<_> = parsed.stories.iter().map(|s| s.export_name.as_str()).collect();
        assert!(names.contains(&"Primary"), "got {names:?}");
        assert!(names.contains(&"Secondary"), "got {names:?}");

        let primary = parsed
            .stories
            .iter()
            .find(|s| s.export_name == "Primary")
            .unwrap();
        assert_eq!(primary.args.get("label"), Some(&CsfValue::Str("Hi".into())));
        assert_eq!(primary.args.get("on"), Some(&CsfValue::Bool(true)));
        assert!(primary.has_render, "bound template supplies render");
    }

    #[test]
    fn spread_args_merge_static_const() {
        let src = r#"
import { Panel } from './Panel';
export default { title: 'Layout/Panel', component: Panel };

const base = { x: 1, y: 1 };
export const Spread = { args: { ...base, x: 2 } };
export const Dynamic = { args: { ...imported, only: 9 } };
"#;
        let parsed = parse_csf(src, true).expect("parses");
        let spread = parsed
            .stories
            .iter()
            .find(|s| s.export_name == "Spread")
            .unwrap();
        assert_eq!(spread.args.get("y"), Some(&CsfValue::Number("1".into())));
        // explicit `x: 2` overrides spread `base.x = 1`.
        assert_eq!(spread.args.get("x"), Some(&CsfValue::Number("2".into())));

        let dynamic = parsed
            .stories
            .iter()
            .find(|s| s.export_name == "Dynamic")
            .unwrap();
        assert_eq!(dynamic.args.get("only"), Some(&CsfValue::Number("9".into())));
        assert!(!dynamic.args.contains_key("x"), "unresolvable spread dropped");
    }

    #[test]
    fn re_exports_are_surfaced_for_the_caller() {
        let src = r#"
export { Primary } from './button.stories';
export { A as B } from './other.stories';
"#;
        let parsed = parse_csf(src, true).expect("re-export-only file is valid CSF");
        assert_eq!(parsed.re_exports.len(), 2);

        let primary = &parsed.re_exports[0];
        assert_eq!(primary.exported_name, "Primary");
        assert_eq!(primary.local_name, "Primary");
        assert_eq!(primary.relative_source, "./button.stories");

        let renamed = &parsed.re_exports[1];
        assert_eq!(renamed.local_name, "A");
        assert_eq!(renamed.exported_name, "B");
        assert_eq!(renamed.relative_source, "./other.stories");
    }
}
// HANDWRITE-END
