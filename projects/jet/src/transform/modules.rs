// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};

use super::TransformResult;

/// Module mapping for resolving import paths
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone)]
pub enum ModuleMapping {
    /// Internal module with numeric ID
    Internal(usize),
    /// External module with package name
    External(String),
}

/// Transform ES6 module syntax (import/export) to CommonJS (require/module.exports)
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_modules(
    source: &str,
    module_map: &HashMap<PathBuf, usize>,
) -> Result<TransformResult> {
    transform_modules_with_dir(source, module_map, None)
}

/// Transform ES6 module syntax with current module directory for relative path resolution.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_modules_with_dir(
    source: &str,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> Result<TransformResult> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_javascript::LANGUAGE.into())?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse JavaScript"))?;

    let root = tree.root_node();

    let has_esm_module_syntax = contains_esm_module_syntax(&root);
    let transformed = transform_node(source, &root, module_map, current_dir)?;
    let transformed = if has_esm_module_syntax {
        format!(
            "Object.defineProperty(module.exports, \"__esModule\", {{ value: true }});\n{}",
            transformed
        )
    } else {
        transformed
    };

    Ok(TransformResult {
        code: transformed,
        source_map: None,
    })
}

fn contains_esm_module_syntax(root: &Node) -> bool {
    let mut cursor = root.walk();
    let has_esm_module_syntax = root
        .children(&mut cursor)
        .any(|child| matches!(child.kind(), "import_statement" | "export_statement"));
    has_esm_module_syntax
}

