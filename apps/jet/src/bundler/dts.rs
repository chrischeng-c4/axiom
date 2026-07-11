// <HANDWRITE gap="missing-generator:logic:d172c696" tracker="standardize-gap-projects-jet-src-bundler-dts-rs" reason="New isolatedDeclarations-style declaration emitter: parse a library entry with tree-sitter-typescript, walk top-level exported declarations, emit type/interface/enum decls verbatim and `export declare` signatures for explicitly-typed exported values, error on untyped exports, and return the assembled `<entry>.d.ts` text (external type imports preserved).">
//! isolatedDeclarations-style `.d.ts` emission for `jet build --lib`.
//!
//! Mirrors the TypeScript 5.5 `isolatedDeclarations` model where practical:
//! declarations are emitted from explicit export-boundary types or from a small
//! deterministic set of local return-expression inferences, never from a whole
//! program type-check. Per library entry we:
//!
//!   1. tree-sitter parse the entry source (TSX grammar, a superset that also
//!      parses plain TS/JS),
//!   2. walk the top-level statements in source order,
//!   3. for `interface` / `type` / `enum` declarations emit the declaration
//!      verbatim (with a leading `export`/`export declare`); for a `class`,
//!      reduce it to its public ambient surface — method bodies dropped to
//!      signatures, field initializers dropped, `private`/`protected`/
//!      `#private` members dropped, `async` stripped from ambient methods,
//!   4. for exported values (`export const`, `export function`) emit an
//!      `export declare`-style signature with the body dropped — requiring an
//!      explicit type annotation or a locally inferable return type
//!      (isolatedDeclarations: error otherwise),
//!   5. preserve `import`/`export … from "pkg"` re-exports so external type
//!      references still resolve,
//!   6. assemble and return the entry's `.d.ts` text.
//!
//! Shapes that cannot be handled cleanly are passed through best-effort with a
//! `// TODO(#171 follow-up)` marker rather than crashing the build.
//!
//! @issue #171
//! @issue #722
//! @issue #784
//! @issue #796
//! @issue #797
//! @issue #799
//! @issue #937

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tree_sitter::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclarationEmit {
    pub(crate) text: String,
    pub(crate) diagnostics: Vec<DtsDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DtsDiagnostic {
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) message: String,
}

impl DtsDiagnostic {
    fn new(node: Node, message: String) -> Self {
        let position = node.start_position();
        Self {
            line: position.row + 1,
            column: position.column + 1,
            message,
        }
    }
}

/// Emit the `.d.ts` text for one library entry's source.
///
/// `entry_source` is the raw TypeScript/TSX source of the entry module. The
/// returned string is the full `.d.ts` content (imports preserved, exported
/// declarations reduced to type-only signatures).
///
/// Errors (isolatedDeclarations contract): an exported `const`/`let`/`var` that
/// lacks an explicit type annotation, or an exported function/member whose
/// return type is neither explicit nor locally inferable, cannot have its type
/// emitted safely, so this returns `Err`.
pub fn emit_declarations(entry_source: &str) -> Result<String> {
    let emit = emit_declarations_with_diagnostics(entry_source)?;
    if emit.diagnostics.is_empty() {
        Ok(emit.text)
    } else {
        Err(anyhow!(format_diagnostics(&emit.diagnostics)))
    }
}

/// Emit declaration text plus all isolatedDeclarations diagnostics for one
/// source module. Fatal parser/setup errors still return `Err`; declaration
/// contract violations are collected in source order.
pub(crate) fn emit_declarations_with_diagnostics(entry_source: &str) -> Result<DeclarationEmit> {
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TSX.into();
    parser
        .set_language(&language)
        .map_err(|e| anyhow!("dts: failed to set tree-sitter TSX language: {e}"))?;
    let tree = parser
        .parse(entry_source, None)
        .ok_or_else(|| anyhow!("dts: failed to parse entry source"))?;
    let root = tree.root_node();

    let mut out = String::new();
    let mut diagnostics = Vec::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        match child.kind() {
            "import_statement" => {
                // Preserve imports verbatim: an emitted declaration may
                // reference an external type by name (`import type { T } from
                // "pkg"`). Keeping the import keeps the reference resolvable.
                push_line(&mut out, node_text(child, entry_source).trim_end());
            }
            "export_statement" => {
                emit_export_statement(child, entry_source, &mut out, &mut diagnostics)?;
            }
            // Top-level (non-exported) declarations are NOT part of the public
            // API surface, so they are dropped from the `.d.ts`. The exception
            // is an ambient declaration the author wrote by hand, which we
            // leave alone.
            "ambient_declaration" => {
                push_line(&mut out, node_text(child, entry_source).trim_end());
            }
            _ => {}
        }
    }

    Ok(DeclarationEmit {
        text: out,
        diagnostics,
    })
}

fn format_diagnostics(diagnostics: &[DtsDiagnostic]) -> String {
    let mut message = format!(
        "dts: isolatedDeclarations found {} error(s)",
        diagnostics.len()
    );
    for diagnostic in diagnostics {
        message.push_str(&format!(
            "\n  - line {}:{}: {}",
            diagnostic.line, diagnostic.column, diagnostic.message
        ));
    }
    message
}

/// Emit one top-level `export_statement` into `out`.
fn emit_export_statement(
    node: Node,
    source: &str,
    out: &mut String,
    diagnostics: &mut Vec<DtsDiagnostic>,
) -> Result<()> {
    // `export { A, B }` / `export { A } from "./x"` / `export type { … }` /
    // `export * from "./x"` — re-export forms have no inner declaration node.
    if let Some(lines) = svgr_reexport_declarations(node, source) {
        for line in lines {
            push_line(out, &line);
        }
        return Ok(());
    }
    if let Some(line) = reexport_line(node, source) {
        push_line(out, &line);
        return Ok(());
    }

    let is_default = has_child_kind(node, "default");

    // Find the declaration the export wraps.
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            // Pure type declarations: emit verbatim with `export`.
            "interface_declaration" | "type_alias_declaration" => {
                push_decl(out, "export ", node_text(child, source).trim_end());
                return Ok(());
            }
            "enum_declaration" => {
                // `const enum` and plain `enum` both emit verbatim; enums carry
                // their member values, which are part of the type surface.
                push_decl(out, "export declare ", node_text(child, source).trim_end());
                return Ok(());
            }
            "class_declaration" | "abstract_class_declaration" => {
                let is_abstract = child.kind() == "abstract_class_declaration";
                let decl = emit_class_declaration(child, source, diagnostics)?;
                // Ambient classes are valid as `export declare class` /
                // `export declare abstract class`; a default-exported class is
                // emitted as `export default class` (no `declare` — TS forbids
                // `declare` on a default-export class).
                let prefix = match (is_default, is_abstract) {
                    (true, true) => "export default abstract class ",
                    (true, false) => "export default class ",
                    (false, true) => "export declare abstract class ",
                    (false, false) => "export declare class ",
                };
                // `emit_class_declaration` returns the body starting at the
                // class name; prepend the chosen prefix.
                push_line(out, &format!("{prefix}{decl}"));
                return Ok(());
            }
            "function_declaration" | "generator_function_declaration" => {
                if let Some(sig) = emit_function_signature(child, source, diagnostics)? {
                    let prefix = if is_default {
                        "export default function "
                    } else {
                        "export declare function "
                    };
                    push_line(out, &format!("{prefix}{sig};"));
                }
                return Ok(());
            }
            // `export function f(): R;` with no body already parses as a
            // function_signature node.
            "function_signature" => {
                let text = node_text(child, source);
                let body = text.trim_start_matches("function").trim_start();
                let prefix = if is_default {
                    "export default function "
                } else {
                    "export declare function "
                };
                push_line(out, &format!("{prefix}{}", body.trim_end()));
                return Ok(());
            }
            "lexical_declaration" | "variable_declaration" => {
                emit_value_declaration(child, source, out, diagnostics)?;
                return Ok(());
            }
            // `export default <expr>` (identifier / call / object). Without an
            // explicit type at the boundary the declared type is unknowable —
            // emit `export default` of the referenced name when it is a plain
            // identifier, otherwise defer.
            "identifier" if is_default => {
                push_line(out, "export { default };");
                return Ok(());
            }
            // `export default (expr as Type)` / `export default (expr
            // satisfies Type)` — the annotation makes the declared type
            // statically determinable. Emit a synthetic `_default` of that
            // type and re-export it as the default.
            "as_expression" | "satisfies_expression" | "parenthesized_expression" if is_default => {
                if let Some(ty) = annotated_expression_type(child, source) {
                    push_line(out, &format!("declare const _default: {ty};"));
                    push_line(out, "export default _default;");
                    return Ok(());
                }
                // No statically-determinable type — fall through to the
                // graceful TODO skip below.
            }
            _ => {}
        }
    }

    // Anything else (e.g. `export default <complex untyped expr>`, decorators)
    // — the declared type is not statically determinable without inference.
    // Skip the body gracefully (emitting it verbatim would leak the
    // implementation into the `.d.ts`) and leave a clear marker so the gap is
    // visible without crashing the build.
    let snippet = node_text(node, source)
        .lines()
        .next()
        .unwrap_or("")
        .trim_end();
    push_line(
        out,
        &format!(
            "// TODO(#171 follow-up): export shape not statically determinable, \
             skipped: {snippet}"
        ),
    );
    Ok(())
}

