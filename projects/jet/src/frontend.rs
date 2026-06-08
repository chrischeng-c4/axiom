// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! Shared frontend ingest for Jet build backends.
//!
//! Both `jet build` and `jet build --wasm` should start from the
//! same normalized frontend source set: TS/TSX entry source, HTML
//! shell, and CSS side-effect imports. Backend choice happens after
//! this point: regular build bundles to JS/CSS/HTML; wasm build
//! lowers the typed TSX subset to Rust/WASM and emits only a thin
//! browser host bridge.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone)]
pub struct FrontendSources {
    pub root_dir: PathBuf,
    pub entry: PathBuf,
    pub entry_path: PathBuf,
    pub entry_source: String,
    pub html_template: String,
    pub css_side_effect_imports: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalComponentImport {
    pub imported_name: String,
    pub specifier: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl FrontendSources {
    pub fn load(root_dir: &Path, entry: PathBuf) -> Result<Self> {
        let entry_path = root_dir.join(&entry);
        let entry_source = std::fs::read_to_string(&entry_path)
            .with_context(|| format!("reading frontend entry: {}", entry_path.display()))?;
        let html_template = load_html_template(root_dir)?;
        let css_side_effect_imports =
            extract_css_side_effect_imports(&entry_source).context("reading CSS imports")?;

        Ok(Self {
            root_dir: root_dir.to_path_buf(),
            entry,
            entry_path,
            entry_source,
            html_template,
            css_side_effect_imports,
        })
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn load_html_template(root_dir: &Path) -> Result<String> {
    let template_path = root_dir.join("index.html");
    if template_path.exists() {
        std::fs::read_to_string(&template_path)
            .with_context(|| format!("reading HTML template: {}", template_path.display()))
    } else {
        Ok(default_index_html())
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn default_index_html() -> String {
    r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Jet App</title>
  </head>
  <body>
    <div id="root"></div>
  </body>
</html>
"#
    .to_string()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn extract_css_side_effect_imports(source: &str) -> Result<Vec<String>> {
    let tree = parse_tsx_root(source)?;
    let root = tree.root_node();
    let mut imports = Vec::new();
    for child in root.named_children(&mut root.walk()) {
        if child.kind() != "import_statement" {
            continue;
        }
        let Some(source_node) = first_child_of_kind(child, "string") else {
            continue;
        };
        let specifier = strip_quotes(node_text(source_node, source));
        if !is_style_specifier(&specifier) {
            continue;
        }
        if first_child_of_kind(child, "import_clause").is_none() {
            imports.push(specifier);
        }
    }
    Ok(imports)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn extract_local_component_imports(source: &str) -> Result<Vec<LocalComponentImport>> {
    let tree = parse_tsx_root(source)?;
    let root = tree.root_node();
    let mut imports = Vec::new();
    for child in root.named_children(&mut root.walk()) {
        if child.kind() != "import_statement" {
            continue;
        }
        let Some(source_node) = first_child_of_kind(child, "string") else {
            continue;
        };
        let specifier = strip_quotes(node_text(source_node, source));
        if !(specifier.starts_with("./") || specifier.starts_with("../")) {
            continue;
        }
        if is_style_specifier(&specifier) {
            continue;
        }

        let text = node_text(child, source).trim();
        if let Some(default_name) = parse_default_import_name(text) {
            imports.push(LocalComponentImport {
                imported_name: default_name,
                specifier: specifier.clone(),
            });
        }
        for named in parse_named_import_names(text) {
            imports.push(LocalComponentImport {
                imported_name: named,
                specifier: specifier.clone(),
            });
        }
    }
    Ok(imports)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn resolve_local_import_path(importer_path: &Path, specifier: &str) -> Result<PathBuf> {
    let importer_dir = importer_path.parent().unwrap_or_else(|| Path::new("."));
    let base = importer_dir.join(specifier);
    if base.is_file() {
        return Ok(base);
    }

    for ext in ["tsx", "ts", "jsx", "js"] {
        let candidate = base.with_extension(ext);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    if base.is_dir() {
        for index in ["index.tsx", "index.ts", "index.jsx", "index.js"] {
            let candidate = base.join(index);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    anyhow::bail!(
        "local import `{specifier}` from {} could not be resolved",
        importer_path.display()
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn contains_function_component(source: &str, component_name: &str) -> Result<bool> {
    let tree = parse_tsx_root(source)?;
    let root = tree.root_node();
    for child in root.named_children(&mut root.walk()) {
        match child.kind() {
            "function_declaration" => {
                if function_name(child, source).as_deref() == Some(component_name) {
                    return Ok(true);
                }
            }
            "export_statement" => {
                if let Some(decl) = first_child_of_kind(child, "function_declaration") {
                    if function_name(decl, source).as_deref() == Some(component_name) {
                        return Ok(true);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

fn parse_tsx_root(source: &str) -> Result<tree_sitter::Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TSX.into();
    parser
        .set_language(&language)
        .context("failed to set tree-sitter TSX language")?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("tree-sitter parse failed"))?;
    Ok(tree)
}

fn function_name(node: Node, source: &str) -> Option<String> {
    first_child_of_kind(node, "identifier").map(|name| node_text(name, source).to_string())
}

fn parse_default_import_name(import_text: &str) -> Option<String> {
    let default_re = regex::Regex::new(r#"(?is)^import\s+([A-Za-z_$][A-Za-z0-9_$]*)\s+from\b"#)
        .expect("valid default import regex");
    default_re
        .captures(import_text)
        .and_then(|captures| captures.get(1))
        .map(|name| name.as_str().to_string())
}

fn parse_named_import_names(import_text: &str) -> Vec<String> {
    let Some(open) = import_text.find('{') else {
        return Vec::new();
    };
    let Some(close) = import_text[open + 1..].find('}') else {
        return Vec::new();
    };
    let specifiers = &import_text[open + 1..open + 1 + close];
    let mut names = Vec::new();
    for spec in specifiers.split(',') {
        let spec = spec.trim();
        if spec.is_empty() {
            continue;
        }
        if spec.starts_with("type ") {
            continue;
        }
        let name = if let Some((_, alias)) = spec.split_once(" as ") {
            alias.trim()
        } else {
            spec.split_whitespace().next().unwrap_or_default()
        };
        if !name.is_empty() {
            names.push(name.to_string());
        }
    }
    names
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn render_js_index_html(
    template: &str,
    entry: &Path,
    js_filename: &str,
    css_filenames: &[String],
) -> String {
    let entry = entry.to_string_lossy().replace('\\', "/");
    let script_tag = format!(r#"<script type="module" src="./{}"></script>"#, js_filename);
    let (html, script_replaced) = replace_module_entry_script(template, &entry, &script_tag);
    let html = if script_replaced {
        html
    } else {
        inject_before_body_end(&html, &format!("    {}\n", script_tag))
    };

    inject_stylesheet_links(&html, css_filenames)
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn render_wasm_index_html(
    template: &str,
    title: &str,
    stylesheet: Option<&str>,
    dom_mount: bool,
) -> String {
    let mut html = if dom_mount {
        ensure_dom_mount(template)
    } else {
        ensure_canvas_mount(template)
    };
    html = replace_first_module_script_or_inject(
        &html,
        r#"<script type="module" src="./boot.js"></script>"#,
    );

    let mut head_injections = String::new();
    head_injections.push_str(&format!("  <title>{title}</title>\n"));
    if let Some(href) = stylesheet {
        head_injections.push_str(&format!(r#"  <link rel="stylesheet" href="./{href}" />"#));
        head_injections.push('\n');
    }
    if dom_mount {
        head_injections.push_str(
            r#"  <style>
    html, body, #jet-root {
      margin: 0;
      padding: 0;
      min-height: 100%;
      background: #fafafa;
      font-family: -apple-system, Segoe UI, system-ui, sans-serif;
    }
  </style>
"#,
        );
    } else {
        head_injections.push_str(
            r#"  <style>
    html, body {
      margin: 0;
      padding: 0;
      height: 100%;
      background: #fafafa;
      font-family: -apple-system, Segoe UI, system-ui, sans-serif;
    }
    #jet-canvas {
      display: block;
      width: 100vw;
      height: 100vh;
    }
  </style>
"#,
        );
    }
    html = remove_title_tags(&html);
    inject_before_head_end(&html, &head_injections)
}

fn ensure_canvas_mount(html: &str) -> String {
    if html.contains("id=\"jet-canvas\"") || html.contains("id='jet-canvas'") {
        return html.to_string();
    }

    let root_re =
        regex::Regex::new(r#"(?is)<div\b([^>]*)\bid\s*=\s*["']root["']([^>]*)>\s*</div\s*>"#)
            .expect("valid root div regex");
    if root_re.is_match(html) {
        return root_re
            .replace(html, r#"<canvas id="jet-canvas"></canvas>"#)
            .to_string();
    }

    inject_before_body_end(
        html,
        r#"  <canvas id="jet-canvas"></canvas>
"#,
    )
}

fn ensure_dom_mount(html: &str) -> String {
    if html.contains("id=\"jet-root\"") || html.contains("id='jet-root'") {
        return html.to_string();
    }

    let root_re =
        regex::Regex::new(r#"(?is)<div\b([^>]*)\bid\s*=\s*["']root["']([^>]*)>\s*</div\s*>"#)
            .expect("valid root div regex");
    if root_re.is_match(html) {
        return root_re
            .replace(html, r#"<div id="jet-root"></div>"#)
            .to_string();
    }

    inject_before_body_end(
        html,
        r#"  <div id="jet-root"></div>
"#,
    )
}

fn replace_first_module_script_or_inject(html: &str, script_tag: &str) -> String {
    let script_re =
        regex::Regex::new(r#"(?is)<script\b[^>]*>\s*</script\s*>"#).expect("valid script regex");
    let module_re =
        regex::Regex::new(r#"(?is)\btype\s*=\s*["']module["']"#).expect("valid type regex");

    for script_match in script_re.find_iter(html) {
        let tag = script_match.as_str();
        if !module_re.is_match(tag) {
            continue;
        }
        let mut out = String::with_capacity(html.len() + script_tag.len());
        out.push_str(&html[..script_match.start()]);
        out.push_str(script_tag);
        out.push_str(&html[script_match.end()..]);
        return out;
    }

    inject_before_body_end(html, &format!("    {script_tag}\n"))
}

fn replace_module_entry_script(html: &str, entry: &str, script_tag: &str) -> (String, bool) {
    let script_re =
        regex::Regex::new(r#"(?is)<script\b[^>]*>\s*</script\s*>"#).expect("valid script regex");
    let src_re =
        regex::Regex::new(r#"(?is)\bsrc\s*=\s*["']([^"']+)["']"#).expect("valid src regex");
    let module_re =
        regex::Regex::new(r#"(?is)\btype\s*=\s*["']module["']"#).expect("valid type regex");

    let mut first_module_script = None;
    for script_match in script_re.find_iter(html) {
        let tag = script_match.as_str();
        if !module_re.is_match(tag) {
            continue;
        }
        first_module_script.get_or_insert(script_match);
        let Some(src_match) = src_re.captures(tag).and_then(|caps| caps.get(1)) else {
            continue;
        };
        if module_src_matches_entry(src_match.as_str(), entry) {
            let mut out = String::with_capacity(html.len() + script_tag.len());
            out.push_str(&html[..script_match.start()]);
            out.push_str(script_tag);
            out.push_str(&html[script_match.end()..]);
            return (out, true);
        }
    }

    if let Some(script_match) = first_module_script {
        let mut out = String::with_capacity(html.len() + script_tag.len());
        out.push_str(&html[..script_match.start()]);
        out.push_str(script_tag);
        out.push_str(&html[script_match.end()..]);
        return (out, true);
    }

    (html.to_string(), false)
}

fn module_src_matches_entry(src: &str, entry: &str) -> bool {
    let normalized_src = src.trim_start_matches("./").trim_start_matches('/');
    normalized_src == entry || normalized_src.ends_with(&format!("/{entry}"))
}

fn inject_stylesheet_links(html: &str, css_filenames: &[String]) -> String {
    if css_filenames.is_empty() {
        return html.to_string();
    }

    let links = css_filenames
        .iter()
        .map(|filename| format!(r#"    <link rel="stylesheet" href="./{filename}" />"#))
        .collect::<Vec<_>>()
        .join("\n");
    inject_before_head_end(html, &format!("{links}\n"))
}

fn inject_before_head_end(html: &str, snippet: &str) -> String {
    let lower = html.to_lowercase();
    if let Some(pos) = lower.find("</head>") {
        let mut out = String::with_capacity(html.len() + snippet.len());
        out.push_str(&html[..pos]);
        out.push_str(snippet);
        out.push_str(&html[pos..]);
        out
    } else {
        format!("{snippet}{html}")
    }
}

fn inject_before_body_end(html: &str, snippet: &str) -> String {
    let lower = html.to_lowercase();
    if let Some(pos) = lower.find("</body>") {
        let mut out = String::with_capacity(html.len() + snippet.len());
        out.push_str(&html[..pos]);
        out.push_str(snippet);
        out.push_str(&html[pos..]);
        out
    } else {
        format!("{html}\n{snippet}")
    }
}

fn remove_title_tags(html: &str) -> String {
    let title_re =
        regex::Regex::new(r#"(?is)\s*<title\b[^>]*>.*?</title\s*>\s*"#).expect("valid title regex");
    title_re.replace_all(html, "\n").to_string()
}

fn is_style_specifier(specifier: &str) -> bool {
    specifier.ends_with(".css")
        || specifier.ends_with(".scss")
        || specifier.ends_with(".sass")
        || specifier.ends_with(".less")
}

fn first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut walker = node.walk();
    for child in node.named_children(&mut walker) {
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

fn node_text<'a>(node: Node, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

fn strip_quotes(raw: &str) -> String {
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(raw)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_css_side_effect_imports() {
        let imports = extract_css_side_effect_imports(
            r#"
import React from 'react';
import './styles.css';
import styles from './module.css';
import '../theme.scss';
"#,
        )
        .unwrap();

        assert_eq!(imports, vec!["./styles.css", "../theme.scss"]);
    }

    #[test]
    fn extracts_local_component_imports() {
        let imports = extract_local_component_imports(
            r#"
import React from 'react';
import AppDefault from './DefaultApp';
import { App, Shell as RenamedShell, type Props } from './App.tsx';
import './styles.css';
"#,
        )
        .unwrap();

        assert_eq!(
            imports,
            vec![
                LocalComponentImport {
                    imported_name: "AppDefault".to_string(),
                    specifier: "./DefaultApp".to_string(),
                },
                LocalComponentImport {
                    imported_name: "App".to_string(),
                    specifier: "./App.tsx".to_string(),
                },
                LocalComponentImport {
                    imported_name: "RenamedShell".to_string(),
                    specifier: "./App.tsx".to_string(),
                },
            ]
        );
    }

    #[test]
    fn detects_exported_function_component() {
        assert!(contains_function_component(
            "export function App({}: AppProps) { return <div /> }",
            "App"
        )
        .unwrap());
        assert!(
            !contains_function_component("function Other() { return <div /> }", "App").unwrap()
        );
    }

    #[test]
    fn js_index_rewrites_entry_script() {
        let template = r#"<!doctype html>
<html>
  <head></head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>"#;

        let html = render_js_index_html(template, Path::new("src/main.tsx"), "main.1234.js", &[]);

        assert!(html.contains(r#"<script type="module" src="./main.1234.js"></script>"#));
        assert!(!html.contains("/src/main.tsx"));
    }

    #[test]
    fn wasm_index_reuses_html_shell_and_replaces_root_mount() {
        let template = r#"<!doctype html>
<html lang="zh-Hant">
  <head>
    <meta charset="UTF-8" />
    <title>Cue Artifact Studio</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>"#;

        let html = render_wasm_index_html(template, "App", Some("style.css"), false);

        assert!(html.contains(r#"<html lang="zh-Hant">"#));
        assert!(html.contains(r#"<canvas id="jet-canvas"></canvas>"#));
        assert!(html.contains(r#"<script type="module" src="./boot.js"></script>"#));
        assert!(html.contains(r#"<link rel="stylesheet" href="./style.css" />"#));
        assert!(html.contains("<title>App</title>"));
        assert!(!html.contains("/src/main.tsx"));
        assert!(!html.contains(r#"id="root""#));
    }
}
// CODEGEN-END
