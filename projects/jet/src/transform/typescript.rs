// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use tree_sitter::{Node, Parser};

use super::type_strip::strip_unused_named_imports;
use super::{TransformOptions, TransformResult};

/// Transform TypeScript to JavaScript by removing type annotations
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_typescript(source: &str, options: &TransformOptions) -> Result<TransformResult> {
    tracing::debug!("Transforming TypeScript (target: {:?})", options.ts_target);

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript"))?;

    let root = tree.root_node();

    let transformed = strip_unused_named_imports(&remove_types(source, &root)?);

    Ok(TransformResult {
        code: transformed,
        source_map: if options.source_maps {
            Some(generate_source_map())
        } else {
            None
        },
    })
}

/// Remove type annotations from TypeScript code
fn remove_types(source: &str, node: &Node) -> Result<String> {
    let mut result = String::new();
    let mut last_pos = 0;
    let mut cursor = node.walk();

    visit_node(source, node, &mut last_pos, &mut result, &mut cursor)?;

    if last_pos < source.len() {
        result.push_str(&source[last_pos..]);
    }

    Ok(result)
}

/// Visit AST node and remove type-related nodes
fn visit_node<'a>(
    source: &str,
    node: &Node<'a>,
    last_pos: &mut usize,
    result: &mut String,
    cursor: &mut tree_sitter::TreeCursor<'a>,
) -> Result<()> {
    for child in node.children(cursor) {
        match child.kind() {
            "type_annotation"
            | "type_arguments"
            | "type_parameters"
            | "type_predicate_annotation"
            | "accessibility_modifier"
            | "readonly"
            | "interface_declaration"
            | "type_alias_declaration"
            | "function_signature"
            | "internal_module" => {
                if *last_pos < child.start_byte() {
                    result.push_str(&source[*last_pos..child.start_byte()]);
                }
                *last_pos = child.end_byte();
            }

            // TypeScript parameter properties compile to instance assignments.
            // e.g. `constructor(private readonly page: Page) {}` →
            // `constructor(page) { this.page = page; }`
            "method_definition"
                if method_name(source, &child).as_deref() == Some("constructor") =>
            {
                if *last_pos < child.start_byte() {
                    result.push_str(&source[*last_pos..child.start_byte()]);
                }
                result.push_str(&emit_constructor_method(source, &child)?);
                *last_pos = child.end_byte();
            }

            // Compile TypeScript enum to JavaScript IIFE
            "enum_declaration" => {
                if *last_pos < child.start_byte() {
                    result.push_str(&source[*last_pos..child.start_byte()]);
                }
                result.push_str(&compile_enum(source, &child)?);
                *last_pos = child.end_byte();
            }

            // as_expression: keep the expression, strip the type cast
            // e.g. `value as (prev: T) => T` → `value`
            // e.g. `tag as unknown as () => VNode` → `tag`
            "as_expression" => {
                if *last_pos < child.start_byte() {
                    result.push_str(&source[*last_pos..child.start_byte()]);
                }
                result.push_str(&emit_as_expression(source, &child)?);
                *last_pos = child.end_byte();
            }

            // satisfies_expression: keep LHS, drop `satisfies <Type>`
            // e.g. `{ a: 1 } satisfies Foo` → `{ a: 1 }`
            // Regression for jet #1535: browser saw raw `satisfies` keyword.
            "satisfies_expression" => {
                if *last_pos < child.start_byte() {
                    result.push_str(&source[*last_pos..child.start_byte()]);
                }
                result.push_str(&emit_satisfies_expression(source, &child)?);
                *last_pos = child.end_byte();
            }

            // Strip `import type { ... } from '...'`
            "import_statement" => {
                let text = &source[child.byte_range()];
                if text.starts_with("import type ") || text.starts_with("import type{") {
                    if *last_pos < child.start_byte() {
                        result.push_str(&source[*last_pos..child.start_byte()]);
                    }
                    *last_pos = child.end_byte();
                    // Skip the import but preserve the newline so the next line
                    // doesn't concatenate with the previous one.
                    // Only consume \r before \n (Windows line endings).
                    if *last_pos < source.len() && source.as_bytes()[*last_pos] == b'\r' {
                        *last_pos += 1;
                    }
                    // Do NOT consume the trailing \n — it separates the
                    // previous statement from the next one.
                } else {
                    let mut child_cursor = child.walk();
                    visit_node(source, &child, last_pos, result, &mut child_cursor)?;
                }
            }

            // Strip entire export statement if it only exports a type
            "export_statement" => {
                let has_only_type = child_has_only_type(&child);
                if has_only_type {
                    if *last_pos < child.start_byte() {
                        result.push_str(&source[*last_pos..child.start_byte()]);
                    }
                    *last_pos = child.end_byte();
                } else {
                    let mut child_cursor = child.walk();
                    visit_node(source, &child, last_pos, result, &mut child_cursor)?;
                }
            }

            // Strip non-null assertion: expr! → expr
            "non_null_expression" => {
                if *last_pos < child.start_byte() {
                    result.push_str(&source[*last_pos..child.start_byte()]);
                }
                // Emit everything except the trailing "!"
                let text = &source[child.byte_range()];
                if let Some(stripped) = text.strip_suffix('!') {
                    result.push_str(stripped);
                } else {
                    result.push_str(text);
                }
                *last_pos = child.end_byte();
            }

            "optional_parameter" => {
                let param_text = &source[child.byte_range()];
                if let Some(question_pos) = param_text.find('?') {
                    result.push_str(&source[*last_pos..child.start_byte()]);
                    result.push_str(&param_text[..question_pos].trim());
                    *last_pos = child.end_byte();
                }
            }

            _ => {
                if child.child_count() > 0 {
                    let mut child_cursor = child.walk();
                    visit_node(source, &child, last_pos, result, &mut child_cursor)?;
                }
            }
        }
    }

    Ok(())
}

