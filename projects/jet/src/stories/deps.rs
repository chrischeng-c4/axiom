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
pub fn resolve_bare_specifier(root: &Path, importer_file: &Path, specifier: &str) -> Option<PathBuf> {
    // Only bare specifiers are our concern. A leading `.` or `/` is a relative
    // or absolute import the module-serving path already handles.
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }

    let options = ResolveOptions {
        // Anchor the node_modules walk-up at the project root so the resolver
        // never escapes above it.
        base_dirs: vec![root.to_path_buf()],
        ..ResolveOptions::default()
    };
    let resolver = ModuleResolver::new(options).ok()?;
    let resolved = resolver.resolve(specifier, importer_file).ok()?;

    // External (or anything not a package resolution) is not something we serve
    // from disk — leave it for the importmap.
    if resolved.is_external || resolved.kind != ResolveKind::Package {
        return None;
    }

    // Must be a real file genuinely inside node_modules. (`resolve` returns the
    // specifier path verbatim for externals, which would not be a real file.)
    if !resolved.path.is_file() {
        return None;
    }
    if !path_has_node_modules(&resolved.path) {
        return None;
    }
    Some(resolved.path)
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

    for raw in source.lines() {
        let line = raw.trim_start();
        let is_import = line.starts_with("import ")
            || line.starts_with("import\"")
            || line.starts_with("import'")
            || line.starts_with("import{")
            || line == "import";
        let is_reexport = (line.starts_with("export ") || line.starts_with("export{"))
            && line.contains(" from ");
        if !is_import && !is_reexport {
            continue;
        }
        if let Some(spec) = specifier_from_statement(line) {
            push(spec);
        }
    }

    specifiers
}

/// Pull the quoted module specifier out of one import/export statement line.
///
/// Uses the `from "..."` clause when present (named/default/namespace imports
/// and re-exports), otherwise the bare side-effect form `import "..."`.
fn specifier_from_statement(line: &str) -> Option<String> {
    let after = if let Some(pos) = line.rfind(" from ") {
        &line[pos + 6..]
    } else {
        // Side-effect import: `import "m";` — the quote follows `import`.
        line.trim_start_matches("import").trim_start()
    };
    extract_first_string_literal(after)
}

/// Extract the first single- or double-quoted string literal from `s`.
fn extract_first_string_literal(s: &str) -> Option<String> {
    let s = s.trim_start();
    let quote = s.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &s[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
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
    fn extract_dedups() {
        let src = "import a from \"clsx\";\nimport { b } from \"clsx\";\n";
        let specs = extract_all_import_specifiers(src);
        assert_eq!(specs.iter().filter(|s| *s == "clsx").count(), 1);
    }

    #[test]
    fn resolve_returns_none_for_relative() {
        assert!(resolve_bare_specifier(Path::new("/proj"), Path::new("/proj/a.tsx"), "./b").is_none());
        assert!(resolve_bare_specifier(Path::new("/proj"), Path::new("/proj/a.tsx"), "/abs").is_none());
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
        std::fs::write(pkg.join("dist/clsx.mjs"), "export default function clsx(){}\n").unwrap();

        let importer = root.join("src/Button.tsx");
        std::fs::create_dir_all(importer.parent().unwrap()).unwrap();
        let resolved = resolve_bare_specifier(root, &importer, "clsx").expect("resolves clsx");
        assert!(resolved.is_file());
        assert_eq!(dep_key(&resolved), "clsx/dist/clsx.mjs");
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
}
// </HANDWRITE>