/// Transform a single AST node
fn transform_node(
    source: &str,
    node: &Node,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> Result<String> {
    let mut result = String::new();
    let mut cursor = node.walk();
    let mut last_pos = node.start_byte();

    for child in node.children(&mut cursor) {
        if child.start_byte() > last_pos {
            result.push_str(&source[last_pos..child.start_byte()]);
        }

        match child.kind() {
            "import_statement" => {
                result.push_str(&transform_import(source, &child, module_map, current_dir)?);
                last_pos = child.end_byte();
            }
            "export_statement" => {
                result.push_str(&transform_export(source, &child, module_map, current_dir)?);
                last_pos = child.end_byte();
            }
            "call_expression" if is_dynamic_import(source, &child) => {
                result.push_str(&transform_dynamic_import(
                    source,
                    &child,
                    module_map,
                    current_dir,
                )?);
                last_pos = child.end_byte();
            }
            "call_expression" if is_require_call(source, &child) => {
                result.push_str(&transform_require_call(
                    source,
                    &child,
                    module_map,
                    current_dir,
                )?);
                last_pos = child.end_byte();
            }
            _ => {
                if child.child_count() > 0 {
                    result.push_str(&transform_node(source, &child, module_map, current_dir)?);
                } else {
                    result.push_str(&source[child.byte_range()]);
                }
                last_pos = child.end_byte();
            }
        }
    }

    if last_pos < node.end_byte() {
        result.push_str(&source[last_pos..node.end_byte()]);
    }

    Ok(result)
}

/// Transform import statement to require()
fn transform_import(
    source: &str,
    node: &Node,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> Result<String> {
    let mut cursor = node.walk();
    let mut import_clause = None;
    let mut source_path = None;

    for child in node.children(&mut cursor) {
        match child.kind() {
            "import_clause" => {
                import_clause = Some(child);
            }
            "string" => {
                let path_str = &source[child.byte_range()];
                source_path = Some(path_str.trim_matches('"').trim_matches('\'').to_string());
            }
            _ => {}
        }
    }

    if import_clause.is_none() {
        if let Some(path) = source_path {
            let require_target = resolve_module_path(&path, module_map, current_dir);
            return Ok(format!("{};", require_target));
        }
        return Ok(String::new());
    }

    let import_clause = import_clause.unwrap();
    let source_path = source_path.ok_or_else(|| anyhow::anyhow!("Missing import source"))?;

    let require_target = resolve_module_path(&source_path, module_map, current_dir);

    let import_spec = parse_import_clause(source, &import_clause)?;

    match import_spec {
        ImportSpec::DefaultImport(name) => Ok(format!(
            "var {} = {}[\"default\"] || {};",
            name, require_target, require_target
        )),
        ImportSpec::NamespaceImport(name) => Ok(format!("var {} = {};", name, require_target)),
        ImportSpec::NamedImports(names) => {
            let requires: Vec<String> = names
                .iter()
                .map(|(imported, local)| {
                    format!("var {} = {}[\"{}\"];", local, require_target, imported)
                })
                .collect();
            Ok(requires.join(" "))
        }
        ImportSpec::Mixed(default_name, named_imports) => {
            let mut statements = vec![format!(
                "var {} = {}[\"default\"] || {};",
                default_name, require_target, require_target
            )];
            for (imported, local) in named_imports {
                statements.push(format!(
                    "var {} = {}[\"{}\"];",
                    local, require_target, imported
                ));
            }
            Ok(statements.join(" "))
        }
    }
}

/// Transform export statement to module.exports
fn transform_export(
    source: &str,
    node: &Node,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> Result<String> {
    let mut cursor = node.walk();

    // Check for re-export source: export { X } from "./X"
    let reexport_source = extract_export_source(source, node);

    for child in node.children(&mut cursor) {
        match child.kind() {
            "export" => continue,
            "default" => {
                let value = extract_export_value(source, node)?;
                return Ok(format!("module.exports[\"default\"] = {};", value));
            }
            "*" => {
                // export * from './foo' → re-export all named exports
                if let Some(ref src_path) = reexport_source {
                    let require_target = resolve_module_path(src_path, module_map, current_dir);
                    return Ok(format!(
                        "var __re = {}; Object.keys(__re).forEach(function(k) {{ if (k !== \"default\") module.exports[k] = __re[k]; }});",
                        require_target
                    ));
                }
                continue;
            }
            "lexical_declaration"
            | "variable_declaration"
            | "function_declaration"
            | "class_declaration" => {
                let declaration = transform_node(source, &child, module_map, current_dir)?;
                let export_names = extract_declaration_names(&child, source)?;

                let mut result = String::new();
                result.push_str(&declaration);
                result.push_str("; ");

                for name in export_names {
                    result.push_str(&format!("module.exports[\"{}\"] = {}; ", name, name));
                }

                return Ok(result);
            }
            "export_clause" => {
                let names = parse_export_clause(source, &child)?;

                if let Some(ref src_path) = reexport_source {
                    // Re-export: export { X } from "./X" → require source, then assign
                    let require_target = resolve_module_path(src_path, module_map, current_dir);
                    let exports: Vec<String> = names
                        .iter()
                        .map(|(local, exported)| {
                            format!(
                                "module.exports[\"{}\"] = {}[\"{}\"];",
                                exported, require_target, local
                            )
                        })
                        .collect();
                    return Ok(exports.join(" "));
                } else {
                    // Local re-export: export { X } → assign local variable
                    let exports: Vec<String> = names
                        .iter()
                        .map(|(local, exported)| {
                            format!("module.exports[\"{}\"] = {};", exported, local)
                        })
                        .collect();
                    return Ok(exports.join(" "));
                }
            }
            _ => {}
        }
    }

    Ok(String::new())
}

/// Extract source path from re-export: export { X } from "./X" → Some("./X")
fn extract_export_source(source: &str, node: &Node) -> Option<String> {
    let mut cursor = node.walk();
    let mut found_from = false;

    for child in node.children(&mut cursor) {
        if child.kind() == "from" {
            found_from = true;
            continue;
        }
        if found_from && child.kind() == "string" {
            let path_str = &source[child.byte_range()];
            return Some(path_str.trim_matches('"').trim_matches('\'').to_string());
        }
    }

    None
}

/// Check if a call_expression is a dynamic import: import('./path')
fn is_dynamic_import(_source: &str, node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "import" {
            return true;
        }
        // Only check first meaningful child
        if child.kind() != "(" && child.kind() != ")" {
            break;
        }
    }
    false
}

/// Transform dynamic import() to Promise.resolve(require())
fn transform_dynamic_import(
    source: &str,
    node: &Node,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            let mut arg_cursor = child.walk();
            for arg_child in child.children(&mut arg_cursor) {
                if arg_child.kind() == "string" {
                    let path_str = &source[arg_child.byte_range()];
                    let module_path = path_str.trim_matches('"').trim_matches('\'').to_string();
                    let require_target = resolve_module_path(&module_path, module_map, current_dir);
                    return Ok(format!("Promise.resolve({})", require_target));
                }
            }
        }
    }
    // Fallback: return original
    Ok(source[node.byte_range()].to_string())
}

