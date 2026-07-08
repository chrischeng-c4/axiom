// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
// CODEGEN-BEGIN
//! jet::tsx_to_rust — the transpiler spike.
//!
//! Consumes TSX source, emits Rust source that targets the
//! `jet-react-wasm` API. Scope for this spike is **just
//! enough** to round-trip the Counter component from the runtime
//! crate's integration tests — i.e., the following TSX:
//!
//! ```tsx
//! interface CounterProps { start: number }
//! export function Counter({ start }: CounterProps) {
//!   const [n, setN] = useState(start);
//!   return (
//!     <button id="inc" onClick={() => setN(n + 1)}>
//!       count: {n}
//!     </button>
//!   );
//! }
//! ```
//!
//! must produce Rust that, when compiled against
//! `jet-react-wasm`, behaves identically to the hand-written
//! `counter_integration.rs`. The test that enforces this is in
//! `tests/transpile_counter.rs`.
//!
//! The transpiler intentionally **fails loud** on syntax it hasn't
//! been taught: the point of a spike is to find out what the support
//! matrix actually needs, not to ship a general TSX compiler on day
//! one. Errors carry the offending node's `start_position` so users
//! can see exactly which line fell off the subset.

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use tree_sitter::{Node, Parser};

pub mod emit;

/// Public entry: transpile a TSX source string and return the Rust
/// source as a string. No TSX source-position annotations in the
/// output — use `transpile_with_source` to opt in.
///
/// Errors: any TSX construct outside the spike's subset. Each error
/// names the construct and points at `file:line:col`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub fn transpile(tsx_source: &str) -> Result<String> {
    transpile_with_source(tsx_source, "").map(|r| r.rust_source)
}

/// Rich transpile result. `rust_source` is what gets written to the
/// scaffolded `src/lib.rs`; `position_map` is the side-car consumed
/// by `jet browser tsx` (and future debug tooling).
/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub struct TranspileResult {
    pub rust_source: String,
    pub position_map: emit::PositionMap,
    pub style_imports: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportAction {
    Ignore,
    Style(String),
    LoweredComponents(Vec<(String, String)>),
}

/// Transpile with a known TSX source filename so `// @tsx <file>:L:C`
/// annotations can be emitted inline. Pass an empty string to opt
/// out of annotations (same behaviour as `transpile`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub fn transpile_with_source(tsx_source: &str, source_file: &str) -> Result<TranspileResult> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TSX.into();
    parser
        .set_language(&language)
        .context("failed to set tree-sitter TSX language")?;
    let tree = parser
        .parse(tsx_source, None)
        .ok_or_else(|| anyhow!("tree-sitter parse failed"))?;
    let root = tree.root_node();

    let mut out = emit::Emitter::new();
    out.source_file = source_file.to_string();
    emit::prelude(&mut out);
    let mut style_imports = Vec::new();
    let mut component_aliases = HashMap::new();

    for child in root.named_children(&mut root.walk()) {
        match child.kind() {
            "comment" => {}
            "import_statement" => match handle_import_statement(child, tsx_source)? {
                ImportAction::Ignore => {}
                ImportAction::Style(path) => style_imports.push(path),
                ImportAction::LoweredComponents(pairs) => {
                    component_aliases.extend(pairs);
                }
            },
            "interface_declaration" => emit::props_interface(&mut out, child, tsx_source)?,
            "export_statement" => {
                // The export might wrap a function declaration — the
                // only exported shape the spike handles today.
                let decl = first_child_of_kind(child, "function_declaration").ok_or_else(|| {
                    reject(child, "export of non-function is outside the spike subset")
                })?;
                emit::function_component(&mut out, decl, tsx_source, &component_aliases)?;
            }
            "function_declaration" => {
                // Non-exported component — handle identically.
                emit::function_component(&mut out, child, tsx_source, &component_aliases)?;
            }
            // Readonly copy constants — `const X = "literal"` or
            // `const COPY = { key: "literal", … }`. Used by Cue's
            // i18n boundary so the WASM shell can share the regular
            // React shell's dictionary instead of inlining literals.
            // @issue #1409
            "lexical_declaration" | "variable_declaration" => {
                emit::top_level_const(&mut out, child, tsx_source)?;
            }
            "ERROR" => bail!("TSX parse error at {}", format_pos(child)),
            other => bail!(
                "top-level construct `{other}` outside the spike subset at {}",
                format_pos(child)
            ),
        }
    }

    let position_map = out.take_position_map();
    Ok(TranspileResult {
        rust_source: out.finish(),
        position_map,
        style_imports,
    })
}

