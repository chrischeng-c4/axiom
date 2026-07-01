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
                if let Some(ty) = default_export_annotated_type(child, source) {
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
fn default_export_annotated_type(node: Node, source: &str) -> Option<String> {
    match node.kind() {
        // `(inner)` — the inner expression has no field name on this grammar,
        // so unwrap the first named child and recurse.
        "parenthesized_expression" => {
            let inner = first_named_child(node)?;
            default_export_annotated_type(inner, source)
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
    infer_object_literal_type(value, source)
}

fn infer_object_literal_type(node: Node, source: &str) -> Option<String> {
    if node.kind() != "object" {
        return None;
    }
    let text = node_text(node, source).trim();
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
        let (key, value) = split_once_top_level(property, ':')?;
        let key = key.trim();
        if !is_supported_object_literal_key(key) {
            return None;
        }
        let ty = infer_expression_type(value.trim(), &empty_param_types)?;
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
            _ if ch == delimiter && depth == 0 => {
                parts.push(text[start..idx].to_string());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(text[start..].to_string());
    parts
}

fn split_once_top_level<'a>(text: &'a str, delimiter: char) -> Option<(&'a str, &'a str)> {
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
            _ if ch == delimiter && depth == 0 => {
                return Some((&text[..idx], &text[idx + ch.len_utf8()..]));
            }
            _ => {}
        }
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
}
// </HANDWRITE>
