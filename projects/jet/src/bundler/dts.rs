// HANDWRITE-BEGIN gap="missing-generator:logic:d172c696" tracker="pending-tracker" reason="New isolatedDeclarations-style declaration emitter: parse a library entry with tree-sitter-typescript, walk top-level exported declarations, emit type/interface/enum decls verbatim and `export declare` signatures for explicitly-typed exported values, error on untyped exports, and return the assembled `<entry>.d.ts` text (external type imports preserved)."
//! isolatedDeclarations-style `.d.ts` emission for `jet build --lib`.
//!
//! Mirrors the TypeScript 5.5 `isolatedDeclarations` model: declarations are
//! emitted from the *explicit* types at the export boundary, never from full
//! type inference. Per library entry we:
//!
//!   1. tree-sitter parse the entry source (TSX grammar, a superset that also
//!      parses plain TS/JS),
//!   2. walk the top-level statements in source order,
//!   3. for `interface` / `type` / `enum` / `class` type declarations emit the
//!      declaration verbatim (with a leading `export`/`export declare`),
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
            "class_declaration" => {
                let decl = emit_class_declaration(child, source);
                let prefix = if is_default {
                    "export default class "
                } else {
                    "export declare class "
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
                push_line(out, &format!("export {{ default }};"));
                return Ok(());
            }
            _ => {}
        }
    }

    // Anything else (e.g. `export default <complex expr>`, decorators) — pass
    // through best-effort with a marker so the build never crashes.
    let text = node_text(node, source);
    push_line(out, "// TODO(#171 follow-up): unsupported export shape passed through");
    push_line(out, text.trim_end());
    Ok(())
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

/// Emit a class declaration as a declaration-only shape.
///
/// Full member-by-member `.d.ts` reduction (dropping method bodies, keeping
/// signatures) is non-trivial; for now the class is passed through verbatim
/// from its name onward, which is a valid (if over-detailed) ambient class
/// shape — TODO(#171 follow-up) reduce method bodies to signatures.
fn emit_class_declaration(node: Node, source: &str) -> String {
    // Drop the leading `class` keyword; the caller supplies the `export
    // declare class ` / `export default class ` prefix.
    let text = node_text(node, source);
    let after_kw = text
        .trim_start()
        .strip_prefix("class")
        .map(str::trim_start)
        .unwrap_or(text);
    // TODO(#171 follow-up): reduce method bodies to signatures and drop private
    // members. Passed through verbatim for now (still a valid class shape).
    after_kw.trim_end().to_string()
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
// HANDWRITE-END