fn method_name(source: &str, node: &Node) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "property_identifier" || child.kind() == "private_property_identifier" {
            return Some(source[child.byte_range()].to_string());
        }
    }
    None
}

fn emit_constructor_method(source: &str, node: &Node) -> Result<String> {
    let mut cursor = node.walk();
    let mut params = None;
    let mut body = None;
    for child in node.children(&mut cursor) {
        match child.kind() {
            "formal_parameters" => params = Some(child),
            "statement_block" => body = Some(child),
            _ => {}
        }
    }
    let Some(params) = params else {
        return Ok(source[node.byte_range()].to_string());
    };
    let Some(body) = body else {
        return Ok(source[node.byte_range()].to_string());
    };

    let assignments = constructor_parameter_property_assignments(source, &params);
    let params_text = emit_formal_parameters(source, &params)?;
    let mut out = String::new();
    out.push_str(&source[node.start_byte()..params.start_byte()]);
    out.push_str(&params_text);
    if params.end_byte() < body.start_byte() {
        out.push_str(&source[params.end_byte()..body.start_byte()]);
    }
    out.push_str(&emit_constructor_body_with_assignments(
        source,
        &body,
        &assignments,
    )?);
    Ok(out)
}

fn constructor_parameter_property_assignments(source: &str, params: &Node) -> Vec<String> {
    let mut assignments = Vec::new();
    let mut cursor = params.walk();
    for child in params.children(&mut cursor) {
        if !is_parameter_node(&child) || !has_parameter_property_modifier(&child) {
            continue;
        }
        if let Some(name) = parameter_identifier(source, &child) {
            assignments.push(format!("this.{name} = {name};"));
        }
    }
    assignments
}

fn emit_formal_parameters(source: &str, params: &Node) -> Result<String> {
    let mut out = String::new();
    let mut cursor = params.walk();
    for child in params.children(&mut cursor) {
        match child.kind() {
            "(" | ")" => out.push_str(&source[child.byte_range()]),
            "," => {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push(',');
                out.push(' ');
            }
            kind if is_parameter_kind(kind) => {
                out.push_str(&emit_parameter(source, &child)?);
            }
            _ => out.push_str(&source[child.byte_range()]),
        }
    }
    Ok(out)
}

fn emit_parameter(source: &str, node: &Node) -> Result<String> {
    let mut result = String::new();
    let mut last_pos = node.start_byte();
    let mut cursor = node.walk();
    visit_node(source, node, &mut last_pos, &mut result, &mut cursor)?;
    if last_pos < node.end_byte() {
        result.push_str(&source[last_pos..node.end_byte()]);
    }
    let text = result.trim().replace('?', "");
    Ok(text)
}