/// Resolve module path to require() target.
///
/// If `current_dir` is provided, relative paths (./foo) are resolved
/// against it and matched against absolute paths in the module map.
fn normalize_path_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() && !out.has_root() {
                    out.push("..");
                }
            }
            _ => out.push(component.as_os_str()),
        }
    }
    out
}

fn lookup_module_id(module_map: &HashMap<PathBuf, usize>, path: &Path) -> Option<usize> {
    if let Some(&id) = module_map.get(path) {
        return Some(id);
    }

    let normalized = normalize_path_lexical(path);
    if let Some(&id) = module_map.get(&normalized) {
        return Some(id);
    }

    if let Ok(canonical) = path.canonicalize() {
        if let Some(&id) = module_map.get(&canonical) {
            return Some(id);
        }
        let normalized_canonical = normalize_path_lexical(&canonical);
        if let Some(&id) = module_map.get(&normalized_canonical) {
            return Some(id);
        }
    }

    module_map.iter().find_map(|(module_path, id)| {
        (normalize_path_lexical(module_path) == normalized).then_some(*id)
    })
}

fn collect_package_entry_candidates(pkg: &serde_json::Value, out: &mut Vec<String>) {
    fn push_string(value: Option<&serde_json::Value>, out: &mut Vec<String>) {
        if let Some(s) = value.and_then(|v| v.as_str()) {
            out.push(s.to_string());
        }
    }

    if let Some(exports) = pkg.get("exports") {
        let root = exports.get(".").unwrap_or(exports);
        collect_export_candidate_strings(root, out);
    }

    push_string(pkg.get("browser"), out);
    push_string(pkg.get("module"), out);
    push_string(pkg.get("main"), out);
}

fn collect_export_candidate_strings(value: &serde_json::Value, out: &mut Vec<String>) {
    if let Some(s) = value.as_str() {
        out.push(s.to_string());
        return;
    }

    let Some(obj) = value.as_object() else {
        return;
    };

    for key in [
        "browser",
        "default",
        "import",
        "require",
        "module",
        "production",
        "development",
    ] {
        if let Some(child) = obj.get(key) {
            collect_export_candidate_strings(child, out);
        }
    }
}

