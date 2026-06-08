// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
//! Source code analysis helpers for the dev server.
//!
//! Provides import scanning, HMR accept detection, error location extraction,
//! JSX component heuristics, and code frame building.

/// Extract import specifiers from transformed JavaScript source.
///
/// Handles both single-line and multi-line import statements.
/// Returns URL-style paths (starting with `/` or `.`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn extract_imports_from_source(code: &str) -> Vec<String> {
    let mut imports = Vec::new();

    // Normalize multi-line imports by joining lines within import statements.
    // We accumulate characters from `import` until we hit a semicolon or a `from '...'`
    let mut chars = code.chars().peekable();
    let mut buf = String::new();
    let mut in_import = false;

    while let Some(ch) = chars.next() {
        if !in_import {
            buf.push(ch);
            // Check if we've started an import statement
            if buf.ends_with("import") {
                // Peek ahead: must be followed by whitespace, '{', or quote
                if let Some(&next) = chars.peek() {
                    if next.is_whitespace() || next == '{' || next == '\'' || next == '"' {
                        in_import = true;
                        buf.clear();
                        buf.push_str("import");
                    }
                }
            }
            // Don't let buffer grow unbounded when not in import.
            // UTF-8 safety: `buf.drain(..buf.len() - 6)` operates on
            // BYTE indices and panics if the end falls mid-codepoint
            // — which is exactly what happened on zh-TW / accented
            // TSX copy (issue #1485). Round the split point forward
            // to the nearest char boundary; the cap stays in spirit
            // (≤ 6 trailing bytes) and we never panic.
            if !in_import && buf.len() > 10 {
                trim_buf_to_tail_chars(&mut buf, 6);
            }
        } else {
            // Inside an import — collapse newlines to spaces
            if ch == '\n' || ch == '\r' {
                buf.push(' ');
            } else {
                buf.push(ch);
            }

            // End of import statement
            if ch == ';' || (ch == '\n' && buf.contains("from ")) {
                extract_import_from_statement(&buf, &mut imports);
                buf.clear();
                in_import = false;
            }
        }
    }

    // Handle case where last import has no trailing semicolon
    if in_import && !buf.is_empty() {
        extract_import_from_statement(&buf, &mut imports);
    }

    imports
}

/// Trim `buf` so the remaining suffix is at most `tail_bytes` bytes
/// AND ends on a UTF-8 char boundary. Equivalent in spirit to
/// `buf.drain(..buf.len().saturating_sub(tail_bytes))`, but rounds
/// the split point forward to the next char boundary when the
/// naive offset would land mid-codepoint — which would panic in
/// `String::drain`. Losing a few extra prefix bytes from a multi-
/// byte codepoint is harmless: the prefix is being discarded
/// anyway, and the suffix still contains every byte that could
/// possibly start the `import` keyword on the next character.
///
/// @spec projects/jet/docs/dev-server-source-analysis-utf8-safety.md#interface
/// @issue #1485
fn trim_buf_to_tail_chars(buf: &mut String, tail_bytes: usize) {
    let mut split = buf.len().saturating_sub(tail_bytes);
    // Round forward to a char boundary. `buf.len()` is always a
    // boundary, so the loop terminates.
    while !buf.is_char_boundary(split) {
        split += 1;
    }
    buf.drain(..split);
}

/// Extract the import specifier from a normalized (single-line) import statement.
fn extract_import_from_statement(stmt: &str, imports: &mut Vec<String>) {
    let trimmed = stmt.trim();

    let spec = if let Some(pos) = trimmed.rfind("from ") {
        let after = &trimmed[pos + 5..];
        extract_string_literal(after)
    } else if trimmed.starts_with("import '") || trimmed.starts_with("import \"") {
        extract_string_literal(&trimmed[7..])
    } else {
        None
    };

    if let Some(s) = spec {
        // Only track relative / absolute imports (not bare specifiers like 'react')
        if s.starts_with('.') || s.starts_with('/') {
            imports.push(s);
        }
    }
}

/// Extract a string literal value from a slice like `'./foo'` or `"./foo"`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn extract_string_literal(s: &str) -> Option<String> {
    let s = s.trim();
    let (quote, rest) = if s.starts_with('\'') {
        ('\'', &s[1..])
    } else if s.starts_with('"') {
        ('"', &s[1..])
    } else {
        return None;
    };

    rest.find(quote).map(|end| rest[..end].to_string())
}

