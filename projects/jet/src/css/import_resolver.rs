// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css.md#schema
// CODEGEN-BEGIN
//! CSS `@import` resolution.
//!
//! Inlines all `@import` statements recursively.  Circular imports are
//! detected via a visited-path set and returned as an error.

use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Resolve and inline all `@import` statements starting from `entry`.
///
/// Returns the fully-inlined CSS string, or an error if a circular import
/// or an unreadable file is encountered.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn resolve_imports(entry: &Path) -> Result<String> {
    let canonical = entry
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("Cannot resolve entry {:?}: {}", entry, e))?;
    let mut visited: HashSet<PathBuf> = HashSet::new();
    resolve_file(&canonical, &mut visited)
}

/// Process a CSS source string given a base directory for relative imports.
///
/// Used when the CSS content is already available in memory (e.g. from an
/// in-memory virtual file) — no file I/O for the root, but imported files
/// are still read from disk.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css.md#schema
pub fn resolve_source(source: &str, base_dir: &Path) -> Result<String> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    process_source(source, base_dir, &mut visited)
}

// ─── internals ───────────────────────────────────────────────────────────────

fn resolve_file(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<String> {
    let canonical = path.canonicalize().map_err(|e| {
        anyhow::anyhow!(
            "Cannot canonicalize CSS import {:?}: {} (GH #3114)",
            path,
            e
        )
    })?;

    if visited.contains(&canonical) {
        bail!("Circular CSS import detected: {:?}", path);
    }
    visited.insert(canonical);

    let source = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Cannot read CSS file {:?}: {}", path, e))?;
    let base_dir = path.parent().unwrap_or(Path::new("."));
    process_source(&source, base_dir, visited)
}

fn process_source(source: &str, base_dir: &Path, visited: &mut HashSet<PathBuf>) -> Result<String> {
    let mut output = String::with_capacity(source.len());

    for line in source.lines() {
        if let Some(import_path) = extract_import_path(line.trim()) {
            let resolved = resolve_import_path(base_dir, &import_path);
            if resolved.exists() {
                let inlined = resolve_file(&resolved, visited)?;
                output.push_str(&inlined);
            } else {
                // Unresolvable (e.g. CDN URL) — preserve verbatim.
                output.push_str(line);
                output.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    Ok(output)
}

/// Build the filesystem path for an `@import` specifier.
///
/// Relative paths (`./foo` or `../foo`) are resolved against `base_dir`.
/// Bare specifiers (e.g. `normalize.css`) are looked up in `node_modules`
/// by walking up the directory tree.
fn resolve_import_path(base_dir: &Path, import_path: &str) -> PathBuf {
    if import_path.starts_with('.') {
        return base_dir.join(import_path);
    }

    // node_modules walk-up resolution
    let mut dir = base_dir.to_path_buf();
    loop {
        let candidate = dir.join("node_modules").join(import_path);
        if candidate.exists() {
            return candidate;
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }

    // Fallback — let caller handle the missing file
    base_dir.join(import_path)
}

/// Extract the import path string from a CSS `@import` line.
///
/// Handles:
/// - `@import "path.css";`
/// - `@import 'path.css';`
/// - `@import url("path.css");`
/// - `@import url('path.css');`
/// - `@import url(path.css);`
///
/// Returns `None` for remote URLs (`http://`, `https://`, `//`) and for
/// lines that are not `@import` statements.
fn extract_import_path(line: &str) -> Option<String> {
    if !line.starts_with("@import ") {
        return None;
    }

    let rest = line["@import ".len()..].trim();

    // `@import url(...)`
    if let Some(url_inner) = rest.strip_prefix("url(") {
        let end = url_inner.find(')')?;
        let inner = url_inner[..end].trim();
        let path = strip_quotes(inner);
        if is_remote(path) {
            return None;
        }
        return Some(path.to_string());
    }

    // `@import "..."` or `@import '...'`
    let stripped = rest.trim_end_matches(';').trim();
    let path = strip_quotes(stripped);
    if path == stripped {
        return None; // no surrounding quotes
    }
    if is_remote(path) {
        return None;
    }

    Some(path.to_string())
}

fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn is_remote(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://") || path.starts_with("//")
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// T3: @import Relative Resolution (R2)
    ///
    /// Verifies that a relative @import is inlined at the @import position.
    #[test]
    fn t3_import_relative_resolution() {
        let dir = TempDir::new().unwrap();

        // tokens.css contains a CSS custom property
        let tokens_path = dir.path().join("tokens.css");
        fs::write(&tokens_path, ":root { --bg: white; }").unwrap();

        // index.css imports tokens.css
        let index_path = dir.path().join("index.css");
        fs::write(&index_path, "@import \"./tokens.css\";\n").unwrap();

        let result = resolve_imports(&index_path);
        assert!(
            result.is_ok(),
            "resolve_imports should succeed: {:?}",
            result
        );
        let content = result.unwrap();
        assert!(
            content.contains("--bg: white"),
            "Output should contain inlined tokens.css content, got: {}",
            content
        );
        // The @import statement itself should not remain
        assert!(
            !content.contains("@import"),
            "Output should not contain @import after resolution, got: {}",
            content
        );
    }

    /// T4: @import Circular Detection (R2)
    ///
    /// Verifies that a circular @import chain returns an Err.
    #[test]
    fn t4_import_circular_detection() {
        let dir = TempDir::new().unwrap();

        // a.css imports b.css, b.css imports a.css → circular
        let a_path = dir.path().join("a.css");
        let b_path = dir.path().join("b.css");
        fs::write(&a_path, "@import \"./b.css\";\n").unwrap();
        fs::write(&b_path, "@import \"./a.css\";\n").unwrap();

        let result = resolve_imports(&a_path);
        assert!(result.is_err(), "Should return Err for circular import");
        let err = result.unwrap_err().to_string().to_lowercase();
        assert!(
            err.contains("circular"),
            "Error message should mention circular import, got: {}",
            err
        );
    }

    /// Unit test: resolve_source inlines relative @import from in-memory CSS.
    #[test]
    fn resolve_source_inlines_import() {
        let dir = TempDir::new().unwrap();
        let tokens_path = dir.path().join("vars.css");
        fs::write(&tokens_path, "a { color: red; }").unwrap();

        let source = "@import \"./vars.css\";";
        let result = resolve_source(source, dir.path());
        assert!(
            result.is_ok(),
            "resolve_source should succeed: {:?}",
            result
        );
        let content = result.unwrap();
        assert!(
            content.contains("color: red"),
            "Output should contain inlined content, got: {}",
            content
        );
    }

    /// Unit test: remote @import URLs are preserved verbatim.
    #[test]
    fn remote_imports_preserved() {
        let source = "@import \"https://example.com/style.css\";";
        let result = resolve_source(source, std::path::Path::new("."));
        assert!(
            result.is_ok(),
            "resolve_source should succeed for remote URL"
        );
        let content = result.unwrap();
        assert!(
            content.contains("https://example.com/style.css"),
            "Remote @import should be preserved: {}",
            content
        );
    }

    /// GH #3114 — when canonicalize fails on an inner import path, the
    /// resolver must surface a typed `Err` rather than silently insert
    /// the raw (potentially `..`-bearing) path into the `visited` set
    /// and risk non-terminating recursion.
    ///
    /// We test `resolve_file` directly with a path that does not exist;
    /// `canonicalize` returns ENOENT, and the resolver must propagate
    /// it as an anyhow error tagged with the GH issue.
    #[test]
    fn resolve_file_surfaces_canonicalize_error_for_missing_path() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("never-was.css");
        let mut visited = HashSet::new();
        let err = resolve_file(&missing, &mut visited)
            .expect_err("missing CSS file must not silently fall back to a raw path");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("Cannot canonicalize CSS import") && msg.contains("3114"),
            "expected canonicalize error tagged with #3114, got: {msg}"
        );
    }

    /// S15: Three-level @import chain (a→b→c) merges depth-first. (TR7)
    ///
    /// Verifies that a.css importing b.css importing c.css produces all
    /// three rule sets inlined in depth-first order: c before b before a.
    #[test]
    fn three_level_import_chain_merged() {
        let dir = TempDir::new().unwrap();

        // c.css — leaf, no imports
        let c_path = dir.path().join("c.css");
        fs::write(&c_path, ".c { color: green; }\n").unwrap();

        // b.css — imports c.css
        let b_path = dir.path().join("b.css");
        fs::write(&b_path, "@import \"./c.css\";\n.b { color: blue; }\n").unwrap();

        // a.css — imports b.css
        let a_path = dir.path().join("a.css");
        fs::write(&a_path, "@import \"./b.css\";\n.a { color: red; }\n").unwrap();

        let result = resolve_imports(&a_path);
        assert!(
            result.is_ok(),
            "resolve_imports should succeed: {:?}",
            result
        );
        let content = result.unwrap();

        // All three rules must be present
        assert!(
            content.contains(".c { color: green; }"),
            "Output must contain .c rule, got: {}",
            content
        );
        assert!(
            content.contains(".b { color: blue; }"),
            "Output must contain .b rule, got: {}",
            content
        );
        assert!(
            content.contains(".a { color: red; }"),
            "Output must contain .a rule, got: {}",
            content
        );

        // Depth-first order: c before b before a
        let pos_c = content.find(".c { color: green; }").unwrap();
        let pos_b = content.find(".b { color: blue; }").unwrap();
        let pos_a = content.find(".a { color: red; }").unwrap();
        assert!(
            pos_c < pos_b && pos_b < pos_a,
            "Order must be c < b < a (depth-first), positions: c={}, b={}, a={}",
            pos_c,
            pos_b,
            pos_a
        );

        // No @import statements remain
        assert!(
            !content.contains("@import"),
            "Output must not contain @import after resolution, got: {}",
            content
        );
    }
}
// CODEGEN-END