fn resolve_module_path(
    path: &str,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> String {
    let path_buf = PathBuf::from(path);

    // Direct match (works for absolute paths or exact relative matches)
    if let Some(id) = lookup_module_id(module_map, &path_buf) {
        return format!("require({})", id);
    }

    // Try relative path resolution with extensions
    if path.starts_with('.') {
        // First try without current_dir (legacy behavior)
        for ext in &["", ".js", ".jsx", ".ts", ".tsx"] {
            let mut test_path = path_buf.clone();
            if !ext.is_empty() {
                test_path.set_extension(&ext[1..]);
            }
            if let Some(id) = lookup_module_id(module_map, &test_path) {
                return format!("require({})", id);
            }
        }

        // Resolve relative to current module directory
        if let Some(dir) = current_dir {
            let resolved = dir.join(path);
            for ext in &["", ".js", ".jsx", ".ts", ".tsx"] {
                let mut test_path = resolved.clone();
                if !ext.is_empty() {
                    test_path.set_extension(&ext[1..]);
                }
                // Try exact match
                if let Some(id) = lookup_module_id(module_map, &test_path) {
                    return format!("require({})", id);
                }
            }
            // Directory package resolution, e.g. ./createTheme with
            // package.json { "module": "../esm/createTheme/index.js" }.
            let pkg_json = resolved.join("package.json");
            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut entries = Vec::new();
                    collect_package_entry_candidates(&pkg, &mut entries);
                    for entry in entries {
                        let entry = entry.trim_start_matches("./");
                        let entry_path = resolved.join(entry);
                        if let Some(id) = lookup_module_id(module_map, &entry_path) {
                            return format!("require({})", id);
                        }
                    }
                }
            }
            // Also try index files
            for index in &["index.js", "index.ts", "index.tsx"] {
                let test_path = resolved.join(index);
                if let Some(id) = lookup_module_id(module_map, &test_path) {
                    return format!("require({})", id);
                }
            }
        }
    }

    // Bare specifier resolution (e.g. "react", "react/jsx-runtime", "scheduler")
    if !path.starts_with('.') && !path.starts_with('/') {
        if let Some(dir) = current_dir {
            let mut search_dir = Some(dir);
            while let Some(d) = search_dir {
                let nm_dir = d.join("node_modules");
                if nm_dir.is_dir() {
                    let candidate = nm_dir.join(path);

                    // Try direct file with extensions
                    for ext in &["", ".js", ".json"] {
                        let test = if ext.is_empty() {
                            candidate.clone()
                        } else {
                            let mut p = candidate.clone();
                            p.set_extension(&ext[1..]);
                            p
                        };
                        if let Some(id) = lookup_module_id(module_map, &test) {
                            return format!("require({})", id);
                        }
                    }

                    // Try package.json resolution
                    let pkg_json = candidate.join("package.json");
                    if pkg_json.exists() {
                        match std::fs::read_to_string(&pkg_json) {
                            Ok(content) => {
                                match serde_json::from_str::<serde_json::Value>(&content) {
                                    Ok(pkg) => {
                                        let mut entries = Vec::new();
                                        collect_package_entry_candidates(&pkg, &mut entries);
                                        for entry in entries {
                                            let entry = entry.trim_start_matches("./");
                                            let main_path = candidate.join(entry);
                                            if let Some(id) =
                                                lookup_module_id(module_map, &main_path)
                                            {
                                                return format!("require({})", id);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        tracing::warn!(
                                            target: "jet::transform::modules",
                                            path = %pkg_json.display(),
                                            specifier = %path,
                                            error = %err,
                                            "GH #3222 failed to parse node_modules package.json; falling through to index.js"
                                        );
                                    }
                                }
                            }
                            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                                // Race with deletion after .exists() returned true; stay silent.
                            }
                            Err(err) => {
                                tracing::warn!(
                                    target: "jet::transform::modules",
                                    path = %pkg_json.display(),
                                    specifier = %path,
                                    error = %err,
                                    "GH #3222 failed to read node_modules package.json; falling through to index.js"
                                );
                            }
                        }

                        // Fallback: try index.js
                        let index = candidate.join("index.js");
                        if let Some(id) = lookup_module_id(module_map, &index) {
                            return format!("require({})", id);
                        }
                    }
                }
                search_dir = d.parent();
            }
        }
    }

    format!("require('{}')", path)
}

/// Check if a call_expression is a CJS require('path') call
fn is_require_call(source: &str, node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" && &source[child.byte_range()] == "require" {
            return true;
        }
        // Only check the function name (first child)
        break;
    }
    false
}

/// Transform CJS require('path') to require(numericId)
fn transform_require_call(
    source: &str,
    node: &Node,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> Result<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            let mut arg_cursor = child.walk();
            for arg_child in child.children(&mut arg_cursor) {
                if arg_child.kind() == "string" {
                    let path_str = &source[arg_child.byte_range()];
                    let module_path = path_str.trim_matches('"').trim_matches('\'').to_string();
                    let resolved = resolve_module_path(&module_path, module_map, current_dir);
                    return Ok(resolved);
                }
            }
        }
    }
    // Fallback: return original (e.g. require(variable))
    Ok(source[node.byte_range()].to_string())
}

/// Extract value from export default statement
fn extract_export_value(source: &str, node: &Node) -> Result<String> {
    let mut cursor = node.walk();
    let mut found_default = false;

    for child in node.children(&mut cursor) {
        if child.kind() == "default" {
            found_default = true;
            continue;
        }
        if found_default && child.kind() != "export" && child.kind() != ";" {
            return Ok(source[child.byte_range()].to_string());
        }
    }

    Err(anyhow::anyhow!("Could not extract export default value"))
}