/// Extract the explicit type annotation from a default-export expression that
/// carries one: `expr as Type`, `expr satisfies Type`, or a parenthesized
/// wrapper around either. Returns the type text, or `None` when the expression
/// has no boundary annotation.
fn annotated_expression_type(node: Node, source: &str) -> Option<String> {
    match node.kind() {
        // `(inner)` — the inner expression has no field name on this grammar,
        // so unwrap the first named child and recurse.
        "parenthesized_expression" => {
            let inner = first_named_child(node)?;
            annotated_expression_type(inner, source)
        }
        // `expr as Type` / `expr satisfies Type` — the type is the trailing
        // named child (`type` field is not set on this grammar).
        "as_expression" | "satisfies_expression" => {
            let ty = node
                .child_by_field_name("type")
                .or_else(|| last_named_child(node))?;
            Some(node_text(ty, source).trim().to_string())
        }
        _ => None,
    }
}

fn last_named_child<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let mut last = None;
    for child in node.named_children(&mut cursor) {
        last = Some(child);
    }
    last
}

fn first_named_child<'a>(node: Node<'a>) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        return Some(child);
    }
    None
}

/// Emit an exported value declaration (`export const`/`let`/`var`).
///
/// isolatedDeclarations: each declarator must carry an explicit type
/// annotation; otherwise we cannot emit its declared type without inference.
fn emit_value_declaration(
    node: Node,
    source: &str,
    out: &mut String,
    diagnostics: &mut Vec<DtsDiagnostic>,
) -> Result<()> {
    // `const` / `let` / `var` keyword text precedes the declarators.
    let kind_kw = leading_value_keyword(node, source).unwrap_or("const");

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() != "variable_declarator" {
            continue;
        }
        let Some(name_node) = child.child_by_field_name("name") else {
            diagnostics.push(DtsDiagnostic::new(
                child,
                format!("export {kind_kw} without a name"),
            ));
            continue;
        };
        let name = node_text(name_node, source);
        let type_node = child.child_by_field_name("type");
        match type_node {
            Some(t) => {
                // `type` field includes the leading `:` (type_annotation node).
                let annotation = node_text(t, source).trim();
                let annotation = annotation.trim_start_matches(':').trim();
                push_line(
                    out,
                    &format!("export declare {kind_kw} {name}: {annotation};"),
                );
            }
            None => {
                if kind_kw == "const" {
                    if let Some(inferred) = infer_variable_declarator_type(child, source) {
                        push_line(
                            out,
                            &format!("export declare {kind_kw} {name}: {inferred};"),
                        );
                        continue;
                    }
                }
                diagnostics.push(DtsDiagnostic::new(
                    child,
                    format!(
                        "isolatedDeclarations error — exported `{kind_kw} {name}` \
                         lacks an explicit type annotation; add `: <Type>` so its \
                         declaration can be emitted without type inference"
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn infer_variable_declarator_type(node: Node, source: &str) -> Option<String> {
    let value = node
        .child_by_field_name("value")
        .or_else(|| last_named_child(node))?;
    if let Some(inferred) = annotated_expression_type(value, source) {
        return Some(inferred);
    }
    if let Some(inferred) = infer_arrow_function_type(value, source) {
        return Some(inferred);
    }
    if let Some(inferred) = infer_arrow_body_return_object_type(value, source) {
        return Some(inferred);
    }
    infer_object_literal_type(value, source)
        .or_else(|| infer_single_arrow_property_object_literal_type(value, source))
}

// @spec .aw/tech-design/projects/jet/logic/jet-lib-dts-isolateddeclarations-false-positive-on-arrow-functio.md#logic
fn infer_arrow_function_type(node: Node, source: &str) -> Option<String> {
    if node.kind() != "arrow_function" {
        return None;
    }

    let params_node = node
        .child_by_field_name("parameters")
        .or_else(|| find_child_by_kind(node, "formal_parameters"))?;
    let params = normalize_arrow_parameters_for_type(node_text(params_node, source).trim())?;

    let ret_node = node.child_by_field_name("return_type")?;
    let ret = node_text(ret_node, source);
    let ret = ret.trim().trim_start_matches(':').trim();
    if ret.is_empty() {
        return None;
    }

    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| node_text(n, source).trim().to_string())
        .unwrap_or_default();
    Some(format!("{type_params}{params} => {ret}"))
}

// @spec .aw/tech-design/projects/jet/logic/jet-lib-dts-isolateddeclarations-false-positive-on-arrow-functio.md#logic
//
// Narrow fallback for #1264: an arrow function with no explicit `return_type`
// field (handled above by `infer_arrow_function_type`) whose body is a
// `statement_block` containing exactly one `return_statement` of an object
// literal. Member typing is delegated to the existing
// `infer_object_literal_type` routine so no new member-inference logic is
// introduced; any other body shape (multiple statements, a non-object
// return, or a partially-typed object literal) falls through unchanged to
// the caller's fail-loud isolatedDeclarations diagnostic.
fn infer_arrow_body_return_object_type(node: Node, source: &str) -> Option<String> {
    if node.kind() != "arrow_function" {
        return None;
    }
    // Arrows with their own explicit return type are handled by
    // `infer_arrow_function_type` before this fallback is tried.
    if node.child_by_field_name("return_type").is_some() {
        return None;
    }

    let body = node.child_by_field_name("body")?;
    if body.kind() != "statement_block" {
        return None;
    }

    let mut cursor = body.walk();
    let mut statements = body.named_children(&mut cursor);
    let statement = statements.next()?;
    if statements.next().is_some() {
        // More than one statement in the body -- not the narrow
        // single-return shape this fallback covers.
        return None;
    }
    if statement.kind() != "return_statement" {
        return None;
    }
    let returned = first_named_child(statement)?;
    let object_type = infer_object_literal_type(returned, source)?;

    let params_node = node
        .child_by_field_name("parameters")
        .or_else(|| find_child_by_kind(node, "formal_parameters"))?;
    let params = normalize_arrow_parameters_for_type(node_text(params_node, source).trim())?;

    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| node_text(n, source).trim().to_string())
        .unwrap_or_default();
    Some(format!("{type_params}{params} => {object_type}"))
}

fn normalize_arrow_parameters_for_type(params: &str) -> Option<String> {
    let inner = params
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(params)
        .trim();
    if inner.is_empty() {
        return Some("()".to_string());
    }

    let empty_param_types = HashMap::new();
    let mut normalized = Vec::new();
    for raw_param in split_top_level(inner, ',') {
        let raw_param = raw_param.trim();
        let is_rest = raw_param.starts_with("...");
        let param_without_rest = raw_param.trim_start_matches("...").trim();
        let (param_head, default_value) = split_once_top_level(param_without_rest, '=')
            .map(|(left, right)| (left.trim(), Some(right.trim())))
            .unwrap_or((param_without_rest, None));
        let param = param_head.trim();
        if param.is_empty() {
            continue;
        }
        if let Some((name, ty)) = split_once_top_level(param, ':') {
            let name = name.trim();
            let ty = ty.trim();
            if name.is_empty() || ty.is_empty() {
                return None;
            }
            let optional = name.ends_with('?') || default_value.is_some();
            let name = name.trim_end_matches('?').trim();
            let is_binding_pattern = is_supported_binding_pattern(name);
            if !is_identifier(name) && !is_binding_pattern {
                return None;
            }
            if is_binding_pattern && optional {
                return None;
            }
            let rest = if is_rest { "..." } else { "" };
            let marker = if optional && !is_rest && !is_binding_pattern {
                "?"
            } else {
                ""
            };
            normalized.push(format!("{rest}{name}{marker}: {ty}"));
            continue;
        }
        let Some(default_value) = default_value else {
            return None;
        };
        if is_rest {
            return None;
        }
        let name = param.trim_end_matches('?').trim();
        if !is_identifier(name) {
            return None;
        }
        let ty = infer_expression_type(default_value, &empty_param_types)?;
        normalized.push(format!("{name}?: {ty}"));
    }
    Some(format!("({})", normalized.join(", ")))
}

fn is_supported_binding_pattern(name: &str) -> bool {
    (name.starts_with('{') && name.ends_with('}')) || (name.starts_with('[') && name.ends_with(']'))
}

fn infer_object_literal_type(node: Node, source: &str) -> Option<String> {
    if node.kind() != "object" {
        return None;
    }
    let text = node_text(node, source).trim();
    infer_object_literal_type_from_text(text)
}

fn infer_object_literal_type_from_text(text: &str) -> Option<String> {
    let inner = text.strip_prefix('{')?.strip_suffix('}')?.trim();
    if inner.is_empty() {
        return Some("{}".to_string());
    }

    let mut members = Vec::new();
    let empty_param_types = HashMap::new();
    for raw_property in split_top_level(inner, ',') {
        let property = raw_property.trim();
        if property.is_empty() {
            continue;
        }
        if property.starts_with("...") || property.starts_with('[') {
            return None;
        }
        if let Some(member) = infer_object_method_member_type(property) {
            members.push(format!("    {member};"));
            continue;
        }
        let (key, value) = split_once_top_level(property, ':')?;
        let key = key.trim();
        if !is_supported_object_literal_key(key) {
            return None;
        }
        let value = value.trim();
        let ty = infer_arrow_function_type_from_text(value)
            .or_else(|| {
                if value.starts_with('{') && value.ends_with('}') {
                    infer_object_literal_type_from_text(value)
                } else {
                    None
                }
            })
            .or_else(|| infer_expression_type(value, &empty_param_types))?;
        members.push(format!("    {key}: {ty};"));
    }

    if members.is_empty() {
        return Some("{}".to_string());
    }
    Some(format!("{{\n{}\n}}", members.join("\n")))
}

fn is_supported_object_literal_key(key: &str) -> bool {
    is_identifier(key) || is_string_literal(key) || is_number_literal(key)
}

fn infer_single_arrow_property_object_literal_type(node: Node, source: &str) -> Option<String> {
    if node.kind() != "object" {
        return None;
    }
    let text = node_text(node, source).trim();
    let inner = text.strip_prefix('{')?.strip_suffix('}')?.trim();
    let (key, value) = split_once_top_level(inner, ':')?;
    let key = key.trim();
    if !is_supported_object_literal_key(key) {
        return None;
    }
    let ty = infer_arrow_function_type_from_text(value.trim().trim_end_matches(','))?;
    Some(format!(
        "{{
    {key}: {ty};
}}"
    ))
}

fn infer_object_method_member_type(property: &str) -> Option<String> {
    let open = property.find('(')?;
    let close = matching_delimiter(property, open, '(', ')')?;
    let prefix = property[..open].trim();
    let name = prefix.strip_prefix("async").unwrap_or(prefix).trim();
    if !is_supported_object_literal_key(name) {
        return None;
    }
    let params = &property[open..=close];
    let rest = property[close + 1..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    let body_start = rest.find('{')?;
    let ret = rest[..body_start].trim();
    if ret.is_empty() {
        return None;
    }
    Some(format!("{name}{params}: {ret}"))
}

fn infer_arrow_function_type_from_text(expr: &str) -> Option<String> {
    let (left, _) = split_once_top_level_arrow(expr)?;
    let (params, ret) = split_arrow_head_params_and_return(left)?;
    let params = normalize_arrow_parameters_for_type(params.trim())?;
    let ret = ret.trim();
    if ret.is_empty() {
        return None;
    }
    Some(format!("{params} => {ret}"))
}

fn split_arrow_head_params_and_return(head: &str) -> Option<(&str, &str)> {
    let head = head.trim();
    let head = head.strip_prefix("async").unwrap_or(head).trim_start();
    if head.starts_with('(') {
        let close = matching_delimiter(head, 0, '(', ')')?;
        let params = &head[..=close];
        let rest = head[close + 1..].trim_start();
        let ret = rest.strip_prefix(':')?.trim_start();
        return Some((params, ret));
    }
    split_once_top_level(head, ':')
}

fn split_once_top_level_arrow(text: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in text.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            '=' if depth == 0 && text[idx..].starts_with("=>") => {
                return Some((&text[..idx], &text[idx + 2..]));
            }
            _ => {}
        }
    }
    None
}

fn matching_delimiter(text: &str, open_idx: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in text.char_indices().skip_while(|(idx, _)| *idx < open_idx) {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            _ if ch == open => depth += 1,
            _ if ch == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

/// Build a function signature string (name + type params + params + return
/// type) with the body dropped.
///
/// isolatedDeclarations: an exported function should declare its return type
/// explicitly. For compatibility with `tsc --declaration` on common library
/// shapes, the emitter also infers a small set of local return expressions
/// (`number`, `string`, `boolean`, primitive unions, and `void`) instead of
/// silently turning them into implicit `any`.
fn emit_function_signature(
    node: Node,
    source: &str,
    diagnostics: &mut Vec<DtsDiagnostic>,
) -> Result<Option<String>> {
    let Some(name_node) = node.child_by_field_name("name") else {
        diagnostics.push(DtsDiagnostic::new(
            node,
            "exported function without a name".to_string(),
        ));
        return Ok(None);
    };
    let name = node_text(name_node, source);

    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| node_text(n, source))
        .unwrap_or("");
    let params = node
        .child_by_field_name("parameters")
        .map(|n| node_text(n, source))
        .unwrap_or("()");
    let ret = match node.child_by_field_name("return_type") {
        Some(n) => node_text(n, source).to_string(),
        None => infer_function_return_type(node, source)?
            .map(|ty| format!(": {ty}"))
            .unwrap_or_else(|| {
                diagnostics.push(DtsDiagnostic::new(
                    node,
                    format!(
                        "isolatedDeclarations error — exported function `{name}` \
                         lacks an explicit or locally inferable return type; add \
                         `: <Type>` so its declaration can be emitted safely"
                    ),
                ));
                String::new()
            }),
    };
    if ret.is_empty() && !matches!(node.child_by_field_name("return_type"), Some(_)) {
        return Ok(None);
    }

    Ok(Some(format!("{name}{type_params}{params}{ret}")))
}

/// Emit a class declaration reduced to its public ambient surface.
///
/// Returns the text *from the class name onward* (the caller supplies the
/// `export declare class ` / `export default class ` prefix and any
/// `abstract`). The reduction:
///
///   * header: `Name<T…> extends Base<…> implements I…` (name, type params,
///     and heritage clauses are reproduced from their structured nodes),
///   * `method_definition` → signature only — the `{ … }` body is dropped and
///     a `;` terminator is appended. `static` / `readonly` / `get` / `set`
///     modifiers are kept (valid in ambient context); `async` is dropped (an
///     ambient method cannot be `async`) but the declared return type is kept,
///   * `public_field_definition` → `field: Type;` — the initializer is
///     dropped, `static` / `readonly` are kept,
///   * `private` / `protected` accessibility members are dropped, as are
///     `#private` fields and methods (not part of the public ambient surface
///     for an isolatedDeclarations-style emit).
fn emit_class_declaration(
    node: Node,
    source: &str,
    diagnostics: &mut Vec<DtsDiagnostic>,
) -> Result<String> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(n, source))
        .unwrap_or("");
    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| node_text(n, source))
        .unwrap_or("");

    // Heritage: `extends …` / `implements …` clauses, reproduced verbatim.
    let mut heritage = String::new();
    if let Some(class_heritage) = find_child_by_kind(node, "class_heritage") {
        let mut cursor = class_heritage.walk();
        for clause in class_heritage.named_children(&mut cursor) {
            if matches!(clause.kind(), "extends_clause" | "implements_clause") {
                heritage.push(' ');
                heritage.push_str(node_text(clause, source).trim());
            }
        }
    }

    let mut header = format!("{name}{type_params}{heritage}");

    // Reduce the class body member by member.
    let Some(body) = node.child_by_field_name("body") else {
        // No body field — emit an empty ambient class shape.
        header.push_str(" {\n}");
        return Ok(header);
    };

    let mut members = String::new();
    let mut cursor = body.walk();
    for member in body.named_children(&mut cursor) {
        if let Some(line) = reduce_class_member(member, source, diagnostics)? {
            members.push_str("    ");
            members.push_str(&line);
            members.push('\n');
        }
    }

    let decl = if members.is_empty() {
        format!("{header} {{\n}}")
    } else {
        format!("{header} {{\n{members}}}")
    };
    Ok(decl)
}

