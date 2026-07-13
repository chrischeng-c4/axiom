// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use std::collections::{HashMap, HashSet};
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

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
#[derive(Debug, Clone, Default)]
pub struct ModuleResolutionIndex {
    module_ids: HashMap<PathBuf, usize>,
    package_roots: HashMap<String, Vec<PathBuf>>,
    /// Nx/tsconfig path alias entries `(prefix, target)`, threaded from
    /// `ResolveOptions::alias` via `Bundler` (WI #1305) so the codegen-time
    /// resolver can consult the same alias table `resolver/mod.rs::resolve_alias`
    /// already used during graph-walk resolution. Empty for any caller that
    /// builds this index via `from_module_map` (unchanged prior behavior).
    alias_entries: Vec<(String, PathBuf)>,
    /// Explicit tsconfig `baseUrl`, threaded from the bundler's resolver so
    /// codegen resolves local bare specifiers the same way graph discovery did.
    base_url: Option<PathBuf>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
impl ModuleResolutionIndex {
    pub fn from_module_map(module_map: &HashMap<PathBuf, usize>) -> Self {
        Self::from_module_map_and_aliases(module_map, &[])
    }

    /// As [`Self::from_module_map`], but also carries the Nx/tsconfig path
    /// alias entries (WI #1305) so `resolve_module_path`'s alias-consultation
    /// branch can re-derive and look up the same candidate path
    /// `resolver/mod.rs::resolve_alias` already resolved during `build_graph`.
    pub fn from_module_map_and_aliases(
        module_map: &HashMap<PathBuf, usize>,
        alias_entries: &[(String, PathBuf)],
    ) -> Self {
        Self::from_module_map_and_aliases_and_base_url(module_map, alias_entries, None)
    }

