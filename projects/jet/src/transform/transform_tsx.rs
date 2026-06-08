// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use tree_sitter::{Node, Parser};

use super::type_strip::{
    has_inline_type_specifiers, is_type_only_export, is_type_only_import,
    transform_import_with_inline_types, transform_satisfies_expression,
};
use super::{TransformOptions, TransformResult};

/// Normalize JSX text content per React/Babel JSX whitespace rules:
/// - Trim START of each line if it's not the first line (removes indentation)
/// - Trim END of each line if it's not the last non-empty line
/// - Skip whitespace-only lines
/// - Join remaining parts with a single space
///
/// This preserves trailing spaces on text like `"Counter: "` (before an expression).
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub(super) fn normalize_jsx_text(raw: &str) -> String {
    let lines: Vec<&str> = raw.split('\n').collect();

    // Find the last line that has non-whitespace content
    let last_non_empty = lines
        .iter()
        .rposition(|l| l.contains(|c: char| !c.is_whitespace()));

    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        let mut s = line.replace('\t', " ");

        // Trim start if not the first line
        if i > 0 {
            s = s.trim_start().to_string();
        }

        // Trim end if not the last non-empty line
        if Some(i) != last_non_empty {
            s = s.trim_end().to_string();
        }

        if !s.is_empty() {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&s);
        }
    }

    result
}

/// Transform TSX to JavaScript in a single pass
///
/// This module addresses the critical bugs identified in Issue #101:
/// - Bug 1: Uses LANGUAGE_TSX parser (not JavaScript parser)
/// - Bug 2: Single-pass transformation (not dual pipeline)
/// - Bug 3: Proper error handling (no default "div" fallback)
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_tsx(source: &str, options: &TransformOptions) -> Result<TransformResult> {
    // Detect @jsx pragma → override to classic mode with custom factory
    let has_classic_pragma = source.contains("@jsxRuntime classic") || source.contains("@jsx ");
    let use_automatic = options.jsx_automatic && !has_classic_pragma;

    // Extract @jsx factory name (e.g. /** @jsx createElement */ → "createElement")
    let pragma_factory = if has_classic_pragma {
        extract_jsx_pragma(source)
    } else {
        None
    };
    let frag_factory = extract_jsx_frag_pragma(source);

    tracing::debug!(
        "Transforming TSX (single-pass, jsx_automatic: {}, pragma: {:?}, frag: {:?})",
        use_automatic,
        pragma_factory,
        frag_factory
    );

    let effective_options;
    let opts = if use_automatic != options.jsx_automatic
        || pragma_factory.is_some()
        || frag_factory.is_some()
    {
        effective_options = TransformOptions {
            jsx_automatic: use_automatic,
            jsx_pragma: pragma_factory.or_else(|| options.jsx_pragma.clone()),
            jsx_fragment: frag_factory.or_else(|| options.jsx_fragment.clone()),
            ..options.clone()
        };
        &effective_options
    } else {
        options
    };

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse TSX"))?;

    let root = tree.root_node();

    let mut transformed = transform_node(source, &root, opts)?;

    if use_automatic && has_jsx(&root) {
        let runtime_import = "import { jsx, jsxs, Fragment } from 'react/jsx-runtime';\n";
        transformed = runtime_import.to_string() + &transformed;
    }

    // React Fast Refresh injection (dev mode only, JSX files only)
    if opts.dev_mode && has_jsx(&root) {
        transformed = super::react_refresh::inject_react_fast_refresh(&transformed, source, &root);
    }

    Ok(TransformResult {
        code: transformed,
        source_map: if options.source_maps {
            Some(generate_source_map(source))
        } else {
            None
        },
    })
}

/// Check if the AST contains JSX elements
fn has_jsx(node: &Node) -> bool {
    if matches!(
        node.kind(),
        "jsx_element" | "jsx_self_closing_element" | "jsx_fragment"
    ) {
        return true;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if has_jsx(&child) {
            return true;
        }
    }

    false
}