fn emit_constructor_body_with_assignments(
    source: &str,
    body: &Node,
    assignments: &[String],
) -> Result<String> {
    if assignments.is_empty() {
        let mut result = String::new();
        let mut last_pos = body.start_byte();
        let mut cursor = body.walk();
        visit_node(source, body, &mut last_pos, &mut result, &mut cursor)?;
        if last_pos < body.end_byte() {
            result.push_str(&source[last_pos..body.end_byte()]);
        }
        return Ok(result);
    }
    let body_text = &source[body.byte_range()];
    let joined = assignments.join(" ");
    if let Some(rest) = body_text.strip_prefix('{') {
        Ok(format!("{{ {joined} {rest}"))
    } else {
        Ok(body_text.to_string())
    }
}

fn has_parameter_property_modifier(node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "accessibility_modifier" || child.kind() == "readonly" {
            return true;
        }
    }
    false
}

fn parameter_identifier(source: &str, node: &Node) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return Some(source[child.byte_range()].to_string());
        }
    }
    None
}

fn is_parameter_node(node: &Node) -> bool {
    is_parameter_kind(node.kind())
}

fn is_parameter_kind(kind: &str) -> bool {
    matches!(
        kind,
        "required_parameter" | "optional_parameter" | "rest_parameter" | "assignment_pattern"
    )
}

/// Check if an export_statement only exports type-only declarations.
fn child_has_only_type(node: &Node) -> bool {
    let mut cursor = node.walk();
    let mut saw_export = false;
    for child in node.children(&mut cursor) {
        match child.kind() {
            "export" => {
                saw_export = true;
                continue;
            }
            ";" | "comment" => continue,
            "type_alias_declaration"
            | "interface_declaration"
            | "function_signature"
            | "internal_module" => {
                return true;
            }
            // `export type { ... } from '...'`
            "type" if saw_export => return true,
            _ => return false,
        }
    }
    false
}

/// Compile a TypeScript enum declaration to a JavaScript IIFE.
///
/// `enum Priority { Low, Medium, High }` →
/// `var Priority; (function(Priority) { Priority[Priority["Low"] = 0] = "Low"; ... })(Priority || (Priority = {}));`
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn compile_enum(source: &str, node: &Node) -> Result<String> {
    // Find enum name (identifier child)
    let mut enum_name = None;
    let mut body_node = None;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => enum_name = Some(source[child.byte_range()].to_string()),
            "enum_body" => body_node = Some(child),
            _ => {}
        }
    }
    let enum_name = enum_name.ok_or_else(|| anyhow::anyhow!("enum missing name"))?;
    let body = body_node.ok_or_else(|| anyhow::anyhow!("enum missing body"))?;

    // Parse members: property_identifier (auto) or enum_assignment (explicit)
    let mut members: Vec<(String, String, bool)> = Vec::new();
    let mut body_cursor = body.walk();
    let mut next_value: i64 = 0;

    for child in body.children(&mut body_cursor) {
        match child.kind() {
            "property_identifier" => {
                // Auto-increment numeric member
                let name = source[child.byte_range()].to_string();
                members.push((name, next_value.to_string(), true));
                next_value += 1;
            }
            "enum_assignment" => {
                // Explicit initializer: property_identifier = value
                let mut member_name = None;
                let mut member_value = None;
                let mut ac = child.walk();
                for gc in child.children(&mut ac) {
                    match gc.kind() {
                        "property_identifier" => {
                            member_name = Some(source[gc.byte_range()].to_string());
                        }
                        "=" => {}
                        _ if member_name.is_some() && gc.kind() != "," => {
                            member_value = Some(source[gc.byte_range()].to_string());
                        }
                        _ => {}
                    }
                }
                if let (Some(name), Some(value)) = (member_name, member_value) {
                    if let Ok(v) = value.trim().parse::<i64>() {
                        next_value = v + 1;
                        members.push((name, value, true));
                    } else {
                        members.push((name, value, false));
                    }
                }
            }
            _ => {}
        }
    }

    // Use single-expression pattern: var E = (function(E) { ...; return E; })({});
    // This ensures the var is assigned the populated object in one statement,
    // so `module.exports["E"] = E;` after it gets the correct value.
    let mut result = format!("var {} = (function({}) {{ ", enum_name, enum_name);
    for (name, value, is_numeric) in &members {
        if *is_numeric {
            result.push_str(&format!(
                "{}[{}[\"{}\"] = {}] = \"{}\"; ",
                enum_name, enum_name, name, value, name
            ));
        } else {
            result.push_str(&format!("{}[\"{}\"] = {}; ", enum_name, name, value));
        }
    }
    result.push_str(&format!("return {}; }})({{}})", enum_name));

    Ok(result)
}