/// Compatibility lowering for real-world React/MUI entries that are
/// not yet inside the strict TSX -> Rust subset.
///
/// This still emits Rust/WASM, not a JS app fallback. The generated
/// tree is a conservative Jet-native shell: MUI view components lower
/// to intrinsic/fragment nodes, event/data runtime behaviour remains
/// in WASM TODO territory, and unsupported dynamic expressions become
/// empty elements instead of pulling a JS runtime into the bundle.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub fn transpile_compat_with_source(
    tsx_source: &str,
    source_file: &str,
    root_component: &str,
) -> Result<TranspileResult> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TSX.into();
    parser
        .set_language(&language)
        .context("failed to set tree-sitter TSX language")?;
    let tree = parser
        .parse(tsx_source, None)
        .ok_or_else(|| anyhow!("tree-sitter parse failed"))?;
    let root = tree.root_node();

    let mut style_imports = Vec::new();
    let mut aliases = HashMap::new();
    let mut has_artifact_studio_api = false;

    for child in root.named_children(&mut root.walk()) {
        if child.kind() != "import_statement" {
            continue;
        }
        let source_node = first_child_of_kind(child, "string")
            .ok_or_else(|| reject(child, "import statement without module specifier"))?;
        let module = strip_quotes(node_text(source_node, tsx_source));
        if is_style_side_effect_import(child, tsx_source, &module) {
            style_imports.push(module.clone());
        }
        if (module == "./api" || module.ends_with("/api") || module.ends_with("/api.ts"))
            && imported_local_names(node_text(child, tsx_source))
                .iter()
                .any(|name| name == "fetchProjects")
        {
            has_artifact_studio_api = true;
        }
        collect_compat_import_aliases(child, tsx_source, &module, &mut aliases)?;
    }

    let mut out = emit::Emitter::new();
    out.source_file = source_file.to_string();
    emit::prelude(&mut out);
    if has_artifact_studio_api
        && (tsx_source.contains("Artifact Studio") || tsx_source.contains("Project workstream"))
    {
        emit::compat_artifact_studio_component(&mut out, root_component)?;
    } else if let Some(component) = find_function_declaration(root, tsx_source, root_component) {
        emit::compat_function_component(&mut out, component, tsx_source, root_component, &aliases)?;
    } else {
        let component_source =
            extract_function_source(tsx_source, root_component).ok_or_else(|| {
                anyhow!("root component `{root_component}` not found for compat lowering")
            })?;
        let component_tree = parser.parse(&component_source, None).ok_or_else(|| {
            anyhow!("tree-sitter parse failed for root component `{root_component}`")
        })?;
        let component = find_function_declaration(
            component_tree.root_node(),
            &component_source,
            root_component,
        )
        .ok_or_else(|| {
            anyhow!("root component `{root_component}` not found after source extraction")
        })?;
        emit::compat_function_component(
            &mut out,
            component,
            &component_source,
            root_component,
            &aliases,
        )?;
    }

    let position_map = out.take_position_map();
    Ok(TranspileResult {
        rust_source: out.finish(),
        position_map,
        style_imports,
    })
}