/// Extract names from declaration (const, function, class).
/// Only extracts top-level declaration names — does NOT recurse into function/class bodies.
fn extract_declaration_names(node: &Node, source: &str) -> Result<Vec<String>> {
    let mut names = Vec::new();

    // Handle the node itself if it's a function/class declaration
    match node.kind() {
        "function_declaration" | "class_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                names.push(source[name_node.byte_range()].to_string());
            }
            return Ok(names);
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "variable_declarator" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    names.push(source[name_node.byte_range()].to_string());
                }
            }
            "function_declaration" | "class_declaration" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    names.push(source[name_node.byte_range()].to_string());
                }
            }
            "lexical_declaration" | "variable_declaration" => {
                let mut inner_cursor = child.walk();
                for inner in child.children(&mut inner_cursor) {
                    if inner.kind() == "variable_declarator" {
                        if let Some(name_node) = inner.child_by_field_name("name") {
                            names.push(source[name_node.byte_range()].to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(names)
}

/// Parse export clause: export { foo, bar as baz }
fn parse_export_clause(source: &str, clause: &Node) -> Result<Vec<(String, String)>> {
    let mut exports = Vec::new();
    let mut cursor = clause.walk();

    for child in clause.children(&mut cursor) {
        if child.kind() == "export_specifier" {
            let (local, exported) = parse_export_specifier(source, &child)?;
            exports.push((local, exported));
        }
    }

    Ok(exports)
}

/// Parse single export specifier.
///
/// Handles patterns:
/// - `Foo`                   → local=Foo, exported=Foo
/// - `Foo as Bar`            → local=Foo, exported=Bar
/// - `type Foo`              → skip "type" keyword, local=Foo
/// - `type Foo as Bar`       → skip "type", local=Foo, exported=Bar
/// - `default as Foo`        → local=default, exported=Foo
/// - string literal exports  → skip gracefully
fn parse_export_specifier(source: &str, node: &Node) -> Result<(String, String)> {
    let raw = source[node.byte_range()].trim();
    if let Some((local, exported)) = raw.split_once(" as ") {
        let clean = |value: &str| {
            value
                .trim()
                .trim_start_matches("type ")
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        };
        return Ok((clean(local), clean(exported)));
    }

    let mut local = None;
    let mut exported = None;
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            // Regular identifier
            "identifier" | "property_identifier" => {
                let name = source[child.byte_range()].to_string();
                // Skip the "type" keyword in `export { type Foo }`
                if name == "type" && local.is_none() {
                    continue;
                }
                if local.is_none() {
                    local = Some(name);
                } else {
                    exported = Some(name);
                }
            }
            // `export { "string" as name }` — use the string content
            "string" | "string_fragment" => {
                let text = source[child.byte_range()].to_string();
                let clean = text.trim_matches('"').trim_matches('\'').to_string();
                if local.is_none() {
                    local = Some(clean);
                } else {
                    exported = Some(clean);
                }
            }
            // `as` keyword, `default` keyword
            _ => {
                let text = source[child.byte_range()].to_string();
                if text == "default" {
                    if local.is_none() {
                        local = Some("default".to_string());
                    }
                }
            }
        }
    }

    // Graceful fallback: if no identifier found, use the raw text
    let local = local.unwrap_or_else(|| source[node.byte_range()].trim().to_string());
    let exported = exported.unwrap_or_else(|| local.clone());

    Ok((local, exported))
}

/// Import specification types
#[derive(Debug)]
enum ImportSpec {
    DefaultImport(String),
    NamespaceImport(String),
    NamedImports(Vec<(String, String)>),
    Mixed(String, Vec<(String, String)>),
}

/// Parse import clause
fn parse_import_clause(source: &str, clause: &Node) -> Result<ImportSpec> {
    let mut cursor = clause.walk();
    let mut default_import = None;
    let mut namespace_import = None;
    let mut named_imports = Vec::new();

    for child in clause.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                default_import = Some(source[child.byte_range()].to_string());
            }
            "namespace_import" => {
                namespace_import = Some(parse_namespace_import(source, &child)?);
            }
            "named_imports" => {
                named_imports = parse_named_imports(source, &child)?;
            }
            _ => {}
        }
    }

    match (default_import, namespace_import, named_imports.is_empty()) {
        (Some(default), None, true) => Ok(ImportSpec::DefaultImport(default)),
        (None, Some(namespace), _) => Ok(ImportSpec::NamespaceImport(namespace)),
        (None, None, false) => Ok(ImportSpec::NamedImports(named_imports)),
        (Some(default), None, false) => Ok(ImportSpec::Mixed(default, named_imports)),
        _ => Err(anyhow::anyhow!("Invalid import clause")),
    }
}