/// Reduce one class-body member to its ambient signature line (without the
/// trailing newline / leading indentation), or `None` when the member is
/// dropped (`private` / `protected` / `#private`, or an unreducible shape).
fn reduce_class_member(
    node: Node,
    source: &str,
    diagnostics: &mut Vec<DtsDiagnostic>,
) -> Result<Option<String>> {
    let line = match node.kind() {
        "method_definition" => reduce_method(node, source, diagnostics),
        "public_field_definition" => reduce_field(node, source, diagnostics),
        // index signatures (`[key: string]: T;`) are already declaration-only.
        "index_signature" => Ok(Some(format!(
            "{};",
            node_text(node, source).trim_end_matches(';')
        ))),
        // Static initialization blocks, decorators-only members, etc. carry no
        // public type surface — drop them.
        _ => Ok(None),
    }?;
    Ok(line)
}

/// Reduce a `method_definition` to a signature line. Drops the body and
/// `async`; keeps `static` / `get` / `set` / `readonly` modifiers.
fn reduce_method(
    node: Node,
    source: &str,
    diagnostics: &mut Vec<DtsDiagnostic>,
) -> Result<Option<String>> {
    // `#private` methods are never part of the public surface.
    let Some(name_node) = node.child_by_field_name("name") else {
        return Ok(None);
    };
    if name_node.kind() == "private_property_identifier" {
        return Ok(None);
    }
    // `private` / `protected` members are dropped from the ambient surface.
    if has_dropped_accessibility(node, source) {
        return Ok(None);
    }

    let name = node_text(name_node, source);

    // Preserved leading modifiers, in source order, minus `async`.
    let mut modifiers = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Stop once we reach the name; anything after is params/return/body.
        if child.id() == name_node.id() {
            break;
        }
        match child.kind() {
            "static" | "get" | "set" | "readonly" => {
                modifiers.push_str(node_text(child, source));
                modifiers.push(' ');
            }
            // `async` is invalid on an ambient method — drop it, keep the
            // declared return type.
            "async" => {}
            // `accessibility_modifier` holding `public` is harmless to keep;
            // `private`/`protected` were already filtered above.
            "accessibility_modifier" => {
                let kw = node_text(child, source).trim();
                if kw == "public" {
                    modifiers.push_str(kw);
                    modifiers.push(' ');
                }
            }
            _ => {}
        }
    }

    // `?` optional-method marker sits between the name and parameters.
    let optional = if has_child_kind(node, "?") { "?" } else { "" };

    let params = node
        .child_by_field_name("parameters")
        .map(|n| node_text(n, source))
        .unwrap_or("()");
    let is_constructor = name == "constructor";
    let is_setter = has_child_kind(node, "set");
    let ret = match node.child_by_field_name("return_type") {
        Some(n) => node_text(n, source).to_string(),
        None if is_constructor || is_setter => String::new(),
        None => infer_function_return_type(node, source)?
            .map(|ty| format!(": {ty}"))
            .unwrap_or_else(|| {
                diagnostics.push(DtsDiagnostic::new(
                    node,
                    format!(
                        "isolatedDeclarations error — exported class member `{name}` \
                         lacks an explicit or locally inferable return type; add \
                         `: <Type>` so its declaration can be emitted safely"
                    ),
                ));
                String::new()
            }),
    };
    if ret.is_empty()
        && !is_constructor
        && !is_setter
        && node.child_by_field_name("return_type").is_none()
    {
        return Ok(None);
    }

    Ok(Some(format!("{modifiers}{name}{optional}{params}{ret};")))
}

