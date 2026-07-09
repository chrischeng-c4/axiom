// <HANDWRITE gap="missing-generator:logic:5570b214" tracker="standardize-gap-projects-jet-src-stories-csf-rs" reason="CSF parser: given a story file source, extract the default export (meta: component ref, title, args, argTypes) and named exports (stories: name, args, render) using the existing extract_imports/tree-sitter surface.">
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
    /// `parameters:` object — render-path metadata such as `layout`.
    pub parameters: BTreeMap<String, CsfValue>,
    /// `globals:` object — file-level globals overrides.
    pub globals: BTreeMap<String, CsfValue>,
    /// `globalTypes:` object — global default values and toolbar metadata.
    pub global_types: BTreeMap<String, CsfValue>,
    /// Whether `decorators:` is authored on the meta object.
    pub has_decorators: bool,
    /// Whether `loaders:` is authored on the meta object.
    pub has_loaders: bool,
    /// Storybook tags such as `autodocs` / `!autodocs`.
    pub tags: Vec<String>,
}

/// Parsed named-export story.
#[derive(Debug, Clone, PartialEq)]
pub struct CsfStory {
    /// The export identifier (`Primary`, `Disabled`).
    pub export_name: String,
    /// Copyable source slice for the story panel. `parameters.docs.source.code`
    /// overrides the extracted source when authored.
    pub source: Option<String>,
    /// Story-level `args:` object (merged over meta args by the index).
    pub args: BTreeMap<String, CsfValue>,
    /// Story-level docs description from `parameters.docs.description.story`
    /// or the JSDoc comment immediately preceding the story export.
    pub description: String,
    /// Whether the story declares its own `render:` function.
    ///
    /// For a CSF2 `Template.bind({})` story this is `true`: the story renders
    /// through the bound template, so it carries its own render just like a
    /// CSF3 story with an explicit `render:` field.
    pub has_render: bool,
    /// `parameters:` object authored on this story.
    pub parameters: BTreeMap<String, CsfValue>,
    /// `globals:` object authored on this story.
    pub globals: BTreeMap<String, CsfValue>,
    /// Whether `decorators:` is authored on this story.
    pub has_decorators: bool,
    /// Whether `loaders:` is authored on this story.
    pub has_loaders: bool,
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
    let mut named_stories: Vec<(String, Node, Node)> = Vec::new();
    // CSF2 `const X = Template.bind({})` story identifiers (in source order).
    let mut bound_stories: Vec<(String, Option<String>)> = Vec::new();
    // Top-level/exported declaration statements keyed by declared identifier.
    let mut source_decls: BTreeMap<String, Node> = BTreeMap::new();
    // Re-exported stories: `export { A as B } from './x'`.
    let mut re_exports: Vec<CsfReExport> = Vec::new();

    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        match child.kind() {
            "lexical_declaration" | "variable_declaration" => {
                for (name, value) in declarators(source, child) {
                    source_decls.insert(name.clone(), child);
                    if let Some(obj) = unwrap_to_object(value) {
                        top_level_consts.insert(name, obj);
                    } else if let Some(template) = bind_template_name(source, value) {
                        // CSF2 `const Primary = Template.bind({})`.
                        bound_stories.push((name, Some(template)));
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
                    // `export const Primary = Template.bind({})` (CSF2), or a
                    // callable/factory story such as `export const Primary =
                    // (args) => <Button {...args} />` followed by
                    // `Primary.args = {...}`.
                    for (name, value) in declarators(source, decl) {
                        source_decls.insert(name.clone(), child);
                        if let Some(obj) = unwrap_to_object(value) {
                            named_stories.push((name, obj, child));
                        } else if let Some(template) = bind_template_name(source, value) {
                            bound_stories.push((name, Some(template)));
                        } else if is_exported_callable_story(&name, value) {
                            bound_stories.push((name, None));
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
        .map(|(export_name, obj, source_node)| {
            let extracted_source = Some(node_text(source_node, source).trim().to_string());
            let jsdoc_description = jsdoc_comment_before_node(source, source_node);
            parse_story_object(
                source,
                &export_name,
                obj,
                &scope,
                &mutations,
                extracted_source,
                jsdoc_description,
            )
        })
        .collect();

    // CSF2 bound-template stories: render comes from the bound template, args
    // come from the later `X.args = {...}` mutation (if any).
    for (name, template_name) in bound_stories {
        // A name may be both a bound template and (mistakenly) re-declared as an
        // object; the object form already produced a story, so skip duplicates.
        if stories.iter().any(|s| s.export_name == name) {
            continue;
        }
        let args = mutations
            .get(&name)
            .map(|m| resolve_args(&m.args_pairs, source, &scope, &mutations))
            .unwrap_or_default();
        let jsdoc_description = source_decls
            .get(&name)
            .and_then(|decl| jsdoc_comment_before_node(source, *decl));
        stories.push(CsfStory {
            source: bound_story_source(
                source,
                &name,
                template_name.as_deref(),
                &source_decls,
                &mutations,
            ),
            export_name: name,
            args,
            description: jsdoc_description.unwrap_or_default(),
            // The render is supplied by the bound template.
            has_render: true,
            parameters: BTreeMap::new(),
            globals: BTreeMap::new(),
            has_decorators: false,
            has_loaders: false,
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
            "parameters" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    meta.parameters = map;
                }
            }
            "globals" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    meta.globals = map;
                }
            }
            "globalTypes" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    meta.global_types = map;
                }
            }
            "decorators" => meta.has_decorators = true,
            "loaders" => meta.has_loaders = true,
            "tags" => meta.tags = string_array_values(source, value),
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
    extracted_source: Option<String>,
    jsdoc_description: Option<String>,
) -> CsfStory {
    let mut args = BTreeMap::new();
    let mut has_render = false;
    let mut parameters = BTreeMap::new();
    let mut globals = BTreeMap::new();
    let mut has_decorators = false;
    let mut has_loaders = false;
    for (key, value) in object_pairs(source, obj) {
        match key.as_str() {
            "args" => {
                if value.kind() == "object" {
                    args = resolve_object_args(value, source, scope, mutations);
                }
            }
            "render" => has_render = true,
            "parameters" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    parameters = map;
                }
            }
            "globals" => {
                if let CsfValue::Object(map) = value_of(source, value) {
                    globals = map;
                }
            }
            "decorators" => has_decorators = true,
            "loaders" => has_loaders = true,
            _ => {}
        }
    }
    let source = docs_source_override(&parameters).or(extracted_source);
    let description = docs_story_description_from_parameters(&parameters)
        .or(jsdoc_description)
        .unwrap_or_default();
    CsfStory {
        export_name: export_name.to_string(),
        source,
        args,
        description,
        has_render,
        parameters,
        globals,
        has_decorators,
        has_loaders,
    }
}

fn docs_story_description_from_parameters(
    parameters: &BTreeMap<String, CsfValue>,
) -> Option<String> {
    let Some(CsfValue::Object(docs)) = parameters.get("docs") else {
        return None;
    };
    let Some(CsfValue::Object(description)) = docs.get("description") else {
        return None;
    };
    match description.get("story") {
        Some(CsfValue::Str(text)) => Some(text.clone()),
        Some(CsfValue::Raw(text)) => Some(strip_quotes(text)),
        _ => None,
    }
}

fn docs_source_override(parameters: &BTreeMap<String, CsfValue>) -> Option<String> {
    let Some(CsfValue::Object(docs)) = parameters.get("docs") else {
        return None;
    };
    let Some(CsfValue::Object(source)) = docs.get("source") else {
        return None;
    };
    match source.get("code") {
        Some(CsfValue::Str(code)) => Some(code.clone()),
        Some(CsfValue::Raw(code)) => Some(strip_quotes(code)),
        _ => None,
    }
}

fn jsdoc_comment_before_node(source: &str, node: Node) -> Option<String> {
    let before = source[..node.start_byte()].trim_end();
    let close = before.rfind("*/")?;
    if close + 2 != before.len() {
        return None;
    }
    let open = before[..close].rfind("/**")?;
    let raw = &before[open + 3..close];
    let mut lines = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        let line = line.strip_prefix('*').unwrap_or(line).trim();
        if !line.is_empty() {
            lines.push(line.to_string());
        }
    }
    let text = lines.join("\n").trim().to_string();
    (!text.is_empty()).then_some(text)
}

