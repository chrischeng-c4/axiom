// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
//! AST-based TypeScript type stripping helpers.
//!
//! Functions in this module identify and remove TypeScript-only syntax from
//! the AST produced by tree-sitter-typescript.  They are called from the main
//! `transform_tsx` walker in `transform_tsx.rs`.

use anyhow::Result;
use tree_sitter::Node;

use super::transform_tsx::transform_node;
use super::TransformOptions;

/// Check if an export_statement exports only type-level constructs.
///
/// Matches patterns:
/// - `export type { Foo } from './foo'`
/// - `export type { Foo }`
/// - `export interface Foo { ... }`
/// - `export type Foo = ...`
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn is_type_only_export(source: &str, node: &Node) -> bool {
    let text = &source[node.byte_range()];

    // `export type {` pattern
    if text.starts_with("export type {") || text.starts_with("export type{") {
        return true;
    }

    // Check if the export has a "type" keyword right after "export"
    let mut cursor = node.walk();
    let mut saw_export = false;
    for child in node.children(&mut cursor) {
        if child.kind() == "export" || source[child.byte_range()].trim() == "export" {
            saw_export = true;
            continue;
        }
        if saw_export {
            let child_text = source[child.byte_range()].trim();
            if child_text == "type" {
                return true;
            }
            break;
        }
    }

    // `export interface ...`
    let mut cursor2 = node.walk();
    for child in node.children(&mut cursor2) {
        if child.kind() == "interface_declaration" {
            return true;
        }
    }

    // `export type Foo = ...` (type alias export)
    let mut cursor3 = node.walk();
    for child in node.children(&mut cursor3) {
        if child.kind() == "type_alias_declaration" {
            return true;
        }
    }

    false
}

/// Check if an import_statement is type-only: `import type { ... } from '...'`
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn is_type_only_import(_source: &str, node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "import" {
            continue;
        }
        // The next non-whitespace token after "import" should be "type"
        if child.kind() == "type" {
            return true;
        }
        break;
    }
    false
}

/// Check if an import_statement has inline `type` specifiers in the named imports.
/// E.g. `import { type Foo, Bar } from '...'`
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn has_inline_type_specifiers(node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "import_clause" {
            let mut cc = child.walk();
            for clause_child in child.children(&mut cc) {
                if clause_child.kind() == "named_imports" {
                    let mut nc = clause_child.walk();
                    for import_child in clause_child.children(&mut nc) {
                        if import_child.kind() == "import_specifier" {
                            // Check if the first child of import_specifier is "type"
                            let mut sc = import_child.walk();
                            for spec_child in import_child.children(&mut sc) {
                                if spec_child.kind() == "type" {
                                    return true;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Transform an import with inline type specifiers.
/// Removes `type`-prefixed specifiers and keeps value specifiers.
/// Returns `None` if all specifiers were type-only (remove entire import).
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_import_with_inline_types(source: &str, node: &Node) -> Result<Option<String>> {
    let text = &source[node.byte_range()];

    // Find the named imports section: { ... }
    let brace_start = match text.find('{') {
        Some(pos) => pos,
        None => return Ok(Some(text.to_string())),
    };
    let brace_end = match text.rfind('}') {
        Some(pos) => pos,
        None => return Ok(Some(text.to_string())),
    };

    let before_braces = &text[..brace_start + 1];
    let after_braces = &text[brace_end..];
    let inside = &text[brace_start + 1..brace_end];

    // Parse specifiers
    let specifiers: Vec<&str> = inside.split(',').collect();
    let mut kept: Vec<String> = Vec::new();

    for spec in &specifiers {
        let trimmed = spec.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip `type Foo` or `type Foo as Bar`
        if trimmed.starts_with("type ") || trimmed.starts_with("type\t") {
            continue;
        }
        kept.push(trimmed.to_string());
    }

    if kept.is_empty() {
        // All specifiers were type-only → remove entire import
        return Ok(None);
    }

    let result = format!(
        "{} {} {}",
        before_braces.trim_end(),
        kept.join(", "),
        after_braces.trim_start()
    );
    // Clean up: `{ ` and ` }`
    let result = result.replace("{  ", "{ ").replace("  }", " }");

    Ok(Some(result))
}

/// Transform satisfies_expression: keep LHS, drop `satisfies Type`
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_satisfies_expression(
    source: &str,
    node: &Node,
    options: &TransformOptions,
) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Skip the "satisfies" keyword and the type that follows
        if child.kind() == "satisfies" {
            continue;
        }
        // Skip type nodes (the type after satisfies)
        if matches!(
            child.kind(),
            "type_identifier"
                | "generic_type"
                | "parenthesized_type"
                | "function_type"
                | "predefined_type"
                | "object_type"
                | "union_type"
                | "intersection_type"
                | "array_type"
                | "tuple_type"
        ) {
            continue;
        }
        // This is the expression to keep — recurse
        return transform_node(source, &child, options);
    }
    // Fallback
    Ok(source[node.byte_range()].to_string())
}
// CODEGEN-END