/// Heuristic check: does the source contain React component patterns?
///
/// Looks for JSX elements or known React patterns to determine if the file
/// should be treated as having React Fast Refresh boundaries.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn source_has_react_components(source: &str) -> bool {
    // Look for JSX syntax: <Component or <div
    for line in source.lines() {
        let trimmed = line.trim();
        // JSX return statements
        if trimmed.contains("return (") || trimmed.contains("return(") {
            continue; // Will be detected by JSX tag check
        }
        // Self-closing JSX: <Foo /> or <div />
        if trimmed.contains("/>") && trimmed.contains('<') {
            return true;
        }
        // Opening JSX: <Foo> or <div>
        if trimmed.contains("</") && trimmed.contains('>') {
            return true;
        }
    }
    false
}

/// Build a code frame string for error reporting.
///
/// If `error_line` is provided, shows surrounding lines (±3 lines) with the error
/// line highlighted. Otherwise falls back to showing the first 10 lines.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn build_error_frame(source: &str, error_line: Option<u32>) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut frame = String::new();

    if let Some(line_num) = error_line {
        let line_idx = line_num.saturating_sub(1) as usize;
        let start = line_idx.saturating_sub(3);
        let end = (line_idx + 4).min(lines.len());

        for i in start..end {
            let marker = if i == line_idx { ">" } else { " " };
            frame.push_str(&format!("{} {:>4} | {}\n", marker, i + 1, lines[i]));
        }
    } else {
        // Fallback: show the first 10 lines
        for (i, line) in lines.iter().take(10).enumerate() {
            frame.push_str(&format!("  {:>4} | {}\n", i + 1, line));
        }
    }

    frame
}

/// Detect `import.meta.hot.accept()` calls in source code.
///
/// Returns `(is_self_accepting, accepted_deps)`:
/// - `is_self_accepting`: true if `import.meta.hot.accept()` or `import.meta.hot.accept(cb)` found
/// - `accepted_deps`: list of dep paths from `import.meta.hot.accept(['./dep'], cb)` calls
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn detect_hmr_accept_calls(source: &str) -> (bool, Vec<String>) {
    let mut is_self_accepting = false;
    let mut accepted_deps = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Look for import.meta.hot.accept patterns
        if !trimmed.contains("import.meta.hot") {
            continue;
        }

        if let Some(pos) = trimmed.find("import.meta.hot.accept") {
            let after = &trimmed[pos + "import.meta.hot.accept".len()..];
            let after = after.trim();

            if after.starts_with('(') {
                let inner = &after[1..];
                let inner = inner.trim();

                if inner.starts_with('[') {
                    // Dependency accept: accept(['./dep1', './dep2'], cb)
                    if let Some(bracket_end) = inner.find(']') {
                        let deps_str = &inner[1..bracket_end];
                        for dep in deps_str.split(',') {
                            let dep = dep.trim();
                            // Extract string literal
                            if let Some(s) = extract_string_literal(dep) {
                                accepted_deps.push(s);
                            }
                        }
                    }
                } else if inner.starts_with(')') || inner.starts_with("()") {
                    // Self-accepting with no callback: accept()
                    is_self_accepting = true;
                } else {
                    // Self-accepting with callback: accept((mod) => {...})
                    // or accept(function(mod) {...})
                    is_self_accepting = true;
                }
            }
        }
    }

    (is_self_accepting, accepted_deps)
}