fn extract_function_source(source: &str, name: &str) -> Option<String> {
    let export_needle = format!("export function {name}");
    let plain_needle = format!("function {name}");
    let start = source
        .find(&export_needle)
        .or_else(|| source.find(&plain_needle))?;
    let rest = &source[start..];
    let open_rel = rest.find('{')?;
    let mut depth = 0usize;
    let mut end_rel = None;
    for (idx, ch) in rest[open_rel..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_rel = Some(open_rel + idx + ch.len_utf8());
                    break;
                }
            }
            _ => {}
        }
    }
    end_rel.map(|end| rest[..end].to_string())
}

fn handle_import_statement(node: Node, source: &str) -> Result<ImportAction> {
    let source_node = first_child_of_kind(node, "string")
        .ok_or_else(|| reject(node, "import statement without module specifier"))?;
    let module = strip_quotes(node_text(source_node, source));
    let text = node_text(node, source).trim();

    if text.starts_with("import type ") {
        return Ok(ImportAction::Ignore);
    }

    if is_style_side_effect_import(node, source, &module) {
        return Ok(ImportAction::Style(module));
    }

    if module == "react" || module == "react/jsx-runtime" {
        return Ok(ImportAction::Ignore);
    }

    if has_only_inline_type_specifiers(text) {
        return Ok(ImportAction::Ignore);
    }

    if let Some(pairs) = supported_library_import_aliases(text, &module) {
        return Ok(ImportAction::LoweredComponents(pairs));
    }

    Err(reject(
        node,
        &format!(
            "runtime import from `{module}` is not lowered by jet build --wasm yet; \
             keep WASM entries to React hooks, type-only imports, CSS side-effect imports, \
             and local code the TSX→Rust compiler can see directly"
        ),
    ))
}

fn supported_library_import_aliases(
    import_text: &str,
    module: &str,
) -> Option<Vec<(String, String)>> {
    let aliases = if module.starts_with("@mui/icons-material/") {
        imported_local_names(import_text)
            .into_iter()
            .map(|local| (local, "span".to_string()))
            .collect::<Vec<_>>()
    } else if module.starts_with("@mui/material") {
        imported_local_names(import_text)
            .into_iter()
            .map(|local| {
                let lower = mui_component_lowering(&local).to_string();
                (local, lower)
            })
            .collect::<Vec<_>>()
    } else if module == "antd" || module.starts_with("antd/") {
        imported_local_names(import_text)
            .into_iter()
            .map(|local| {
                let lower = antd_component_lowering(&local).to_string();
                (local, lower)
            })
            .collect::<Vec<_>>()
    } else {
        return None;
    };

    Some(aliases)
}

fn find_function_declaration<'a>(root: Node<'a>, source: &str, name: &str) -> Option<Node<'a>> {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.kind() == "function_declaration" {
            if let Some(ident) = first_child_of_kind(node, "identifier") {
                if node_text(ident, source) == name {
                    return Some(node);
                }
            }
        }
        for child in node.named_children(&mut node.walk()) {
            if child.kind() == "statement_block" && node.kind() == "function_declaration" {
                continue;
            }
            stack.push(child);
        }
    }
    None
}

fn collect_compat_import_aliases(
    node: Node,
    source: &str,
    module: &str,
    aliases: &mut HashMap<String, String>,
) -> Result<()> {
    if module == "react"
        || module == "react/jsx-runtime"
        || is_style_side_effect_import(node, source, module)
    {
        return Ok(());
    }

    let text = node_text(node, source).trim();
    if module.starts_with("@mui/icons-material/") {
        for local in imported_local_names(text) {
            aliases.insert(local, "span".to_string());
        }
        return Ok(());
    }

    if module.starts_with("@mui/material") {
        for local in imported_local_names(text) {
            let lower = mui_component_lowering(&local);
            aliases.insert(local, lower.to_string());
        }
        return Ok(());
    }

    // @spec .aw/tech-design/projects/jet/specs/4041.md#logic
    if module == "antd" || module.starts_with("antd/") {
        for local in imported_local_names(text) {
            let lower = antd_component_lowering(&local);
            aliases.insert(local, lower.to_string());
        }
        return Ok(());
    }

    if module.starts_with("./") || module.starts_with("../") {
        // Local runtime imports are accepted in compat mode. Calls are
        // intentionally not executed by JS; unsupported dynamic use is
        // lowered to empty/static nodes by the compat JSX emitter.
        return Ok(());
    }

    Ok(())
}