    /// As [`Self::from_module_map_and_aliases`], also retaining an explicit
    /// tsconfig `baseUrl` for local non-relative module lookups.
    pub fn from_module_map_and_aliases_and_base_url(
        module_map: &HashMap<PathBuf, usize>,
        alias_entries: &[(String, PathBuf)],
        base_url: Option<PathBuf>,
    ) -> Self {
        let mut seen = HashSet::new();
        let mut module_ids = HashMap::new();
        let mut package_roots: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for (module_path, id) in module_map {
            module_ids.entry(module_path.clone()).or_insert(*id);
            module_ids
                .entry(normalize_path_lexical(module_path))
                .or_insert(*id);

            if let Some((package_name, root)) = module_path_package_name_and_root(module_path) {
                if seen.insert((package_name.clone(), root.clone())) {
                    package_roots.entry(package_name).or_default().push(root);
                }
            }
        }

        Self {
            module_ids,
            package_roots,
            alias_entries: alias_entries.to_vec(),
            base_url,
        }
    }
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
    transform_modules_with_dir_and_index(source, module_map, None, current_dir)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_modules_with_dir_and_index(
    source: &str,
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    current_dir: Option<&Path>,
) -> Result<TransformResult> {
    transform_modules_with_dir_index_and_tree(
        source,
        module_map,
        resolution_index,
        current_dir,
        None,
    )
}

/// As [`transform_modules_with_dir_and_index`], but reuses a tree-sitter tree
/// parsed earlier (during graph construction) when one is supplied, avoiding a
/// second parse of the same source. The caller guarantees `reuse_tree`, if
/// `Some`, is the JS-grammar parse of exactly this `source`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn transform_modules_with_dir_index_and_tree(
    source: &str,
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    current_dir: Option<&Path>,
    reuse_tree: Option<tree_sitter::Tree>,
) -> Result<TransformResult> {
    let tree = match reuse_tree {
        Some(tree) => tree,
        None => {
            let mut parser = Parser::new();
            parser.set_language(&tree_sitter_javascript::LANGUAGE.into())?;
            parser
                .parse(source, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse JavaScript"))?
        }
    };

    let root = tree.root_node();

    let has_esm_module_syntax = contains_esm_module_syntax(&root);
    let transformed = transform_node(source, &root, module_map, resolution_index, current_dir)?;
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
    resolution_index: Option<&ModuleResolutionIndex>,
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
                result.push_str(&transform_import(
                    source,
                    &child,
                    module_map,
                    resolution_index,
                    current_dir,
                )?);
                last_pos = child.end_byte();
            }
            "export_statement" => {
                result.push_str(&transform_export(
                    source,
                    &child,
                    module_map,
                    resolution_index,
                    current_dir,
                )?);
                last_pos = child.end_byte();
            }
            "call_expression" if is_dynamic_import(source, &child) => {
                result.push_str(&transform_dynamic_import(
                    source,
                    &child,
                    module_map,
                    resolution_index,
                    current_dir,
                )?);
                last_pos = child.end_byte();
            }
            "call_expression" if is_require_call(source, &child) => {
                result.push_str(&transform_require_call(
                    source,
                    &child,
                    module_map,
                    resolution_index,
                    current_dir,
                )?);
                last_pos = child.end_byte();
            }
            _ => {
                if child.child_count() > 0 {
                    result.push_str(&transform_node(
                        source,
                        &child,
                        module_map,
                        resolution_index,
                        current_dir,
                    )?);
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
    resolution_index: Option<&ModuleResolutionIndex>,
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
            let require_target =
                resolve_module_path(&path, module_map, resolution_index, current_dir);
            return Ok(format!("{};", require_target));
        }
        return Ok(String::new());
    }

    let import_clause = import_clause.unwrap();
    let source_path = source_path.ok_or_else(|| anyhow::anyhow!("Missing import source"))?;

    let require_target =
        resolve_module_path(&source_path, module_map, resolution_index, current_dir);

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
    resolution_index: Option<&ModuleResolutionIndex>,
    current_dir: Option<&Path>,
) -> Result<String> {
    let mut cursor = node.walk();

    // Check for re-export source: export { X } from "./X"
    let reexport_source = extract_export_source(source, node);

    for child in node.children(&mut cursor) {
        match child.kind() {
            "export" => continue,
            "default" => {
                if let Some(transformed) = transform_named_default_declaration_export(
                    source,
                    node,
                    module_map,
                    resolution_index,
                    current_dir,
                )? {
                    return Ok(transformed);
                }
                let value = extract_export_value(source, node)?;
                return Ok(format!("module.exports[\"default\"] = {};", value));
            }
            "*" => {
                // export * from './foo' → re-export all named exports
                if let Some(ref src_path) = reexport_source {
                    let require_target =
                        resolve_module_path(src_path, module_map, resolution_index, current_dir);
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
                let declaration =
                    transform_node(source, &child, module_map, resolution_index, current_dir)?;
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
                    let require_target =
                        resolve_module_path(src_path, module_map, resolution_index, current_dir);
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

fn transform_named_default_declaration_export(
    source: &str,
    node: &Node,
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    current_dir: Option<&Path>,
) -> Result<Option<String>> {
    let mut cursor = node.walk();
    let mut found_default = false;

    for child in node.children(&mut cursor) {
        if child.kind() == "default" {
            found_default = true;
            continue;
        }
        if !found_default || is_export_default_value_noise(&child) {
            continue;
        }
        if !matches!(child.kind(), "function_declaration" | "class_declaration") {
            return Ok(None);
        }

        let names = extract_declaration_names(&child, source)?;
        let Some(name) = names.first() else {
            return Ok(None);
        };
        let declaration =
            transform_node(source, &child, module_map, resolution_index, current_dir)?;
        return Ok(Some(format!(
            "{}; module.exports[\"default\"] = {};",
            declaration, name
        )));
    }

    Ok(None)
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
    resolution_index: Option<&ModuleResolutionIndex>,
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
                    let require_target = resolve_module_path(
                        &module_path,
                        module_map,
                        resolution_index,
                        current_dir,
                    );
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

fn lookup_module_id_for_resolution(
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    path: &Path,
) -> Option<usize> {
    let Some(index) = resolution_index else {
        return lookup_module_id(module_map, path);
    };

    if let Some(&id) = index.module_ids.get(path) {
        return Some(id);
    }

    let normalized = normalize_path_lexical(path);
    index.module_ids.get(&normalized).copied()
}

fn collect_package_entry_candidates(pkg: &serde_json::Value, out: &mut Vec<String>) {
    fn push_string(value: Option<&serde_json::Value>, out: &mut Vec<String>) {
        if let Some(s) = value.and_then(|v| v.as_str()) {
            out.push(s.to_string());
        }
    }

    fn browser_replacement_for<'a>(browser: &'a serde_json::Value, entry: &str) -> Option<&'a str> {
        let map = browser.as_object()?;
        for key in [
            entry.to_string(),
            format!("./{}", entry.trim_start_matches("./")),
        ] {
            if let Some(replacement) = map.get(&key).and_then(|v| v.as_str()) {
                return Some(replacement);
            }
        }
        None
    }

    if let Some(exports) = pkg.get("exports") {
        let root = exports.get(".").unwrap_or(exports);
        collect_export_candidate_strings(root, out);
    }

    if let Some(browser) = pkg.get("browser") {
        if let Some(s) = browser.as_str() {
            out.push(s.to_string());
        }
        for entry in [pkg.get("module"), pkg.get("main")]
            .into_iter()
            .flatten()
            .filter_map(|value| value.as_str())
        {
            if let Some(replacement) = browser_replacement_for(browser, entry) {
                out.push(replacement.to_string());
            }
        }
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

fn lookup_file_or_directory_module_id(
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    candidate: &Path,
) -> Option<usize> {
    if let Some(id) = lookup_file_module_id_with_extensions(module_map, resolution_index, candidate)
    {
        return Some(id);
    }

    if let Some(id) = lookup_directory_index_module_id(module_map, resolution_index, candidate) {
        return Some(id);
    }

    lookup_package_entry_module_id(module_map, resolution_index, candidate)
        .or_else(|| lookup_directory_index_module_id(module_map, resolution_index, candidate))
}

/// Node builtins `resolver/mod.rs::resolve_browser_builtin` generates a real
/// browser polyfill module for (gated on the `browser` export condition,
/// which `jet build`'s `ResolveOptions::for_browser_production()` always
/// sets). Mirrors `resolver/mod.rs`'s const of the same name 1:1 -- see WI
/// #1306; `transform/modules.rs` and `resolver/mod.rs` share no common lib,
/// the same intentional duplication precedent as `append_extension` (WI
/// #1304).
const NODE_BUILTINS_WITH_BROWSER_FALLBACK: &[&str] = &[
    "assert",
    "buffer",
    "child_process",
    "cluster",
    "constants",
    "crypto",
    "dgram",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "worker_threads",
    "zlib",
];

/// Strips an optional `node:` prefix and checks the result against
/// [`NODE_BUILTINS_WITH_BROWSER_FALLBACK`]. Mirrors
/// `resolver/mod.rs::node_builtin_name` 1:1 -- see WI #1306.
fn node_builtin_name(specifier: &str) -> Option<&str> {
    let name = specifier.strip_prefix("node:").unwrap_or(specifier);
    NODE_BUILTINS_WITH_BROWSER_FALLBACK
        .contains(&name)
        .then_some(name)
}

/// Appends `ext` to `base` via string concatenation rather than
/// `PathBuf::set_extension`, which replaces everything after the LAST `.`
/// in the file name. A dotted basename such as `router.config` must keep
/// its full name when probed for a candidate extension (`router.config.ts`,
/// not `router.ts`). Mirrors `resolver/mod.rs::append_extension`.
fn append_extension(base: &Path, ext: &str) -> PathBuf {
    let mut path = base.as_os_str().to_os_string();
    path.push(".");
    path.push(ext.trim_start_matches('.'));
    PathBuf::from(path)
}

fn lookup_file_module_id_with_extensions(
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    candidate: &Path,
) -> Option<usize> {
    for ext in &["", ".js", ".jsx", ".ts", ".tsx", ".json"] {
        let test = if ext.is_empty() {
            candidate.to_path_buf()
        } else {
            append_extension(candidate, ext)
        };
        if let Some(id) = lookup_module_id_for_resolution(module_map, resolution_index, &test) {
            return Some(id);
        }
    }

    None
}

fn lookup_directory_index_module_id(
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    candidate: &Path,
) -> Option<usize> {
    for index in &[
        "index.js",
        "index.jsx",
        "index.ts",
        "index.tsx",
        "index.json",
    ] {
        let index_path = candidate.join(index);
        if let Some(id) = lookup_module_id_for_resolution(module_map, resolution_index, &index_path)
        {
            return Some(id);
        }
    }

    None
}

fn lookup_package_entry_module_id(
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    candidate: &Path,
) -> Option<usize> {
    let pkg_json = candidate.join("package.json");
    if pkg_json.exists() {
        match std::fs::read_to_string(&pkg_json) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(pkg) => {
                    let mut entries = Vec::new();
                    collect_package_entry_candidates(&pkg, &mut entries);
                    for entry in entries {
                        let entry = entry.trim_start_matches("./");
                        let main_path = candidate.join(entry);
                        if let Some(id) = lookup_file_module_id_with_extensions(
                            module_map,
                            resolution_index,
                            &main_path,
                        )
                        .or_else(|| {
                            lookup_directory_index_module_id(
                                module_map,
                                resolution_index,
                                &main_path,
                            )
                        }) {
                            return Some(id);
                        };
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        target: "jet::transform::modules",
                        path = %pkg_json.display(),
                        error = %err,
                        "GH #3222 failed to parse node_modules package.json; falling through to index.js"
                    );
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                tracing::warn!(
                    target: "jet::transform::modules",
                    path = %pkg_json.display(),
                    error = %err,
                    "GH #3222 failed to read node_modules package.json; falling through to index.js"
                );
            }
        }
    }

    None
}

fn split_bare_specifier(path: &str) -> Option<(String, Option<String>)> {
    if path.starts_with('.') || path.starts_with('/') {
        return None;
    }

    let mut parts = path.split('/');
    let first = parts.next()?;
    if first.is_empty() {
        return None;
    }

    if first.starts_with('@') {
        let name = parts.next()?;
        let package_name = format!("{first}/{name}");
        let rest = parts.collect::<Vec<_>>().join("/");
        return Some((package_name, (!rest.is_empty()).then_some(rest)));
    }

    let rest = parts.collect::<Vec<_>>().join("/");
    Some((first.to_string(), (!rest.is_empty()).then_some(rest)))
}

fn path_from_components(components: &[std::path::Component<'_>], end_inclusive: usize) -> PathBuf {
    let mut path = PathBuf::new();
    for component in &components[..=end_inclusive] {
        path.push(component.as_os_str());
    }
    path
}

fn module_path_package_root(path: &Path, package_name: &str) -> Option<PathBuf> {
    let components: Vec<_> = path.components().collect();
    let package_parts: Vec<&str> = package_name.split('/').collect();

    for idx in 0..components.len() {
        let current = components[idx].as_os_str().to_string_lossy();

        if current == "node_modules" {
            if package_parts.len() == 2 {
                let scope = components.get(idx + 1)?.as_os_str().to_string_lossy();
                let name = components.get(idx + 2)?.as_os_str().to_string_lossy();
                if scope == package_parts[0] && name == package_parts[1] {
                    return Some(path_from_components(&components, idx + 2));
                }
            } else {
                let name = components.get(idx + 1)?.as_os_str().to_string_lossy();
                if name == package_name {
                    return Some(path_from_components(&components, idx + 1));
                }
            }
        }

        if current == ".jet-store" {
            if package_parts.len() == 2 {
                let scope = components.get(idx + 1)?.as_os_str().to_string_lossy();
                let versioned_name = components.get(idx + 2)?.as_os_str().to_string_lossy();
                let expected_prefix = format!("{}@", package_parts[1]);
                if scope == package_parts[0] && versioned_name.starts_with(&expected_prefix) {
                    return Some(path_from_components(&components, idx + 2));
                }
            } else {
                let versioned_name = components.get(idx + 1)?.as_os_str().to_string_lossy();
                let expected_prefix = format!("{package_name}@");
                if versioned_name.starts_with(&expected_prefix) {
                    return Some(path_from_components(&components, idx + 1));
                }
            }
        }
    }

    None
}

fn module_path_package_name_and_root(path: &Path) -> Option<(String, PathBuf)> {
    let components: Vec<_> = path.components().collect();

    for idx in 0..components.len() {
        let current = components[idx].as_os_str().to_string_lossy();

        if current == "node_modules" {
            let first = components.get(idx + 1)?.as_os_str().to_string_lossy();
            if first.starts_with('@') {
                let second = components.get(idx + 2)?.as_os_str().to_string_lossy();
                let package_name = format!("{first}/{second}");
                return Some((package_name, path_from_components(&components, idx + 2)));
            }
            return Some((
                first.to_string(),
                path_from_components(&components, idx + 1),
            ));
        }

        if current == ".jet-store" {
            let first = components.get(idx + 1)?.as_os_str().to_string_lossy();
            if first.starts_with('@') {
                let versioned_name = components.get(idx + 2)?.as_os_str().to_string_lossy();
                let package_leaf = versioned_name.rsplit_once('@')?.0;
                let package_name = format!("{first}/{package_leaf}");
                return Some((package_name, path_from_components(&components, idx + 2)));
            }
            let package_leaf = first.rsplit_once('@')?.0;
            return Some((
                package_leaf.to_string(),
                path_from_components(&components, idx + 1),
            ));
        }
    }

    None
}

fn resolve_bare_specifier_from_module_map(
    path: &str,
    module_map: &HashMap<PathBuf, usize>,
) -> Option<usize> {
    let (package_name, subpath) = split_bare_specifier(path)?;
    let mut seen = HashSet::new();
    let mut roots = Vec::new();

    for module_path in module_map.keys() {
        if let Some(root) = module_path_package_root(module_path, &package_name) {
            if seen.insert(root.clone()) {
                roots.push(root);
            }
        }
    }

    for root in roots {
        let candidate = subpath
            .as_deref()
            .map_or_else(|| root.clone(), |subpath| root.join(subpath));
        if let Some(id) = lookup_file_or_directory_module_id(module_map, None, &candidate) {
            return Some(id);
        }
    }

    None
}

fn resolve_bare_specifier_from_index(
    path: &str,
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: &ModuleResolutionIndex,
) -> Option<usize> {
    let (package_name, subpath) = split_bare_specifier(path)?;
    for root in resolution_index.package_roots.get(&package_name)? {
        let candidate = subpath
            .as_deref()
            .map_or_else(|| root.clone(), |subpath| root.join(subpath));
        if let Some(id) =
            lookup_file_or_directory_module_id(module_map, Some(resolution_index), &candidate)
        {
            return Some(id);
        }
    }
    None
}

fn jet_store_root_from_path(path: &Path) -> Option<PathBuf> {
    let components: Vec<_> = path.components().collect();
    for idx in 0..components.len() {
        if components[idx].as_os_str().to_string_lossy() == ".jet-store" {
            return Some(path_from_components(&components, idx));
        }
    }
    None
}

fn matching_jet_store_package_roots(store_root: &Path, package_name: &str) -> Vec<PathBuf> {
    let package_parts: Vec<&str> = package_name.split('/').collect();
    if package_parts.len() == 2 {
        let scope_dir = store_root.join(package_parts[0]);
        let expected_prefix = format!("{}@", package_parts[1]);
        return std::fs::read_dir(scope_dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(Result::ok))
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(&expected_prefix))
            })
            .collect();
    }

    let expected_prefix = format!("{package_name}@");
    std::fs::read_dir(store_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&expected_prefix))
        })
        .collect()
}

fn resolve_bare_specifier_from_jet_store(
    path: &str,
    module_map: &HashMap<PathBuf, usize>,
    current_dir: Option<&Path>,
) -> Option<usize> {
    let (package_name, subpath) = split_bare_specifier(path)?;
    let store_root = jet_store_root_from_path(current_dir?)?;
    for root in matching_jet_store_package_roots(&store_root, &package_name) {
        let candidate = subpath
            .as_deref()
            .map_or_else(|| root.clone(), |subpath| root.join(subpath));
        if let Some(id) = lookup_file_or_directory_module_id(module_map, None, &candidate) {
            return Some(id);
        }
    }
    None
}

fn resolve_module_path(
    path: &str,
    module_map: &HashMap<PathBuf, usize>,
    resolution_index: Option<&ModuleResolutionIndex>,
    current_dir: Option<&Path>,
) -> String {
    let path_buf = PathBuf::from(path);

    // Direct match (works for absolute paths or exact relative matches)
    if let Some(id) = lookup_module_id_for_resolution(module_map, resolution_index, &path_buf) {
        return format!("require({})", id);
    }

    // Try relative path resolution with extensions
    if path.starts_with('.') {
        // First try without current_dir (legacy behavior)
        for ext in &["", ".js", ".jsx", ".ts", ".tsx"] {
            let test_path = if ext.is_empty() {
                path_buf.clone()
            } else {
                append_extension(&path_buf, ext)
            };
            if let Some(id) =
                lookup_module_id_for_resolution(module_map, resolution_index, &test_path)
            {
                return format!("require({})", id);
            }
        }

        // Resolve relative to current module directory
        if let Some(dir) = current_dir {
            let resolved = dir.join(path);
            for ext in &["", ".js", ".jsx", ".ts", ".tsx"] {
                let test_path = if ext.is_empty() {
                    resolved.clone()
                } else {
                    append_extension(&resolved, ext)
                };
                // Try exact match
                if let Some(id) =
                    lookup_module_id_for_resolution(module_map, resolution_index, &test_path)
                {
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
                        if let Some(id) = lookup_module_id_for_resolution(
                            module_map,
                            resolution_index,
                            &entry_path,
                        ) {
                            return format!("require({})", id);
                        }
                    }
                }
            }
            // Also try index files
            for index in &["index.js", "index.ts", "index.tsx"] {
                let test_path = resolved.join(index);
                if let Some(id) =
                    lookup_module_id_for_resolution(module_map, resolution_index, &test_path)
                {
                    return format!("require({})", id);
                }
            }
        }
    }