/// Emit the expression part of a `satisfies_expression`, stripping
/// the `satisfies <Type>` tail. The expression itself is emitted with
/// type stripping (so nested `as` / `satisfies` / `!` collapse too).
fn emit_satisfies_expression(source: &str, node: &Node) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Stop at the "satisfies" keyword; everything after it is the
        // type to discard.
        if child.kind() == "satisfies" {
            break;
        }
        // Nested `satisfies` (e.g. `x satisfies A satisfies B`) — recurse
        // to peel both type tails off.
        if child.kind() == "satisfies_expression" {
            return emit_satisfies_expression(source, &child);
        }
        // `x as A satisfies B` — the inner `as_expression` keeps `x`.
        if child.kind() == "as_expression" {
            return emit_as_expression(source, &child);
        }
        // First non-keyword child is the value expression to keep.
        // Emit it with type stripping so any inner TS syntax also goes away.
        let mut result = String::new();
        let mut last_pos = child.start_byte();
        let mut inner_cursor = child.walk();
        visit_node(
            source,
            &child,
            &mut last_pos,
            &mut result,
            &mut inner_cursor,
        )?;
        if last_pos < child.end_byte() {
            result.push_str(&source[last_pos..child.end_byte()]);
        }
        return Ok(result);
    }
    // Fallback: return raw text (best-effort, shouldn't happen on valid input).
    Ok(source[node.byte_range()].to_string())
}

/// Emit the expression part of an as_expression, stripping the type cast.
/// Handles nested as_expressions: `x as T1 as T2` → `x`
fn emit_as_expression(source: &str, node: &Node) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Skip the "as" keyword and everything after it (type nodes)
        if child.kind() == "as" {
            break;
        }
        // The first child before "as" is the expression to keep
        if child.kind() == "as_expression" {
            // Nested: `expr as T1 as T2` → recurse to get inner expression
            return emit_as_expression(source, &child);
        }
        // Non-as_expression child: emit it with type stripping
        let mut result = String::new();
        let mut last_pos = child.start_byte();
        let mut inner_cursor = child.walk();
        visit_node(
            source,
            &child,
            &mut last_pos,
            &mut result,
            &mut inner_cursor,
        )?;
        if last_pos < child.end_byte() {
            result.push_str(&source[last_pos..child.end_byte()]);
        }
        return Ok(result);
    }
    // Fallback: return raw text
    Ok(source[node.byte_range()].to_string())
}

