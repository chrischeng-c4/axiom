// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! CSS bundling: resolve @import and concatenate CSS files.
//!
//! Basic CSS bundler that inlines @import statements by reading
//! referenced files and concatenating them in dependency order.

use std::path::{Path, PathBuf};

/// GH #3817 — fallback stem used when an inlined CSS asset path has no
/// `file_stem()` (e.g., a `..` path). Kept as a named constant so call
/// sites and tests pin the same value.
pub(crate) const CSS_BUNDLE_ASSET_STEM_FALLBACK: &str = "asset";

/// GH #3817 — fallback extension used when an inlined CSS asset path
/// has no `extension()`. Empty string flips the hashed-filename builder
/// onto the `stem.hash` (no-extension) branch, preserving legacy
/// behaviour.
pub(crate) const CSS_BUNDLE_ASSET_EXT_FALLBACK: &str = "";

/// GH #3817 — warn shown when `css_bundle` builds a hashed filename
/// for a CSS-referenced asset whose `file_stem()` is non-UTF-8. The
/// prior code silently dropped to `"asset"`, collapsing every non-UTF-8
/// stem variant onto the same filename pattern and losing the user's
/// original filename signal.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn format_css_bundle_asset_non_utf8_stem_warn(path: &Path, lossy: &str) -> String {
    format!(
        "gh3817 css_bundle saw inlined CSS asset path with non-UTF-8 file_stem path={:?}; \
         lossy form is {:?}; routing through the lossy form so distinct non-UTF-8 stems \
         do not collapse onto the {:?} fallback in dist/",
        path, lossy, CSS_BUNDLE_ASSET_STEM_FALLBACK
    )
}

/// GH #3817 — warn shown when `css_bundle` builds a hashed filename
/// for a CSS-referenced asset whose `extension()` is non-UTF-8. The
/// prior code silently dropped to `""`, collapsing non-UTF-8 extensions
/// onto the no-extension branch (`stem.hash`) — the asset is then
/// written without an extension and the browser cannot guess the
/// content type.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn format_css_bundle_asset_non_utf8_ext_warn(path: &Path, lossy: &str) -> String {
    format!(
        "gh3817 css_bundle saw inlined CSS asset path with non-UTF-8 extension path={:?}; \
         lossy form is {:?}; routing through the lossy form so the hashed asset preserves \
         the extension instead of collapsing onto the no-extension `stem.hash` form",
        path, lossy
    )
}

/// GH #3817 — coerce a CSS-asset path's file_stem into a string for
/// the hashed-filename builder. Three-way branch:
/// - `None` (no `file_stem()`) → silent `Cow::Borrowed("asset")`
///   (legitimate degenerate path);
/// - `Some(utf8)` → silent `Cow::Borrowed(utf8)`;
/// - `Some(non-UTF-8)` → gh3817 warn + `Cow::Owned(lossy)`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn coerce_css_bundle_asset_stem_or_warn(path: &Path) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    match path.file_stem() {
        None => Cow::Borrowed(CSS_BUNDLE_ASSET_STEM_FALLBACK),
        Some(os) => match os.to_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                let lossy = os.to_string_lossy().into_owned();
                tracing::warn!(
                    target: "jet::bundler::css",
                    path = %path.display(),
                    lossy = %lossy,
                    "{}",
                    format_css_bundle_asset_non_utf8_stem_warn(path, &lossy)
                );
                Cow::Owned(lossy)
            }
        },
    }
}

/// GH #3817 — coerce a CSS-asset path's extension into a string for
/// the hashed-filename builder. Three-way branch:
/// - `None` (no extension) → silent `Cow::Borrowed("")` (legitimate);
/// - `Some(utf8)` → silent `Cow::Borrowed(utf8)`;
/// - `Some(non-UTF-8)` → gh3817 warn + `Cow::Owned(lossy)`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub(crate) fn coerce_css_bundle_asset_ext_or_warn(path: &Path) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    match path.extension() {
        None => Cow::Borrowed(CSS_BUNDLE_ASSET_EXT_FALLBACK),
        Some(os) => match os.to_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                let lossy = os.to_string_lossy().into_owned();
                tracing::warn!(
                    target: "jet::bundler::css",
                    path = %path.display(),
                    lossy = %lossy,
                    "{}",
                    format_css_bundle_asset_non_utf8_ext_warn(path, &lossy)
                );
                Cow::Owned(lossy)
            }
        },
    }
}