/// Parse namespace import: * as name
fn parse_namespace_import(source: &str, node: &Node) -> Result<String> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return Ok(source[child.byte_range()].to_string());
        }
    }

    Err(anyhow::anyhow!("Missing identifier in namespace import"))
}

/// Parse named imports: { foo, bar as baz }
fn parse_named_imports(source: &str, node: &Node) -> Result<Vec<(String, String)>> {
    let mut imports = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "import_specifier" {
            let (imported, local) = parse_import_specifier(source, &child)?;
            imports.push((imported, local));
        }
    }

    Ok(imports)
}

/// Parse import specifier: foo or bar as baz
fn parse_import_specifier(source: &str, node: &Node) -> Result<(String, String)> {
    let mut imported = None;
    let mut local = None;
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            if imported.is_none() {
                imported = Some(source[child.byte_range()].to_string());
            } else {
                local = Some(source[child.byte_range()].to_string());
            }
        }
    }

    let imported =
        imported.ok_or_else(|| anyhow::anyhow!("Missing imported name in import specifier"))?;
    let local = local.unwrap_or_else(|| imported.clone());

    Ok((imported, local))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_module_map() -> HashMap<PathBuf, usize> {
        let mut map = HashMap::new();
        map.insert(PathBuf::from("./utils.js"), 1);
        map.insert(PathBuf::from("./components/Button.jsx"), 2);
        map
    }

    #[test]
    fn test_import_default() {
        let source = "import React from 'react';";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(result.code.contains("var React"));
        assert!(result.code.contains("require('react')"));
    }

    #[test]
    fn test_import_named() {
        let source = "import { useState, useEffect } from 'react';";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(result.code.contains("var useState"));
        assert!(result.code.contains("var useEffect"));
    }

    #[test]
    fn test_import_namespace() {
        let source = "import * as utils from './utils.js';";
        let map = test_module_map();
        let result = transform_modules(source, &map).unwrap();
        assert!(result.code.contains("var utils"));
        assert!(result.code.contains("require(1)"));
    }

    #[test]
    fn test_export_default() {
        let source = "export default App;";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(result
            .code
            .contains("Object.defineProperty(module.exports, \"__esModule\""));
        assert!(result.code.contains("module.exports"));
        assert!(result.code.contains("App"));
    }

    #[test]
    fn test_export_named() {
        let source = "export const foo = 1;";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(result.code.contains("const foo"));
        assert!(result.code.contains("module.exports"));
    }

    #[test]
    fn test_side_effect_import() {
        let source = "import './styles.css';";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(result.code.contains("require('./styles.css')"));
    }

    #[test]
    fn test_export_star_from() {
        let source = "export * from './math';";
        let mut map = HashMap::new();
        map.insert(PathBuf::from("./math.js"), 5);
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result.code.contains("require(5)"),
            "should resolve to module ID"
        );
        assert!(
            result.code.contains("Object.keys"),
            "should use Object.keys for star re-export"
        );
        assert!(
            result.code.contains("module.exports[k]"),
            "should assign to module.exports"
        );
    }

    #[test]
    fn test_export_named_alias_to_default() {
        let source = "function helper() {}\nexport { helper as default };";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result.code.contains("module.exports[\"default\"] = helper"),
            "default alias must assign module.exports.default: {}",
            result.code
        );
    }

    #[test]
    fn esm_transform_marks_exports_for_babel_interop() {
        let source = "export { default } from './createTheme';";
        let mut map = HashMap::new();
        map.insert(PathBuf::from("./createTheme.js"), 62);

        let result = transform_modules(source, &map).unwrap();

        assert!(
            result.code.starts_with(
                "Object.defineProperty(module.exports, \"__esModule\", { value: true });"
            ),
            "ESM output must be marked before Babel interop helpers see it: {}",
            result.code
        );
        assert!(result
            .code
            .contains("module.exports[\"default\"] = require(62)[\"default\"]"));
    }

    #[test]
    fn cjs_transform_does_not_mark_exports_for_babel_interop() {
        let source = "const dep = require('./utils.js');\nmodule.exports = dep;";
        let map = test_module_map();

        let result = transform_modules(source, &map).unwrap();

        assert!(
            !result.code.contains("__esModule"),
            "pure CJS modules must not be marked as ESM: {}",
            result.code
        );
        assert!(result.code.contains("const dep = require(1);"));
    }

    // GH #3222 regression: resolve_module_path used to silently swallow
    // node_modules/<pkg>/package.json read+parse errors. The bare-specifier
    // branch must now fall through to the literal `require('<spec>')` form
    // without panicking, and (for non-NotFound errors) emit a tracing::warn.
    #[test]
    fn resolve_module_path_malformed_pkg_json_falls_through() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let pkg_dir = tmp.path().join("node_modules").join("brokenpkg");
        std::fs::create_dir_all(&pkg_dir).expect("create pkg dir");
        std::fs::write(pkg_dir.join("package.json"), b"{ this is : not json")
            .expect("write malformed package.json");

        let map: HashMap<PathBuf, usize> = HashMap::new();
        let out = resolve_module_path("brokenpkg", &map, Some(tmp.path()));
        assert_eq!(
            out, "require('brokenpkg')",
            "malformed package.json must fall through to literal require, got: {out}"
        );
    }

    #[test]
    fn resolve_module_path_missing_pkg_json_falls_through_silently() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // node_modules/somepkg exists but has no package.json and no index.js
        let pkg_dir = tmp.path().join("node_modules").join("somepkg");
        std::fs::create_dir_all(&pkg_dir).expect("create pkg dir");

        let map: HashMap<PathBuf, usize> = HashMap::new();
        let out = resolve_module_path("somepkg", &map, Some(tmp.path()));
        assert_eq!(out, "require('somepkg')");
    }

    #[test]
    fn resolve_module_path_bare_subpath_matches_uncanonical_module_map_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let helper = tmp
            .path()
            .join("node_modules")
            .join("@babel")
            .join("runtime")
            .join("helpers")
            .join("interopRequireDefault.js");
        std::fs::create_dir_all(helper.parent().unwrap()).expect("create helper dir");
        std::fs::write(&helper, "module.exports = function(x) { return x; };")
            .expect("write helper");

        let importer_dir = tmp.path().join("node_modules").join("@mui").join("system");
        std::fs::create_dir_all(&importer_dir).expect("create importer dir");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(helper.clone(), 7);

        let out = resolve_module_path(
            "@babel/runtime/helpers/interopRequireDefault",
            &map,
            Some(&importer_dir),
        );
        assert_eq!(out, "require(7)");
    }

    #[test]
    fn resolve_module_path_relative_directory_package_module_entry() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let system_dir = tmp.path().join("node_modules").join("@mui").join("system");
        let create_theme_dir = system_dir.join("createTheme");
        let esm_entry = system_dir.join("esm").join("createTheme").join("index.js");
        std::fs::create_dir_all(&create_theme_dir).expect("create package dir");
        std::fs::create_dir_all(esm_entry.parent().unwrap()).expect("create esm dir");
        std::fs::write(
            create_theme_dir.join("package.json"),
            br#"{"module":"../esm/createTheme/index.js","main":"./index.js"}"#,
        )
        .expect("write package.json");
        std::fs::write(&esm_entry, "export default function createTheme() {}")
            .expect("write esm entry");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(esm_entry, 12);

        let out = resolve_module_path("./createTheme", &map, Some(&system_dir));
        assert_eq!(out, "require(12)");
    }

    #[cfg(unix)]
    #[test]
    fn resolve_module_path_unreadable_pkg_json_falls_through_without_panic() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().expect("tempdir");
        let pkg_dir = tmp.path().join("node_modules").join("lockedpkg");
        std::fs::create_dir_all(&pkg_dir).expect("create pkg dir");
        let pj = pkg_dir.join("package.json");
        std::fs::write(&pj, br#"{"main":"index.js"}"#).expect("write package.json");
        std::fs::set_permissions(&pj, std::fs::Permissions::from_mode(0o000)).expect("chmod 000");

        let map: HashMap<PathBuf, usize> = HashMap::new();
        let out = resolve_module_path("lockedpkg", &map, Some(tmp.path()));

        // Restore perms so tempdir cleanup succeeds.
        let _ = std::fs::set_permissions(&pj, std::fs::Permissions::from_mode(0o644));

        assert_eq!(
            out, "require('lockedpkg')",
            "unreadable package.json must fall through to literal require, got: {out}"
        );
    }
}
// CODEGEN-END
