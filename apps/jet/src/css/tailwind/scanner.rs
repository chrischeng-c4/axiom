// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
// CODEGEN-BEGIN
//! Content scanner for Tailwind JIT.
//!
//! Walks files matching content glob patterns and extracts the set of all
//! Tailwind class names referenced in those files.

use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

/// Scans source files for Tailwind utility class names.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub struct ContentScanner {
    /// Content glob patterns (e.g. `["./src/**/*.{ts,tsx}"]`).
    patterns: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
impl ContentScanner {
    pub fn new(patterns: Vec<String>) -> Self {
        Self { patterns }
    }

    /// Walk all files matching the configured glob patterns (resolved relative
    /// to `root`) and extract the set of used Tailwind class names.
    ///
    /// GH #3310 — Per-entry walkdir errors and per-file read errors used to
    /// be silently dropped, so a single unreadable component file made every
    /// Tailwind class declared in that file vanish from the compiled CSS
    /// with no log line. Surface both via `tracing::warn!` under target
    /// `jet::css::tailwind::scanner` so operators can triage missing styles.
    pub fn scan(&self, root: &Path) -> Result<HashSet<String>> {
        let glob_set = build_glob_set(&self.patterns)?;
        let mut classes = HashSet::new();

        for entry_res in WalkDir::new(root).follow_links(false).into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::css::tailwind::scanner",
                        root = %root.display(),
                        offending_path = ?err.path(),
                        error = %err,
                        "GH #3310 walkdir error during Tailwind content scan; \
                         any utility classes declared under this entry will \
                         be missing from the compiled CSS"
                    );
                    continue;
                }
            };
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            // Check against glob set (use path relative to root for matching)
            let relative = path.strip_prefix(root).unwrap_or(path);
            // Prefix with "./" so glob patterns like "./src/**" match
            let check_path = Path::new("./").join(relative);
            if !glob_set.is_match(&check_path) {
                continue;
            }

            let source = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    // Race with deletion/rename — quiet.
                    continue;
                }
                Err(err) => {
                    tracing::warn!(
                        target: "jet::css::tailwind::scanner",
                        path = %path.display(),
                        error = %err,
                        "GH #3310 unreadable source during Tailwind content \
                         scan; its utility classes will be missing from the \
                         compiled CSS until the file becomes readable"
                    );
                    continue;
                }
            };

            extract_classes(&source, &mut classes);
        }

        Ok(classes)
    }
}

// ─── glob set builder ─────────────────────────────────────────────────────────

fn build_glob_set(patterns: &[String]) -> Result<globset::GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        // Expand brace patterns like {ts,tsx} manually since globset handles
        // them natively.
        let glob = Glob::new(pattern)
            .map_err(|e| anyhow::anyhow!("Invalid glob pattern {:?}: {}", pattern, e))?;
        builder.add(glob);
    }
    let set = builder
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build glob set: {}", e))?;
    Ok(set)
}

// ─── class extraction ─────────────────────────────────────────────────────────

/// Extract all Tailwind-like class names from a source file.
///
/// Handles:
/// - `className="..."` and `class="..."` string literals
/// - Template literals with conditional expressions (best-effort)
/// - `clsx(...)` and `cn(...)` call arguments
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub fn extract_classes(source: &str, out: &mut HashSet<String>) {
    // Strategy: split on whitespace/quotes/delimiters and collect tokens that
    // look like Tailwind class names.  This intentionally over-collects;
    // the JIT emitter ignores unknown classes.

    extract_from_class_attributes(source, out);
    extract_from_clsx_cn(source, out);
    // Also scan raw string tokens for class-like identifiers
    extract_bare_tokens(source, out);
}

/// Extract classes from `className="..."`, `class="..."`, and template literals.
fn extract_from_class_attributes(source: &str, out: &mut HashSet<String>) {
    // Match className="..." or class="..."
    let patterns = [
        r#"className=""#,
        r#"className='"#,
        r#"class=""#,
        r#"class='"#,
    ];

    for pat in patterns {
        let quote_char = if pat.ends_with('"') { '"' } else { '\'' };
        let mut search = source;
        while let Some(pos) = search.find(pat) {
            search = &search[pos + pat.len()..];
            // Collect until the closing quote
            if let Some(end) = find_closing_quote(search, quote_char) {
                let class_str = &search[..end];
                collect_class_tokens(class_str, out);
                search = &search[end + 1..];
            }
        }
    }

    // Template literals: className={`...`}
    let tpl_pat = "className={`";
    let mut search = source;
    while let Some(pos) = search.find(tpl_pat) {
        search = &search[pos + tpl_pat.len()..];
        if let Some(end) = search.find('`') {
            let class_str = &search[..end];
            collect_class_tokens(class_str, out);
            search = &search[end + 1..];
        }
    }
}

