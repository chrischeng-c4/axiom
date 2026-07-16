// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
//! AST-based TypeScript type stripping helpers.
//!
//! Functions in this module identify and remove TypeScript-only syntax from
//! the AST produced by tree-sitter-typescript.  They are called from the main
//! `transform_tsx` walker in `transform_tsx.rs`.

use anyhow::Result;
use regex::Regex;
use std::{collections::HashSet, sync::OnceLock};
use tree_sitter::{Node, Parser};

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
    let runtime_identifiers = collect_runtime_identifier_names(&usage);

    let mut result = String::with_capacity(code.len());
    let mut cursor = 0usize;
    for (start, end) in import_ranges {
        result.push_str(&code[cursor..start]);
        let statement = &code[start..end];
        if let Some(rewritten) = rewrite_named_import(statement, &usage, &runtime_identifiers) {
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

fn rewrite_named_import(
    statement: &str,
    usage: &str,
    runtime_identifiers: &Option<HashSet<String>>,
) -> Option<String> {
    static NAMED_IMPORT_RE: OnceLock<Regex> = OnceLock::new();
    let re = NAMED_IMPORT_RE.get_or_init(|| {
        Regex::new(
            r#"(?s)^(\s*import\s+)(?:(?P<default>[A-Za-z_$][\w$]*)\s*,\s*)?\{(?P<named>.*?)\}\s+from\s+(?P<module>['"][^'"]+['"])[\t ]*;?[\t ]*$"#,
        )
        .expect("valid named import regex")
    });
    // Preserve the physical line ending outside the regex. A trailing `\s*`
    // in the old expression greedily consumed it before the optional newline
    // capture, joining two rewritten import declarations into one line.
    let (statement, newline) = if let Some(without_newline) = statement.strip_suffix("\r\n") {
        (without_newline, "\r\n")
    } else if let Some(without_newline) = statement.strip_suffix('\n') {
        (without_newline, "\n")
    } else {
        (statement, "")
    };
    let Some(captures) = re.captures(statement) else {
        return Some(format!("{statement}{newline}"));
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
            if identifier_is_used(usage, local, runtime_identifiers)
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

#[cfg(test)]
mod tests {
    use super::{collect_runtime_identifier_names, identifier_is_used, strip_unused_named_imports};

    #[test]
    fn rewritten_named_imports_preserve_physical_line_endings() {
        let source = "import { TypeOnly } from \"./types\";\nimport { value } from \"./value\";\nconsole.log(value);\n";
        let rewritten = strip_unused_named_imports(source);

        assert!(
            rewritten.contains("import { value } from \"./value\";\nconsole.log(value);"),
            "rewriting one import must not concatenate the next statement: {rewritten}"
        );
        assert!(
            !rewritten.contains("import { value } from \"./value\";console.log"),
            "the next statement must retain its original newline: {rewritten}"
        );
    }

    #[test]
    fn rewritten_named_imports_preserve_windows_line_endings() {
        let source = "import { TypeOnly } from \"./types\";\r\nimport { value } from \"./value\";\r\nconsole.log(value);\r\n";
        let rewritten = strip_unused_named_imports(source);

        assert!(
            rewritten.contains("import { value } from \"./value\";\r\nconsole.log(value);"),
            "rewriting must retain CRLF boundaries: {rewritten}"
        );
    }

    #[test]
    fn identifier_usage_ignores_strings_but_keeps_runtime_references() {
        let quoted_source = r#"test("Dayjs is only a type", () => {});"#;
        let quoted_identifiers = collect_runtime_identifier_names(quoted_source);
        assert!(
            !identifier_is_used(quoted_source, "Dayjs", &quoted_identifiers),
            "quoted text must not retain a type-only import"
        );
        let runtime_source = "const value = Dayjs.now();";
        let runtime_identifiers = collect_runtime_identifier_names(runtime_source);
        assert!(
            identifier_is_used(runtime_source, "Dayjs", &runtime_identifiers),
            "a runtime identifier must retain its import"
        );
    }
}

fn import_specifier_local_name(spec: &str) -> Option<&str> {
    if let Some((_, local)) = spec.rsplit_once(" as ") {
        return Some(local.trim());
    }
    spec.split_whitespace().next().map(str::trim)
}

fn collect_runtime_identifier_names(source: &str) -> Option<HashSet<String>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_javascript::LANGUAGE.into())
        .ok()?;
    let tree = parser.parse(source, None)?;
    let mut identifiers = HashSet::new();
    collect_runtime_identifier_names_from_node(tree.root_node(), source, &mut identifiers);
    Some(identifiers)
}

fn collect_runtime_identifier_names_from_node(
    node: Node<'_>,
    source: &str,
    identifiers: &mut HashSet<String>,
) {
    if matches!(node.kind(), "identifier" | "shorthand_property_identifier") {
        identifiers.insert(source[node.byte_range()].to_string());
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_runtime_identifier_names_from_node(child, source, identifiers);
    }
}

fn identifier_is_used(
    haystack: &str,
    ident: &str,
    runtime_identifiers: &Option<HashSet<String>>,
) -> bool {
    if ident.is_empty() {
        return false;
    }

    if let Some(runtime_identifiers) = runtime_identifiers {
        return runtime_identifiers.contains(ident);
    }

    identifier_is_used_textually(haystack, ident)
}

fn identifier_is_used_textually(haystack: &str, ident: &str) -> bool {
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

/// Check whether a named export contains a per-specifier `type` modifier.
///
/// For example, this returns true for `export { Value, type ValueShape }`.
/// `export type { ValueShape }` is a separate all-type export form and is
/// handled by [`is_type_only_export`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn has_inline_type_export_specifiers(node: &Node) -> bool {
    let Some(export_clause) = named_export_clause(node) else {
        return false;
    };

    let mut cursor = export_clause.walk();
    for specifier in export_clause.children(&mut cursor) {
        if specifier.kind() == "export_specifier" && export_specifier_is_type_only(&specifier) {
            return true;
        }
    }
    false
}

/// Transform a named export with inline `type` specifiers.
///
/// TypeScript permits a type-only name beside runtime values in the same named
/// export list. Node's JavaScript parser does not understand that modifier, so
/// remove only those type-only specifiers and keep the runtime re-exports.
/// Returns `None` when all named specifiers were type-only.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_export_with_inline_types(source: &str, node: &Node) -> Result<Option<String>> {
    let Some(export_clause) = named_export_clause(node) else {
        return Ok(Some(source[node.byte_range()].to_string()));
    };

    let mut cursor = export_clause.walk();
    let specifiers: Vec<Node<'_>> = export_clause
        .children(&mut cursor)
        .filter(|child| child.kind() == "export_specifier")
        .collect();

    if specifiers.is_empty() {
        return Ok(Some(source[node.byte_range()].to_string()));
    }

    let kept: Vec<&str> = specifiers
        .iter()
        .filter(|specifier| !export_specifier_is_type_only(specifier))
        .map(|specifier| source[specifier.byte_range()].trim())
        .collect();

    if kept.len() == specifiers.len() {
        return Ok(Some(source[node.byte_range()].to_string()));
    }
    if kept.is_empty() {
        return Ok(None);
    }

    let before_clause = &source[node.start_byte()..export_clause.start_byte()];
    let after_clause = &source[export_clause.end_byte()..node.end_byte()];
    Ok(Some(format!(
        "{before_clause}{{ {} }}{after_clause}",
        kept.join(", ")
    )))
}

fn named_export_clause<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "export_clause" {
            return Some(child);
        }
    }
    None
}

fn export_specifier_is_type_only(node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "type" {
            return true;
        }
    }
    false
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
