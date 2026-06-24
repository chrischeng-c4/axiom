// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
/// Import/Export detection using Tree-sitter.
///
/// Also provides `apply_alias` — a lightweight pre-processor that substitutes
/// module path aliases (e.g. `@/components/Foo` → `./src/components/Foo`)
/// before the specifier is handed to the Node.js resolver.  This mirrors
/// what Vite does: alias resolution happens in the module graph construction
/// step, before any `node_modules` lookup.
use anyhow::Result;
use tree_sitter::{Node, Parser};

/// Import/export information extracted from a module
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct ModuleImports {
    pub static_imports: Vec<ImportDeclaration>,
    pub dynamic_imports: Vec<String>,
    pub exports: Vec<ExportDeclaration>,
}

/// Static import declaration
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct ImportDeclaration {
    pub source: String,
    pub kind: ImportKind,
}

/// Kind of import
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Default,
    Named,
    Namespace,
    SideEffect,
}

/// Export declaration
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct ExportDeclaration {
    pub kind: ExportKind,
    pub source: Option<String>,
}

/// Kind of export
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportKind {
    Named,
    Default,
    All,
}

/// Extract imports from JavaScript/TypeScript source code
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn extract_imports(source: &str, is_typescript: bool) -> Result<ModuleImports> {
    extract_imports_with_tree(source, is_typescript).map(|(imports, _tree)| imports)
}

/// Like [`extract_imports`] but also hands back the parsed tree-sitter tree.
///
/// jet otherwise tree-sitter-parses every module twice — once here during
/// graph construction to discover imports, and again in the module transform
/// for codegen. For a plain-JS module (`.js`/`.cjs`/`.mjs`) whose source the
/// transform does not rewrite first, this tree (parsed with the JS grammar) is
/// byte-for-byte the same parse the transform would redo, so the caller can
/// stash it and skip the second parse. TS/TSX/JSX trees are NOT reusable: their
/// source is rewritten before the module transform parses it.
pub fn extract_imports_with_tree(
    source: &str,
    is_typescript: bool,
) -> Result<(ModuleImports, tree_sitter::Tree)> {
    let mut parser = Parser::new();

    let language = if is_typescript {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    } else {
        tree_sitter_javascript::LANGUAGE.into()
    };

    parser.set_language(&language)?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse source"))?;

    let root = tree.root_node();

    let mut imports = ModuleImports {
        static_imports: Vec::new(),
        dynamic_imports: Vec::new(),
        exports: Vec::new(),
    };

    extract_from_node(source, &root, &mut imports);

    Ok((imports, tree))
}

/// Recursively extract imports/exports from AST node
fn extract_from_node(source: &str, node: &Node, imports: &mut ModuleImports) {
    match node.kind() {
        "import_statement" => {
            if let Some(import_decl) = parse_import_statement(source, node) {
                imports.static_imports.push(import_decl);
            }
        }

        "call_expression" => {
            if is_dynamic_import(node) {
                if let Some(specifier) = extract_dynamic_import(source, node) {
                    imports.dynamic_imports.push(specifier);
                }
            } else if is_require_call(source, node) {
                if let Some(specifier) = extract_require_specifier(source, node) {
                    imports.static_imports.push(ImportDeclaration {
                        source: specifier,
                        kind: ImportKind::Default,
                    });
                }
            }
        }

        "export_statement" => {
            if let Some(export_decl) = parse_export_statement(source, node) {
                // Re-exports with a source need to be tracked as static imports too
                if let Some(ref src) = export_decl.source {
                    imports.static_imports.push(ImportDeclaration {
                        source: src.clone(),
                        kind: ImportKind::Named,
                    });
                }
                imports.exports.push(export_decl);
            }
        }

        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_from_node(source, &child, imports);
    }
}