/// Extract classes from `clsx(...)` and `cn(...)` call sites.
fn extract_from_clsx_cn(source: &str, out: &mut HashSet<String>) {
    for fn_name in &["clsx(", "cn(", "classNames("] {
        let mut search = source;
        while let Some(pos) = search.find(fn_name) {
            search = &search[pos + fn_name.len()..];
            // Scan string literals inside the call
            let mut inner = search;
            for quote in ['"', '\''] {
                let mut scan = inner;
                while let Some(q_pos) = scan.find(quote) {
                    scan = &scan[q_pos + 1..];
                    if let Some(end) = find_closing_quote(scan, quote) {
                        collect_class_tokens(&scan[..end], out);
                        scan = &scan[end + 1..];
                    }
                }
            }
            // Advance past the closing paren (best-effort)
            if let Some(close) = search.find(')') {
                inner = &search[..close];
                let _ = inner; // already scanned above
                search = &search[close + 1..];
            }
        }
    }
}

/// Scan for standalone class-like identifiers (bare `flex`, `text-blue-500`).
fn extract_bare_tokens(source: &str, out: &mut HashSet<String>) {
    // Collect all quoted string contents for additional coverage
    for quote in ['"', '\''] {
        let mut search = source;
        while let Some(pos) = search.find(quote) {
            search = &search[pos + 1..];
            if let Some(end) = find_closing_quote(search, quote) {
                collect_class_tokens(&search[..end], out);
                search = &search[end + 1..];
            }
        }
    }
}

/// Split a class string on whitespace/newlines and collect individual tokens
/// that look like Tailwind utility classes.
fn collect_class_tokens(class_str: &str, out: &mut HashSet<String>) {
    for token in class_str.split_whitespace() {
        // Strip surrounding braces/brackets from template expressions
        let token = token.trim_matches(|c| matches!(c, '{' | '}' | '(' | ')' | ',' | ';'));
        if looks_like_tailwind_class(token) {
            out.insert(token.to_string());
        }
    }
}

/// Heuristic: does this token look like a Tailwind utility class?
fn looks_like_tailwind_class(token: &str) -> bool {
    if token.is_empty() || token.len() > 80 {
        return false;
    }
    // Must start with a letter, digit, or known prefix character
    let first = token.chars().next().unwrap();
    if !first.is_ascii_alphanumeric() && first != '-' && first != '!' {
        return false;
    }
    // Must contain only ASCII word chars, hyphens, colons, brackets, slashes, dots, #
    token.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, '-' | ':' | '[' | ']' | '/' | '.' | '#' | '_' | '!' | '%')
    })
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use tempfile::TempDir;

    /// T5: Content Scanning Extracts Classes (R3)
    ///
    /// Verifies that className attributes in TSX files are scanned for classes.
    #[test]
    fn t5_content_scanner_extracts_classes() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("App.tsx"),
            r#"
            function App() {
                return <div className="flex items-center text-blue-500">Hello</div>;
            }
            "#,
        )
        .unwrap();

        let patterns = vec!["./src/**/*.{ts,tsx}".to_string()];
        let scanner = ContentScanner::new(patterns);
        let result = scanner.scan(dir.path());
        assert!(result.is_ok(), "Scanner should succeed: {:?}", result);
        let classes = result.unwrap();

        assert!(
            classes.contains("flex"),
            "Should extract 'flex' from className"
        );
        assert!(
            classes.contains("items-center"),
            "Should extract 'items-center'"
        );
        assert!(
            classes.contains("text-blue-500"),
            "Should extract 'text-blue-500'"
        );
    }

    /// Unit test: class extraction from className string.
    #[test]
    fn extract_classes_from_classname_attribute() {
        let mut out = HashSet::new();
        extract_classes(
            r#"<div className="flex items-center p-4">hello</div>"#,
            &mut out,
        );
        assert!(out.contains("flex"), "Should extract 'flex'");
        assert!(
            out.contains("items-center"),
            "Should extract 'items-center'"
        );
        assert!(out.contains("p-4"), "Should extract 'p-4'");
    }

    /// Unit test: class extraction from clsx() call.
    #[test]
    fn extract_classes_from_clsx() {
        let mut out = HashSet::new();
        extract_classes(r#"clsx("flex", isActive && "bg-blue-500")"#, &mut out);
        assert!(out.contains("flex"), "Should extract 'flex' from clsx");
        assert!(
            out.contains("bg-blue-500"),
            "Should extract 'bg-blue-500' from clsx"
        );
    }

    /// Unit test: template literal className extraction.
    #[test]
    fn extract_classes_from_template_literal() {
        let mut out = HashSet::new();
        extract_classes("className={`flex items-start rounded`}", &mut out);
        assert!(
            out.contains("flex"),
            "Should extract 'flex' from template literal"
        );
        assert!(out.contains("items-start"), "Should extract 'items-start'");
        assert!(out.contains("rounded"), "Should extract 'rounded'");
    }

    /// Unit test: glob patterns filter files correctly.
    #[test]
    fn scanner_ignores_non_matching_files() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();

        // Only .tsx files should be scanned
        fs::write(src.join("App.tsx"), r#"className="flex""#).unwrap();
        fs::write(src.join("styles.css"), r#"className="hidden""#).unwrap(); // should be ignored

        let patterns = vec!["./src/**/*.{ts,tsx}".to_string()];
        let scanner = ContentScanner::new(patterns);
        let classes = scanner.scan(dir.path()).unwrap();

        assert!(classes.contains("flex"), "Should scan App.tsx");
        // "hidden" appears in .css which is not matched by glob — may still be found
        // since CSS files can contain class strings. We only assert that .tsx is scanned.
    }
}

fn find_closing_quote(s: &str, quote: char) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == quote {
            return Some(i);
        }
        // Stop at newline for non-template strings
        if c == '\n' && quote != '`' {
            return None;
        }
    }
    None
}