/// Infer a safe return type for a function-like node from its local body. This
/// is intentionally bounded: it handles primitive literal returns, typed
/// parameter identifiers, template strings, and arithmetic/string/boolean
/// binary expressions. Unknown shapes return `None`, keeping the build
/// fail-loud instead of emitting `any`.
fn infer_function_return_type(node: Node, source: &str) -> Result<Option<String>> {
    let param_types = node
        .child_by_field_name("parameters")
        .map(|n| parse_parameter_type_map(node_text(n, source)))
        .unwrap_or_default();
    let Some(body) = node
        .child_by_field_name("body")
        .or_else(|| find_child_by_kind(node, "statement_block"))
    else {
        return Ok(None);
    };

    let mut returns = Vec::new();
    collect_return_statement_types(body, source, &param_types, &mut returns);
    if returns.is_empty() {
        return Ok(Some("void".to_string()));
    }
    union_return_types(returns)
}

fn collect_return_statement_types(
    node: Node,
    source: &str,
    param_types: &HashMap<String, String>,
    out: &mut Vec<Option<String>>,
) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        match child.kind() {
            "return_statement" => out.push(infer_return_statement_type(child, source, param_types)),
            kind if nested_return_scope(kind) => {}
            _ => collect_return_statement_types(child, source, param_types, out),
        }
    }
}

fn nested_return_scope(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "generator_function_declaration"
            | "function"
            | "function_expression"
            | "generator_function"
            | "arrow_function"
            | "method_definition"
            | "class_declaration"
            | "abstract_class_declaration"
            | "class"
    )
}

fn infer_return_statement_type(
    node: Node,
    source: &str,
    param_types: &HashMap<String, String>,
) -> Option<String> {
    if let Some(expr_node) = first_named_child(node) {
        if let Some(ty) = annotated_expression_type(expr_node, source) {
            return Some(ty);
        }
    }

    let text = node_text(node, source).trim();
    let expr = text
        .strip_prefix("return")
        .unwrap_or(text)
        .trim()
        .trim_end_matches(';')
        .trim();
    if expr.is_empty() {
        return Some("void".to_string());
    }
    infer_expression_type(expr, param_types)
}

fn union_return_types(types: Vec<Option<String>>) -> Result<Option<String>> {
    let mut known = Vec::new();
    for ty in types {
        let Some(ty) = ty else {
            return Ok(None);
        };
        known.push(ty);
    }
    if known.is_empty() {
        return Ok(Some("void".to_string()));
    }

    let mixed_with_void = known.len() > 1 && known.iter().any(|ty| ty == "void");
    let mut unique = Vec::new();
    for ty in known {
        let ty = if mixed_with_void && ty == "void" {
            "undefined".to_string()
        } else {
            ty
        };
        if !unique.iter().any(|seen| seen == &ty) {
            unique.push(ty);
        }
    }
    Ok(Some(unique.join(" | ")))
}

fn parse_parameter_type_map(params: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let inner = params
        .trim()
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(params)
        .trim();
    if inner.is_empty() {
        return out;
    }

    for raw_param in split_top_level(inner, ',') {
        let param_head = split_once_top_level(&raw_param, '=')
            .map(|(left, _)| left)
            .unwrap_or(raw_param.as_str());
        let param = param_head.trim().trim_start_matches("...").trim();
        let Some((name, ty)) = split_once_top_level(param, ':') else {
            continue;
        };
        let name = name.trim().trim_end_matches('?').trim();
        if is_identifier(name) {
            out.insert(name.to_string(), ty.trim().to_string());
        }
    }
    out
}

fn infer_expression_type(expr: &str, param_types: &HashMap<String, String>) -> Option<String> {
    let expr = trim_wrapping_parens(expr.trim());
    if expr.is_empty() {
        return None;
    }
    if is_string_literal(expr) || expr.starts_with('`') {
        return Some("string".to_string());
    }
    if is_number_literal(expr) {
        return Some("number".to_string());
    }
    if matches!(expr, "true" | "false") {
        return Some("boolean".to_string());
    }
    if matches!(expr, "null" | "undefined") {
        return Some(expr.to_string());
    }
    if is_identifier(expr) {
        return param_types.get(expr).cloned();
    }

    if let Some((left, op, right)) = split_binary_expression(expr) {
        let left_ty = infer_expression_type(left, param_types)?;
        let right_ty = infer_expression_type(right, param_types)?;
        return match op {
            "+" if left_ty == "string" || right_ty == "string" => Some("string".to_string()),
            "+" if left_ty == "number" && right_ty == "number" => Some("number".to_string()),
            "-" | "*" | "/" | "%" if left_ty == "number" && right_ty == "number" => {
                Some("number".to_string())
            }
            "===" | "!==" | "==" | "!=" | "<" | "<=" | ">" | ">=" => Some("boolean".to_string()),
            "&&" | "||" if left_ty == right_ty => Some(left_ty),
            "??" if left_ty == right_ty => Some(left_ty),
            _ => None,
        };
    }

    None
}

fn trim_wrapping_parens(mut expr: &str) -> &str {
    loop {
        let trimmed = expr.trim();
        if !(trimmed.starts_with('(') && trimmed.ends_with(')')) {
            return trimmed;
        }
        if !outer_parens_wrap(trimmed) {
            return trimmed;
        }
        expr = &trimmed[1..trimmed.len() - 1];
    }
}