/// Transform a single AST node
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub(super) fn transform_node(
    source: &str,
    node: &Node,
    options: &TransformOptions,
) -> Result<String> {
    let mut result = String::new();
    let mut cursor = node.walk();
    let mut last_pos = node.start_byte();

    for child in node.children(&mut cursor) {
        // Handle as_expression: keep expression, strip type cast
        if is_as_expression(&child) {
            if child.start_byte() > last_pos {
                result.push_str(&source[last_pos..child.start_byte()]);
            }
            result.push_str(&transform_as_expression(source, &child, options)?);
            last_pos = child.end_byte();
            continue;
        }

        // Handle satisfies_expression: keep LHS, drop satisfies + type
        if child.kind() == "satisfies_expression" {
            if child.start_byte() > last_pos {
                result.push_str(&source[last_pos..child.start_byte()]);
            }
            result.push_str(&transform_satisfies_expression(source, &child, options)?);
            last_pos = child.end_byte();
            continue;
        }

        // Handle export_statement that exports only type-level constructs
        if child.kind() == "export_statement" && is_type_only_export(source, &child) {
            if last_pos < child.start_byte() {
                result.push_str(&source[last_pos..child.start_byte()]);
            }
            last_pos = child.end_byte();
            // Consume trailing newline
            if last_pos < source.len() && source.as_bytes()[last_pos] == b'\n' {
                last_pos += 1;
            }
            continue;
        }

        // Handle import_statement with type modifier → remove entire statement
        if child.kind() == "import_statement" && is_type_only_import(source, &child) {
            if last_pos < child.start_byte() {
                result.push_str(&source[last_pos..child.start_byte()]);
            }
            last_pos = child.end_byte();
            if last_pos < source.len() && source.as_bytes()[last_pos] == b'\n' {
                last_pos += 1;
            }
            continue;
        }

        // Handle import_statement with inline type specifiers
        if child.kind() == "import_statement" && has_inline_type_specifiers(&child) {
            if child.start_byte() > last_pos {
                result.push_str(&source[last_pos..child.start_byte()]);
            }
            let transformed = transform_import_with_inline_types(source, &child)?;
            if let Some(import_str) = transformed {
                result.push_str(&import_str);
            }
            // If None, the import was entirely type-only → remove
            last_pos = child.end_byte();
            continue;
        }

        if should_skip_node(&child) {
            if last_pos < child.start_byte() {
                let before_type = &source[last_pos..child.start_byte()];
                result.push_str(before_type.trim_end());
            }
            last_pos = child.end_byte();
            if last_pos < source.len() {
                let next_char = source.as_bytes().get(last_pos).copied();
                if let Some(ch) = next_char {
                    if ch != b' '
                        && ch != b'\n'
                        && ch != b'\r'
                        && ch != b'\t'
                        && ch != b';'
                        && ch != b','
                        && ch != b')'
                        && ch != b'}'
                        && ch != b'='
                        && ch != b'>'
                    {
                        result.push(' ');
                    }
                }
            }
            continue;
        }

        if child.start_byte() > last_pos {
            result.push_str(&source[last_pos..child.start_byte()]);
        }

        match child.kind() {
            "jsx_element" | "jsx_self_closing_element" => {
                result.push_str(&transform_jsx_element(source, &child, options)?);
                last_pos = child.end_byte();
            }
            "jsx_fragment" => {
                result.push_str(&transform_jsx_fragment(source, &child, options)?);
                last_pos = child.end_byte();
            }

            "optional_parameter" => {
                let param_text = &source[child.byte_range()];
                if let Some(question_pos) = param_text.find('?') {
                    result.push_str(&param_text[..question_pos].trim());
                } else {
                    result.push_str(param_text);
                }
                last_pos = child.end_byte();
            }

            // Compile enum to JavaScript IIFE
            "enum_declaration" => {
                result.push_str(&crate::transform::typescript::compile_enum(source, &child)?);
                last_pos = child.end_byte();
            }

            // Strip non-null assertion: expr! → expr
            "non_null_expression" => {
                let text = &source[child.byte_range()];
                if let Some(stripped) = text.strip_suffix('!') {
                    result.push_str(stripped);
                } else {
                    result.push_str(text);
                }
                last_pos = child.end_byte();
            }

            _ => {
                if child.child_count() > 0 {
                    result.push_str(&transform_node(source, &child, options)?);
                    last_pos = child.end_byte();
                } else {
                    result.push_str(&source[child.byte_range()]);
                    last_pos = child.end_byte();
                }
            }
        }
    }

    if last_pos < node.end_byte() {
        result.push_str(&source[last_pos..node.end_byte()]);
    }

    Ok(result)
}