fn parse_import_statement(source: &str, node: &Node) -> Option<ImportDeclaration> {
    let source_node = find_child_by_kind(node, "string")?;
    let source_text = node_text(source, &source_node);
    let import_source = strip_quotes(&source_text);

    let kind = determine_import_kind(node);

    Some(ImportDeclaration {
        source: import_source,
        kind,
    })
}

fn determine_import_kind(node: &Node) -> ImportKind {
    if let Some(import_clause) = find_child_by_kind(node, "import_clause") {
        if find_child_by_kind(&import_clause, "identifier").is_some() {
            return ImportKind::Default;
        }
        if find_child_by_kind(&import_clause, "namespace_import").is_some() {
            return ImportKind::Namespace;
        }
        return ImportKind::Named;
    }
    ImportKind::SideEffect
}

fn is_dynamic_import(node: &Node) -> bool {
    if let Some(function) = find_child_by_kind(node, "import") {
        return function.kind() == "import";
    }
    false
}

fn extract_dynamic_import(source: &str, node: &Node) -> Option<String> {
    let args = find_child_by_kind(node, "arguments")?;
    let string_node = find_child_by_kind(&args, "string")?;
    let source_text = node_text(source, &string_node);
    Some(strip_quotes(&source_text))
}

fn is_require_call(source: &str, node: &Node) -> bool {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    if let Some(function) = children.first() {
        if function.kind() == "identifier" {
            let function_name = node_text(source, function);
            return function_name == "require";
        }
    }
    false
}

fn extract_require_specifier(source: &str, node: &Node) -> Option<String> {
    let args = find_child_by_kind(node, "arguments")?;

    let mut cursor = args.walk();
    let children: Vec<_> = args.children(&mut cursor).collect();

    for child in children {
        if child.kind() == "string" {
            let source_text = node_text(source, &child);
            let specifier = strip_quotes(&source_text);

            if specifier.contains(".development.") || specifier.contains("-development.") {
                tracing::debug!("Skipping development build: {}", specifier);
                return None;
            }

            return Some(specifier);
        }
    }

    None
}

fn parse_export_statement(source: &str, node: &Node) -> Option<ExportDeclaration> {
    // Check for star re-export: export * from "./foo"
    if find_child_by_kind(node, "*").is_some() {
        let source_node = find_child_by_kind(node, "string");
        let source_val = source_node.map(|n| strip_quotes(&node_text(source, &n)));
        return Some(ExportDeclaration {
            kind: ExportKind::All,
            source: source_val,
        });
    }

    // Named re-export: export { X } from "./X" or local export { X }
    if find_child_by_kind(node, "export_clause").is_some() {
        let source_node = find_child_by_kind(node, "string");
        let source_val = source_node.map(|n| strip_quotes(&node_text(source, &n)));
        return Some(ExportDeclaration {
            kind: ExportKind::Named,
            source: source_val,
        });
    }

    if node_text(source, node).contains("export default") {
        return Some(ExportDeclaration {
            kind: ExportKind::Default,
            source: None,
        });
    }

    Some(ExportDeclaration {
        kind: ExportKind::Named,
        source: None,
    })
}

fn find_child_by_kind<'a>(node: &'a Node, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    children.into_iter().find(|child| child.kind() == kind)
}

fn node_text(source: &str, node: &Node) -> String {
    source[node.byte_range()].to_string()
}

fn strip_quotes(s: &str) -> String {
    s.trim_matches(|c| c == '"' || c == '\'' || c == '`')
        .to_string()
}

// ─── Alias resolution helper ─────────────────────────────────────────────────

/// Apply module path alias mappings to a specifier string.
///
/// Called during module graph construction (before the Node.js resolver) so
/// that alias-based imports like `"@/components/Foo"` are normalised to
/// `"./src/components/Foo"` before any `node_modules` lookup.
///
/// `aliases` is a slice of `(prefix, replacement_path_str)` pairs sorted by
/// descending prefix length so longer prefixes win.  For example:
///
/// ```text
/// [("@/", "./src/")]
/// ```
///
/// Given specifier `"@/components/Foo"`, returns `"./src/components/Foo"`.
/// If no prefix matches the specifier is returned unchanged.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn apply_alias(specifier: &str, aliases: &[(String, String)]) -> String {
    for (prefix, replacement) in aliases {
        if specifier.starts_with(prefix.as_str()) {
            let rest = &specifier[prefix.len()..];
            return format!("{}{}", replacement, rest);
        }
    }
    specifier.to_string()
}