/// Bundle CSS starting from an entry CSS file.
///
/// Resolves @import statements recursively and concatenates all CSS
/// into a single output string. Handles circular imports by tracking visited files.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn bundle_css(entry: &Path) -> std::io::Result<String> {
    let mut visited = std::collections::HashSet::new();
    let mut output = String::new();
    resolve_css_imports(entry, &mut visited, &mut output)?;
    Ok(output)
}

/// Bundle CSS from source string with a base directory for resolving imports.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn bundle_css_from_source(source: &str, base_dir: &Path) -> std::io::Result<String> {
    let mut visited = std::collections::HashSet::new();
    let mut output = String::new();
    process_css_source(source, base_dir, &mut visited, &mut output)?;
    Ok(output)
}

/// Recursively resolve @import in a CSS file.
///
/// GH #3132 — the circular-import guard is only correct when both
/// sides of the comparison are in canonical form. The previous code
/// swallowed a `canonicalize` failure via `unwrap_or_else(|_| ...)`,
/// inserting the raw (potentially `..`-bearing) input path into the
/// visited set and letting two textually-different paths to the same
/// file recurse without termination. Same bug pattern (and same fix
/// shape) as the dev-server-side fix in #3114; the two CSS pipelines
/// were independently authored, so each copy needs its own patch.
fn resolve_css_imports(
    file: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
    output: &mut String,
) -> std::io::Result<()> {
    let canonical = file.canonicalize().map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("Cannot canonicalize CSS import {:?}: {e} (GH #3132)", file),
        )
    })?;
    if visited.contains(&canonical) {
        return Ok(());
    }
    visited.insert(canonical);

    // SCSS/Sass compile step: a `.scss`/`.sass` file is compiled to CSS via
    // grass (pure-Rust Sass) BEFORE inlining, so Sass nesting/variables/
    // mixins and its own `@use`/`@import` partials are flattened. Plain
    // `.css` reads straight from disk. The compiled CSS then flows through
    // the same `@import` inlining as before.
    let source = if crate::css::scss::is_sass_family_path(file) {
        crate::css::scss::compile_sass_file(file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
    } else {
        std::fs::read_to_string(file)?
    };
    let base_dir = file.parent().unwrap_or(Path::new("."));

    process_css_source(&source, base_dir, visited, output)
}