fn string_array_values(source: &str, node: Node) -> Vec<String> {
    if node.kind() != "array" {
        return Vec::new();
    }
    named_children(node)
        .into_iter()
        .filter(|child| child.kind() == "string")
        .map(|child| strip_quotes(node_text(child, source)))
        .collect()
}

// ── CSF2 mutations + spread resolution ────────────────────────────────────────

fn bound_story_source(
    source: &str,
    story_name: &str,
    template_name: Option<&str>,
    decls: &BTreeMap<String, Node>,
    mutations: &BTreeMap<String, StoryMutation>,
) -> Option<String> {
    let mut chunks = Vec::new();
    if let Some(template) = template_name.and_then(|name| decls.get(name)) {
        chunks.push(node_text(*template, source).trim().to_string());
    }
    if let Some(story_decl) = decls.get(story_name) {
        let text = node_text(*story_decl, source).trim().to_string();
        if !chunks.iter().any(|chunk| chunk == &text) {
            chunks.push(text);
        }
    }
    if let Some(mutation) = mutations.get(story_name) {
        for statement in &mutation.source_statements {
            chunks.push(node_text(*statement, source).trim().to_string());
        }
    }
    if chunks.is_empty() {
        None
    } else {
        Some(chunks.join("\n"))
    }
}

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
    /// Full top-level assignment statements that should appear in the source panel.
    source_statements: Vec<Node<'a>>,
    /// `X.storyName = '...'` value, if assigned. (Surfaced for completeness;
    /// the story index keys off the export identifier today.)
    #[allow(dead_code)]
    story_name: Option<String>,
}