/// Generate source map
fn generate_source_map() -> String {
    r#"{"version":3,"sources":[],"names":[],"mappings":""}"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_basic() {
        let source = "const x: number = 42;";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(!result.code.contains(": number"));
        assert!(result.code.contains("const x"));
        assert!(result.code.contains("= 42"));
    }

    #[test]
    fn test_typescript_function() {
        let source = "function add(a: number, b: number): number { return a + b; }";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(!result.code.contains(": number"));
        assert!(result.code.contains("function add"));
        assert!(result.code.contains("return a + b"));
    }

    #[test]
    fn test_arrow_function_type_predicate() {
        let source = "const edges = items.filter((edge): edge is { id: string } => edge !== null);";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            result
                .code
                .contains("items.filter((edge) => edge !== null)"),
            "must preserve arrow callback without type predicate: {}",
            result.code
        );
        assert!(
            !result.code.contains("edge is"),
            "must strip type predicate annotation: {}",
            result.code
        );
    }

    #[test]
    fn test_typescript_interface() {
        let source = r#"
interface User {
    name: string;
    age: number;
}

const user: User = { name: "Alice", age: 30 };
        "#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(!result.code.contains("interface"));
        assert!(result.code.contains("const user"));
    }

    #[test]
    fn test_as_expression_leaf() {
        // Leaf identifier: `value as Type` → `value`
        let source = "const x = (value as string);";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(result.code.contains("(value)"), "got: {}", result.code);
        assert!(!result.code.contains("as string"));
    }

    #[test]
    fn test_as_expression_member() {
        // Member expression: `el as HTMLInputElement` → `el`
        let source = "(el as HTMLInputElement).checked = true;";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(result.code.contains("(el).checked"), "got: {}", result.code);
    }

    #[test]
    fn test_as_expression_function_type() {
        // Complex type cast in call: `(value as (prev: T) => T)(current)` → `(value)(current)`
        let source = "const next = (value as (prev: number) => number)(current);";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            result.code.contains("(value)(current)"),
            "got: {}",
            result.code
        );
    }

    #[test]
    fn test_as_expression_nested() {
        // Nested member: `(vnode.tag as Function)(props)` → `(vnode.tag)(props)`
        let source = "(vnode.tag as Function)(vnode.props);";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            result.code.contains("(vnode.tag)(vnode.props)"),
            "got: {}",
            result.code
        );
    }

    #[test]
    fn test_export_function_preserved() {
        let source = "export function render(vnode: VNode, container: HTMLElement): void { }";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            result.code.contains("export function render"),
            "got: {:?}",
            result.code
        );
    }

    #[test]
    fn test_export_function_with_generics() {
        let source =
            "export function useState<T>(initial: T): [T, (v: T | ((prev: T) => T)) => void] { }";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            result.code.contains("export function useState"),
            "got: {:?}",
            result.code
        );
    }

    #[test]
    fn test_export_function_overload_signature_stripped() {
        let source = r#"export function withTheme<C>(Component: C): C;
export function withTheme(Component) { return Component; }"#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            !result.code.contains("withTheme<C>") && !result.code.contains("Component: C"),
            "must strip overload-only type syntax: {}",
            result.code
        );
        assert_eq!(
            result.code.matches("export function withTheme").count(),
            1,
            "must preserve only the implementation export: {}",
            result.code
        );
    }

    #[test]
    fn test_exported_type_namespace_stripped() {
        let source = r#"import { Interpolation } from '@emotion/serialize'
import { Theme } from './theming'
export namespace ReactJSX {
  export type ElementType = string
  export interface Element {}
}
export const value = 1"#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            !result.code.contains("namespace")
                && !result.code.contains("Interpolation")
                && !result.code.contains("Theme"),
            "must strip type-only namespace and imports: {}",
            result.code
        );
        assert!(
            result.code.contains("export const value = 1"),
            "must preserve value export: {}",
            result.code
        );
    }

    #[test]
    fn test_named_type_only_imports_removed_after_strip() {
        let source = r#"import createCache, { EmotionCache } from '@emotion/cache'
import { Theme, ThemeContext } from './theming'
const cache = createCache()
export const value = ThemeContext"#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            result
                .code
                .contains("import createCache from '@emotion/cache';"),
            "must preserve default value import: {}",
            result.code
        );
        assert!(
            result
                .code
                .contains("import { ThemeContext } from './theming';"),
            "must preserve used named value import: {}",
            result.code
        );
        assert!(
            !result.code.contains("EmotionCache") && !result.code.contains("Theme,"),
            "must remove unused type-only named imports: {}",
            result.code
        );
    }

    #[test]
    fn test_export_function_multiline() {
        // Multi-line export function with complex type annotations
        let source = r#"export function createElement(
  tag: string | Function,
  props: Record<string, any> | null,
  ...children: any[]
): VNode {
  return { tag, props: props || {}, children };
}"#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            result.code.contains("export function createElement"),
            "missing export: {:?}",
            result.code
        );
    }

    #[test]
    fn test_private_readonly_class_field_stripped() {
        let source = r#"class ManualRecorder {
  private readonly steps: ManualStep[] = [];
}"#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            !result.code.contains("private") && !result.code.contains("readonly"),
            "must strip TypeScript-only class field modifiers: {}",
            result.code
        );
        assert!(
            result.code.contains("steps = []"),
            "must preserve runtime class field initializer: {}",
            result.code
        );
    }

    #[test]
    fn test_constructor_parameter_property_initializes_this() {
        let source = r#"class TaskEventRecorder {
  constructor(private readonly page: Page) {}
  install() { return this.page; }
}"#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();

        assert!(
            !result.code.contains("private")
                && !result.code.contains("readonly")
                && !result.code.contains(": Page"),
            "must strip parameter property syntax: {}",
            result.code
        );
        assert!(
            result.code.contains("constructor(page)") && result.code.contains("this.page = page;"),
            "must compile constructor parameter property to an instance assignment: {}",
            result.code
        );
    }

    #[test]
    fn test_ts_then_module_transform() {
        // Full pipeline: TS transform → module transform
        let source = r#"export function render(vnode: VNode, container: HTMLElement): void {
  container.innerHTML = "";
}