fn outer_parens_wrap(expr: &str) -> bool {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in expr.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 && idx != expr.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

fn is_string_literal(expr: &str) -> bool {
    (expr.starts_with('"') && expr.ends_with('"'))
        || (expr.starts_with('\'') && expr.ends_with('\''))
}

fn is_number_literal(expr: &str) -> bool {
    let expr = expr.trim();
    if expr.is_empty() {
        return false;
    }
    let expr = expr.strip_prefix('-').unwrap_or(expr);
    expr.parse::<f64>().is_ok()
        || expr.starts_with("0x")
        || expr.starts_with("0b")
        || expr.starts_with("0o")
}

fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn split_binary_expression(expr: &str) -> Option<(&str, &str, &str)> {
    const GROUPS: &[&[&str]] = &[
        &["??", "||"],
        &["&&"],
        &["===", "!==", "==", "!=", "<=", ">=", "<", ">"],
        &["+", "-"],
        &["*", "/", "%"],
    ];
    for ops in GROUPS {
        if let Some((idx, op)) = find_top_level_operator(expr, ops) {
            let left = expr[..idx].trim();
            let right = expr[idx + op.len()..].trim();
            if !left.is_empty() && !right.is_empty() {
                return Some((left, op, right));
            }
        }
    }
    None
}

fn find_top_level_operator<'a>(expr: &str, ops: &'a [&str]) -> Option<(usize, &'a str)> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    let mut found = None;
    for (idx, ch) in expr.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ if depth == 0 => {
                for op in ops {
                    if expr[idx..].starts_with(op) && !is_unary_sign(expr, idx, op) {
                        found = Some((idx, *op));
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    found
}

fn is_unary_sign(expr: &str, idx: usize, op: &str) -> bool {
    if op != "+" && op != "-" {
        return false;
    }
    let left = expr[..idx].trim_end();
    left.is_empty()
        || left.ends_with('(')
        || left.ends_with('[')
        || left.ends_with('{')
        || left.ends_with(',')
        || left.ends_with('=')
        || left.ends_with(':')
        || left.ends_with('?')
        || left.ends_with('+')
        || left.ends_with('-')
        || left.ends_with('*')
        || left.ends_with('/')
        || left.ends_with('%')
        || left.ends_with('!')
        || left.ends_with('<')
        || left.ends_with('>')
        || left.ends_with('&')
        || left.ends_with('|')
}

fn split_top_level(text: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    let mut prev = '\0';
    for (idx, ch) in text.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                prev = ch;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                prev = ch;
                continue;
            }
            if ch == q {
                quote = None;
            }
            prev = ch;
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            // `=>` (arrow function token): the trailing `>` is not a
            // closing generic bracket, so it must not decrement depth --
            // otherwise multiple arrow-typed members/params in the same
            // top-level list (e.g. two object properties whose arrow
            // return types are themselves generic, `Promise<string>`)
            // would be merged into one part (#1264).
            '>' if prev == '=' => {}
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            _ if ch == delimiter && depth == 0 => {
                parts.push(text[start..idx].to_string());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
        prev = ch;
    }
    parts.push(text[start..].to_string());
    parts
}

fn split_once_top_level<'a>(text: &'a str, delimiter: char) -> Option<(&'a str, &'a str)> {
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    let mut prev = '\0';
    for (idx, ch) in text.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
                prev = ch;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                prev = ch;
                continue;
            }
            if ch == q {
                quote = None;
            }
            prev = ch;
            continue;
        }
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            // See the matching comment in `split_top_level` (#1264): `=>`'s
            // trailing `>` is not a closing generic bracket.
            '>' if prev == '=' => {}
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' | '>' => depth -= 1,
            _ if ch == delimiter && depth == 0 => {
                return Some((&text[..idx], &text[idx + ch.len_utf8()..]));
            }
            _ => {}
        }
        prev = ch;
    }
    None
}

/// Reduce a `public_field_definition` to a `field: Type;` line, dropping the
/// initializer. Keeps `static` / `readonly`. Drops `private` / `protected` /
/// `#private` fields.
fn reduce_field(
    node: Node,
    source: &str,
    diagnostics: &mut Vec<DtsDiagnostic>,
) -> Result<Option<String>> {
    let Some(name_node) = node.child_by_field_name("name") else {
        return Ok(None);
    };
    if name_node.kind() == "private_property_identifier" {
        return Ok(None);
    }
    if has_dropped_accessibility(node, source) {
        return Ok(None);
    }

    let name = node_text(name_node, source);

    let mut modifiers = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.id() == name_node.id() {
            break;
        }
        match child.kind() {
            "static" | "readonly" => {
                modifiers.push_str(node_text(child, source));
                modifiers.push(' ');
            }
            "accessibility_modifier" => {
                let kw = node_text(child, source).trim();
                if kw == "public" {
                    modifiers.push_str(kw);
                    modifiers.push(' ');
                }
            }
            _ => {}
        }
    }

    // `?` / `!` definite-assignment markers sit between name and type.
    let marker = if has_child_kind(node, "?") {
        "?"
    } else if has_child_kind(node, "!") {
        "!"
    } else {
        ""
    };

    let ty = node
        .child_by_field_name("type")
        .map(|n| node_text(n, source).trim().to_string())
        .unwrap_or_default();
    if ty.is_empty() {
        diagnostics.push(DtsDiagnostic::new(
            node,
            format!(
                "isolatedDeclarations error — exported class field `{name}` \
                 lacks an explicit type annotation; add `: <Type>` so its \
                 declaration can be emitted without type inference"
            ),
        ));
        return Ok(None);
    }

    Ok(Some(format!("{modifiers}{name}{marker}{ty};")))
}

/// True when the member carries a `private` or `protected` accessibility
/// modifier (these are dropped from the ambient surface).
fn has_dropped_accessibility(node: Node, source: &str) -> bool {
    find_child_by_kind(node, "accessibility_modifier")
        .map(|m| {
            let kw = node_text(m, source).trim();
            kw == "private" || kw == "protected"
        })
        .unwrap_or(false)
}

/// First *named* child of `node` with the given kind.
fn find_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

/// Recognize a re-export `export_statement` (`export { … }`, `export { … }
/// from "x"`, `export * from "x"`, `export type { … } from "x"`) and return
/// the line to emit, or `None` when the statement wraps a declaration.
fn reexport_line(node: Node, source: &str) -> Option<String> {
    let has_clause = has_child_kind(node, "export_clause");
    let has_namespace = has_child_kind(node, "namespace_export") || star_export(node, source);
    if !has_clause && !has_namespace {
        return None;
    }
    // A re-export never wraps a declaration node; emit verbatim.
    Some(node_text(node, source).trim_end().to_string())
}

fn svgr_reexport_declarations(node: Node, source: &str) -> Option<Vec<String>> {
    if !has_child_kind(node, "export_clause") {
        return None;
    }
    let text = node_text(node, source).trim_end();
    let spec = export_from_specifier(text)?;
    if !is_svg_specifier_for_dts(spec) {
        return None;
    }
    let aliases = svgr_reexport_aliases(text);
    if aliases.is_empty() {
        return None;
    }

    let mut lines = vec!["import type { FC, SVGProps } from \"react\";".to_string()];
    for alias in aliases {
        lines.push(format!(
            "export declare const {alias}: FC<SVGProps<SVGSVGElement>>;"
        ));
    }
    Some(lines)
}

fn export_from_specifier(text: &str) -> Option<&str> {
    let after = text.rsplit_once(" from ")?.1.trim();
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn is_svg_specifier_for_dts(spec: &str) -> bool {
    let path = spec.split(['?', '#']).next().unwrap_or(spec);
    path.ends_with(".svg")
}

fn svgr_reexport_aliases(text: &str) -> Vec<String> {
    let Some(open) = text.find('{') else {
        return Vec::new();
    };
    let Some(close) = text[open..].find('}').map(|idx| open + idx) else {
        return Vec::new();
    };
    text[open + 1..close]
        .split(',')
        .filter_map(|binding| svgr_reexport_alias(binding.trim()))
        .collect()
}

fn svgr_reexport_alias(binding: &str) -> Option<String> {
    let mut parts = binding.split_whitespace();
    let first = parts.next()?;
    if first != "ReactComponent" {
        return None;
    }
    match (parts.next(), parts.next(), parts.next()) {
        (None, None, None) => Some(first.to_string()),
        (Some("as"), Some(alias), None) if is_identifier(alias) => Some(alias.to_string()),
        _ => None,
    }
}

/// Detect `export * from "x"` whose `*` is an anonymous token, not a named
/// child node.
fn star_export(node: Node, source: &str) -> bool {
    let text = node_text(node, source).trim_start();
    text.starts_with("export *") || text.starts_with("export type *")
}

/// Extract the leading `const`/`let`/`var` keyword of a value declaration.
fn leading_value_keyword<'a>(node: Node, source: &'a str) -> Option<&'a str> {
    let text = node_text(node, source).trim_start();
    for kw in ["const", "let", "var"] {
        if text.starts_with(kw) {
            return Some(kw);
        }
    }
    None
}

fn has_child_kind(node: Node, kind: &str) -> bool {
    let mut cursor = node.walk();
    let found = node.children(&mut cursor).any(|c| c.kind() == kind);
    found
}

fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

/// Push `line` to `out`, ensuring a trailing newline. Skips empty lines.
fn push_line(out: &mut String, line: &str) {
    if line.is_empty() {
        return;
    }
    out.push_str(line);
    out.push('\n');
}

/// Push a declaration as `<prefix><decl>` on its own line(s).
fn push_decl(out: &mut String, prefix: &str, decl: &str) {
    out.push_str(prefix);
    out.push_str(decl);
    if !decl.ends_with('\n') {
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_exported_interface_verbatim() {
        let src = "export interface User { id: number; name: string; }\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export interface User"),
            "interface emitted verbatim, got:\n{dts}"
        );
        assert!(dts.contains("id: number"), "members preserved, got:\n{dts}");
    }

    #[test]
    fn emits_type_alias_verbatim() {
        let src = "export type ID = string | number;\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export type ID = string | number"),
            "type alias emitted verbatim, got:\n{dts}"
        );
    }

    #[test]
    fn emits_function_signature_without_body() {
        let src = "export function add(a: number, b: number): number { return a + b; }\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare function add(a: number, b: number): number;"),
            "function reduced to declare signature, got:\n{dts}"
        );
        assert!(
            !dts.contains("return a + b"),
            "function body must be dropped, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_function_number_return() {
        let src = "export function add(a: number, b: number) { return a + b; }\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare function add(a: number, b: number): number;"),
            "function return inferred from typed numeric params, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_function_as_expression_return() {
        let src = r#"export interface UploadApi {
    open(): Promise<void>;
}
export function createUploadApi() {
    const api = {};
    return api as UploadApi;
}
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare function createUploadApi(): UploadApi;"),
            "return as-expression should use its asserted type, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_class_member_string_return() {
        let src = r#"export class Greeter {
    greet(name: string) { return `hi ${name}`; }
}
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("greet(name: string): string;"),
            "method return inferred from template string, got:\n{dts}"
        );
    }

    #[test]
    fn uninferrable_exported_function_return_errors() {
        let src = "export function makeThing() { return createThing(); }\n";
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("locally inferable return type"),
            "unknown return expression must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn emits_typed_const_signature() {
        let src = "export const VERSION: string = \"1.0.0\";\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const VERSION: string;"),
            "typed const reduced to declare signature, got:\n{dts}"
        );
        assert!(
            !dts.contains("1.0.0"),
            "const initializer must be dropped, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_const_as_expression_signature() {
        let src = r#"export interface UploadApi {
    open(): Promise<void>;
}
export const uploadApi = {
    async open() {},
} as UploadApi;
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const uploadApi: UploadApi;"),
            "const as-expression should use its asserted type, got:\n{dts}"
        );
    }

    #[test]
    fn infers_plain_object_literal_const_signature() {
        let src = r#"export const UPLOAD_ACCEPT_TYPE = {
    JPG: "image/jpeg",
    PNG: "image/png",
    PDF: "application/pdf",
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const UPLOAD_ACCEPT_TYPE: {"),
            "object literal const should synthesize a declaration type, got:\n{dts}"
        );
        for expected in ["JPG: string;", "PNG: string;", "PDF: string;"] {
            assert!(
                dts.contains(expected),
                "object property {expected:?} should be emitted, got:\n{dts}"
            );
        }
        assert!(
            !dts.contains("image/jpeg"),
            "object literal values must not leak into .d.ts, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_const_nested_plain_object_literal_signature() {
        // R1 (#1263): minimal repro -- an object literal member whose value
        // is itself a plain, all-string-literal nested object literal
        // (nesting depth 2) must infer instead of raising a false-positive
        // isolatedDeclarations error.
        let src = r#"export const flatLiteral = {
    ltr: "ltr",
    rtl: "rtl",
};

export const nestedLiteral = {
    ltr: "ltr",
    heading: {
        h1: "editor-heading--h1",
    },
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const flatLiteral: {"),
            "flat object literal const should still synthesize a declaration type, got:\n{dts}"
        );
        assert!(
            dts.contains("export declare const nestedLiteral: {"),
            "nested object literal const should synthesize a declaration type, got:\n{dts}"
        );
        assert!(
            dts.contains("ltr: string;"),
            "flat string member should be emitted, got:\n{dts}"
        );
        assert!(
            dts.contains("heading: {") && dts.contains("h1: string;"),
            "nested member should recurse into a nested object literal type, got:\n{dts}"
        );
        assert!(
            !dts.contains("editor-heading--h1"),
            "nested object literal values must not leak into .d.ts, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_const_deeply_nested_plain_object_literal_signature() {
        // R2 (#1263): real-world shape control mirroring the issue's
        // fe-shared `lexicalTheme` hit -- nesting depth 3+, all leaves plain
        // string literals. Proves the new recursive branch has no
        // hard-coded depth cap.
        let src = r#"export const theme = {
    list: {
        nested: {
            listitem: "editor-list-item",
        },
    },
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const theme: {"),
            "deeply nested object literal const should synthesize a declaration type, got:\n{dts}"
        );
        assert!(
            dts.contains("list: {")
                && dts.contains("nested: {")
                && dts.contains("listitem: string;"),
            "every nesting level should recurse into a nested object literal type, got:\n{dts}"
        );
        assert!(
            !dts.contains("editor-list-item"),
            "deeply nested object literal values must not leak into .d.ts, got:\n{dts}"
        );
    }

    #[test]
    fn uninferrable_exported_const_nested_object_literal_with_untyped_member_errors() {
        // R3 (#1263): negative control -- same shape as R1, but the nested
        // object's own member value is itself uninferrable (a bare
        // identifier with no locally resolvable type). Must stay fail-loud,
        // proving the new recursive branch does not silently widen to
        // accept a nested object literal with a genuinely untyped leaf.
        let src = r#"export const nestedLiteral = {
    ltr: "ltr",
    heading: {
        h1: someUntypedImport,
    },
};
"#;
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "nested object literal with a genuinely untyped leaf must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn infers_exported_const_arrow_function_type() {
        let src = "export const delay = (ms: number): Promise<void> => new Promise<void>((resolve) => setTimeout(resolve, ms));\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const delay: (ms: number) => Promise<void>;"),
            "typed arrow const should synthesize a callable declaration type, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_const_arrow_default_param_type() {
        let src = "export const withDefault = (a: number, b = 6): number => a + b;\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const withDefault: (a: number, b?: number) => number;"),
            "default-valued arrow param should synthesize an optional parameter type, got:\n{dts}"
        );
    }

    #[test]
    fn infers_object_literal_function_property_type() {
        let src = r#"export const _Table = {
    rowNo: (idx: number, page: number, pageSize: number): number =>
        idx + 1 + (page - 1) * pageSize,
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("rowNo: (idx: number, page: number, pageSize: number) => number;"),
            "object literal function property should synthesize a callable property type, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_const_object_assign_computed_key_arrow_property_chained_calls_signature() {
        // R1 (#1238): WI minimal repro -- an object literal with a single
        // arrow-function property carrying its own explicit return type, whose
        // concise body is Object.assign({}, ...chain.of.calls.map(callback))
        // where the callback returns an object with a computed key (the issue's
        // exact _Query.parse shape: chained replace/split/filter/map calls, a
        // block-bodied map callback with array destructuring and a computed-key
        // returned object literal). Must emit the correct .d.ts instead of an
        // isolatedDeclarations error.
        let src = r#"export const _Query = {
    parse: (search: string): Record<string, string> =>
        Object.assign(
            {},
            ...search
                .replace(/^\?/, '')
                .split('&')
                .filter(Boolean)
                .map((pair) => {
                    const [key, value] = pair.split('=');
                    return { [key]: value };
                }),
        ),
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("parse: (search: string) => Record<string, string>;"),
            "object arrow property with Object.assign computed-key body should emit correct type, got:\n{dts}"
        );
    }

    #[test]
    fn infers_object_assign_computed_key_arrow_property_followed_by_sibling_member_signature() {
        // R2 (#1238): non-regression control pinning the entanglement with the
        // still-open WI #1262 (silent property truncation). The same
        // Object.assign+computed-key arrow-property shape as R1, followed by a
        // sibling object-literal member. Must emit BOTH members in the output
        // without silently dropping the sibling property, proving the shared
        // split_top_level bracket-depth fix does not regress to multi-property
        // truncation.
        let src = r#"export const _Query = {
    parse: (search: string): Record<string, string> =>
        Object.assign(
            {},
            ...search
                .split('&')
                .map((pair) => {
                    const [k, v] = pair.split('=');
                    return { [k]: v };
                }),
        ),
    stringify: (obj: Record<string, string>): string =>
        Object.keys(obj).join('&'),
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("parse: (search: string) => Record<string, string>;"),
            "first arrow property should be emitted, got:\n{dts}"
        );
        assert!(
            dts.contains("stringify: (obj: Record<string, string>) => string;"),
            "second arrow property should be emitted (not truncated), got:\n{dts}"
        );
    }

    #[test]
    fn uninferrable_object_assign_computed_key_arrow_property_without_explicit_return_type_errors()
    {
        // R3 (#1238): negative control -- the same Object.assign+computed-key
        // arrow-property shape as R1, but with the arrow's own explicit return
        // type annotation removed. Must still raise an isolatedDeclarations
        // error, proving the inference only fires because the arrow itself
        // carries an explicit return type (infer_arrow_function_type_from_text
        // never inspects the Object.assign(...) body) and does not silently
        // widen to genuinely untyped members.
        let src = r#"export const _Query = {
    parse: (rows: Array<{ key: string }>) =>
        Object.assign({}, ...rows.map((row) => ({ [row.key]: 1 }))),
};
"#;
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "object arrow property without explicit return type must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn infers_object_literal_method_with_object_assign_computed_key_body() {
        let src = r#"export const columns = {
    render(rows: Array<{ key: string }>): Record<string, number> {
        return Object.assign({}, ...rows.map((row) => ({ [row.key]: 1 })));
    },
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("render(rows: Array<{ key: string }>): Record<string, number>;"),
            "object method with computed object body should use explicit boundary types, got:\n{dts}"
        );
    }

    #[test]
    fn infers_object_literal_async_arrow_with_destructured_typed_param() {
        let src = r#"export const handlers = {
    load: async ({ id }: { id: string }): Promise<string> => id,
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("load: ({ id }: { id: string }) => Promise<string>;"),
            "async object arrow with typed destructured param should emit, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_const_arrow_with_destructured_typed_param() {
        let src = r#"export const load = ({ id }: { id: string }): Promise<string> => Promise.resolve(id);
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const load: ({ id }: { id: string }) => Promise<string>;"),
            "const arrow with typed destructured param should emit, got:\n{dts}"
        );
    }

    #[test]
    fn infers_object_literal_async_method_with_plain_typed_params() {
        let src = r#"export const handlers = {
    async save(id: string, count: number): Promise<number> {
        return count;
    },
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("save(id: string, count: number): Promise<number>;"),
            "async object method with typed params should emit, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_generic_const_arrow_function_type() {
        let src = "export const identity = <T>(value: T): T => value;\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const identity: <T>(value: T) => T;"),
            "generic typed arrow const should preserve type parameters, got:\n{dts}"
        );
    }

    #[test]
    fn untyped_exported_const_arrow_param_errors() {
        let src = "export const delay = (ms): Promise<void> => Promise.resolve();\n";
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "untyped arrow params must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn exported_const_arrow_without_return_type_errors() {
        let src = "export const delay = (ms: number) => Promise.resolve();\n";
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "arrow const without return type must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn untyped_const_errors() {
        let src = "export const VERSION = \"1.0.0\";\n";
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "untyped export must error, got: {err}"
        );
    }

    #[test]
    fn preserves_type_imports() {
        let src = "import type { Foo } from \"some-pkg\";\nexport type Bar = Foo;\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("import type { Foo } from \"some-pkg\""),
            "external type import preserved, got:\n{dts}"
        );
        assert!(dts.contains("export type Bar = Foo"));
    }

    #[test]
    fn preserves_reexport_from() {
        let src = "export { Helper } from \"./helper\";\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export { Helper } from \"./helper\""),
            "re-export preserved, got:\n{dts}"
        );
    }

    #[test]
    fn preserves_svgr_asset_reexport_from() {
        let src = "export { ReactComponent as ErrorCircleIcon } from \"./icons/error.svg\";\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("import type { FC, SVGProps } from \"react\";"),
            "SVGR declaration should import React component types, got:\n{dts}"
        );
        assert!(
            dts.contains("export declare const ErrorCircleIcon: FC<SVGProps<SVGSVGElement>>;"),
            "SVGR asset re-export must emit a concrete component declaration, got:\n{dts}"
        );
    }

    #[test]
    fn emits_enum_verbatim() {
        let src = "export enum Color { Red, Green, Blue }\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare enum Color"),
            "enum emitted as declare, got:\n{dts}"
        );
        assert!(dts.contains("Red"), "enum members preserved, got:\n{dts}");
    }

    #[test]
    fn reduces_class_members_to_signatures() {
        let src = r#"export class Button {
    constructor(p: Props) { this.p = p; }
    render(): Node { return null; }
    private x = 1;
    readonly id: string = "";
}
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare class Button"),
            "class reduced to declare class, got:\n{dts}"
        );
        assert!(
            dts.contains("constructor(p: Props);"),
            "constructor signature, got:\n{dts}"
        );
        assert!(
            dts.contains("render(): Node;"),
            "method signature, got:\n{dts}"
        );
        assert!(
            dts.contains("readonly id: string;"),
            "readonly field kept, got:\n{dts}"
        );
        assert!(
            !dts.contains("return null") && !dts.contains("this.p = p"),
            "bodies dropped, got:\n{dts}"
        );
        assert!(
            !dts.contains("private x") && !dts.contains("= 1"),
            "private member dropped, got:\n{dts}"
        );
        assert!(
            !dts.contains("= \"\""),
            "field initializer dropped, got:\n{dts}"
        );
    }

    #[test]
    fn class_keeps_heritage_generics_and_static() {
        let src = r#"export class Store<T> extends Base<T> implements IStore {
    static create(): Store<number> { return new Store(); }
    async load(): Promise<void> {}
    get size(): number { return 0; }
    #hidden(): void {}
    protected note = "x";
}
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare class Store<T> extends Base<T> implements IStore"),
            "header with generics + heritage preserved, got:\n{dts}"
        );
        assert!(
            dts.contains("static create(): Store<number>;"),
            "static modifier kept, got:\n{dts}"
        );
        // `async` dropped (invalid on ambient method) but return type kept.
        assert!(
            dts.contains("load(): Promise<void>;") && !dts.contains("async load"),
            "async stripped, return type kept, got:\n{dts}"
        );
        assert!(
            dts.contains("get size(): number;"),
            "get accessor kept, got:\n{dts}"
        );
        assert!(
            !dts.contains("#hidden"),
            "#private method dropped, got:\n{dts}"
        );
        assert!(
            !dts.contains("note"),
            "protected member dropped, got:\n{dts}"
        );
    }

    #[test]
    fn emits_abstract_class_as_declare_abstract() {
        let src = "export abstract class Shape { abstract area(): number; }\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare abstract class Shape"),
            "abstract class kept, got:\n{dts}"
        );
    }

    #[test]
    fn emits_annotated_default_export_type() {
        let src = "export default (loadConfig() as Config);\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("declare const _default: Config;")
                && dts.contains("export default _default;"),
            "annotated default export resolves to its type, got:\n{dts}"
        );
        assert!(
            !dts.contains("loadConfig"),
            "default expression must not leak, got:\n{dts}"
        );
    }

    #[test]
    fn defers_unannotated_complex_default_export() {
        let src = "export default { a: 1, b: doThing() };\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("TODO(#171 follow-up)"),
            "undeterminable default export left as a TODO, got:\n{dts}"
        );
        // The expression is skipped, not emitted as an active declaration: any
        // mention of it survives only inside the `// TODO` comment line.
        for line in dts.lines() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            assert!(
                !line.contains("doThing()"),
                "undeterminable default body must not be emitted as a real \
                 declaration, got active line:\n{line}\nfull dts:\n{dts}"
            );
        }
    }

    #[test]
    fn drops_non_exported_declarations() {
        let src = "const internal = 1;\nexport const VALUE: number = internal;\n";
        let dts = emit_declarations(src).unwrap();
        assert!(
            !dts.contains("internal"),
            "non-exported binding must not leak into .d.ts, got:\n{dts}"
        );
        assert!(dts.contains("export declare const VALUE: number;"));
    }

    #[test]
    fn infers_exported_const_plain_object_literal_as_expression_signature() {
        // R1 (#937): minimal repro -- a plain object-literal `expr as Type`
        // const initializer must emit the asserted type with the initializer
        // dropped, not an isolatedDeclarations error.
        let src = r#"interface Foo {
    a: number;
}
export const asCastConst = { a: 1 } as Foo;
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const asCastConst: Foo;"),
            "plain object-literal as-expression should use its asserted type, got:\n{dts}"
        );
        assert!(
            !dts.contains("{ a: 1 }"),
            "const initializer must be dropped, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_function_return_via_local_variable_as_expression() {
        // R2 (#937): minimal repro -- a function that assigns to a local
        // variable and returns it cast via `x as Type` must resolve through
        // the cast, not the (unresolvable) local variable.
        let src = r#"interface Foo {
    a: number;
}
export function asCastReturn() {
    const x: unknown = { a: 1 };
    return x as Foo;
}
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare function asCastReturn(): Foo;"),
            "local-variable-then-cast return should use its asserted type, got:\n{dts}"
        );
    }

    #[test]
    fn infers_exported_const_identifier_as_expression_signature() {
        // R3 (#937): WI #937 cited real-code `SpAlert` shape -- a bare
        // identifier cast via `as Type`, confirming the fix is not limited
        // to object-literal initializers.
        let src = r#"interface AlertInterface {
    type: string;
}
export const SpAlert = Alert as AlertInterface;
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const SpAlert: AlertInterface;"),
            "identifier as-expression should use its asserted type, got:\n{dts}"
        );
    }

    #[test]
    fn uninferrable_exported_function_return_of_local_variable_without_as_expression_errors() {
        // R4 (#937): negative control -- same local-variable-return shape as
        // R2 but with the `as Foo` cast removed. Must still raise an
        // isolatedDeclarations error, proving `annotated_expression_type`
        // stays scoped to explicit `as`/`satisfies` casts and does not
        // broaden return-expression inference to uncast local variables.
        let src = r#"export function notCast() {
    const x: unknown = { a: 1 };
    return x;
}
"#;
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "uncast local-variable return must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn infers_exported_const_arrow_body_single_return_typed_object_literal_signature() {
        // R1 (#1264): WI minimal repro -- arrow const with no explicit
        // return type, single-statement block body that returns an object
        // literal whose own members all carry explicit types. tsc accepts
        // this and infers the return type from the returned literal.
        let src = r#"export const funcReturningTypedObject = (a: string, b: number = 1) => {
    return {
        fromOutsource: (x: string): Promise<string> => Promise.resolve(x),
        toOutsource: (y?: string): string => y ?? a,
    };
};
"#;
        let dts = emit_declarations(src).unwrap();
        assert!(
            dts.contains("export declare const funcReturningTypedObject: (a: string, b?: number) => {")
                && dts.contains("fromOutsource: (x: string) => Promise<string>;")
                && dts.contains("toOutsource: (y?: string) => string;"),
            "arrow body single-return typed object literal should synthesize a return-object signature, got:\n{dts}"
        );
    }

    #[test]
    fn uninferrable_exported_const_arrow_body_return_partially_typed_object_literal_errors() {
        // R2 (#1264): negative control -- same shape as R1 but one member's
        // arrow value has no explicit return type. Must stay fail-loud,
        // proving the new path only fires when every returned-object member
        // is itself locally inferable.
        let src = r#"export const funcReturningTypedObject = (a: string, b: number = 1) => {
    return {
        fromOutsource: (x: string): Promise<string> => Promise.resolve(x),
        toOutsource: (y?: string) => y ?? a,
    };
};
"#;
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "arrow body return object with a partially-typed member must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn uninferrable_exported_const_arrow_multi_statement_body_return_object_literal_errors() {
        // R3 (#1264): negative control -- multi-statement block body before
        // the `return { ... }`. Must stay fail-loud, proving the new
        // inference is scoped to exactly one return statement.
        let src = r#"export const funcReturningTypedObject = (a: string, b: number = 1) => {
    const prefix = a;
    return {
        fromOutsource: (x: string): Promise<string> => Promise.resolve(x),
        toOutsource: (y?: string): string => y ?? prefix,
    };
};
"#;
        let err = emit_declarations(src).unwrap_err();
        assert!(
            err.to_string().contains("isolatedDeclarations"),
            "multi-statement arrow body returning a typed object literal must stay fail-loud, got: {err}"
        );
    }

    #[test]
    fn infers_object_assign_computed_key_arrow_property_at_real_world_scale_signature() {
        // R1 (#1262): WI's real-world-scale repro -- an 11-method
        // reconstruction of the issue's own "Real-world impact" description
        // (fe-shared's `_Query`), with a single Object.assign({},
        // ...arr.map(cb))-valued arrow property (`parse`) followed by TEN
        // sibling properties in the same object literal. On jet 0.4.16
        // (pre-WI #1264's split_top_level bracket-depth fix), only `parse`
        // survived and all ten siblings were silently dropped with no error.
        // Must emit all eleven members, matching tsc's ground-truth output,
        // proving the truncation does not resurface at real-world scale.
        let src = r#"export const _Query = {
    parse: (search: string): Record<string, string> =>
        Object.assign(
            {},
            ...search
                .replace(/^\?/, '')
                .split('&')
                .filter(Boolean)
                .map((pair) => {
                    const [key, value] = pair.split('=');
                    return { [key]: value };
                }),
        ),
    formatToQueryObject: (obj: Record<string, string>): string =>
        Object.entries(obj)
            .map(([k, v]) => `${k}=${v}`)
            .join('&'),
    getOperatorsInOrder: (obj: Record<string, string>): string[] =>
        Object.keys(obj),
    transformCase: (str: string): string => str.trim(),
    camelCase: (str: string): string => str.replace(/-./g, (m) => m[1].toUpperCase()),
    snakeCase: (str: string): string => str.replace(/([A-Z])/g, '_$1').toLowerCase(),
    kebabCase: (str: string): string => str.replace(/([A-Z])/g, '-$1').toLowerCase(),
    formatToQueryString: (obj: Record<string, string>): string =>
        Object.entries(obj)
            .map(([k, v]) => `${k}=${v}`)
            .join('&'),
    int: (str: string): number => parseInt(str, 10),
    genOrderList: (list: string[]): string[] => [...list],
    genOrderStrList: (list: string[]): string => list.join(','),
};
"#;
        let dts = emit_declarations(src).unwrap();
        let expected_members = [
            "parse: (search: string) => Record<string, string>;",
            "formatToQueryObject: (obj: Record<string, string>) => string;",
            "getOperatorsInOrder: (obj: Record<string, string>) => string[];",
            "transformCase: (str: string) => string;",
            "camelCase: (str: string) => string;",
            "snakeCase: (str: string) => string;",
            "kebabCase: (str: string) => string;",
            "formatToQueryString: (obj: Record<string, string>) => string;",
            "int: (str: string) => number;",
            "genOrderList: (list: string[]) => string[];",
            "genOrderStrList: (list: string[]) => string;",
        ];
        for member in expected_members {
            assert!(
                dts.contains(member),
                "expected member `{member}` missing from real-world-scale Object.assign object literal, got:\n{dts}"
            );
        }
        // Truncation-detection assertion: count emitted top-level member
        // signature lines directly rather than relying only on substring
        // presence, so a partial-truncation regression that happens to
        // preserve unrelated member text still fails this test.
        let member_line_count = dts
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.ends_with(';') && !trimmed.starts_with("export") && trimmed != "};"
            })
            .count();
        assert_eq!(
            member_line_count, 11,
            "expected exactly 11 emitted members (silent truncation drops the tail), got {member_line_count}:\n{dts}"
        );
    }

    #[test]
    fn infers_object_assign_computed_key_arrow_property_multiple_members_same_literal_signature() {
        // R2 (#1262): non-regression stress control -- TWO separate
        // Object.assign({}, ...arr.map(cb))-valued arrow properties in the
        // same object literal (`parse` and `second`), each followed by
        // further sibling properties, none of which was covered by the
        // #1238 TD's single-Object.assign probes. Must emit all twelve
        // members, proving the split_top_level fix generalizes to more than
        // one Object.assign-valued member per literal.
        let src = r#"export const _Query = {
    simpleMethod: (x: number): number => x + 1,
    parse: (search: string): Record<string, string> =>
        Object.assign(
            {},
            ...search
                .split('&')
                .filter(Boolean)
                .map((pair) => {
                    const [key, value] = pair.split('=');
                    return { [key]: value };
                }),
        ),
    formatToQueryObject: (obj: Record<string, string>): string =>
        Object.entries(obj)
            .map(([k, v]) => `${k}=${v}`)
            .join('&'),
    int: (str: string): number => parseInt(str, 10),
    second: (list: string[]): string[] =>
        Object.assign(
            [],
            ...list.map((item) => {
                return [item.trim()];
            }),
        ),
    genOrderList: (list: string[]): string[] => [...list],
};
"#;
        let dts = emit_declarations(src).unwrap();
        let expected_members = [
            "simpleMethod: (x: number) => number;",
            "parse: (search: string) => Record<string, string>;",
            "formatToQueryObject: (obj: Record<string, string>) => string;",
            "int: (str: string) => number;",
            "second: (list: string[]) => string[];",
            "genOrderList: (list: string[]) => string[];",
        ];
        for member in expected_members {
            assert!(
                dts.contains(member),
                "expected member `{member}` missing from dual-Object.assign object literal, got:\n{dts}"
            );
        }
    }
}
// </HANDWRITE>