/// Check if a node should be skipped entirely (TypeScript-specific syntax)
fn should_skip_node(node: &Node) -> bool {
    matches!(
        node.kind(),
        "type_annotation"
            | "type_arguments"
            | "type_parameters"
            | "type_predicate_annotation"
            | "interface_declaration"
            | "type_alias_declaration"
            | "ambient_declaration"
    )
}

/// Check if node is an as_expression that needs special handling
fn is_as_expression(node: &Node) -> bool {
    node.kind() == "as_expression"
}

/// Transform as_expression: keep expression, drop type cast
fn transform_as_expression(
    source: &str,
    node: &Node,
    options: &TransformOptions,
) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "as"
            && child.kind() != "type_identifier"
            && child.kind() != "generic_type"
            && child.kind() != "parenthesized_type"
            && child.kind() != "function_type"
            && child.kind() != "predefined_type"
        {
            // This is the expression to keep — recurse into it
            return transform_node(source, &child, options);
        }
    }
    // Fallback: return the raw text
    Ok(source[node.byte_range()].to_string())
}

/// Transform JSX element to jsx() or React.createElement() call
fn transform_jsx_element(source: &str, node: &Node, options: &TransformOptions) -> Result<String> {
    let tag_name = extract_tag_name(source, node);

    // tree-sitter-typescript v0.23 parses <>..</> as jsx_element with no tag name
    if tag_name.is_none() {
        return transform_jsx_fragment(source, node, options);
    }
    let tag_name = tag_name.unwrap();

    let props = extract_props(source, node, options)?;
    let children = extract_children(source, node, options)?;

    if options.jsx_automatic {
        transform_to_jsx_runtime(&tag_name, &props, &children)
    } else {
        transform_to_create_element(&tag_name, &props, &children, options)
    }
}

/// Transform JSX fragment <>...</>
fn transform_jsx_fragment(source: &str, node: &Node, options: &TransformOptions) -> Result<String> {
    let children = extract_children(source, node, options)?;

    if options.jsx_automatic {
        Ok(format!(
            "jsxs(Fragment, {{ children: [{}] }})",
            children.join(", ")
        ))
    } else {
        let fragment = options.jsx_fragment.as_deref().unwrap_or("React.Fragment");
        Ok(format!(
            "{}({}, null, {})",
            options
                .jsx_pragma
                .as_deref()
                .unwrap_or("React.createElement"),
            fragment,
            children.join(", ")
        ))
    }
}

/// Extract tag name from JSX element
fn extract_tag_name(source: &str, node: &Node) -> Option<String> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "jsx_opening_element" => {
                return extract_tag_from_opening(&child, source);
            }
            "identifier" | "member_expression" => {
                return Some(source[child.byte_range()].to_string());
            }
            _ => {}
        }
    }

    tracing::warn!(
        "Could not extract tag name from JSX element: {}",
        &source[node.byte_range()]
    );
    None
}

/// Extract tag name from opening element
fn extract_tag_from_opening(opening: &Node, source: &str) -> Option<String> {
    let mut cursor = opening.walk();

    for child in opening.children(&mut cursor) {
        match child.kind() {
            "identifier" | "member_expression" => {
                return Some(source[child.byte_range()].to_string());
            }
            _ => {}
        }
    }

    tracing::warn!(
        "Could not extract tag name from opening element: {}",
        &source[opening.byte_range()]
    );
    None
}

/// Extract props from JSX element
fn extract_props(
    source: &str,
    node: &Node,
    options: &TransformOptions,
) -> Result<Vec<(String, String)>> {
    // For self-closing elements, attributes are directly on the node
    if node.kind() == "jsx_self_closing_element" {
        return extract_props_from_opening(source, node, options);
    }

    // For regular elements, look for the opening element child
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "jsx_opening_element" {
            return extract_props_from_opening(source, &child, options);
        }
    }

    Ok(vec![])
}