// ─── SVGR routing (import `.svg` as a React component) ────────────────────────

/// True when `specifier` points at a `.svg` file (case-insensitive, query
/// strings like `?url` / `?react` stripped).
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_svg_specifier(specifier: &str) -> bool {
    let path = specifier.split(['?', '#']).next().unwrap_or(specifier);
    path.to_ascii_lowercase().ends_with(".svg")
}

// ─── SCSS / Sass routing (compile `.scss`/`.sass` imports to CSS) ─────────────

/// True when `specifier` points at a `.scss`/`.sass` Sass source
/// (case-insensitive, query strings stripped). These imports must be routed
/// through the grass SCSS compile step ([`crate::css::scss`]) before the
/// lightningcss pipeline, rather than read as plain CSS.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn is_scss_specifier(specifier: &str) -> bool {
    let path = specifier.split(['?', '#']).next().unwrap_or(specifier);
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".scss") || lower.ends_with(".sass")
}

/// Classify a style import specifier into the route the build must take.
///
/// - `.scss`/`.sass` → [`StyleImportRoute::Sass`] (compile via grass first).
/// - any other style specifier (`.css`, `.less`, …) → [`StyleImportRoute::PlainCss`].
///
/// Routing strictly by extension keeps plain `.css` on the existing path and
/// only diverts the Sass family through the compile step.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn classify_style_import(specifier: &str) -> StyleImportRoute {
    if is_scss_specifier(specifier) {
        StyleImportRoute::Sass
    } else {
        StyleImportRoute::PlainCss
    }
}

/// How a style import is fed into the CSS pipeline.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleImportRoute {
    /// `.scss`/`.sass` — compile to CSS via grass before lightningcss.
    Sass,
    /// `.css` (and other already-CSS forms) — pass straight through.
    PlainCss,
}