/// Process CSS source, resolving @import inline.
fn process_css_source(
    source: &str,
    base_dir: &Path,
    visited: &mut std::collections::HashSet<PathBuf>,
    output: &mut String,
) -> std::io::Result<()> {
    for line in source.lines() {
        let trimmed = line.trim();

        if let Some(import_path) = extract_css_import(trimmed) {
            let resolved = base_dir.join(&import_path);
            if resolved.exists() {
                resolve_css_imports(&resolved, visited, output)?;
            } else {
                // Keep unresolvable @import as-is
                output.push_str(line);
                output.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    Ok(())
}

/// Extract the path from a CSS @import statement.
///
/// Handles:
/// - `@import "path.css";`
/// - `@import 'path.css';`
/// - `@import url("path.css");`
/// - `@import url('path.css');`
/// - `@import url(path.css);`
fn extract_css_import(line: &str) -> Option<String> {
    if !line.starts_with("@import ") {
        return None;
    }

    let rest = line["@import ".len()..].trim();

    // @import url(...)
    if let Some(url_content) = rest.strip_prefix("url(") {
        let end = url_content.find(')')?;
        let inner = url_content[..end].trim();
        // Strip quotes if present
        let path = strip_quotes(inner);
        // Skip URLs (http://, https://, //)
        if is_remote(path) {
            return None;
        }
        return Some(path.to_string());
    }

    // @import "..." or @import '...'
    let stripped = rest.trim_end_matches(';').trim();
    let path = strip_quotes(stripped);
    if path == stripped {
        // No quotes found — skip
        return None;
    }
    if is_remote(path) {
        return None;
    }

    Some(path.to_string())
}

/// Strip surrounding quotes from a string.
fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Check if a path is a remote URL.
fn is_remote(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://") || path.starts_with("//")
}

/// Result of CSS URL rewriting.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct CssRewriteResult {
    /// The rewritten CSS content.
    pub css: String,
    /// Assets discovered and processed (path -> hashed filename).
    pub discovered_assets: Vec<RewrittenAsset>,
}

/// An asset that was discovered and rewritten in CSS.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone)]
pub struct RewrittenAsset {
    /// Original resolved path of the asset.
    pub original_path: PathBuf,
    /// Hashed output filename (e.g. "logo.abc12345.svg").
    pub hashed_filename: String,
}

/// Rewrite `url()` references in CSS to point to hashed asset paths.
///
/// Scans the CSS for `url(...)` references, resolves them relative to
/// `css_dir`, computes content hashes, and rewrites the URLs to
/// `{asset_prefix}/{stem}.{hash}.{ext}`.
///
/// Handles:
/// - Quoted URLs: `url("path")`, `url('path')`
/// - Unquoted URLs: `url(path)`
/// - URLs with query strings: `url(path?v=1)` (query is stripped)
/// - Remote URLs (http://, https://, //, data:) are skipped
///
/// `asset_prefix` is the path prefix for rewritten URLs (e.g. "assets").
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn rewrite_css_urls(css: &str, css_dir: &Path, asset_prefix: &str) -> CssRewriteResult {
    let mut result = String::with_capacity(css.len());
    let mut discovered = Vec::new();
    let chars: Vec<char> = css.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Look for url( — case insensitive
        if i + 3 < len
            && (chars[i] == 'u' || chars[i] == 'U')
            && (chars[i + 1] == 'r' || chars[i + 1] == 'R')
            && (chars[i + 2] == 'l' || chars[i + 2] == 'L')
            && chars[i + 3] == '('
        {
            let url_start = i;
            i += 4; // skip "url("

            // Skip whitespace
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            // Determine if quoted
            let quote_char = if i < len && (chars[i] == '"' || chars[i] == '\'') {
                let q = chars[i];
                i += 1;
                Some(q)
            } else {
                None
            };

            // Read the URL value
            let value_start = i;
            while i < len {
                if let Some(q) = quote_char {
                    if chars[i] == q {
                        break;
                    }
                } else if chars[i] == ')' || chars[i].is_whitespace() {
                    break;
                }
                i += 1;
            }
            let url_value: String = chars[value_start..i].iter().collect();

            // Skip closing quote
            if quote_char.is_some() && i < len {
                i += 1;
            }

            // Skip whitespace before closing paren
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            // Skip closing paren
            if i < len && chars[i] == ')' {
                i += 1;
            }

            // Determine if we should rewrite this URL
            let trimmed_url = url_value.trim();
            if is_remote(trimmed_url)
                || trimmed_url.starts_with("data:")
                || trimmed_url.starts_with('#')
                || trimmed_url.is_empty()
            {
                // Copy original url() verbatim
                let original: String = chars[url_start..i].iter().collect();
                result.push_str(&original);
                continue;
            }

            // Strip query string for file resolution
            let clean_path = trimmed_url.split('?').next().unwrap_or(trimmed_url);
            let clean_path = clean_path.split('#').next().unwrap_or(clean_path);

            // Resolve relative to CSS file directory
            let resolved = css_dir.join(clean_path);

            if resolved.exists() {
                // GH #3207 — distinguish race-with-deletion (NotFound after
                // .exists()) from real IO failures; the latter surface via
                // tracing::warn! so the bundle 404 has a breadcrumb.
                match std::fs::read(&resolved) {
                    Ok(content) => {
                        let hash = compute_asset_hash(&content);
                        let stem = resolved
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("asset");
                        let ext = resolved.extension().and_then(|e| e.to_str()).unwrap_or("");
                        let hashed_filename = if ext.is_empty() {
                            format!("{}.{}", stem, hash)
                        } else {
                            format!("{}.{}.{}", stem, hash, ext)
                        };
                        let rewritten_url = format!("{}/{}", asset_prefix, hashed_filename);

                        result.push_str(&format!("url({})", rewritten_url));

                        discovered.push(RewrittenAsset {
                            original_path: resolved,
                            hashed_filename,
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        // Lost a race with deletion between .exists() and
                        // read(); keep the original URL silently — matches
                        // the truly-doesn't-exist fallback below.
                        let original: String = chars[url_start..i].iter().collect();
                        result.push_str(&original);
                    }
                    Err(e) => {
                        tracing::warn!(
                            target: "jet::bundler::css",
                            "GH #3207 failed to read CSS asset {}: {} — \
                             keeping original URL; the bundled page will 404 \
                             this asset at runtime",
                            resolved.display(),
                            e,
                        );
                        let original: String = chars[url_start..i].iter().collect();
                        result.push_str(&original);
                    }
                }
            } else {
                // File doesn't exist, keep original
                let original: String = chars[url_start..i].iter().collect();
                result.push_str(&original);
            }
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    CssRewriteResult {
        css: result,
        discovered_assets: discovered,
    }
}

/// Compute a short content hash for an asset.
fn compute_asset_hash(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_css_import_double_quotes() {
        assert_eq!(
            extract_css_import(r#"@import "base.css";"#),
            Some("base.css".to_string())
        );
    }

    #[test]
    fn test_extract_css_import_single_quotes() {
        assert_eq!(
            extract_css_import("@import 'reset.css';"),
            Some("reset.css".to_string())
        );
    }

    #[test]
    fn test_extract_css_import_url() {
        assert_eq!(
            extract_css_import(r#"@import url("theme.css");"#),
            Some("theme.css".to_string())
        );
    }

    #[test]
    fn test_extract_css_import_url_no_quotes() {
        assert_eq!(
            extract_css_import("@import url(vars.css);"),
            Some("vars.css".to_string())
        );
    }

    #[test]
    fn test_skip_remote_import() {
        assert_eq!(
            extract_css_import(r#"@import "https://fonts.googleapis.com/css";"#),
            None
        );
    }

    #[test]
    fn test_not_an_import() {
        assert_eq!(extract_css_import("body { color: red; }"), None);
    }

    #[test]
    fn test_bundle_css_from_source() {
        let source = "body { color: red; }\n.app { margin: 0; }\n";
        let result = bundle_css_from_source(source, Path::new(".")).unwrap();
        assert!(result.contains("body { color: red; }"));
        assert!(result.contains(".app { margin: 0; }"));
    }

    #[test]
    fn test_bundle_css_with_import() {
        let dir = tempfile::tempdir().unwrap();
        let base_css = dir.path().join("base.css");
        let main_css = dir.path().join("main.css");

        let mut f = std::fs::File::create(&base_css).unwrap();
        writeln!(f, "* {{ margin: 0; }}").unwrap();

        let mut f = std::fs::File::create(&main_css).unwrap();
        writeln!(f, "@import \"base.css\";").unwrap();
        writeln!(f, "body {{ color: red; }}").unwrap();

        let result = bundle_css(&main_css).unwrap();
        assert!(result.contains("margin: 0"));
        assert!(result.contains("color: red"));
    }

    #[test]
    fn test_circular_import_protection() {
        let dir = tempfile::tempdir().unwrap();
        let a_css = dir.path().join("a.css");
        let b_css = dir.path().join("b.css");

        std::fs::write(&a_css, "@import \"b.css\";\n.a { color: red; }\n").unwrap();
        std::fs::write(&b_css, "@import \"a.css\";\n.b { color: blue; }\n").unwrap();

        // Should not infinite loop
        let result = bundle_css(&a_css).unwrap();
        assert!(result.contains(".a { color: red; }"));
        assert!(result.contains(".b { color: blue; }"));
    }

    /// GH #3132 — when `canonicalize` fails on an inner import path,
    /// the resolver must surface a typed `Err` rather than silently
    /// inserting the raw (potentially `..`-bearing) path into the
    /// `visited` set and risking non-terminating recursion. Same
    /// shape as the dev-server-side regression test added in #3114
    /// (`resolve_file_surfaces_canonicalize_error_for_missing_path`).
    ///
    /// We test `resolve_css_imports` directly with a path that does
    /// not exist; `canonicalize` returns ENOENT, and the resolver
    /// must propagate it as an IO error tagged with the GH issue.
    #[test]
    fn resolve_css_imports_surfaces_canonicalize_error_for_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("never-was.css");
        let mut visited = std::collections::HashSet::new();
        let mut output = String::new();
        let err = resolve_css_imports(&missing, &mut visited, &mut output)
            .expect_err("missing CSS file must not silently fall back to a raw path");
        let msg = err.to_string();
        assert!(
            msg.contains("Cannot canonicalize CSS import") && msg.contains("3132"),
            "expected canonicalize error tagged with #3132, got: {msg}"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // CSS URL rewriting tests (R14 / T21)
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_css_url_rewrite_basic() {
        // T21: Rewrite url() references to hashed asset paths
        let dir = tempfile::tempdir().unwrap();
        let img_dir = dir.path().join("img");
        std::fs::create_dir_all(&img_dir).unwrap();

        // Create a test SVG file
        let svg_content = "<svg><rect/></svg>";
        let svg_path = img_dir.join("logo.svg");
        std::fs::write(&svg_path, svg_content).unwrap();

        let css = "background: url(../img/logo.svg);\n";
        // CSS is in a subdirectory relative to the image
        let css_dir = dir.path().join("css");
        std::fs::create_dir_all(&css_dir).unwrap();

        let result = rewrite_css_urls(css, &css_dir, "assets");

        assert!(
            result.css.contains("url(assets/logo."),
            "URL should be rewritten to hashed path, got: {}",
            result.css
        );
        assert!(
            result.css.contains(".svg)"),
            "Extension should be preserved, got: {}",
            result.css
        );
        assert_eq!(
            result.discovered_assets.len(),
            1,
            "Should discover one asset"
        );
        assert!(
            result.discovered_assets[0]
                .hashed_filename
                .starts_with("logo."),
            "Hashed filename should start with 'logo.'"
        );
        assert!(
            result.discovered_assets[0]
                .hashed_filename
                .ends_with(".svg"),
            "Hashed filename should end with '.svg'"
        );
    }

    #[test]
    fn test_css_url_rewrite_quoted() {
        let dir = tempfile::tempdir().unwrap();
        let font_path = dir.path().join("font.woff2");
        std::fs::write(&font_path, b"fake font data").unwrap();

        let css = r#"src: url("font.woff2");"#;
        let result = rewrite_css_urls(css, dir.path(), "assets");

        assert!(
            result.css.contains("url(assets/font."),
            "Quoted URL should be rewritten, got: {}",
            result.css
        );
    }

    #[test]
    fn test_css_url_rewrite_single_quotes() {
        let dir = tempfile::tempdir().unwrap();
        let img_path = dir.path().join("bg.png");
        std::fs::write(&img_path, b"fake png data").unwrap();

        let css = "background: url('bg.png');";
        let result = rewrite_css_urls(css, dir.path(), "assets");

        assert!(
            result.css.contains("url(assets/bg."),
            "Single-quoted URL should be rewritten, got: {}",
            result.css
        );
    }

    #[test]
    fn test_css_url_rewrite_skip_remote() {
        let css = "background: url(https://example.com/img.png);";
        let result = rewrite_css_urls(css, Path::new("."), "assets");

        assert!(
            result.css.contains("https://example.com/img.png"),
            "Remote URLs should be left unchanged, got: {}",
            result.css
        );
        assert!(
            result.discovered_assets.is_empty(),
            "No assets should be discovered for remote URLs"
        );
    }

    #[test]
    fn test_css_url_rewrite_skip_data_uri() {
        let css = "background: url(data:image/png;base64,abc123);";
        let result = rewrite_css_urls(css, Path::new("."), "assets");

        assert!(
            result.css.contains("data:image/png"),
            "Data URIs should be left unchanged, got: {}",
            result.css
        );
    }

    #[test]
    fn test_css_url_rewrite_query_string() {
        let dir = tempfile::tempdir().unwrap();
        let font_path = dir.path().join("icon.woff");
        std::fs::write(&font_path, b"font data here").unwrap();

        let css = "src: url(icon.woff?v=1.2);";
        let result = rewrite_css_urls(css, dir.path(), "assets");

        assert!(
            result.css.contains("url(assets/icon."),
            "URL with query string should be rewritten, got: {}",
            result.css
        );
    }

    #[test]
    fn test_css_url_rewrite_missing_file() {
        let css = "background: url(nonexistent.png);";
        let result = rewrite_css_urls(css, Path::new("/tmp"), "assets");

        assert!(
            result.css.contains("url(nonexistent.png)"),
            "Missing files should keep original URL, got: {}",
            result.css
        );
    }

    #[test]
    fn test_css_url_rewrite_multiple() {
        let dir = tempfile::tempdir().unwrap();
        let img1 = dir.path().join("a.png");
        let img2 = dir.path().join("b.svg");
        std::fs::write(&img1, b"png data").unwrap();
        std::fs::write(&img2, b"svg data").unwrap();

        let css = "bg1: url(a.png); bg2: url(b.svg);";
        let result = rewrite_css_urls(css, dir.path(), "assets");

        assert_eq!(
            result.discovered_assets.len(),
            2,
            "Should discover two assets"
        );
        assert!(
            result.css.contains("url(assets/a."),
            "First URL should be rewritten"
        );
        assert!(
            result.css.contains("url(assets/b."),
            "Second URL should be rewritten"
        );
    }

    /// GH #3207 — when an asset file is referenced from CSS but cannot be
    /// read at bundle time (Unix permission 0 here), `rewrite_css_urls`
    /// MUST NOT panic and MUST NOT silently discover the asset. The
    /// original URL is kept verbatim (matching the pre-fix
    /// runtime-404 behaviour) but the failure is logged so a developer
    /// hunting a missing asset has a breadcrumb back to the read error.
    #[cfg(unix)]
    #[test]
    fn rewrite_css_urls_unreadable_asset_keeps_original_and_does_not_discover() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let img = dir.path().join("locked.png");
        std::fs::write(&img, b"png data").unwrap();
        // Strip all permissions so `std::fs::read` returns PermissionDenied.
        std::fs::set_permissions(&img, std::fs::Permissions::from_mode(0o000)).unwrap();

        let css = "bg: url(locked.png);";
        let result = rewrite_css_urls(css, dir.path(), "assets");

        // The unreadable file must NOT be reported as discovered; otherwise
        // the bundler would copy a zero-byte/error placeholder into the
        // shipped bundle and the CSS would still reference an unhashed URL.
        assert!(
            result.discovered_assets.is_empty(),
            "unreadable asset must not be discovered; got {:?}",
            result.discovered_assets
        );
        // The original url() literal is preserved so the surrounding CSS
        // remains valid — same fallback as the truly-missing-file branch.
        assert!(
            result.css.contains("url(locked.png)"),
            "original URL must be preserved when read fails; got CSS: {}",
            result.css
        );

        // Restore permissions so tempdir cleanup succeeds.
        let _ = std::fs::set_permissions(&img, std::fs::Permissions::from_mode(0o644));
    }

    /// GH #3207 — when the referenced asset is missing entirely (no
    /// .exists()), `rewrite_css_urls` must continue to fall through
    /// silently. This pins the contract so future authors don't widen
    /// the warn path to swallow the "asset not yet built" case.
    #[test]
    fn rewrite_css_urls_missing_asset_falls_through_silently() {
        let dir = tempfile::tempdir().unwrap();
        let css = "bg: url(never-existed.png);";
        let result = rewrite_css_urls(css, dir.path(), "assets");

        assert!(
            result.discovered_assets.is_empty(),
            "no asset should be discovered for a missing file"
        );
        assert!(
            result.css.contains("url(never-existed.png)"),
            "missing asset must keep original URL verbatim; got CSS: {}",
            result.css
        );
    }
}

#[cfg(test)]
mod gh3817_css_bundle_asset_non_utf8_warn_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn utf8_stem_borrows_silently() {
        let cow = coerce_css_bundle_asset_stem_or_warn(Path::new("/foo/bar.png"));
        assert_eq!(cow.as_ref(), "bar");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn utf8_ext_borrows_silently() {
        let cow = coerce_css_bundle_asset_ext_or_warn(Path::new("/foo/bar.png"));
        assert_eq!(cow.as_ref(), "png");
        assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn no_stem_falls_back_to_named_constant() {
        // Path ending in `..` has no file_stem
        let cow = coerce_css_bundle_asset_stem_or_warn(Path::new(".."));
        assert_eq!(cow.as_ref(), CSS_BUNDLE_ASSET_STEM_FALLBACK);
        assert_eq!(cow.as_ref(), "asset");
    }

    #[test]
    fn no_ext_falls_back_to_named_constant() {
        let cow = coerce_css_bundle_asset_ext_or_warn(Path::new("/foo/noext"));
        assert_eq!(cow.as_ref(), CSS_BUNDLE_ASSET_EXT_FALLBACK);
        assert_eq!(cow.as_ref(), "");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_stem_produces_lossy_form_not_fallback() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let raw = b"foo/\xffbad.png";
        let path = std::path::PathBuf::from(OsStr::from_bytes(raw));
        let cow = coerce_css_bundle_asset_stem_or_warn(&path);
        assert_ne!(
            cow.as_ref(),
            CSS_BUNDLE_ASSET_STEM_FALLBACK,
            "non-UTF-8 stem must not collapse onto fallback"
        );
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_ext_produces_lossy_form_not_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let raw = b"foo/bar.\xffweird";
        let path = std::path::PathBuf::from(OsStr::from_bytes(raw));
        let cow = coerce_css_bundle_asset_ext_or_warn(&path);
        assert!(
            !cow.as_ref().is_empty(),
            "non-UTF-8 ext must not collapse to empty"
        );
        assert!(matches!(cow, std::borrow::Cow::Owned(_)));
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_stems_do_not_collide() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let a = std::path::PathBuf::from(OsStr::from_bytes(b"d/\xffone.png"));
        let b = std::path::PathBuf::from(OsStr::from_bytes(b"d/\xfetwo.png"));
        let ca = coerce_css_bundle_asset_stem_or_warn(&a).into_owned();
        let cb = coerce_css_bundle_asset_stem_or_warn(&b).into_owned();
        assert_ne!(ca, cb, "distinct non-UTF-8 stems must remain distinct");
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let _: fn(&Path, &str) -> String = format_css_bundle_asset_non_utf8_stem_warn;
        let _: fn(&Path, &str) -> String = format_css_bundle_asset_non_utf8_ext_warn;
        let _: fn(&Path) -> std::borrow::Cow<'_, str> = coerce_css_bundle_asset_stem_or_warn;
        let _: fn(&Path) -> std::borrow::Cow<'_, str> = coerce_css_bundle_asset_ext_or_warn;
        assert_eq!(CSS_BUNDLE_ASSET_STEM_FALLBACK, "asset");
        assert_eq!(CSS_BUNDLE_ASSET_EXT_FALLBACK, "");
    }

    #[test]
    fn each_warn_string_carries_gh3817_tag() {
        let p = Path::new("a.png");
        let stem = format_css_bundle_asset_non_utf8_stem_warn(p, "lossy");
        let ext = format_css_bundle_asset_non_utf8_ext_warn(p, "lossy");
        assert!(stem.contains("gh3817"), "stem warn lacks tag: {stem}");
        assert!(ext.contains("gh3817"), "ext warn lacks tag: {ext}");
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let p = Path::new("a.png");
        let stem = format_css_bundle_asset_non_utf8_stem_warn(p, "x");
        let ext = format_css_bundle_asset_non_utf8_ext_warn(p, "x");
        for prior in [
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799", "gh3801", "gh3803",
            "gh3805", "gh3807", "gh3809", "gh3811", "gh3813", "gh3815",
        ] {
            assert!(
                !stem.contains(prior),
                "stem warn collides with {prior}: {stem}"
            );
            assert!(
                !ext.contains(prior),
                "ext warn collides with {prior}: {ext}"
            );
        }
    }

    #[test]
    fn two_sibling_warns_are_mutually_distinct() {
        let p = Path::new("a.png");
        let stem = format_css_bundle_asset_non_utf8_stem_warn(p, "x");
        let ext = format_css_bundle_asset_non_utf8_ext_warn(p, "x");
        assert_ne!(stem, ext);
        assert!(stem.contains("file_stem"));
        assert!(ext.contains("extension"));
    }
}
// CODEGEN-END