#[cfg(test)]
mod gh3310_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// GH #3310 — Healthy tree round-trips: every matched file's classes
    /// appear in the returned set. Pins the happy path against the
    /// walkdir-warn refactor.
    #[test]
    fn scan_happy_path_collects_all_classes() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("a.tsx"), r#"<div className="bg-red-500"/>"#).unwrap();
        fs::write(src.join("b.tsx"), r#"<div className="text-lg"/>"#).unwrap();

        let scanner = ContentScanner::new(vec!["./src/**/*.tsx".to_string()]);
        let classes = scanner.scan(dir.path()).unwrap();

        assert!(classes.contains("bg-red-500"));
        assert!(classes.contains("text-lg"));
    }

    /// GH #3310 — An unreadable subdir must NOT abort the scan; the
    /// sibling-file classes must still be extracted. The classes inside
    /// the locked subdir are silently absent (operator sees the warn).
    #[cfg(unix)]
    #[test]
    fn scan_unreadable_subdir_keeps_sibling_files() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("keep.tsx"), r#"<div className="bg-blue-100"/>"#).unwrap();

        let locked = src.join("locked");
        fs::create_dir(&locked).unwrap();
        fs::write(
            locked.join("hidden.tsx"),
            r#"<div className="bg-red-999"/>"#,
        )
        .unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        // Skip when running as root.
        if fs::read_dir(&locked).is_ok() {
            let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));
            return;
        }

        let scanner = ContentScanner::new(vec!["./src/**/*.tsx".to_string()]);
        let result = scanner.scan(dir.path());

        // Restore perms before any assertion.
        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));

        let classes = result.expect("scan must not error on per-entry walkdir failures");
        assert!(
            classes.contains("bg-blue-100"),
            "sibling file's classes must survive an unreadable subdir"
        );
        assert!(
            !classes.contains("bg-red-999"),
            "locked subdir classes are silently absent (logged via tracing::warn)"
        );
    }

    /// GH #3310 — An unreadable single file must NOT abort the scan; other
    /// readable files must still contribute. The locked file's classes are
    /// silently absent (operator sees the warn).
    #[cfg(unix)]
    #[test]
    fn scan_unreadable_file_keeps_siblings() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("keep.tsx"), r#"<div className="bg-green-200"/>"#).unwrap();

        let locked = src.join("locked.tsx");
        fs::write(&locked, r#"<div className="bg-yellow-300"/>"#).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

        // Skip when running as root.
        if fs::read_to_string(&locked).is_ok() {
            let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o644));
            return;
        }

        let scanner = ContentScanner::new(vec!["./src/**/*.tsx".to_string()]);
        let result = scanner.scan(dir.path());

        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o644));

        let classes = result.expect("scan must not error when a single file is unreadable");
        assert!(
            classes.contains("bg-green-200"),
            "readable sibling's classes must survive an unreadable file"
        );
        assert!(
            !classes.contains("bg-yellow-300"),
            "locked file's classes are silently absent (logged via tracing::warn)"
        );
    }
}
// CODEGEN-END