/// Walk the top level for `X.args = {...}` and `X.storyName = '...'`
/// assignment statements, grouping them by the mutated identifier `X`.
fn collect_story_mutations<'a>(
    source: &str,
    root: Node<'a>,
) -> BTreeMap<String, StoryMutation<'a>> {
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
            source_statements: Vec::new(),
            story_name: None,
        });
        match prop_name {
            "args" => {
                if rhs.kind() == "object" {
                    entry.args_pairs = object_member_nodes(rhs);
                }
                entry.source_statements.push(child);
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
                    let resolved = resolve_args_guarded(&base, source, scope, mutations, depth + 1);
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

/// Return `Template` when `value` is a `Template.bind({...})`-shaped call.
fn bind_template_name(source: &str, value: Node) -> Option<String> {
    // Unwrap `as`/`satisfies`/parens around the call (rare, but cheap).
    let call = unwrap_to_call(value)?;
    let callee = first_child_of_kind(call, "member_expression")?;
    let kids = named_children(callee);
    let base = kids.iter().find(|c| c.kind() == "identifier")?;
    let has_bind = kids
        .iter()
        .filter(|c| c.kind() == "property_identifier")
        .any(|c| node_text(*c, source) == "bind");
    has_bind.then(|| node_text(*base, source).to_string())
}

fn is_exported_callable_story(name: &str, value: Node) -> bool {
    starts_with_uppercase(name) && is_callable_story_value(value)
}

fn starts_with_uppercase(name: &str) -> bool {
    name.chars()
        .next()
        .map(|ch| ch.is_ascii_uppercase())
        .unwrap_or(false)
}

fn is_callable_story_value(value: Node) -> bool {
    match value.kind() {
        "arrow_function" | "function" | "function_declaration" | "call_expression" => true,
        "satisfies_expression" | "as_expression" | "parenthesized_expression" => {
            named_children(value)
                .into_iter()
                .any(is_callable_story_value)
        }
        _ => false,
    }
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
    let trimmed = raw.trim();
    let Some(first) = trimmed.chars().next() else {
        return String::new();
    };
    let Some(last) = trimmed.chars().next_back() else {
        return String::new();
    };
    if matches!(first, '"' | '\'' | '`') && first == last && trimmed.len() >= 2 {
        trimmed[first.len_utf8()..trimmed.len() - last.len_utf8()].to_string()
    } else {
        trimmed.to_string()
    }
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
        assert!(primary
            .source
            .as_deref()
            .unwrap()
            .contains("export const Primary: Story ="));

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
        let names: Vec<_> = parsed
            .stories
            .iter()
            .map(|s| s.export_name.as_str())
            .collect();
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
        let primary_source = primary.source.as_deref().unwrap();
        assert!(primary_source.contains("const Template = (args) => <Toggle {...args} />;"));
        assert!(primary_source.contains("const Primary = Template.bind({});"));
        assert!(primary_source.contains("Primary.args = { label: \"Hi\", on: true };"));

        let secondary = parsed
            .stories
            .iter()
            .find(|s| s.export_name == "Secondary")
            .unwrap();
        let secondary_source = secondary.source.as_deref().unwrap();
        assert!(secondary_source.contains("export const Secondary = Template.bind({});"));
        assert!(secondary_source.contains("Secondary.args = { label: \"Lo\" };"));
    }

    #[test]
    fn docs_source_code_overrides_extracted_source() {
        let src = r#"
import { Button } from './Button';
export default { title: 'Components/Button', component: Button };

export const Primary = {
  args: { label: 'Hidden' },
  parameters: { docs: { source: { code: '<Button label="Copy me" />' } } },
};
"#;
        let parsed = parse_csf(src, true).expect("parses");
        assert_eq!(
            parsed.stories[0].source.as_deref(),
            Some("<Button label=\"Copy me\" />")
        );
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
        assert_eq!(
            dynamic.args.get("only"),
            Some(&CsfValue::Number("9".into()))
        );
        assert!(
            !dynamic.args.contains_key("x"),
            "unresolvable spread dropped"
        );
    }

    #[test]
    fn parses_render_path_core_fields() {
        let src = r#"
import { Button } from './Button';
export default {
  title: 'Components/Button',
  component: Button,
  decorators: [(Story) => <section><Story /></section>],
  parameters: { layout: 'centered', chromatic: { disable: true } },
  globals: { theme: 'light' },
  globalTypes: { theme: { defaultValue: 'dark' } },
  loaders: [async () => ({ project: true })],
  tags: ['autodocs'],
};

export const Primary = {
  decorators: [(Story) => <div><Story /></div>],
  parameters: { layout: 'fullscreen' },
  globals: { locale: 'en' },
  loaders: [() => ({ story: true })],
};
"#;
        let parsed = parse_csf(src, true).expect("parses");
        assert!(parsed.meta.has_decorators);
        assert!(parsed.meta.has_loaders);
        assert_eq!(
            parsed.meta.parameters.get("layout"),
            Some(&CsfValue::Str("centered".into()))
        );
        assert!(parsed.meta.global_types.contains_key("theme"));
        assert_eq!(
            parsed.meta.globals.get("theme"),
            Some(&CsfValue::Str("light".into()))
        );
        assert_eq!(parsed.meta.tags, vec!["autodocs"]);

        let primary = &parsed.stories[0];
        assert!(primary.has_decorators);
        assert!(primary.has_loaders);
        assert_eq!(
            primary.parameters.get("layout"),
            Some(&CsfValue::Str("fullscreen".into()))
        );
        assert_eq!(
            primary.globals.get("locale"),
            Some(&CsfValue::Str("en".into()))
        );
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
// </HANDWRITE>
