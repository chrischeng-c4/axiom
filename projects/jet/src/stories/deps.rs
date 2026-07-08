// <HANDWRITE gap="missing-generator:logic:b3d9a1f2" tracker="standardize-gap-projects-jet-src-stories-deps-rs" reason="Shared node_modules bare-import resolution for the stories workbench (dev server + static export): resolve a bare specifier to an on-disk node_modules file via the project ModuleResolver, extract every import specifier (incl. bare) from source, and key a dep by its node_modules-relative path so both the dev route and the static layout map a dep consistently.">
//! Shared `node_modules` bare-import resolution for `jet stories` (#197).
//!
//! Both the dev workbench server ([`super::server`]) and the static exporter
//! ([`super::build`]) need to turn a bare import specifier in a served/emitted
//! module — `import x from "clsx"` — into a real file inside the project's
//! `node_modules`, so a real component's third-party deps actually load in the
//! preview. This module is the single place that:
//!
//! 1. [`resolve_bare_specifier`] — resolve a bare specifier against an importing
//!    file using the project's [`crate::resolver::ModuleResolver`] (the same
//!    Node-resolution + `package.json` `exports`/`module`/`main` honoring that
//!    `jet install` / the bundler use), returning the on-disk file **only** when
//!    it resolves into `node_modules` (so React-class specifiers with no local
//!    install fall through to the esm.sh importmap, unchanged).
//! 2. [`extract_all_import_specifiers`] — extract every import specifier in a
//!    source file, **including** bare ones (the dev server's
//!    [`crate::dev_server::source_analysis::extract_imports_from_source`]
//!    deliberately drops bare specifiers, so we need our own pass here).
//! 3. [`dep_key`] — the `node_modules`-relative key (`clsx/dist/clsx.mjs`) that
//!    both surfaces share: the dev server serves it under `/@dep/<key>` and the
//!    static exporter emits it under `out_dir/deps/<key>.js`.
//!
//! ## Scope (#197)
//! The common case — a component imports one or two simple deps whose entry is a
//! `package.json` `main`/`module`/`exports` pointing at a single JS file, plus
//! that file's own relative imports — works in both dev and static and is
//! tested. The resolver itself already handles conditional `exports`, scoped
//! packages, and the monorepo `node_modules` walk-up, so those ride along.
//! TODO(#197 follow-up): advanced conditional-exports edge cases (deeply nested
//! `import`/`require`/`browser` branch selection that diverges from the
//! workbench's chosen condition order) and CommonJS deps that need an interop
//! shim are not specially handled — a dep authored as ESM is the expectation.

use std::path::{Path, PathBuf};

use crate::resolver::{ModuleResolver, ResolveKind, ResolveOptions};

/// Resolve a bare specifier (`clsx`, `@scope/pkg`, `clsx/dist/x`) to an existing
/// file inside `root`'s `node_modules`, resolving from `importer_file`.
///
/// Returns `Some(absolute_path)` only when the specifier resolves to a real file
/// whose path contains a `node_modules` segment (so it is a genuinely-installed
/// dep we can serve/emit locally). Returns `None` for:
///   - relative / absolute specifiers (the caller handles those separately),
///   - specifiers that do not resolve on disk (e.g. `react` with no local
///     install) — the caller leaves them for the esm.sh importmap/CDN,
///   - anything the resolver flags external.
///
/// The resolver uses the default browser ESM conditions
/// (`import`/`browser`/`default`), matching the dev preview's runtime.
pub fn resolve_bare_specifier(
    root: &Path,
    importer_file: &Path,
    specifier: &str,
) -> Option<PathBuf> {
    // Only bare specifiers are our concern. A leading `.` or `/` is a relative
    // or absolute import the module-serving path already handles.
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }
    if is_preview_importmap_specifier(specifier) {
        return None;
    }

    let options = ResolveOptions {
        // Anchor the node_modules walk-up at the project root so the resolver
        // never escapes above it.
        base_dirs: vec![root.to_path_buf()],
        ..ResolveOptions::default()
    };
    let resolver = ModuleResolver::new(options).ok()?;
    let resolved = match resolver.resolve(specifier, importer_file).ok() {
        Some(resolved) => resolved,
        None => {
            return resolve_nested_bare_specifier(importer_file, specifier)
                .or_else(|| resolve_bare_asset_export(root, specifier));
        }
    };

    // External (or anything not a package resolution) is not something we serve
    // from disk — leave it for the importmap.
    if resolved.is_external || resolved.kind != ResolveKind::Package {
        return resolve_nested_bare_specifier(importer_file, specifier)
            .or_else(|| resolve_bare_asset_export(root, specifier));
    }

    // Must be a real file genuinely inside node_modules. (`resolve` returns the
    // specifier path verbatim for externals, which would not be a real file.)
    if !resolved.path.is_file() {
        return resolve_nested_bare_specifier(importer_file, specifier)
            .or_else(|| resolve_bare_asset_export(root, specifier));
    }
    if !path_has_node_modules(&resolved.path) {
        return resolve_nested_bare_specifier(importer_file, specifier)
            .or_else(|| resolve_bare_asset_export(root, specifier));
    }
    Some(canonical_node_modules_path(&resolved.path))
}