    // Bare specifier resolution (e.g. "react", "react/jsx-runtime", "scheduler")
    if !path.starts_with('.') && !path.starts_with('/') {
        // Nx/tsconfig path aliases take precedence over baseUrl and package
        // lookup, matching `resolver/mod.rs::detect_kind`. Re-derive the
        // graph resolver's prefix-strip-and-join arithmetic so emitted code
        // points at the same module ID selected during graph discovery.
        if let Some(index) = resolution_index {
            for (prefix, target) in &index.alias_entries {
                if let Some(rest) = path.strip_prefix(prefix.as_str()) {
                    let candidate = if rest.is_empty() {
                        target.clone()
                    } else {
                        target.join(rest.trim_start_matches('/'))
                    };
                    if let Some(id) =
                        lookup_file_or_directory_module_id(module_map, resolution_index, &candidate)
                    {
                        return format!("require({})", id);
                    }
                }
            }
        }

        // TypeScript resolves an explicit baseUrl before node_modules. The
        // graph resolver already registered this same local module; consult
        // the module map here so codegen emits a numeric require rather than
        // leaving the original bare-looking source specifier in the bundle.
        if let Some(base_url) = resolution_index.and_then(|index| index.base_url.as_deref()) {
            let candidate = base_url.join(path);
            if let Some(id) =
                lookup_file_or_directory_module_id(module_map, resolution_index, &candidate)
            {
                return format!("require({})", id);
            }
        }

        if let Some(dir) = current_dir {
            let mut search_dir = Some(dir);
            while let Some(d) = search_dir {
                let nm_dir = d.join("node_modules");
                if nm_dir.is_dir() {
                    let candidate = nm_dir.join(path);
                    if let Some(id) =
                        lookup_file_or_directory_module_id(module_map, resolution_index, &candidate)
                    {
                        return format!("require({})", id);
                    }
                }
                search_dir = d.parent();
            }
        }

        if let Some(id) = resolution_index
            .and_then(|index| resolve_bare_specifier_from_index(path, module_map, index))
        {
            return format!("require({})", id);
        }

        if resolution_index.is_none() {
            if let Some(id) = resolve_bare_specifier_from_module_map(path, module_map) {
                return format!("require({})", id);
            }
        }

        if resolution_index.is_none() {
            if let Some(id) = resolve_bare_specifier_from_jet_store(path, module_map, current_dir) {
                return format!("require({})", id);
            }
        }

        // Node builtin browser polyfill fallback (WI #1306): every strategy
        // above only knows how to resolve real `node_modules/<pkg>`
        // directories, so a bare Node builtin specifier such as `crypto`
        // (direct or transitive, e.g. via a dependency's own internal
        // `require('crypto')`) falls through all of them, since no
        // `node_modules/crypto` directory exists. `resolver/mod.rs`'s
        // `resolve_browser_builtin` already generated and registered a real
        // browser polyfill module at `<dir>/node_modules/.jet/polyfill-<
        // builtin>.mjs` for this specifier during graph construction (see
        // `build_graph`) -- reuse the same node_modules ancestor walk-up to
        // probe that already-materialized path before giving up.
        if let Some(builtin) = node_builtin_name(path) {
            if let Some(dir) = current_dir {
                let mut search_dir = Some(dir);
                while let Some(d) = search_dir {
                    let nm_dir = d.join("node_modules");
                    if nm_dir.is_dir() {
                        let candidate = nm_dir.join(".jet").join(format!("polyfill-{builtin}.mjs"));
                        if let Some(id) = lookup_module_id_for_resolution(
                            module_map,
                            resolution_index,
                            &candidate,
                        ) {
                            return format!("require({})", id);
                        }
                    }
                    search_dir = d.parent();
                }
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
    resolution_index: Option<&ModuleResolutionIndex>,
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
                    let resolved = resolve_module_path(
                        &module_path,
                        module_map,
                        resolution_index,
                        current_dir,
                    );
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
        if found_default && !is_export_default_value_noise(&child) {
            if child.kind() == "expression_statement" {
                if let Some(expression) = child.named_children(&mut child.walk()).next() {
                    return Ok(source[expression.byte_range()].to_string());
                }
            }
            return Ok(source[child.byte_range()].to_string());
        }
    }

    Err(anyhow::anyhow!("Could not extract export default value"))
}

fn is_export_default_value_noise(node: &Node) -> bool {
    matches!(
        node.kind(),
        "export" | ";" | "comment" | "automatic_semicolon"
    )
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
                    collect_binding_names(&name_node, source, &mut names);
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
                            collect_binding_names(&name_node, source, &mut names);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(names)
}

fn collect_binding_names(node: &Node, source: &str, names: &mut Vec<String>) {
    match node.kind() {
        "identifier" | "shorthand_property_identifier_pattern" => {
            names.push(source[node.byte_range()].to_string());
        }
        "pair_pattern" => {
            if let Some(value) = node.child_by_field_name("value") {
                collect_binding_names(&value, source, names);
            }
        }
        "assignment_pattern" => {
            if let Some(left) = node.child_by_field_name("left") {
                collect_binding_names(&left, source, names);
            } else if let Some(first_named) = node.named_children(&mut node.walk()).next() {
                collect_binding_names(&first_named, source, names);
            }
        }
        "rest_pattern" => {
            for child in node.named_children(&mut node.walk()) {
                collect_binding_names(&child, source, names);
            }
        }
        "object_pattern" | "array_pattern" => {
            for child in node.named_children(&mut node.walk()) {
                collect_binding_names(&child, source, names);
            }
        }
        _ => {}
    }
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
    fn test_export_default_pure_comment_call_expression() {
        let source = "export default /*#__PURE__*/createContext(undefined);";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result
                .code
                .contains("module.exports[\"default\"] = createContext(undefined)"),
            "pure annotation comment must not become the default export value: {}",
            result.code
        );
        assert!(
            !result.code.contains("module.exports[\"default\"] = ;"),
            "default export RHS must not be empty: {}",
            result.code
        );
    }

    #[test]
    fn test_export_default_expression_statement_without_trailing_semicolon() {
        let source = "export default React.createContext({});";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result
                .code
                .contains("module.exports[\"default\"] = React.createContext({})"),
            "default export should preserve expression RHS: {}",
            result.code
        );
    }