/// Extract line and column numbers from a transform error message.
///
/// Tries several common patterns:
/// - "at line X, column Y"
/// - "line X column Y"
/// - "row X column Y" (tree-sitter, 0-based → converted to 1-based)
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn extract_error_location(error_msg: &str) -> (Option<u32>, Option<u32>) {
    let lower = error_msg.to_lowercase();

    // Pattern: "line X" or "at line X"
    if let Some(pos) = lower.find("line ") {
        let after = &error_msg[pos + 5..];
        if let Some(line_num) = parse_leading_number(after) {
            // Look for column after the line number
            let col = if let Some(col_pos) = lower[pos..].find("column ") {
                let col_after = &error_msg[pos + col_pos + 7..];
                parse_leading_number(col_after)
            } else if let Some(col_pos) = lower[pos..].find("col ") {
                let col_after = &error_msg[pos + col_pos + 4..];
                parse_leading_number(col_after)
            } else {
                None
            };
            return (Some(line_num), col);
        }
    }

    // Pattern: "row X" (tree-sitter errors)
    if let Some(pos) = lower.find("row ") {
        let after = &error_msg[pos + 4..];
        if let Some(line_num) = parse_leading_number(after) {
            let col = if let Some(col_pos) = lower[pos..].find("column ") {
                let col_after = &error_msg[pos + col_pos + 7..];
                parse_leading_number(col_after)
            } else {
                None
            };
            // tree-sitter uses 0-based rows; convert to 1-based
            return (Some(line_num + 1), col);
        }
    }

    (None, None)
}

/// Parse a leading integer from the start of a string.
fn parse_leading_number(s: &str) -> Option<u32> {
    let s = s.trim();
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// Convert a filesystem path to a URL path relative to the project root.
///
/// GH #3588 — when `path` is not under `root`, the prior implementation
/// silently emitted the *absolute filesystem path* as a URL (e.g.
/// `/Users/chris/project/foo`). That leaks the absolute path into the
/// browser response and the downstream 404 has no breadcrumb back to
/// the strip_prefix failure. Two changes:
///
/// 1. Fall back to `path.file_name()` only — never leak the absolute
///    path. The resulting URL is still useful for debugging while
///    refusing to expose the host filesystem layout.
/// 2. Emit a `tracing::warn!` tagged `GH #3588` that names both `path`
///    and `root` so the operator can correlate the warn with the
///    downstream 404.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn file_path_to_url(path: &std::path::Path, root: &std::path::Path) -> String {
    match path.strip_prefix(root) {
        Ok(rel) => format!("/{}", rel.to_string_lossy().replace('\\', "/")),
        Err(err) => {
            tracing::warn!(
                target: "jet::dev_server::source_analysis",
                "{}",
                format_file_path_to_url_warn(path, root, &err)
            );
            // Fall back to file_name only — never leak the absolute path.
            let bare = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            format!("/{}", bare.replace('\\', "/"))
        }
    }
}