fn is_preview_importmap_specifier(specifier: &str) -> bool {
    matches!(
        specifier,
        "react"
            | "react-dom"
            | "react-dom/client"
            | "react/jsx-runtime"
            | "@storybook/addon-actions"
            | "@storybook/global"
            | "@storybook/preview-api"
            | "@storybook/instrumenter"
            | "@storybook/test"
    )
}

fn canonical_node_modules_path(path: &Path) -> PathBuf {
    let Ok(canonical) = path.canonicalize() else {
        return path.to_path_buf();
    };
    if path_has_node_modules(&canonical) {
        canonical
    } else {
        path.to_path_buf()
    }
}

fn resolve_bare_asset_export(root: &Path, specifier: &str) -> Option<PathBuf> {
    let (package_name, subpath) = split_package_specifier(specifier)?;
    if !is_raw_asset_specifier(&subpath) {
        return None;
    }
    let package_dir = resolve_asset_package_dir(root, &package_name)?;
    if let Some(direct) = package_asset_file(&package_dir, &subpath) {
        return Some(direct);
    }

    let package_json = package_dir.join("package.json");
    let body = std::fs::read_to_string(package_json).ok()?;
    let package: serde_json::Value = serde_json::from_str(&body).ok()?;
    let exports = package.get("exports")?;
    let target = export_target_for_subpath(exports, specifier_path_without_query(&subpath))?;
    package_asset_file(&package_dir, &target)
}

fn resolve_asset_package_dir(root: &Path, package_name: &str) -> Option<PathBuf> {
    let node_modules = root.join("node_modules").join(package_name);
    if node_modules.is_dir() {
        return Some(node_modules);
    }
    workspace_package_dir(root, package_name)
}

fn workspace_package_dir(root: &Path, package_name: &str) -> Option<PathBuf> {
    for parent in ["packages", "libs"] {
        let dir = root.join(parent);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let package_dir = entry.path();
            let package_json = package_dir.join("package.json");
            let Ok(body) = std::fs::read_to_string(package_json) else {
                continue;
            };
            let Ok(package) = serde_json::from_str::<serde_json::Value>(&body) else {
                continue;
            };
            if package.get("name").and_then(|name| name.as_str()) == Some(package_name) {
                return Some(package_dir);
            }
        }
    }
    None
}

fn resolve_nested_bare_specifier(importer_file: &Path, specifier: &str) -> Option<PathBuf> {
    let (package_name, subpath) = split_package_specifier(specifier)?;
    let importer = importer_file
        .canonicalize()
        .unwrap_or_else(|_| importer_file.to_path_buf());
    for ancestor in importer.ancestors() {
        let mut candidates = Vec::new();
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("node_modules") {
            candidates.push(ancestor.join(&package_name));
        }
        candidates.push(ancestor.join("node_modules").join(&package_name));
        for package_dir in candidates {
            if !package_dir.is_dir() {
                continue;
            }
            if let Some(file) = package_entry_file(&package_dir, &subpath) {
                return Some(canonical_node_modules_path(&file));
            }
        }
    }
    None
}