fn imported_local_names(import_text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let Some(from_pos) = import_text.find(" from ") else {
        return names;
    };
    let head = import_text
        .trim_start_matches("import ")
        .get(..from_pos.saturating_sub("import ".len()))
        .unwrap_or("")
        .trim();

    if let Some(open) = head.find('{') {
        if let Some(close) = head[open + 1..].find('}') {
            for spec in head[open + 1..open + 1 + close].split(',') {
                let spec = spec.trim();
                if spec.is_empty() || spec.starts_with("type ") {
                    continue;
                }
                let local = spec
                    .split(" as ")
                    .nth(1)
                    .unwrap_or(spec)
                    .trim()
                    .trim_start_matches("type ")
                    .trim();
                if !local.is_empty() {
                    names.push(local.to_string());
                }
            }
        }
    }

    let default_part = head.split(',').next().unwrap_or("").trim();
    if !default_part.is_empty() && !default_part.starts_with('{') && !default_part.starts_with('*')
    {
        names.push(default_part.to_string());
    }

    names.sort();
    names.dedup();
    names
}

fn mui_component_lowering(local: &str) -> &'static str {
    match local {
        "ThemeProvider" | "React.Fragment" | "Fragment" => "__fragment",
        "CssBaseline" => "__empty",
        "Button" | "ListItemButton" => "button",
        // @spec .aw/tech-design/projects/jet/specs/4072.md#logic
        "Checkbox" | "Input" | "TextField" => "input",
        "Chip" | "InputAdornment" | "Typography" => "span",
        "Divider" | "LinearProgress" | "CircularProgress" => "div",
        "Box" | "Stack" | "List" => "div",
        _ => "div",
    }
}

fn antd_component_lowering(local: &str) -> &'static str {
    match local {
        "Button" => "button",
        "Input" | "InputNumber" => "input",
        "TextArea" => "textarea",
        "Checkbox" => "input",
        "Typography" => "span",
        "Space" | "Flex" | "Form" => "div",
        _ => "div",
    }
}

fn is_style_side_effect_import(node: Node, source: &str, module: &str) -> bool {
    let has_clause = first_child_of_kind(node, "import_clause").is_some();
    if has_clause {
        return false;
    }
    let text = node_text(node, source).trim_start();
    text.starts_with("import ")
        && (module.ends_with(".css")
            || module.ends_with(".scss")
            || module.ends_with(".sass")
            || module.ends_with(".less"))
}

fn has_only_inline_type_specifiers(import_text: &str) -> bool {
    let Some(open) = import_text.find('{') else {
        return false;
    };
    let Some(close) = import_text[open + 1..].find('}') else {
        return false;
    };
    let specifiers = &import_text[open + 1..open + 1 + close];
    let mut saw_type = false;
    for spec in specifiers.split(',') {
        let spec = spec.trim();
        if spec.is_empty() {
            continue;
        }
        if !spec.starts_with("type ") {
            return false;
        }
        saw_type = true;
    }
    saw_type
}

fn strip_quotes(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(raw)
        .to_string()
}

// ── Tree-sitter helpers ─────────────────────────────────────────────────────

/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub(crate) fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub(crate) fn first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut walker = node.walk();
    for child in node.named_children(&mut walker) {
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub(crate) fn format_pos(node: Node) -> String {
    let p = node.start_position();
    format!("{}:{}", p.row + 1, p.column + 1)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-tsx-to-rust.md#schema
pub(crate) fn reject(node: Node, msg: &str) -> anyhow::Error {
    anyhow!("{msg} at {}", format_pos(node))
}
// CODEGEN-END