/// Decide whether a `.svg` import should be routed through SVGR (emit a React
/// component module) instead of the default asset-URL behavior.
///
/// This mirrors `vite-plugin-svgr`'s routing, which is driven by two things:
///
/// 1. **Explicit query suffix** — `?url` forces the asset-URL path even when
///    SVGR is enabled; `?react` (or `?component`) forces the component path
///    even when SVGR is disabled globally.
/// 2. **The import shape** — with `vite-plugin-svgr`'s `{ exportType: 'named'
///    }` (what `fe-shared` uses) the SVG is a component only when imported via
///    the named `ReactComponent` binding: `import { ReactComponent as Icon }
///    from './icon.svg'`. A bare `import url from './icon.svg'` stays an asset
///    URL. With `exportType: 'default'`, the default import is the component.
///
/// `svgr_enabled` is the global toggle ([`crate::asset::SvgrConfig::enabled`]);
/// `export_type` is the configured [`crate::asset::SvgrExportType`].
///
/// Returns `true` to route through `transform_svg_to_component`, `false` to
/// keep the existing asset-URL emission. Non-`.svg` specifiers always return
/// `false`.
///
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn should_route_svg_as_component(
    specifier: &str,
    import_kind: &ImportKind,
    svgr_enabled: bool,
    export_type: crate::asset::SvgrExportType,
) -> bool {
    if !is_svg_specifier(specifier) {
        return false;
    }

    // 1. Explicit query suffix wins over the global toggle.
    let query = specifier.split('?').nth(1).unwrap_or("");
    if query.contains("url") {
        // `import logo from './logo.svg?url'` — always an asset URL.
        return false;
    }
    if query.contains("react") || query.contains("component") {
        // `import Logo from './logo.svg?react'` — always a component.
        return true;
    }

    if !svgr_enabled {
        return false;
    }

    // 2. Import-shape gate, matching the configured export type.
    use crate::asset::SvgrExportType;
    match export_type {
        // Named (`{ exportType: 'named' }`, fe-shared default): only the named
        // `ReactComponent` binding is the component. A default/namespace/
        // side-effect import keeps the asset-URL behavior.
        SvgrExportType::Named => matches!(import_kind, ImportKind::Named),
        // Default: the default import is the component.
        SvgrExportType::Default => matches!(import_kind, ImportKind::Default),
        // Both: either a named or default import resolves to the component.
        SvgrExportType::Both => {
            matches!(import_kind, ImportKind::Named | ImportKind::Default)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_static_imports() {
        let source = r#"
            import React from 'react';
            import { useState } from 'react';
            import * as utils from './utils';
            import './styles.css';
        "#;

        let imports = extract_imports(source, false).unwrap();

        assert_eq!(imports.static_imports.len(), 4);
        assert_eq!(imports.static_imports[0].source, "react");
        assert_eq!(imports.static_imports[0].kind, ImportKind::Default);
        assert_eq!(imports.static_imports[1].source, "react");
        assert_eq!(imports.static_imports[1].kind, ImportKind::Named);
        assert_eq!(imports.static_imports[2].source, "./utils");
        assert_eq!(imports.static_imports[2].kind, ImportKind::Namespace);
        assert_eq!(imports.static_imports[3].source, "./styles.css");
        assert_eq!(imports.static_imports[3].kind, ImportKind::SideEffect);
    }

    #[test]
    fn test_extract_dynamic_imports() {
        let source = r#"
            const module = import('./dynamic-module');
            async function load() {
                const mod = await import('./lazy-module');
            }
        "#;

        let imports = extract_imports(source, false).unwrap();

        assert_eq!(imports.dynamic_imports.len(), 2);
        assert_eq!(imports.dynamic_imports[0], "./dynamic-module");
        assert_eq!(imports.dynamic_imports[1], "./lazy-module");
    }

    #[test]
    fn test_extract_typescript_imports() {
        let source = r#"
            import type { User } from './types';
            import React from 'react';
        "#;

        let imports = extract_imports(source, true).unwrap();

        assert!(imports.static_imports.len() >= 1);
        assert!(imports.static_imports.iter().any(|i| i.source == "react"));
    }

    #[test]
    fn test_extract_exports() {
        let source = r#"
            export const foo = 1;
            export default function bar() {}
            export * from './other';
        "#;

        let imports = extract_imports(source, false).unwrap();

        assert_eq!(imports.exports.len(), 3);
    }

    // ─── Alias integration tests ──────────────────────────────────────────────

    /// REQ-JET-05/REQ-JET-07: apply_alias correctly maps aliased specifiers to
    /// their target paths — the same function used in both dev (JIT) and prod
    /// (bundler) module graph construction.
    #[test]
    fn alias_works_in_prod_build() {
        let aliases = vec![("@/".to_string(), "./src/".to_string())];

        // Aliased import resolves correctly
        assert_eq!(
            apply_alias("@/components/Foo", &aliases),
            "./src/components/Foo"
        );
        assert_eq!(
            apply_alias("@/utils/helpers", &aliases),
            "./src/utils/helpers"
        );

        // Non-aliased specifiers are returned unchanged
        assert_eq!(apply_alias("react", &aliases), "react");
        assert_eq!(apply_alias("./local-module", &aliases), "./local-module");
        assert_eq!(
            apply_alias("../parent-module", &aliases),
            "../parent-module"
        );
    }

    /// REQ-JET-06: longest alias prefix wins when multiple entries are defined.
    #[test]
    fn alias_longest_prefix_wins() {
        let aliases = vec![
            // Sorted longest-first (as AliasResolver produces)
            ("@/components/".to_string(), "./src/ui/".to_string()),
            ("@/".to_string(), "./src/".to_string()),
        ];

        // More specific prefix wins
        assert_eq!(
            apply_alias("@/components/Button", &aliases),
            "./src/ui/Button"
        );

        // Less specific prefix used when no longer match
        assert_eq!(
            apply_alias("@/hooks/useData", &aliases),
            "./src/hooks/useData"
        );
    }

    /// REQ-JET-07: apply_alias is deterministic across calls (prod == dev).
    #[test]
    fn alias_resolution_is_deterministic() {
        let aliases = vec![("@/".to_string(), "./src/".to_string())];
        let specifier = "@/pages/Home";

        // Calling apply_alias multiple times always yields the same result
        let result_1 = apply_alias(specifier, &aliases);
        let result_2 = apply_alias(specifier, &aliases);
        assert_eq!(result_1, result_2);
    }

    // ─── SVGR routing tests ───────────────────────────────────────────────────

    use crate::asset::SvgrExportType;

    #[test]
    fn is_svg_specifier_detects_svg() {
        assert!(is_svg_specifier("./icon.svg"));
        assert!(is_svg_specifier("./Icon.SVG"));
        assert!(is_svg_specifier("./icon.svg?react"));
        assert!(is_svg_specifier("@/assets/logo.svg?url"));
        assert!(!is_svg_specifier("./icon.png"));
        assert!(!is_svg_specifier("react"));
    }

    #[test]
    fn named_export_routes_only_reactcomponent_named_import() {
        // fe-shared shape: `import { ReactComponent as Icon } from './icon.svg'`
        assert!(should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Named,
            true,
            SvgrExportType::Named,
        ));
        // Bare default import stays an asset URL under `exportType: 'named'`.
        assert!(!should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Default,
            true,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn default_export_routes_default_import() {
        assert!(should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Default,
            true,
            SvgrExportType::Default,
        ));
        assert!(!should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Named,
            true,
            SvgrExportType::Default,
        ));
    }

    #[test]
    fn url_query_forces_asset_url_even_when_enabled() {
        assert!(!should_route_svg_as_component(
            "./icon.svg?url",
            &ImportKind::Named,
            true,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn react_query_forces_component_even_when_disabled() {
        assert!(should_route_svg_as_component(
            "./icon.svg?react",
            &ImportKind::Default,
            false,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn disabled_svgr_keeps_asset_url() {
        assert!(!should_route_svg_as_component(
            "./icon.svg",
            &ImportKind::Named,
            false,
            SvgrExportType::Named,
        ));
    }

    #[test]
    fn non_svg_never_routes_as_component() {
        assert!(!should_route_svg_as_component(
            "./icon.png",
            &ImportKind::Named,
            true,
            SvgrExportType::Named,
        ));
    }

    // ─── SCSS / Sass routing tests ─────────────────────────────────────────────

    #[test]
    fn is_scss_specifier_detects_scss_and_sass() {
        assert!(is_scss_specifier("./theme.scss"));
        assert!(is_scss_specifier("./theme.SCSS"));
        assert!(is_scss_specifier("../styles/main.sass"));
        assert!(is_scss_specifier("@/styles/x.scss?inline"));
        assert!(!is_scss_specifier("./theme.css"));
        assert!(!is_scss_specifier("./icon.svg"));
        assert!(!is_scss_specifier("react"));
    }

    #[test]
    fn classify_routes_sass_family_and_keeps_plain_css() {
        assert_eq!(classify_style_import("./a.scss"), StyleImportRoute::Sass);
        assert_eq!(classify_style_import("./a.sass"), StyleImportRoute::Sass);
        // Plain CSS (and other forms) must NOT be diverted.
        assert_eq!(classify_style_import("./a.css"), StyleImportRoute::PlainCss);
        assert_eq!(
            classify_style_import("./a.less"),
            StyleImportRoute::PlainCss
        );
    }
}
// CODEGEN-END