fn package_entry_file(package_dir: &Path, subpath: &str) -> Option<PathBuf> {
    let package_json = package_dir.join("package.json");
    let package = std::fs::read_to_string(package_json)
        .ok()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(&body).ok());
    if let Some(package) = &package {
        if let Some(exports) = package.get("exports") {
            if let Some(target) =
                export_target_for_subpath(exports, specifier_path_without_query(subpath))
            {
                if let Some(file) = package_file(package_dir, &target) {
                    return Some(file);
                }
            }
        }
        if subpath == "." {
            for field in ["module", "main"] {
                if let Some(target) = package.get(field).and_then(|value| value.as_str()) {
                    if let Some(file) = package_file(package_dir, target) {
                        return Some(file);
                    }
                }
            }
        }
    }
    if subpath == "." {
        for index in ["index.mjs", "index.js", "index.cjs"] {
            let file = package_dir.join(index);
            if file.is_file() {
                return Some(file);
            }
        }
    } else {
        return package_file(package_dir, subpath);
    }
    None
}

fn package_file(package_dir: &Path, target: &str) -> Option<PathBuf> {
    let clean = specifier_path_without_query(target)
        .trim_start_matches("./")
        .trim_start_matches('/');
    let direct = package_dir.join(clean);
    if direct.is_file() {
        return Some(direct);
    }
    for ext in ["js", "mjs", "cjs", "ts", "tsx"] {
        let with_ext = package_dir.join(format!("{clean}.{ext}"));
        if with_ext.is_file() {
            return Some(with_ext);
        }
    }
    let index = package_dir.join(clean).join("index.js");
    if index.is_file() {
        return Some(index);
    }
    None
}

fn package_asset_file(package_dir: &Path, target: &str) -> Option<PathBuf> {
    let clean = specifier_path_without_query(target)
        .trim_start_matches("./")
        .trim_start_matches('/');
    let direct = package_dir.join(clean);
    if direct.is_file() {
        return Some(direct);
    }

    if let Some(rest) = clean.strip_prefix("dist/") {
        let source = package_dir.join("src/lib").join(rest);
        if source.is_file() {
            return Some(source);
        }
    }

    let source = package_dir.join("src/lib").join(clean);
    if source.is_file() {
        return Some(source);
    }

    None
}

fn specifier_path_without_query(path: &str) -> &str {
    path.split(['?', '#']).next().unwrap_or(path)
}

fn split_package_specifier(specifier: &str) -> Option<(String, String)> {
    if specifier.starts_with('@') {
        let mut parts = specifier.splitn(4, '/');
        let scope = parts.next()?;
        let name = parts.next()?;
        let package_end = scope.len() + 1 + name.len();
        let subpath = specifier
            .get(package_end + 1..)
            .map(|rest| format!("./{rest}"))
            .unwrap_or_else(|| ".".to_string());
        return Some((specifier[..package_end].to_string(), subpath));
    }

    let (package_name, rest) = specifier.split_once('/').unwrap_or((specifier, ""));
    let subpath = if rest.is_empty() {
        ".".to_string()
    } else {
        format!("./{rest}")
    };
    Some((package_name.to_string(), subpath))
}

fn export_target_for_subpath(exports: &serde_json::Value, subpath: &str) -> Option<String> {
    match exports {
        serde_json::Value::String(path) if subpath == "." => Some(path.clone()),
        serde_json::Value::Object(map) => {
            if let Some(value) = map.get(subpath) {
                return export_target_value(value);
            }
            None
        }
        _ => None,
    }
}