    #[test]
    fn test_export_default_named_function_preserves_binding() {
        let source = "export default function useUpdate(callback) { return callback(); }\nexport function useUpdateState() { return useUpdate(() => 1); }";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result
                .code
                .contains("function useUpdate(callback) { return callback(); }"),
            "named default function declaration should remain a local binding: {}",
            result.code
        );
        assert!(
            result
                .code
                .contains("module.exports[\"default\"] = useUpdate"),
            "default export should point at the preserved binding: {}",
            result.code
        );
        assert!(
            !result
                .code
                .contains("module.exports[\"default\"] = function useUpdate"),
            "named default function must not be lowered into a function expression: {}",
            result.code
        );
    }

    #[test]
    fn test_export_default_named_class_preserves_binding() {
        let source =
            "export default class Widget {}\nexport function make() { return new Widget(); }";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result.code.contains("class Widget {}"),
            "named default class declaration should remain a local binding: {}",
            result.code
        );
        assert!(
            result.code.contains("module.exports[\"default\"] = Widget"),
            "default export should point at the preserved class binding: {}",
            result.code
        );
        assert!(
            !result
                .code
                .contains("module.exports[\"default\"] = class Widget"),
            "named default class must not be lowered into a class expression: {}",
            result.code
        );
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
    fn test_export_clause_with_comments_exports_all_names() {
        let source = r#"export {
Theme,
createTheme,
// Transformer
legacyLogicalPropertiesTransformer,
px2remTransformer,
// util
token2CSSVar,
unit,
genCalc
};"#;
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        for name in [
            "Theme",
            "createTheme",
            "legacyLogicalPropertiesTransformer",
            "px2remTransformer",
            "token2CSSVar",
            "unit",
            "genCalc",
        ] {
            assert!(
                result
                    .code
                    .contains(&format!("module.exports[\"{name}\"] = {name}")),
                "missing commented export {name}: {}",
                result.code
            );
        }
    }

    #[test]
    fn test_export_var_object_literal_preserves_declaration() {
        let source = r#"export var _experimental = {
  supportModernCSS: function supportModernCSS() {
    return supportWhere() && supportLogicProps();
  }
};"#;
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result.code.contains("var _experimental = {"),
            "exported variable declaration should be preserved: {}",
            result.code
        );
        assert!(
            result
                .code
                .contains("module.exports[\"_experimental\"] = _experimental"),
            "exported variable should assign module.exports: {}",
            result.code
        );
        assert!(
            crate::bundler::dce::js_parses_without_errors(&result.code),
            "transformed export var should parse: {}",
            result.code
        );
    }

    #[test]
    fn test_export_destructured_object_binding_alias() {
        let source = "export const { Consumer: ConfigConsumer } = ConfigContext;";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result
                .code
                .contains("const { Consumer: ConfigConsumer } = ConfigContext"),
            "declaration should be preserved: {}",
            result.code
        );
        assert!(
            result
                .code
                .contains("module.exports[\"ConfigConsumer\"] = ConfigConsumer"),
            "destructuring export should use bound local name: {}",
            result.code
        );
        assert!(
            !result.code.contains("module.exports[\"{\"]"),
            "object pattern must not be treated as an export name: {}",
            result.code
        );
    }

    #[test]
    fn test_export_destructured_object_shorthand_bindings() {
        let source =
            "export const { genStyleHooks, genComponentStyleHook, genSubStyleComponent } = utils;";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        for name in [
            "genStyleHooks",
            "genComponentStyleHook",
            "genSubStyleComponent",
        ] {
            assert!(
                result
                    .code
                    .contains(&format!("module.exports[\"{name}\"] = {name}")),
                "missing destructured shorthand export {name}: {}",
                result.code
            );
        }
    }

    #[test]
    fn test_export_destructured_array_bindings() {
        let source = "export const [first, second] = pair;";
        let map = HashMap::new();
        let result = transform_modules(source, &map).unwrap();
        assert!(
            result.code.contains("module.exports[\"first\"] = first"),
            "missing first array export: {}",
            result.code
        );
        assert!(
            result.code.contains("module.exports[\"second\"] = second"),
            "missing second array export: {}",
            result.code
        );
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
        let out = resolve_module_path("brokenpkg", &map, None, Some(tmp.path()));
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
        let out = resolve_module_path("somepkg", &map, None, Some(tmp.path()));
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
            None,
            Some(&importer_dir),
        );
        assert_eq!(out, "require(7)");
    }

    #[test]
    fn resolve_module_path_bare_directory_subpath_uses_index_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let alert_index = tmp
            .path()
            .join("node_modules")
            .join("antd")
            .join("es")
            .join("alert")
            .join("index.js");
        std::fs::create_dir_all(alert_index.parent().unwrap()).expect("create alert dir");
        std::fs::write(&alert_index, "export default function Alert() {}").expect("write alert");

        let importer_dir = tmp.path().join("src");
        std::fs::create_dir_all(&importer_dir).expect("create importer dir");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(alert_index, 14);

        let out = resolve_module_path("antd/es/alert", &map, None, Some(&importer_dir));
        assert_eq!(out, "require(14)");
    }

    #[test]
    fn resolve_module_path_bare_subpath_uses_jet_store_module_map_root() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let antd_root = tmp.path().join(".jet-store").join("antd@5.29.3");
        let alert_index = antd_root.join("es").join("alert").join("index.js");
        let importer_dir = antd_root.join("es").join("table").join("hooks");
        std::fs::create_dir_all(alert_index.parent().unwrap()).expect("create alert dir");
        std::fs::create_dir_all(&importer_dir).expect("create importer dir");
        std::fs::write(&alert_index, "export default function Alert() {}").expect("write alert");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(alert_index, 14);

        let out = resolve_module_path("antd/es/alert", &map, None, Some(&importer_dir));
        assert_eq!(out, "require(14)");
    }

    #[test]
    fn resolve_module_path_scoped_bare_package_uses_jet_store_package_entry() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let package_root = tmp
            .path()
            .join(".jet-store")
            .join("@ant-design")
            .join("cssinjs@1.24.0");
        let entry = package_root.join("es").join("index.js");
        let importer_dir = tmp.path().join(".jet-store").join("antd@5.29.3").join("es");
        std::fs::create_dir_all(entry.parent().unwrap()).expect("create cssinjs entry dir");
        std::fs::create_dir_all(&importer_dir).expect("create importer dir");
        std::fs::write(
            package_root.join("package.json"),
            br#"{"module":"./es/index","main":"./lib/index"}"#,
        )
        .expect("write package.json");
        std::fs::write(&entry, "export const unit = value => value;").expect("write entry");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(entry, 377);

        let out = resolve_module_path("@ant-design/cssinjs", &map, None, Some(&importer_dir));
        assert_eq!(out, "require(377)");
    }

    #[test]
    fn resolve_module_path_unscoped_bare_package_uses_extensionless_jet_store_entry() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let package_root = tmp.path().join(".jet-store").join("rc-motion@2.9.5");
        let entry = package_root.join("es").join("index.js");
        let importer_dir = tmp.path().join(".jet-store").join("antd@5.29.3").join("es");
        std::fs::create_dir_all(entry.parent().unwrap()).expect("create rc-motion entry dir");
        std::fs::create_dir_all(&importer_dir).expect("create importer dir");
        std::fs::write(
            package_root.join("package.json"),
            br#"{"module":"./es/index","main":"./lib/index"}"#,
        )
        .expect("write package.json");
        std::fs::write(&entry, "export default function CSSMotion() {}").expect("write entry");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(entry, 212);

        let out = resolve_module_path("rc-motion", &map, None, Some(&importer_dir));
        assert_eq!(out, "require(212)");
    }

    #[test]
    fn resolve_module_path_unscoped_bare_package_uses_browser_mapped_module_entry() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let package_root = tmp
            .path()
            .join(".jet-store")
            .join("styled-components@6.1.13");
        let entry = package_root
            .join("dist")
            .join("styled-components.browser.esm.js");
        let importer_dir = tmp.path().join("src");
        std::fs::create_dir_all(entry.parent().unwrap()).expect("create styled entry dir");
        std::fs::create_dir_all(&importer_dir).expect("create importer dir");
        std::fs::write(
            package_root.join("package.json"),
            br#"{
              "module": "./dist/styled-components.esm.js",
              "main": "dist/styled-components.cjs.js",
              "browser": {
                "./dist/styled-components.cjs.js": "./dist/styled-components.browser.cjs.js",
                "./dist/styled-components.esm.js": "./dist/styled-components.browser.esm.js"
              }
            }"#,
        )
        .expect("write package.json");
        std::fs::write(&entry, "export default {};").expect("write entry");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(entry, 812);
        let index = ModuleResolutionIndex::from_module_map(&map);

        let out = resolve_module_path("styled-components", &map, Some(&index), Some(&importer_dir));
        assert_eq!(out, "require(812)");
    }

    #[test]
    fn resolve_module_path_indexed_bare_package_does_not_scan_jet_store_on_miss() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let package_root = tmp.path().join(".jet-store").join("rc-motion@2.9.5");
        let entry = package_root.join("es").join("index.js");
        let importer_dir = tmp.path().join(".jet-store").join("antd@5.29.3").join("es");
        std::fs::create_dir_all(entry.parent().unwrap()).expect("create rc-motion entry dir");
        std::fs::create_dir_all(&importer_dir).expect("create importer dir");
        std::fs::write(
            package_root.join("package.json"),
            br#"{"module":"./es/index","main":"./lib/index"}"#,
        )
        .expect("write package.json");
        std::fs::write(&entry, "export default function CSSMotion() {}").expect("write entry");

        let mut map: HashMap<PathBuf, usize> = HashMap::new();
        map.insert(entry, 212);
        let index = ModuleResolutionIndex::default();

        let out = resolve_module_path("rc-motion", &map, Some(&index), Some(&importer_dir));
        assert_eq!(out, "require('rc-motion')");
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

        let out = resolve_module_path("./createTheme", &map, None, Some(&system_dir));
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
        let out = resolve_module_path("lockedpkg", &map, None, Some(tmp.path()));

        // Restore perms so tempdir cleanup succeeds.
        let _ = std::fs::set_permissions(&pj, std::fs::Permissions::from_mode(0o644));

        assert_eq!(
            out, "require('lockedpkg')",
            "unreadable package.json must fall through to literal require, got: {out}"
        );
    }

    // WI #1304 R4: append_extension must append the extension via string
    // concatenation, never replace text after the last '.' in the base
    // path (the bug in the old PathBuf::set_extension probe).
    #[test]
    fn append_extension_appends_without_replacing_dotted_basename() {
        let base = PathBuf::from("router.config");
        let out = append_extension(&base, "ts");
        assert_eq!(
            out,
            PathBuf::from("router.config.ts"),
            "append_extension must preserve the full dotted basename, got: {out:?}"
        );
        assert_ne!(
            out,
            PathBuf::from("router.ts"),
            "append_extension must not replace text after the last '.' in the base name"
        );
    }

    // WI #1304 R1/R2/R3: full Bundler::bundle() pipeline regressions for the
    // dotted-basename extension probe fix in resolve_module_path.
    mod bundle_dotted_basename_regressions {
        use crate::bundler::{BundleOptions, Bundler};
        use std::io::Write;

        fn write_fixture(dir: &std::path::Path, files: &[(&str, &str)]) -> std::path::PathBuf {
            for (name, contents) in files {
                let path = dir.join(name);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                let mut f = std::fs::File::create(&path).unwrap();
                f.write_all(contents.as_bytes()).unwrap();
            }
            dir.join(files[0].0)
        }

        // WI #1304 AC1: a dotted-basename extensionless relative import
        // (./router.config resolving to router.config.ts) must resolve
        // through the complete bundle pipeline instead of being left as a
        // literal unresolved require('./router.config') string.
        #[tokio::test]
        async fn bundle_resolves_dotted_basename_extensionless_relative_import() {
            let tmp = tempfile::tempdir().unwrap();
            let entry = write_fixture(
                tmp.path(),
                &[
                    (
                        "entry.ts",
                        "import { routerConfig } from './router.config';\nexport const x = routerConfig;\n",
                    ),
                    (
                        "router.config.ts",
                        "export const routerConfig = 'ROUTER_CONFIG_MARKER_1304';\n",
                    ),
                ],
            );

            let opts = BundleOptions {
                entry: entry.clone(),
                output_dir: tmp.path().join("dist"),
                ..Default::default()
            };
            let bundler = Bundler::new(opts).unwrap();
            let output = bundler
                .bundle(entry)
                .await
                .expect("dotted-basename relative import must resolve");

            assert!(
                !output.code.contains("require('./router.config')")
                    && !output.code.contains("require(\"./router.config\")"),
                "dotted-basename import must not be left as a literal unresolved require string:\n{}",
                output.code
            );
            assert!(
                output.code.contains("ROUTER_CONFIG_MARKER_1304"),
                "target module body must be present in the bundle:\n{}",
                output.code
            );
        }

        // WI #1304 AC2: a legacy-CJS-style nested relative import with a
        // dotted basename in a library-style subdirectory
        // (../../modules/es6.object.assign resolving to
        // modules/es6.object.assign.js) must resolve end to end.
        #[tokio::test]
        async fn bundle_resolves_legacy_cjs_nested_dotted_basename_relative_import() {
            let tmp = tempfile::tempdir().unwrap();
            let entry = write_fixture(
                tmp.path(),
                &[
                    (
                        "entry.js",
                        "var assign = require('./library/es-abstract/2020/entry').assign;\nexports.assign = assign;\n",
                    ),
                    (
                        "library/es-abstract/2020/entry.js",
                        "var assign = require('../../../modules/es6.object.assign');\nmodule.exports = { assign: assign };\n",
                    ),
                    (
                        "modules/es6.object.assign.js",
                        "module.exports = 'ES6_OBJECT_ASSIGN_MARKER_1304';\n",
                    ),
                ],
            );

            let opts = BundleOptions {
                entry: entry.clone(),
                output_dir: tmp.path().join("dist"),
                ..Default::default()
            };
            let bundler = Bundler::new(opts).unwrap();
            let output = bundler
                .bundle(entry)
                .await
                .expect("legacy-CJS nested dotted-basename import must resolve");

            assert!(
                !output
                    .code
                    .contains("require('../../../modules/es6.object.assign')")
                    && !output
                        .code
                        .contains("require(\"../../../modules/es6.object.assign\")"),
                "legacy-CJS nested dotted-basename import must not be left as a literal unresolved require string:\n{}",
                output.code
            );
            assert!(
                output.code.contains("ES6_OBJECT_ASSIGN_MARKER_1304"),
                "target module body must be present in the bundle:\n{}",
                output.code
            );
        }

        // WI #1304 R3: no-regression control — a plain (non-dotted)
        // extensionless relative import must continue to resolve exactly
        // as before the fix.
        #[tokio::test]
        async fn bundle_resolves_plain_extensionless_relative_import_unchanged() {
            let tmp = tempfile::tempdir().unwrap();
            let entry = write_fixture(
                tmp.path(),
                &[
                    (
                        "entry.ts",
                        "import { util } from './utils';\nexport const x = util;\n",
                    ),
                    (
                        "utils.ts",
                        "export const util = 'PLAIN_UTILS_MARKER_1304';\n",
                    ),
                ],
            );

            let opts = BundleOptions {
                entry: entry.clone(),
                output_dir: tmp.path().join("dist"),
                ..Default::default()
            };
            let bundler = Bundler::new(opts).unwrap();
            let output = bundler
                .bundle(entry)
                .await
                .expect("plain extensionless relative import must resolve");

            assert!(
                !output.code.contains("require('./utils')")
                    && !output.code.contains("require(\"./utils\")"),
                "plain extensionless import must not be left as a literal unresolved require string:\n{}",
                output.code
            );
            assert!(
                output.code.contains("PLAIN_UTILS_MARKER_1304"),
                "target module body must be present in the bundle:\n{}",
                output.code
            );
        }
    }

    // WI #1306 R4: isolated unit-level pin on node_builtin_name's exact
    // matching and 'node:' prefix-stripping semantics.
    #[test]
    fn node_builtin_name_matches_known_builtins_and_strips_node_prefix() {
        assert_eq!(node_builtin_name("crypto"), Some("crypto"));
        assert_eq!(node_builtin_name("node:crypto"), Some("crypto"));
        assert_eq!(node_builtin_name("react"), None);
    }

    fn write_node_builtin_fixture(
        dir: &std::path::Path,
        files: &[(&str, &str)],
    ) -> std::path::PathBuf {
        use std::io::Write;
        for (name, contents) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(contents.as_bytes()).unwrap();
        }
        dir.join(files[0].0)
    }

    fn node_builtin_polyfill_bundle_options(
        fixture_root: &std::path::Path,
        entry: std::path::PathBuf,
    ) -> crate::bundler::BundleOptions {
        let mut resolve_options = crate::resolver::ResolveOptions::for_browser_production();
        resolve_options.base_dirs = vec![fixture_root.to_path_buf()];
        crate::bundler::BundleOptions {
            entry,
            output_dir: fixture_root.join("dist"),
            resolve_options,
            ..Default::default()
        }
    }

    // WI #1306 R1: a DIRECT Node builtin import (import { randomBytes } from
    // 'crypto') must resolve end to end through the complete bundle pipeline
    // to the browser polyfill module resolver/mod.rs::resolve_browser_builtin
    // already generates -- no literal unresolved require('crypto') string
    // must survive in the emitted bundle.
    //
    // The fixture root is canonicalized before use: `resolve_browser_builtin`
    // resolves its polyfill path from the configured `base_dirs` while
    // `build_graph` canonicalizes the entry path, so on platforms where the
    // OS temp dir is itself a symlink (macOS `/var` -> `/private/var`) an
    // un-canonicalized fixture root would make those two paths disagree by
    // prefix even though they name the same file -- a tempdir artifact, not
    // a real-world condition (ordinary project roots are not symlinked).
    #[tokio::test]
    async fn bundle_resolves_direct_node_builtin_import_to_generated_polyfill() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let entry = write_node_builtin_fixture(
            &root,
            &[(
                "entry.ts",
                "import { randomBytes } from 'crypto';\nexport const x = randomBytes;\n",
            )],
        );

        let opts = node_builtin_polyfill_bundle_options(&root, entry.clone());
        let bundler = crate::bundler::Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("direct Node builtin import must resolve");

        assert!(
            !output.code.contains("require('crypto')") && !output.code.contains("require(\"crypto\")"),
            "direct Node builtin import must not be left as a literal unresolved require string:\n{}",
            output.code
        );
        assert!(
            output.code.contains("globalThis.crypto"),
            "generated crypto polyfill body must be reachable in the bundle:\n{}",
            output.code
        );
    }

    // WI #1306 R2: the WI's explicitly-called-out transitive case, mirroring
    // the original bug report's seedrandom-shaped repro: a fixture
    // node_modules dependency whose own source contains a bare
    // require('crypto') call must have that call rewritten to reference the
    // generated polyfill module id too -- not just the entry module.
    //
    // The fixture root is canonicalized before use -- see the comment on
    // `bundle_resolves_direct_node_builtin_import_to_generated_polyfill`.
    #[tokio::test]
    async fn bundle_resolves_transitive_node_builtin_require_via_dependency() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let entry = write_node_builtin_fixture(
            &root,
            &[
                (
                    "entry.js",
                    "var seedrandom = require('seedrandom-fixture');\nmodule.exports = seedrandom;\n",
                ),
                (
                    "node_modules/seedrandom-fixture/index.js",
                    "var crypto = require('crypto');\nmodule.exports = crypto;\n",
                ),
            ],
        );

        let opts = node_builtin_polyfill_bundle_options(&root, entry.clone());
        let bundler = crate::bundler::Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("transitive Node builtin require must resolve");

        assert!(
            !output.code.contains("require('crypto')") && !output.code.contains("require(\"crypto\")"),
            "transitive Node builtin require must not be left as a literal unresolved require string anywhere in the bundle:\n{}",
            output.code
        );
        assert!(
            output.code.contains("globalThis.crypto"),
            "generated crypto polyfill body must be reachable in the bundle:\n{}",
            output.code
        );
    }

    // WI #1306 R3: no-regression control -- an ordinary (non-builtin) bare
    // package import must continue to resolve exactly as before this fix,
    // proving the new Node-builtin branch is reached only after (and does
    // not interfere with) the pre-existing bare-specifier strategies.
    #[tokio::test]
    async fn bundle_resolves_ordinary_bare_package_import_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let entry = write_node_builtin_fixture(
            tmp.path(),
            &[
                (
                    "entry.js",
                    "var pkg = require('ordinary-pkg');\nmodule.exports = pkg;\n",
                ),
                (
                    "node_modules/ordinary-pkg/index.js",
                    "module.exports = 'ORDINARY_PKG_MARKER_1306';\n",
                ),
            ],
        );

        let opts = node_builtin_polyfill_bundle_options(tmp.path(), entry.clone());
        let bundler = crate::bundler::Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("ordinary bare package import must resolve");

        assert!(
            !output.code.contains("require('ordinary-pkg')")
                && !output.code.contains("require(\"ordinary-pkg\")"),
            "ordinary bare package import must not be left as a literal unresolved require string:\n{}",
            output.code
        );
        assert!(
            output.code.contains("ORDINARY_PKG_MARKER_1306"),
            "target package module body must be present in the bundle:\n{}",
            output.code
        );
    }

    // WI #1305 R1/R2/R3: full Bundler::bundle() pipeline regressions proving
    // an internal Nx workspace library imported via its declared tsconfig
    // path alias resolves through resolve_module_path's new alias-
    // consultation branch, driven through the real
    // `AliasResolver::load(...).to_resolve_aliases()` loading path (not a
    // hand-built alias Vec) exactly as `cli.rs::browser_production_resolve_options`
    // does.
    fn nx_alias_bundle_options(
        fixture_root: &std::path::Path,
        entry: std::path::PathBuf,
    ) -> crate::bundler::BundleOptions {
        let mut resolve_options = crate::resolver::ResolveOptions::for_browser_production();
        resolve_options.base_dirs = vec![fixture_root.to_path_buf()];
        let aliases = crate::resolver::alias::AliasResolver::load(
            fixture_root,
            &std::collections::HashMap::new(),
        );
        resolve_options.alias = aliases.to_resolve_aliases();
        resolve_options.base_url = aliases.base_url().map(Path::to_path_buf);
        crate::bundler::BundleOptions {
            entry,
            output_dir: fixture_root.join("dist"),
            resolve_options,
            ..Default::default()
        }
    }

    // WI #1305 R1/AC1/AC2: the WI's minimal repro at the full pipeline
    // level -- an entry module imports an internal Nx workspace library via
    // its declared tsconfig.base.json path alias
    // (`@operations/tech-platform-lib`). No literal unresolved alias
    // specifier may survive in the emitted bundle, and the aliased
    // library's compiled body must be present in `_mods` via a
    // `require(<id>)` reference.
    #[tokio::test]
    async fn bundle_resolves_nx_workspace_library_via_tsconfig_alias_full_pipeline() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        std::fs::write(
            root.join("tsconfig.base.json"),
            r#"{
              "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                  "@operations/tech-platform-lib": ["libs/tech-platform-lib/src/index.ts"]
                }
              }
            }"#,
        )
        .unwrap();
        let entry = write_node_builtin_fixture(
            &root,
            &[
                (
                    "entry.ts",
                    "import { platformValue } from '@operations/tech-platform-lib';\nexport const x = platformValue;\n",
                ),
                (
                    "libs/tech-platform-lib/src/index.ts",
                    "export const platformValue = 'NX_ALIAS_LIB_MARKER';\n",
                ),
            ],
        );

        let opts = nx_alias_bundle_options(&root, entry.clone());
        let bundler = crate::bundler::Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("Nx workspace library import via tsconfig path alias must resolve");

        assert!(
            !output.code.contains("require('@operations/tech-platform-lib')")
                && !output
                    .code
                    .contains("require(\"@operations/tech-platform-lib\")"),
            "aliased Nx workspace library import must not be left as a literal unresolved require string:\n{}",
            output.code
        );
        assert!(
            output.code.contains("NX_ALIAS_LIB_MARKER"),
            "aliased library's compiled module body must be present in the bundle:\n{}",
            output.code
        );
    }

    #[tokio::test]
    async fn bundle_resolves_tsconfig_base_url_before_node_modules_full_pipeline() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        std::fs::write(
            root.join("tsconfig.base.json"),
            r#"{"compilerOptions":{"baseUrl":"."}}"#,
        )
        .unwrap();
        let entry = write_node_builtin_fixture(
            &root,
            &[
                (
                    "entry.ts",
                    "import { locale } from 'third-party/firebase-ui/esm__zh_tw';\nexport const currentLocale = locale;\n",
                ),
                (
                    "third-party/firebase-ui/esm__zh_tw.ts",
                    "export const locale = 'BASE_URL_LOCAL_MARKER';\n",
                ),
                (
                    "node_modules/third-party/firebase-ui/esm__zh_tw.js",
                    "export const locale = 'NODE_MODULES_MARKER';\n",
                ),
            ],
        );

        let opts = nx_alias_bundle_options(&root, entry.clone());
        let bundler = crate::bundler::Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("tsconfig baseUrl local import must resolve through the full bundle pipeline");

        assert!(
            output.code.contains("BASE_URL_LOCAL_MARKER"),
            "baseUrl module body must be present in the bundle: {}",
            output.code
        );
        assert!(
            !output.code.contains("NODE_MODULES_MARKER"),
            "baseUrl must take precedence over the same node_modules path: {}",
            output.code
        );
        assert!(
            !output
                .code
                .contains("require('third-party/firebase-ui/esm__zh_tw')")
                && !output
                    .code
                    .contains("require(\"third-party/firebase-ui/esm__zh_tw\")"),
            "baseUrl import must be rewritten to an internal require: {}",
            output.code
        );
    }

    #[test]
    fn resolve_module_path_alias_precedes_base_url_when_both_targets_are_in_graph() {
        let tmp = tempfile::tempdir().unwrap();
        let base_url = tmp.path().to_path_buf();
        let alias_target = base_url.join("workspace-alias").join("module.ts");
        let base_url_target = base_url.join("third-party").join("module.ts");
        let mut module_map = HashMap::new();
        module_map.insert(alias_target.clone(), 11);
        module_map.insert(base_url_target, 22);
        let alias_entries = vec![("third-party/".to_string(), base_url.join("workspace-alias"))];
        let index = ModuleResolutionIndex::from_module_map_and_aliases_and_base_url(
            &module_map,
            &alias_entries,
            Some(base_url),
        );

        let resolved = resolve_module_path("third-party/module", &module_map, Some(&index), None);

        assert_eq!(
            resolved, "require(11)",
            "paths/alias must precede baseUrl exactly as graph resolution does"
        );
    }

    // WI #1305 R2/AC3: no-regression control -- with resolve_options.alias
    // populated (as it always is for `jet build --nx`), an ordinary bare
    // node_modules package import must continue to resolve exactly as
    // before this fix, proving an unmatched alias branch does not interfere
    // with the pre-existing node_modules ancestor walk-up / package-root-index
    // bare-specifier resolution strategies.
    #[tokio::test]
    async fn bundle_resolves_ordinary_bare_package_import_unaffected_by_alias_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        std::fs::write(
            root.join("tsconfig.base.json"),
            r#"{
              "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                  "@operations/tech-platform-lib": ["libs/tech-platform-lib/src/index.ts"]
                }
              }
            }"#,
        )
        .unwrap();
        let entry = write_node_builtin_fixture(
            &root,
            &[
                (
                    "entry.js",
                    "var pkg = require('ordinary-nx-pkg');\nmodule.exports = pkg;\n",
                ),
                (
                    "node_modules/ordinary-nx-pkg/index.js",
                    "module.exports = 'ORDINARY_NX_PKG_MARKER_1305';\n",
                ),
            ],
        );

        let opts = nx_alias_bundle_options(&root, entry.clone());
        let bundler = crate::bundler::Bundler::new(opts).unwrap();
        let output = bundler.bundle(entry).await.expect(
            "ordinary bare package import must resolve even when the alias table is populated",
        );

        assert!(
            !output.code.contains("require('ordinary-nx-pkg')")
                && !output.code.contains("require(\"ordinary-nx-pkg\")"),
            "ordinary bare package import must not be left as a literal unresolved require string:\n{}",
            output.code
        );
        assert!(
            output.code.contains("ORDINARY_NX_PKG_MARKER_1305"),
            "target package module body must be present in the bundle:\n{}",
            output.code
        );
    }

    // WI #1305 R3: edge case pinning the alias_entries prefix-strip-and-join
    // arithmetic's `rest.is_empty()` branch (mirrors
    // `resolver/mod.rs::resolve_alias`'s own `candidate = target.clone()`
    // branch 1:1) in the presence of a second, non-matching glob alias
    // entry in the same list -- proving the exact-key entry resolves to its
    // single target file (no further joinable subpath segment) rather than
    // being shadowed or mis-joined by the other entry's arithmetic.
    #[tokio::test]
    async fn bundle_resolves_nx_alias_exact_prefix_match_with_empty_rest() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        std::fs::write(
            root.join("tsconfig.base.json"),
            r#"{
              "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                  "@operations/tech-platform-lib": ["libs/tech-platform-lib/src/index.ts"],
                  "@operations/tech-platform-mock/*": ["libs/tech-platform-mock/src/*"]
                }
              }
            }"#,
        )
        .unwrap();
        let entry = write_node_builtin_fixture(
            &root,
            &[
                (
                    "entry.ts",
                    "import { platformValue } from '@operations/tech-platform-lib';\nexport const x = platformValue;\n",
                ),
                (
                    "libs/tech-platform-lib/src/index.ts",
                    "export const platformValue = 'NX_ALIAS_EXACT_PREFIX_MARKER';\n",
                ),
            ],
        );

        let opts = nx_alias_bundle_options(&root, entry.clone());
        let bundler = crate::bundler::Bundler::new(opts).unwrap();
        let output = bundler
            .bundle(entry)
            .await
            .expect("exact-prefix-match alias with an empty rest segment must resolve");

        assert!(
            !output.code.contains("require('@operations/tech-platform-lib')")
                && !output
                    .code
                    .contains("require(\"@operations/tech-platform-lib\")"),
            "exact-prefix alias import must not be left as a literal unresolved require string:\n{}",
            output.code
        );
        assert!(
            output.code.contains("NX_ALIAS_EXACT_PREFIX_MARKER"),
            "alias target entry file's compiled body must be present in the bundle:\n{}",
            output.code
        );
    }

    // WI #1305 R4: isolated unit-level pin (not full pipeline) on
    // ModuleResolutionIndex::from_module_map_and_aliases and the new
    // alias-consultation branch in resolve_module_path, proving prior
    // fallback behavior is preserved on a miss: a resolution_index built
    // with a non-empty alias_entries list, resolved against a bare
    // specifier that does not match any alias prefix, must fall through to
    // the pre-existing final literal require('<spec>') string exactly as
    // before this fix, and from_module_map (empty alias_entries) must
    // behave identically for the same specifier -- pinning that the new
    // field/branch is purely additive.
    #[test]
    fn resolve_module_path_alias_miss_falls_through_to_unresolved_literal() {
        let module_map: HashMap<PathBuf, usize> = HashMap::new();
        let alias_entries = vec![(
            "@operations/tech-platform-lib".to_string(),
            PathBuf::from("/workspace/libs/tech-platform-lib/src/index.ts"),
        )];
        let with_alias =
            ModuleResolutionIndex::from_module_map_and_aliases(&module_map, &alias_entries);
        let without_alias = ModuleResolutionIndex::from_module_map(&module_map);

        let specifier = "@some/unrelated-package";

        let resolved_with_alias =
            resolve_module_path(specifier, &module_map, Some(&with_alias), None);
        let resolved_without_alias =
            resolve_module_path(specifier, &module_map, Some(&without_alias), None);

        assert_eq!(
            resolved_with_alias,
            format!("require('{}')", specifier),
            "a non-matching specifier must fall through to the literal unresolved require string even when alias_entries is non-empty: {resolved_with_alias}"
        );
        assert_eq!(
            resolved_with_alias, resolved_without_alias,
            "an alias-miss must resolve identically whether or not alias_entries is populated: {resolved_with_alias} vs {resolved_without_alias}"
        );
    }
}
// CODEGEN-END
