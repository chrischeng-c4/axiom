// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
//! AST-based TypeScript type stripping helpers.
//!
//! Functions in this module identify and remove TypeScript-only syntax from
//! the AST produced by tree-sitter-typescript.  They are called from the main
//! `transform_tsx` walker in `transform_tsx.rs`.

use anyhow::Result;
use regex::Regex;
use std::sync::OnceLock;
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
/// - `export namespace Foo { ... }`
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
        if child.kind() == "type_alias_declaration" || child.kind() == "internal_module" {
            return true;
        }
    }

    // TypeScript overload signatures are type-only declarations. The
    // implementation appears as a later function_declaration.
    let mut cursor4 = node.walk();
    for child in node.children(&mut cursor4) {
        if child.kind() == "function_signature" {
            return true;
        }
    }

    false
}

/// Drop named import specifiers that are no longer referenced after type
/// stripping. Many ecosystem packages still write value-form imports for
/// type-only names; TypeScript erases them, so Jet must do the same before the
/// browser validates ESM export names.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn strip_unused_named_imports(code: &str) -> String {
    let import_ranges = collect_named_import_ranges(code);
    if import_ranges.is_empty() {
        return code.to_string();
    }

    let mut usage = String::with_capacity(code.len());
    let mut cursor = 0usize;
    for (start, end) in &import_ranges {
        usage.push_str(&code[cursor..*start]);
        cursor = *end;
    }
    usage.push_str(&code[cursor..]);

    let mut result = String::with_capacity(code.len());
    let mut cursor = 0usize;
    for (start, end) in import_ranges {
        result.push_str(&code[cursor..start]);
        let statement = &code[start..end];
        if let Some(rewritten) = rewrite_named_import(statement, &usage) {
            result.push_str(&rewritten);
        }
        cursor = end;
    }
    result.push_str(&code[cursor..]);
    result
}

fn collect_named_import_ranges(code: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut offset = 0usize;
    let lines: Vec<&str> = code.split_inclusive('\n').collect();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if !trimmed.starts_with("import ") || !line.contains('{') {
            offset += line.len();
            i += 1;
            continue;
        }

        let start = offset;
        let mut end = offset + line.len();
        let mut statement = line.to_string();
        let mut j = i + 1;
        while !statement.contains(" from ") && j < lines.len() {
            statement.push_str(lines[j]);
            end += lines[j].len();
            j += 1;
        }

        if statement.contains(" from ") && statement.contains('}') {
            ranges.push((start, end));
        }

        offset = end;
        i = j;
    }

    ranges
}

fn rewrite_named_import(statement: &str, usage: &str) -> Option<String> {
    static NAMED_IMPORT_RE: OnceLock<Regex> = OnceLock::new();
    let re = NAMED_IMPORT_RE.get_or_init(|| {
        Regex::new(
            r#"(?s)^(\s*import\s+)(?:(?P<default>[A-Za-z_$][\w$]*)\s*,\s*)?\{(?P<named>.*?)\}\s+from\s+(?P<module>['"][^'"]+['"])\s*;?\s*(?P<newline>\r?\n?)$"#,
        )
        .expect("valid named import regex")
    });
    let Some(captures) = re.captures(statement) else {
        return Some(statement.to_string());
    };

    let default_import = captures.name("default").map(|m| m.as_str());
    let named = captures
        .name("named")
        .map(|m| m.as_str())
        .unwrap_or_default();
    let module = captures
        .name("module")
        .map(|m| m.as_str())
        .unwrap_or_default();
    let newline = captures
        .name("newline")
        .map(|m| m.as_str())
        .unwrap_or_default();

    let kept: Vec<String> = named
        .split(',')
        .filter_map(|raw| {
            let spec = raw.trim();
            if spec.is_empty() {
                return None;
            }
            let spec = spec
                .strip_prefix("type ")
                .or_else(|| spec.strip_prefix("type\t"))
                .unwrap_or(spec)
                .trim();
            if spec.is_empty() {
                return None;
            }
            let local = import_specifier_local_name(spec)?;
            if identifier_is_used(usage, local)
                || (default_import.is_none() && !looks_like_type_import_name(local))
            {
                Some(spec.to_string())
            } else {
                None
            }
        })
        .collect();

    match (default_import, kept.is_empty()) {
        (Some(default_name), true) => {
            Some(format!("import {default_name} from {module};{newline}"))
        }
        (None, true) => None,
        (Some(default_name), false) => Some(format!(
            "import {default_name}, {{ {} }} from {module};{newline}",
            kept.join(", ")
        )),
        (None, false) => Some(format!(
            "import {{ {} }} from {module};{newline}",
            kept.join(", ")
        )),
    }
}

fn import_specifier_local_name(spec: &str) -> Option<&str> {
    if let Some((_, local)) = spec.rsplit_once(" as ") {
        return Some(local.trim());
    }
    spec.split_whitespace().next().map(str::trim)
}

fn identifier_is_used(haystack: &str, ident: &str) -> bool {
    if ident.is_empty() {
        return false;
    }
    let mut start = 0usize;
    while let Some(pos) = haystack[start..].find(ident) {
        let absolute = start + pos;
        let before = haystack[..absolute].chars().next_back();
        let after = haystack[absolute + ident.len()..].chars().next();
        if !is_identifier_part(before) && !is_identifier_part(after) {
            return true;
        }
        start = absolute + ident.len();
    }
    false
}

fn looks_like_type_import_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch == '_' || ch.is_ascii_uppercase())
}

fn is_identifier_part(ch: Option<char>) -> bool {
    ch.is_some_and(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
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