fn export_target_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(path) => Some(path.clone()),
        serde_json::Value::Object(map) => {
            for key in ["import", "browser", "default"] {
                if let Some(nested) = map.get(key) {
                    if let Some(path) = export_target_value(nested) {
                        return Some(path);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn is_raw_asset_specifier(path: &str) -> bool {
    let path = path.split(['?', '#']).next().unwrap_or(path);
    matches!(
        Path::new(path).extension().and_then(|e| e.to_str()),
        Some(ext)
            if ext.eq_ignore_ascii_case("svg")
                || ext.eq_ignore_ascii_case("png")
                || ext.eq_ignore_ascii_case("jpg")
                || ext.eq_ignore_ascii_case("jpeg")
                || ext.eq_ignore_ascii_case("gif")
                || ext.eq_ignore_ascii_case("webp")
                || ext.eq_ignore_ascii_case("avif")
    )
}

/// The `node_modules`-relative key for a resolved dep file: the path *after* the
/// last `node_modules/` segment (`clsx/dist/clsx.mjs`,
/// `@scope/pkg/dist/index.js`). This is the stable identity both surfaces use —
/// the dev server serves it at `/@dep/<key>` and the static exporter writes it
/// to `out_dir/deps/<key>` (extension normalized to `.js`).
///
/// Falls back to the file name when no `node_modules` segment is present (should
/// not happen for a value returned by [`resolve_bare_specifier`]).
pub fn dep_key(resolved_file: &Path) -> String {
    let components: Vec<String> = resolved_file
        .iter()
        .map(|c| c.to_string_lossy().to_string())
        .collect();
    let mut last_nm: Option<usize> = None;
    for (i, c) in components.iter().enumerate() {
        if c == "node_modules" {
            last_nm = Some(i);
        }
    }
    match last_nm {
        Some(i) if i + 1 < components.len() => components[i + 1..].join("/"),
        _ => resolved_file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default(),
    }
}

/// True when `path` contains a `node_modules` path segment.
fn path_has_node_modules(path: &Path) -> bool {
    path.iter().any(|c| c == "node_modules")
}

/// Extract **every** import specifier from `source`, including bare ones.
///
/// The dev server's
/// [`crate::dev_server::source_analysis::extract_imports_from_source`] keeps only
/// relative/absolute specifiers (it filters bare ones out), which is exactly the
/// opposite of what bare-import resolution needs. This pass returns the raw
/// specifier text of each static `import`/`export ... from` and bare `import "x"`
/// in source order, de-duplicated, so the caller can resolve the bare ones and
/// rewrite them.
///
/// Covers: `import x from "m"`, `import {a} from "m"`, `import * as x from "m"`,
/// `import "m"`, and `export {a} from "m"` / `export * from "m"`. Dynamic
/// `import("m")` is intentionally not rewritten (it is rare in transformed
/// component output and would need expression-aware rewriting).
pub fn extract_all_import_specifiers(source: &str) -> Vec<String> {
    let mut specifiers: Vec<String> = Vec::new();
    let mut push = |s: String| {
        if !specifiers.contains(&s) {
            specifiers.push(s);
        }
    };

    let mut idx = 0;
    while idx < source.len() {
        if source[idx..].starts_with("//") {
            idx = source[idx..]
                .find('\n')
                .map(|offset| idx + offset + 1)
                .unwrap_or(source.len());
            continue;
        }
        if source[idx..].starts_with("/*") {
            idx = source[idx + 2..]
                .find("*/")
                .map(|offset| idx + 2 + offset + 2)
                .unwrap_or(source.len());
            continue;
        }
        if let Some((_, end)) = string_literal_at(source, idx) {
            idx = end;
            continue;
        }

        if keyword_at(source, idx, "import") {
            let after = idx + "import".len();
            let Some(next) = next_non_ws(source, after) else {
                break;
            };
            if source[next..].starts_with('(') {
                idx = after;
                continue;
            }
            if let Some((spec, end)) = string_literal_at(source, next) {
                push(spec);
                idx = end;
                continue;
            }
            let end = statement_end(source, after);
            if let Some(spec) = specifier_from_statement(&source[idx..end]) {
                push(spec);
            }
            idx = end;
            continue;
        }

        if keyword_at(source, idx, "export") {
            let end = statement_end(source, idx + "export".len());
            if let Some(spec) = specifier_from_statement(&source[idx..end]) {
                push(spec);
            }
            idx = end;
            continue;
        }

        idx += source[idx..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(1);
    }

    specifiers
}

/// Pull the quoted module specifier out of one import/export statement line.
///
/// Uses the `from "..."` clause when present (named/default/namespace imports
/// and re-exports), otherwise the bare side-effect form `import "..."`.
fn specifier_from_statement(line: &str) -> Option<String> {
    if keyword_at(line, 0, "import") {
        let after = next_non_ws(line, "import".len())?;
        if let Some((spec, _)) = string_literal_at(line, after) {
            return Some(spec);
        }
    }
    find_from_string_literal(line)
}

fn find_from_string_literal(statement: &str) -> Option<String> {
    let mut idx = 0;
    while idx < statement.len() {
        if let Some((_, end)) = string_literal_at(statement, idx) {
            idx = end;
            continue;
        }
        if keyword_at(statement, idx, "from") {
            if let Some(start) = next_non_ws(statement, idx + "from".len()) {
                if let Some((spec, _)) = string_literal_at(statement, start) {
                    return Some(spec);
                }
            }
        }
        idx += statement[idx..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(1);
    }
    None
}

fn statement_end(source: &str, start: usize) -> usize {
    let mut idx = start;
    let mut depth = 0i32;
    while idx < source.len() {
        if source[idx..].starts_with("//") {
            idx = source[idx..]
                .find('\n')
                .map(|offset| idx + offset + 1)
                .unwrap_or(source.len());
            continue;
        }
        if source[idx..].starts_with("/*") {
            idx = source[idx + 2..]
                .find("*/")
                .map(|offset| idx + 2 + offset + 2)
                .unwrap_or(source.len());
            continue;
        }
        if let Some((_, end)) = string_literal_at(source, idx) {
            idx = end;
            continue;
        }
        let Some(ch) = source[idx..].chars().next() else {
            return source.len();
        };
        match ch {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ';' if depth <= 0 => return idx + 1,
            '\n' if depth <= 0 && find_from_string_literal(&source[start..idx]).is_some() => {
                return idx + 1;
            }
            _ => {}
        }
        idx += ch.len_utf8();
    }
    source.len()
}

fn next_non_ws(source: &str, mut idx: usize) -> Option<usize> {
    while idx < source.len() {
        let ch = source[idx..].chars().next()?;
        if !ch.is_whitespace() {
            return Some(idx);
        }
        idx += ch.len_utf8();
    }
    None
}

fn keyword_at(source: &str, idx: usize, keyword: &str) -> bool {
    source[idx..].starts_with(keyword)
        && source[..idx]
            .chars()
            .next_back()
            .map(|ch| !is_ident_char(ch))
            .unwrap_or(true)
        && source[idx + keyword.len()..]
            .chars()
            .next()
            .map(|ch| !is_ident_char(ch))
            .unwrap_or(true)
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

fn string_literal_at(source: &str, idx: usize) -> Option<(String, usize)> {
    let quote = source[idx..].chars().next()?;
    if quote != '"' && quote != '\'' && quote != '`' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    let mut cursor = idx + quote.len_utf8();
    while cursor < source.len() {
        let ch = source[cursor..].chars().next()?;
        cursor += ch.len_utf8();
        if escaped {
            out.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some((out, cursor));
        }
        out.push(ch);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dep_key_strips_through_node_modules() {
        let p = Path::new("/proj/node_modules/clsx/dist/clsx.mjs");
        assert_eq!(dep_key(p), "clsx/dist/clsx.mjs");
    }

    #[test]
    fn dep_key_handles_scoped_and_nested() {
        let p = Path::new("/proj/node_modules/@scope/pkg/dist/index.js");
        assert_eq!(dep_key(p), "@scope/pkg/dist/index.js");
        // Last node_modules wins (nested install).
        let nested = Path::new("/proj/node_modules/a/node_modules/b/index.js");
        assert_eq!(dep_key(nested), "b/index.js");
    }

    #[test]
    fn extract_picks_up_bare_and_relative() {
        let src = r#"
import React from "react";
import { clsx } from 'clsx';
import { Local } from "./Local";
import "side-effect.css";
export { x } from "../shared/x";
const dyn = import("ignored");
"#;
        let specs = extract_all_import_specifiers(src);
        assert!(specs.contains(&"react".to_string()));
        assert!(specs.contains(&"clsx".to_string()));
        assert!(specs.contains(&"./Local".to_string()));
        assert!(specs.contains(&"side-effect.css".to_string()));
        assert!(specs.contains(&"../shared/x".to_string()));
        // Dynamic import is not extracted (no static `import ` / `from`).
        assert!(!specs.contains(&"ignored".to_string()));
    }

    #[test]
    fn extract_picks_up_multiline_imports_and_reexports() {
        let src = r#"
import {
  Button,
  type ButtonProps,
} from "./Button";
import type {
  Meta,
  StoryObj,
} from "@storybook/react";
export {
  assetUrl
} from './asset.svg?url';
const dyn = import("ignored");
"#;
        let specs = extract_all_import_specifiers(src);
        assert!(specs.contains(&"./Button".to_string()), "{specs:?}");
        assert!(specs.contains(&"@storybook/react".to_string()), "{specs:?}");
        assert!(specs.contains(&"./asset.svg?url".to_string()), "{specs:?}");
        assert!(!specs.contains(&"ignored".to_string()));
    }

    #[test]
    fn extract_dedups() {
        let src = "import a from \"clsx\";\nimport { b } from \"clsx\";\n";
        let specs = extract_all_import_specifiers(src);
        assert_eq!(specs.iter().filter(|s| *s == "clsx").count(), 1);
    }

    #[test]
    fn resolve_returns_none_for_relative() {
        assert!(
            resolve_bare_specifier(Path::new("/proj"), Path::new("/proj/a.tsx"), "./b").is_none()
        );
        assert!(
            resolve_bare_specifier(Path::new("/proj"), Path::new("/proj/a.tsx"), "/abs").is_none()
        );
    }

    #[test]
    fn resolve_finds_installed_dep_and_keys_it() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let pkg = root.join("node_modules/clsx");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name":"clsx","version":"2.0.0","module":"dist/clsx.mjs","main":"dist/clsx.js"}"#,
        )
        .unwrap();
        std::fs::create_dir_all(pkg.join("dist")).unwrap();
        std::fs::write(
            pkg.join("dist/clsx.mjs"),
            "export default function clsx(){}\n",
        )
        .unwrap();

        let importer = root.join("src/Button.tsx");
        std::fs::create_dir_all(importer.parent().unwrap()).unwrap();
        let resolved = resolve_bare_specifier(root, &importer, "clsx").expect("resolves clsx");
        assert!(resolved.is_file());
        assert_eq!(dep_key(&resolved), "clsx/dist/clsx.mjs");
    }

    #[test]
    fn resolve_finds_bare_asset_file_without_exports() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let pkg = root.join("node_modules/@tw-tech/shared-assets");
        std::fs::create_dir_all(pkg.join("images")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name":"@tw-tech/shared-assets","version":"1.0.0"}"#,
        )
        .unwrap();
        std::fs::write(pkg.join("images/empty-default.png"), "png").unwrap();

        let importer = root.join("src/AssetBox.tsx");
        std::fs::create_dir_all(importer.parent().unwrap()).unwrap();
        let resolved = resolve_bare_specifier(
            root,
            &importer,
            "@tw-tech/shared-assets/images/empty-default.png?url",
        )
        .expect("resolves direct bare asset file");
        assert_eq!(
            dep_key(&resolved),
            "@tw-tech/shared-assets/images/empty-default.png"
        );
    }

    #[test]
    fn resolve_finds_nested_pnpm_transitive_dependency_from_importer() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let react_pdf = root.join("node_modules/.pnpm/react-pdf@1.0.0/node_modules/react-pdf");
        let pdfjs = root.join("node_modules/.pnpm/react-pdf@1.0.0/node_modules/pdfjs-dist");
        std::fs::create_dir_all(react_pdf.join("dist")).unwrap();
        std::fs::create_dir_all(pdfjs.join("build")).unwrap();
        std::fs::write(
            react_pdf.join("package.json"),
            r#"{"name":"react-pdf","version":"1.0.0","main":"dist/index.js"}"#,
        )
        .unwrap();
        std::fs::write(
            pdfjs.join("package.json"),
            r#"{"name":"pdfjs-dist","version":"5.4.296","main":"build/pdf.mjs"}"#,
        )
        .unwrap();
        std::fs::write(
            react_pdf.join("dist/index.js"),
            "import * as pdfjs from 'pdfjs-dist';\n",
        )
        .unwrap();
        std::fs::write(
            pdfjs.join("build/pdf.mjs"),
            "export const version = '5.4.296';\n",
        )
        .unwrap();

        let resolved = resolve_bare_specifier(root, &react_pdf.join("dist/index.js"), "pdfjs-dist")
            .expect("resolves nested pdfjs-dist from importer package");
        assert_eq!(dep_key(&resolved), "pdfjs-dist/build/pdf.mjs");
    }

    #[test]
    fn resolve_finds_workspace_asset_export_source_fallback() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let pkg = root.join("packages/assets");
        std::fs::create_dir_all(pkg.join("src/lib/icons")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{
  "name": "@tw-tech/shared-assets",
  "version": "1.0.0",
  "exports": {
    "./icons/*": "./dist/icons/*"
  }
}"#,
        )
        .unwrap();
        std::fs::write(pkg.join("src/lib/icons/list.svg"), "<svg />").unwrap();

        let importer = root.join("packages/ui-general/src/sp-empty-box.tsx");
        std::fs::create_dir_all(importer.parent().unwrap()).unwrap();
        let resolved =
            resolve_bare_specifier(root, &importer, "@tw-tech/shared-assets/icons/list.svg")
                .expect("resolves workspace asset export source file");
        assert_eq!(resolved, pkg.join("src/lib/icons/list.svg"));
    }

    #[test]
    fn resolve_returns_none_when_not_installed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let importer = root.join("src/Button.tsx");
        std::fs::create_dir_all(importer.parent().unwrap()).unwrap();
        // `react` is NOT installed locally → falls through to the importmap.
        assert!(resolve_bare_specifier(root, &importer, "react").is_none());
    }

    #[test]
    fn resolve_keeps_preview_importmap_specifiers_external_even_when_installed() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let react = root.join("node_modules/react");
        let react_dom = root.join("node_modules/react-dom");
        let storybook_test = root.join("node_modules/@storybook/test");
        std::fs::create_dir_all(&react).unwrap();
        std::fs::create_dir_all(&react_dom).unwrap();
        std::fs::create_dir_all(&storybook_test).unwrap();
        std::fs::write(
            react.join("package.json"),
            r#"{"name":"react","version":"18.3.1","main":"index.js","exports":{"./jsx-runtime":"./jsx-runtime.js",".":"./index.js"}}"#,
        )
        .unwrap();
        std::fs::write(react.join("index.js"), "module.exports = {};\n").unwrap();
        std::fs::write(react.join("jsx-runtime.js"), "module.exports = {};\n").unwrap();
        std::fs::write(
            react_dom.join("package.json"),
            r#"{"name":"react-dom","version":"18.3.1","main":"index.js","exports":{"./client":"./client.js",".":"./index.js"}}"#,
        )
        .unwrap();
        std::fs::write(react_dom.join("index.js"), "module.exports = {};\n").unwrap();
        std::fs::write(react_dom.join("client.js"), "module.exports = {};\n").unwrap();
        std::fs::write(
            storybook_test.join("package.json"),
            r#"{"name":"@storybook/test","version":"8.0.0","main":"dist/index.mjs"}"#,
        )
        .unwrap();
        std::fs::create_dir_all(storybook_test.join("dist")).unwrap();
        std::fs::write(
            storybook_test.join("dist/index.mjs"),
            "export const fn = () => {};\n",
        )
        .unwrap();

        let importer = root.join("src/Button.tsx");
        std::fs::create_dir_all(importer.parent().unwrap()).unwrap();
        for specifier in [
            "react",
            "react-dom",
            "react-dom/client",
            "react/jsx-runtime",
            "@storybook/test",
        ] {
            assert!(
                resolve_bare_specifier(root, &importer, specifier).is_none(),
                "{specifier} should stay on the preview importmap"
            );
        }
    }

    #[test]
    fn resolve_finds_pnpm_nested_dependency_from_dep_importer() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let importer_pkg =
            root.join("node_modules/.pnpm/rc-tree-select@5.27.0/node_modules/rc-tree-select");
        let runtime_pkg =
            root.join("node_modules/.pnpm/rc-tree-select@5.27.0/node_modules/@babel/runtime");
        std::fs::create_dir_all(importer_pkg.join("es")).unwrap();
        std::fs::create_dir_all(runtime_pkg.join("helpers/esm")).unwrap();
        std::fs::write(
            importer_pkg.join("package.json"),
            r#"{"name":"rc-tree-select","version":"5.27.0","module":"es/index.js"}"#,
        )
        .unwrap();
        std::fs::write(importer_pkg.join("es/TreeSelect.js"), "").unwrap();
        std::fs::write(
            runtime_pkg.join("package.json"),
            r#"{"name":"@babel/runtime","version":"7.29.7","exports":{"./helpers/esm/extends":"./helpers/esm/extends.js"}}"#,
        )
        .unwrap();
        std::fs::write(
            runtime_pkg.join("helpers/esm/extends.js"),
            "export default function _extends() {}\n",
        )
        .unwrap();

        let importer = importer_pkg.join("es/TreeSelect.js");
        let resolved =
            resolve_bare_specifier(root, &importer, "@babel/runtime/helpers/esm/extends")
                .expect("resolves pnpm nested @babel/runtime from dependency importer");
        assert_eq!(
            resolved,
            runtime_pkg
                .join("helpers/esm/extends.js")
                .canonicalize()
                .unwrap()
        );
    }

    #[test]
    #[cfg(unix)]
    fn resolve_canonicalizes_pnpm_symlinked_dep_for_transitive_resolution() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let project_pkg = root.join("packages/ui-form-inputs");
        let store_pkg =
            root.join("node_modules/.pnpm/rc-tree-select@5.24.4/node_modules/rc-tree-select");
        let runtime_pkg =
            root.join("node_modules/.pnpm/rc-tree-select@5.24.4/node_modules/@babel/runtime");
        std::fs::create_dir_all(project_pkg.join("src")).unwrap();
        std::fs::create_dir_all(project_pkg.join("node_modules")).unwrap();
        std::fs::create_dir_all(store_pkg.join("es")).unwrap();
        std::fs::create_dir_all(runtime_pkg.join("helpers/esm")).unwrap();
        symlink(&store_pkg, project_pkg.join("node_modules/rc-tree-select")).unwrap();

        std::fs::write(
            store_pkg.join("package.json"),
            r#"{"name":"rc-tree-select","version":"5.24.4","module":"es/index.js"}"#,
        )
        .unwrap();
        std::fs::write(store_pkg.join("es/TreeSelect.js"), "").unwrap();
        std::fs::write(
            runtime_pkg.join("package.json"),
            r#"{"name":"@babel/runtime","version":"7.29.7","exports":{"./helpers/esm/extends":"./helpers/esm/extends.js"}}"#,
        )
        .unwrap();
        std::fs::write(
            runtime_pkg.join("helpers/esm/extends.js"),
            "export default function _extends() {}\n",
        )
        .unwrap();

        let project_importer = project_pkg.join("src/form-tree-select.tsx");
        let dep_entry =
            resolve_bare_specifier(root, &project_importer, "rc-tree-select/es/TreeSelect")
                .expect("project import resolves through pnpm symlink");
        assert_eq!(
            dep_entry,
            store_pkg.join("es/TreeSelect.js").canonicalize().unwrap()
        );

        let transitive =
            resolve_bare_specifier(root, &dep_entry, "@babel/runtime/helpers/esm/extends")
                .expect("transitive import resolves from canonical pnpm package path");
        assert_eq!(
            transitive,
            runtime_pkg
                .join("helpers/esm/extends.js")
                .canonicalize()
                .unwrap()
        );
    }
}
// </HANDWRITE>