/// Extract props from opening element
fn extract_props_from_opening(
    source: &str,
    opening: &Node,
    options: &TransformOptions,
) -> Result<Vec<(String, String)>> {
    let mut props = vec![];
    let mut cursor = opening.walk();

    for child in opening.children(&mut cursor) {
        if child.kind() == "jsx_attribute" {
            let prop = extract_single_prop(source, &child, options)?;
            props.push(prop);
        } else if child.kind() == "jsx_expression" && is_spread_expression(&child) {
            // tree-sitter-typescript: spread props are jsx_expression > spread_element
            let expr = extract_spread_expression(source, &child, options)?;
            props.push(("...".to_string(), expr));
        }
    }

    Ok(props)
}

/// Extract a single prop
fn extract_single_prop(
    source: &str,
    attr: &Node,
    options: &TransformOptions,
) -> Result<(String, String)> {
    let mut cursor = attr.walk();
    let mut name = String::new();
    let mut value = String::from("true");

    for child in attr.children(&mut cursor) {
        match child.kind() {
            "property_identifier" => {
                name = source[child.byte_range()].to_string();
            }
            "jsx_expression" => {
                value = extract_jsx_expression(source, &child, options)?;
            }
            "string" => {
                value = source[child.byte_range()].to_string();
            }
            _ => {}
        }
    }

    Ok((name, value))
}

/// Check if a jsx_expression contains a spread_element: {...expr}
fn is_spread_expression(node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "spread_element" {
            return true;
        }
    }
    false
}

/// Extract expression from spread: jsx_expression > spread_element > expr
fn extract_spread_expression(
    source: &str,
    node: &Node,
    options: &TransformOptions,
) -> Result<String> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "spread_element" {
            // spread_element children: "...", expression
            let mut sc = child.walk();
            for spread_child in child.children(&mut sc) {
                if spread_child.kind() != "..." {
                    return transform_node(source, &spread_child, options);
                }
            }
        }
    }

    Ok(source[node.byte_range()].to_string())
}

/// Extract JSX expression (content inside {})
fn extract_jsx_expression(source: &str, node: &Node, options: &TransformOptions) -> Result<String> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "{" | "}" | "!" => continue,
            "jsx_element" | "jsx_self_closing_element" => {
                return transform_jsx_element(source, &child, options);
            }
            "non_null_expression" => {
                // Strip non-null assertion: expr! → expr
                let text = &source[child.byte_range()];
                return Ok(text.strip_suffix('!').unwrap_or(text).to_string());
            }
            _ => {
                if child.child_count() > 0 {
                    return transform_node(source, &child, options);
                } else {
                    return Ok(source[child.byte_range()].to_string());
                }
            }
        }
    }

    Ok(String::new())
}

/// Extract children from JSX element
fn extract_children(source: &str, node: &Node, options: &TransformOptions) -> Result<Vec<String>> {
    let mut children = vec![];
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "jsx_text" => {
                let text = normalize_jsx_text(&source[child.byte_range()]);
                if !text.is_empty() {
                    children.push(format!("\"{}\"", text.replace('"', "\\\"")));
                }
            }
            "jsx_expression" => {
                // Skip spread expressions in attributes (handled by extract_props)
                if !is_spread_expression(&child) {
                    let expr = extract_jsx_expression(source, &child, options)?;
                    if !expr.is_empty() {
                        children.push(expr);
                    }
                }
            }
            "jsx_element" | "jsx_self_closing_element" => {
                children.push(transform_jsx_element(source, &child, options)?);
            }
            "jsx_fragment" => {
                children.push(transform_jsx_fragment(source, &child, options)?);
            }
            _ => {}
        }
    }

    Ok(children)
}

/// Transform to React 17+ jsx() runtime
fn transform_to_jsx_runtime(
    tag_name: &str,
    props: &[(String, String)],
    children: &[String],
) -> Result<String> {
    let is_component = tag_name.chars().next().unwrap_or('a').is_uppercase();
    let tag = if is_component {
        tag_name.to_string()
    } else {
        format!("\"{}\"", tag_name)
    };

    let jsx_func = if children.len() > 1 { "jsxs" } else { "jsx" };

    // Extract "key" prop — must be passed as 3rd argument to jsx(), not in props
    let mut key_arg: Option<&str> = None;
    let mut props_str = String::new();
    if !props.is_empty() {
        for (key, value) in props {
            if key == "key" {
                key_arg = Some(value.as_str());
                continue;
            }
            if !props_str.is_empty() {
                props_str.push_str(", ");
            }
            if key == "..." {
                // Spread syntax: ...expr
                props_str.push_str(&format!("...{}", value));
            } else if key.contains('-') {
                props_str.push_str(&format!("\"{}\": {}", key, value));
            } else {
                props_str.push_str(&format!("{}: {}", key, value));
            }
        }
    }

    if !children.is_empty() {
        if !props_str.is_empty() {
            props_str.push_str(", ");
        }
        if children.len() == 1 {
            props_str.push_str(&format!("children: {}", children[0]));
        } else {
            props_str.push_str(&format!("children: [{}]", children.join(", ")));
        }
    }

    let props_obj = if props_str.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", props_str)
    };

    if let Some(key_val) = key_arg {
        Ok(format!("{}({}, {}, {})", jsx_func, tag, props_obj, key_val))
    } else {
        Ok(format!("{}({}, {})", jsx_func, tag, props_obj))
    }
}