/// GH #3588 — build the warn message for a `file_path_to_url` lookup
/// whose `path` was not under `root`. Extracted so the wording (issue
/// tag + path + root + consequence) is unit-testable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn format_file_path_to_url_warn(
    path: &std::path::Path,
    root: &std::path::Path,
    err: &std::path::StripPrefixError,
) -> String {
    format!(
        "GH #3588 file_path_to_url: path {path:?} is not under root {root:?} \
         ({err}); falling back to file_name only to avoid leaking the absolute \
         filesystem path into the dev-server URL. The downstream request will \
         likely 404 — fix the caller to pass a path under {root:?}."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_line_imports() {
        let code = r#"
import { useState } from 'react';
import { App } from './App';
import './styles.css';
import { utils } from '/src/utils';
"#;
        let imports = extract_imports_from_source(code);
        assert!(imports.contains(&"./App".to_string()));
        assert!(imports.contains(&"/src/utils".to_string()));
        // Bare specifier 'react' should be excluded
        assert!(!imports.iter().any(|s| s.contains("react")));
        // Side-effect CSS import
        assert!(imports.contains(&"./styles.css".to_string()));
    }

    #[test]
    fn extract_multi_line_imports() {
        let code = r#"
import {
  A,
  B,
  C
} from './components';
import { single } from './single';
"#;
        let imports = extract_imports_from_source(code);
        assert!(
            imports.contains(&"./components".to_string()),
            "must extract multi-line import: {:?}",
            imports
        );
        assert!(imports.contains(&"./single".to_string()));
    }

    #[test]
    fn detect_self_accepting_module() {
        let source = r#"
import { useState } from 'react';
if (import.meta.hot) {
  import.meta.hot.accept();
}
"#;
        let (is_self, deps) = detect_hmr_accept_calls(source);
        assert!(is_self, "must detect self-accepting");
        assert!(deps.is_empty());
    }

    #[test]
    fn detect_self_accepting_with_callback() {
        let source = r#"
if (import.meta.hot) {
  import.meta.hot.accept((mod) => {
    render(mod.default);
  });
}
"#;
        let (is_self, deps) = detect_hmr_accept_calls(source);
        assert!(is_self, "must detect self-accepting with callback");
        assert!(deps.is_empty());
    }

    #[test]
    fn detect_dependency_accept() {
        let source = r#"
import.meta.hot.accept(['./dep1', './dep2'], ([d1, d2]) => {
  update(d1, d2);
});
"#;
        let (is_self, deps) = detect_hmr_accept_calls(source);
        assert!(!is_self, "must not be self-accepting");
        assert_eq!(deps, vec!["./dep1".to_string(), "./dep2".to_string()]);
    }

    #[test]
    fn detect_no_accept_calls() {
        let source = r#"
import { useState } from 'react';
export function App() { return <div/>; }
"#;
        let (is_self, deps) = detect_hmr_accept_calls(source);
        assert!(!is_self);
        assert!(deps.is_empty());
    }

    #[test]
    fn extract_error_location_line_column() {
        let (line, col) = extract_error_location("Unexpected token at line 15, column 8");
        assert_eq!(line, Some(15));
        assert_eq!(col, Some(8));
    }

    #[test]
    fn extract_error_location_row() {
        // tree-sitter uses 0-based row
        let (line, col) = extract_error_location("Syntax error at row 14, column 7");
        assert_eq!(line, Some(15)); // 14 + 1 = 15
        assert_eq!(col, Some(7));
    }

    #[test]
    fn extract_error_location_none() {
        let (line, col) = extract_error_location("Something went wrong");
        assert_eq!(line, None);
        assert_eq!(col, None);
    }

    #[test]
    fn build_error_frame_with_line() {
        let source = (1..=20)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let frame = build_error_frame(&source, Some(10));
        assert!(frame.contains("> "), "must have error marker");
        assert!(frame.contains("line 10"), "must show error line");
        assert!(frame.contains("line 7"), "must show context before");
        assert!(frame.contains("line 13"), "must show context after");
    }

    #[test]
    fn build_error_frame_fallback() {
        let source = (1..=20)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let frame = build_error_frame(&source, None);
        assert!(!frame.contains("> "), "fallback must not have error marker");
        assert!(frame.contains("line 1"), "must show first lines");
        assert!(frame.contains("line 10"), "must show up to 10 lines");
        assert!(!frame.contains("line 11"), "must not show beyond 10 lines");
    }

    #[test]
    fn file_path_to_url_strips_root() {
        let root = std::path::Path::new("/project");
        let path = std::path::Path::new("/project/src/App.tsx");
        assert_eq!(file_path_to_url(path, root), "/src/App.tsx");
    }

    // ─── GH #3588: file_path_to_url leak / silent-fallback ────────────

    /// GH #3588 — when path is NOT under root the fallback URL MUST
    /// NOT leak the absolute filesystem path. The new behavior keeps
    /// only the file_name component so the URL is `/<filename>`
    /// rather than `/Users/chris/...`.
    #[test]
    fn gh3588_path_not_under_root_falls_back_to_filename_only() {
        let root = std::path::Path::new("/project");
        let path = std::path::Path::new("/Users/chris/elsewhere/foo.tsx");
        let url = file_path_to_url(path, root);
        assert_eq!(
            url, "/foo.tsx",
            "fallback must use file_name only — no absolute-path leak"
        );
        assert!(
            !url.contains("/Users/chris"),
            "URL must not contain absolute filesystem path components, got: {url:?}"
        );
        assert!(
            !url.contains("elsewhere"),
            "URL must not leak intermediate directories, got: {url:?}"
        );
    }

    /// GH #3588 — fallback for a path with no file_name (e.g. `/`)
    /// must still produce a syntactically valid URL (`"/"`) and must
    /// NOT panic.
    #[test]
    fn gh3588_path_not_under_root_no_filename_falls_back_to_slash() {
        let root = std::path::Path::new("/project");
        let path = std::path::Path::new("/");
        let url = file_path_to_url(path, root);
        assert_eq!(
            url, "/",
            "rootless path with no file_name must collapse to `/`, not leak `/`-prefixed garbage"
        );
    }

    /// GH #3588 — the warn message must name path, root, and the
    /// issue tag so a CI grep lands on this fix.
    #[test]
    fn gh3588_format_file_path_to_url_warn_names_tag_path_root() {
        let root = std::path::Path::new("/project");
        let path = std::path::Path::new("/Users/chris/elsewhere/foo.tsx");
        // Provoke a real StripPrefixError so the Display impl is exercised.
        let err = path.strip_prefix(root).unwrap_err();
        let msg = format_file_path_to_url_warn(path, root, &err);
        assert!(msg.contains("GH #3588"), "must include tag, got: {msg}");
        assert!(
            msg.contains("/Users/chris/elsewhere/foo.tsx"),
            "must name the offending path, got: {msg}"
        );
        assert!(msg.contains("/project"), "must name the root, got: {msg}");
    }

    /// GH #3588 — happy-path regression: path UNDER root continues
    /// to strip the prefix exactly as before (no behavior change).
    #[test]
    fn gh3588_happy_path_strips_prefix_unchanged() {
        let root = std::path::Path::new("/project");
        let path = std::path::Path::new("/project/src/App.tsx");
        assert_eq!(file_path_to_url(path, root), "/src/App.tsx");
    }

    // Regression for #1485: zh-TW / accented string literals in TSX
    // used to panic the buffer trimmer via `String::drain`'s
    // is_char_boundary assertion. The trim helper now rounds the
    // split point forward to the nearest char boundary.
    #[test]
    fn extract_imports_handles_non_ascii_copy_without_panic() {
        let src = "const greeting = '你好世界 안녕하세요 مرحبا Café';\n\
                   import { Foo } from './foo';\n\
                   import { Bar } from './bar';\n";
        let imports = extract_imports_from_source(src);
        assert!(
            imports.iter().any(|s| s == "./foo"),
            "expected ./foo import, got: {:?}",
            imports
        );
        assert!(
            imports.iter().any(|s| s == "./bar"),
            "expected ./bar import, got: {:?}",
            imports
        );
    }

    #[test]
    fn extract_imports_handles_long_cjk_prose_without_panic() {
        // A long run of multi-byte codepoints before any `import`
        // exercises the buffer-overflow trimmer many times in a row.
        let mut src = String::new();
        for _ in 0..200 {
            src.push_str("中文字串 ");
        }
        src.push_str("import { Baz } from './baz';\n");
        let imports = extract_imports_from_source(&src);
        assert!(
            imports.iter().any(|s| s == "./baz"),
            "expected ./baz import after CJK prose, got: {:?}",
            imports
        );
    }

    #[test]
    fn trim_buf_to_tail_chars_rounds_to_char_boundary() {
        // "abcdef" (6 bytes) + "你好" (each char is 3 bytes UTF-8 = 6 bytes)
        // Naive drain at byte 6 would split inside "你" and panic.
        let mut buf = String::from("abcdef你好");
        let original_len = buf.len();
        assert_eq!(original_len, 12);
        trim_buf_to_tail_chars(&mut buf, 6);
        assert!(
            buf.len() <= 6,
            "trimmed buf must be <= 6 bytes, got {}: {:?}",
            buf.len(),
            buf
        );
        // String invariants: every String is valid UTF-8 by
        // construction, but make the intent explicit.
        assert!(buf.is_char_boundary(0));
        assert!(buf.is_char_boundary(buf.len()));
    }

    #[test]
    fn trim_buf_to_tail_chars_ascii_only_matches_naive_drain() {
        // The fix must not change ASCII behaviour: a 12-byte ASCII
        // buffer asked to keep its last 6 bytes must end up exactly
        // 6 bytes, byte-for-byte identical to the old code path.
        let mut buf = String::from("abcdefghijkl");
        trim_buf_to_tail_chars(&mut buf, 6);
        assert_eq!(buf, "ghijkl");
    }

    #[test]
    fn trim_buf_to_tail_chars_shorter_than_tail_is_noop() {
        let mut buf = String::from("abc");
        trim_buf_to_tail_chars(&mut buf, 6);
        assert_eq!(buf, "abc");
    }
}
// CODEGEN-END