export const h = createElement;"#;
        let options = TransformOptions::default();
        let ts_result = transform_typescript(source, &options).unwrap();
        println!("TS OUTPUT: {:?}", ts_result.code);

        let module_map = std::collections::HashMap::new();
        let module_result =
            crate::transform::modules::transform_modules(&ts_result.code, &module_map).unwrap();
        println!("MODULE OUTPUT: {:?}", module_result.code);
        assert!(
            module_result.code.contains("module.exports[\"render\"]"),
            "missing render export: {:?}",
            module_result.code
        );
        assert!(
            module_result.code.contains("module.exports[\"h\"]"),
            "missing h export: {:?}",
            module_result.code
        );
    }

    #[test]
    fn test_enum_numeric() {
        let source = "enum Priority { Low, Medium, High }";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            result.code.contains("var Priority = (function"),
            "got: {}",
            result.code
        );
        assert!(result.code.contains("\"Low\"] = 0"), "got: {}", result.code);
        assert!(
            result.code.contains("\"Medium\"] = 1"),
            "got: {}",
            result.code
        );
        assert!(
            result.code.contains("\"High\"] = 2"),
            "got: {}",
            result.code
        );
        assert!(
            result.code.contains("return Priority;"),
            "got: {}",
            result.code
        );
    }

    #[test]
    fn test_enum_string() {
        let source = r#"enum Color { Red = "RED", Green = "GREEN" }"#;
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            result.code.contains("var Color = (function"),
            "got: {}",
            result.code
        );
        assert!(
            result.code.contains("\"Red\"] = \"RED\""),
            "got: {}",
            result.code
        );
    }

    #[test]
    fn test_export_enum_preserved() {
        let source = "export enum Priority { Low, Medium, High }";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            result.code.contains("var Priority"),
            "should not be stripped: {}",
            result.code
        );
        assert!(
            result.code.contains("export"),
            "should keep export: {}",
            result.code
        );
    }

    #[test]
    fn test_satisfies_expression_simple() {
        // Single-line `satisfies` in a .ts file — regression for jet #1535.
        let source = "const cfg = { port: 3000 } satisfies Config;";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            !result.code.contains("satisfies"),
            "must strip `satisfies`: {:?}",
            result.code
        );
        assert!(
            !result.code.contains("Config"),
            "must strip type after satisfies: {:?}",
            result.code
        );
        assert!(
            result.code.contains("{ port: 3000 }"),
            "must keep value expression: {:?}",
            result.code
        );
    }

    #[test]
    fn test_satisfies_expression_multiline_generic() {
        // The exact failure shape from jet #1535 — multi-line const dictionary
        // with `satisfies Record<Locale, CueCopy>` in a .ts file. Before the fix
        // this leaks raw `satisfies` syntax to the browser → SyntaxError.
        let source = "const dictionaries = {\n  'zh-TW': { hello: 'a' },\n  'en-US': { hello: 'b' },\n} satisfies Record<Locale, CueCopy>;\n";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(
            !result.code.contains("satisfies"),
            "must strip `satisfies` keyword: {:?}",
            result.code
        );
        assert!(
            !result.code.contains("Record<"),
            "must strip generic type after satisfies: {:?}",
            result.code
        );
        assert!(
            !result.code.contains("CueCopy"),
            "must strip type argument: {:?}",
            result.code
        );
        assert!(
            result.code.contains("const dictionaries"),
            "must keep declaration: {:?}",
            result.code
        );
        assert!(
            result.code.contains("'zh-TW'") && result.code.contains("'en-US'"),
            "must keep object literal: {:?}",
            result.code
        );
    }

    #[test]
    fn test_as_expression_double_cast() {
        // Double cast: `tag as unknown as () => VNode` → `tag`
        let source = "currentComponent = vnode.tag as unknown as () => VNode;";
        let options = TransformOptions::default();
        let result = transform_typescript(source, &options).unwrap();
        assert!(result.code.contains("= vnode.tag;"), "got: {}", result.code);
        assert!(!result.code.contains(" as "), "got: {}", result.code);
    }
}
// CODEGEN-END