/// Transform to classic React.createElement()
fn transform_to_create_element(
    tag_name: &str,
    props: &[(String, String)],
    children: &[String],
    options: &TransformOptions,
) -> Result<String> {
    let is_component = tag_name.chars().next().unwrap_or('a').is_uppercase();
    let tag = if is_component {
        tag_name.to_string()
    } else {
        format!("\"{}\"", tag_name)
    };

    let pragma = options
        .jsx_pragma
        .as_deref()
        .unwrap_or("React.createElement");

    let has_spread = props.iter().any(|(k, _)| k == "...");

    let props_obj = if props.is_empty() {
        "null".to_string()
    } else if has_spread {
        // Use Object.assign to merge spread props with explicit props
        let mut parts: Vec<String> = vec!["{}".to_string()];
        let mut current_obj_props: Vec<String> = vec![];

        for (key, value) in props {
            if key == "..." {
                // Flush accumulated regular props first
                if !current_obj_props.is_empty() {
                    parts.push(format!("{{ {} }}", current_obj_props.join(", ")));
                    current_obj_props.clear();
                }
                parts.push(value.clone());
            } else {
                let prop = if key.contains('-') {
                    format!("\"{}\": {}", key, value)
                } else {
                    format!("{}: {}", key, value)
                };
                current_obj_props.push(prop);
            }
        }

        // Flush remaining regular props
        if !current_obj_props.is_empty() {
            parts.push(format!("{{ {} }}", current_obj_props.join(", ")));
        }

        format!("Object.assign({})", parts.join(", "))
    } else {
        let mut props_str = String::new();
        for (key, value) in props {
            if !props_str.is_empty() {
                props_str.push_str(", ");
            }
            if key.contains('-') {
                props_str.push_str(&format!("\"{}\": {}", key, value));
            } else {
                props_str.push_str(&format!("{}: {}", key, value));
            }
        }
        format!("{{ {} }}", props_str)
    };

    if children.is_empty() {
        Ok(format!("{}({}, {})", pragma, tag, props_obj))
    } else {
        Ok(format!(
            "{}({}, {}, {})",
            pragma,
            tag,
            props_obj,
            children.join(", ")
        ))
    }
}

/// Extract @jsx pragma factory name from source comments.
/// e.g. `/** @jsx createElement */` → Some("createElement")
/// e.g. `/** @jsx h */` → Some("h")
fn extract_jsx_pragma(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        // Match /** @jsx NAME */ or /* @jsx NAME */ or // @jsx NAME
        if let Some(pos) = trimmed.find("@jsx ") {
            let after = &trimmed[pos + 5..];
            let name = after.trim().trim_end_matches("*/").trim();
            if !name.is_empty() && !name.contains(' ') {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract @jsxFrag pragma fragment name from source comments.
/// e.g. `/** @jsxFrag Fragment */` → Some("Fragment")
fn extract_jsx_frag_pragma(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(pos) = trimmed.find("@jsxFrag ") {
            let after = &trimmed[pos + 9..];
            let name = after.trim().trim_end_matches("*/").trim();
            if !name.is_empty() && !name.contains(' ') {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Generate source map (placeholder)
fn generate_source_map(_source: &str) -> String {
    r#"{"version":3,"sources":[],"names":[],"mappings":""}"#.to_string()
}

#[cfg(test)]
#[path = "transform_tsx_tests.rs"]
mod tests;
// CODEGEN-END
