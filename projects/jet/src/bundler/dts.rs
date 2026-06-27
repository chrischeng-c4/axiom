// <HANDWRITE gap="missing-generator:logic:d172c696" tracker="standardize-gap-projects-jet-src-bundler-dts-rs" reason="New isolatedDeclarations-style declaration emitter: parse a library entry with tree-sitter-typescript, walk top-level exported declarations, emit type/interface/enum decls verbatim and `export declare` signatures for explicitly-typed exported values, error on untyped exports, and return the assembled `<entry>.d.ts` text (external type imports preserved).">
//! isolatedDeclarations-style `.d.ts` emission for `jet build --lib`.
//!
//! Mirrors the TypeScript 5.5 `isolatedDeclarations` model: declarations are
//! emitted from the *explicit* types at the export boundary, never from full
//! type inference. Per library entry we:
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
//!      explicit type annotation (isolatedDeclarations: error otherwise),
//!   5. preserve `import`/`export … from "pkg"` re-exports so external type
//!      references still resolve,
//!   6. assemble and return the entry's `.d.ts` text.
//!
//! Shapes that cannot be handled cleanly are passed through best-effort with a
//! `// TODO(#171 follow-up)` marker rather than crashing the build.
//!
//! @issue #171

use anyhow::{anyhow, Result};
use tree_sitter::Node;

/// Emit the `.d.ts` text for one library entry's source.
///
/// `entry_source` is the raw TypeScript/TSX source of the entry module. The
/// returned string is the full `.d.ts` content (imports preserved, exported
/// declarations reduced to type-only signatures).
///
/// Errors (isolatedDeclarations contract): an exported `const`/`let`/`var` or
/// `function` that lacks an explicit type annotation cannot have its type
/// emitted without inference, so this returns `Err`.
pub fn emit_declarations(entry_source: &str) -> Result<String> {
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
                emit_export_statement(child, entry_source, &mut out)?;
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

    Ok(out)
}

/// Emit one top-level `export_statement` into `out`.
fn emit_export_statement(node: Node, source: &str, out: &mut String) -> Result<()> {
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
                let decl = emit_class_declaration(child, source);
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
                let sig = emit_function_signature(child, source)?;
                let prefix = if is_default {
                    "export default function "
                } else {
                    "export declare function "
                };
                push_line(out, &format!("{prefix}{sig};"));
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
                emit_value_declaration(child, source, out)?;
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
fn emit_value_declaration(node: Node, source: &str, out: &mut String) -> Result<()> {
    // `const` / `let` / `var` keyword text precedes the declarators.
    let kind_kw = leading_value_keyword(node, source).unwrap_or("const");

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() != "variable_declarator" {
            continue;
        }
        let name = child
            .child_by_field_name("name")
            .map(|n| node_text(n, source))
            .ok_or_else(|| anyhow!("dts: export {kind_kw} without a name"))?;
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
                return Err(anyhow!(
                    "dts: isolatedDeclarations error — exported `{kind_kw} {name}` \
                     lacks an explicit type annotation; add `: <Type>` so its \
                     declaration can be emitted without type inference"
                ));
            }
        }
    }
    Ok(())
}

/// Build a function signature string (name + type params + params + return
/// type) with the body dropped.
///
/// isolatedDeclarations: an exported function should declare its return type
/// explicitly. We do not hard-error on a missing return type (a `void`-bodied
/// helper is common); instead the signature is emitted as written and the
/// return annotation, if present, is preserved verbatim.
fn emit_function_signature(node: Node, source: &str) -> Result<String> {
    let name = node
        .child_by_field_name("name")
        .map(|n| node_text(n, source))
        .ok_or_else(|| anyhow!("dts: exported function without a name"))?;

    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| node_text(n, source))
        .unwrap_or("");
    let params = node
        .child_by_field_name("parameters")
        .map(|n| node_text(n, source))
        .unwrap_or("()");
    let ret = node
        .child_by_field_name("return_type")
        .map(|n| node_text(n, source))
        .unwrap_or("");

    Ok(format!("{name}{type_params}{params}{ret}"))
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
fn emit_class_declaration(node: Node, source: &str) -> String {
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
        return header;
    };

    let mut members = String::new();
    let mut cursor = body.walk();
    for member in body.named_children(&mut cursor) {
        if let Some(line) = reduce_class_member(member, source) {
            members.push_str("    ");
            members.push_str(&line);
            members.push('\n');
        }
    }

    if members.is_empty() {
        format!("{header} {{\n}}")
    } else {
        format!("{header} {{\n{members}}}")
    }
}

/// Reduce one class-body member to its ambient signature line (without the
/// trailing newline / leading indentation), or `None` when the member is
/// dropped (`private` / `protected` / `#private`, or an unreducible shape).
fn reduce_class_member(node: Node, source: &str) -> Option<String> {
    match node.kind() {
        "method_definition" => reduce_method(node, source),
        "public_field_definition" => reduce_field(node, source),
        // index signatures (`[key: string]: T;`) are already declaration-only.
        "index_signature" => Some(format!(
            "{};",
            node_text(node, source).trim_end_matches(';')
        )),
        // Static initialization blocks, decorators-only members, etc. carry no
        // public type surface — drop them.
        _ => None,
    }
}

/// Reduce a `method_definition` to a signature line. Drops the body and
/// `async`; keeps `static` / `get` / `set` / `readonly` modifiers.
fn reduce_method(node: Node, source: &str) -> Option<String> {
    // `#private` methods are never part of the public surface.
    let name_node = node.child_by_field_name("name")?;
    if name_node.kind() == "private_property_identifier" {
        return None;
    }
    // `private` / `protected` members are dropped from the ambient surface.
    if has_dropped_accessibility(node, source) {
        return None;
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
    let ret = node
        .child_by_field_name("return_type")
        .map(|n| node_text(n, source))
        .unwrap_or("");

    Some(format!("{modifiers}{name}{optional}{params}{ret};"))
}

/// Reduce a `public_field_definition` to a `field: Type;` line, dropping the
/// initializer. Keeps `static` / `readonly`. Drops `private` / `protected` /
/// `#private` fields.
fn reduce_field(node: Node, source: &str) -> Option<String> {
    let name_node = node.child_by_field_name("name")?;
    if name_node.kind() == "private_property_identifier" {
        return None;
    }
    if has_dropped_accessibility(node, source) {
        return None;
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

    Some(format!("{modifiers}{name}{marker}{ty};"))
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
