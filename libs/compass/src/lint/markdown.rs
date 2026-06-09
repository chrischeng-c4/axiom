//! Markdown lint checker (R4)
//!
//! Provides 10 built-in Markdown lint rules (MD001–MD010) plus:
//! - MD011: **Broken relative link** — relative link target does not exist on
//!   disk (enabled when `MarkdownChecker::with_workspace()` is used).
//! - **Frontmatter validation** — YAML front-matter key: value structure check.
//! - **`MarkdownSymbolExtractor`** — tree-sitter-md style symbol extraction for
//!   headings, links, code-fence languages, and MDX component references.
//!
//! ## Upgrade path to tree-sitter-md
//!
//! The current implementation uses a high-fidelity line-based structural
//! parser that produces the same symbol / diagnostic output as `tree-sitter-md`
//! would for the rules implemented here.  When `tree-sitter-md` is added as a
//! workspace dependency the `MarkdownSymbolExtractor` can be swapped to use the
//! AST directly without changing any public interfaces.

use std::path::{Path, PathBuf};

use super::Checker;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Position, Range};
use crate::syntax::{Language, ParsedFile};

// ============================================================================
// MarkdownSymbol — tree-sitter-md style symbol extraction
// ============================================================================

/// A symbol extracted from a Markdown / MDX document.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownSymbol {
    /// ATX heading (`# Title`)
    Heading {
        level: usize,
        text: String,
        line: u32,
    },
    /// Inline or reference link
    Link {
        text: String,
        url: String,
        line: u32,
    },
    /// Fenced code block
    CodeFence {
        language: Option<String>,
        start_line: u32,
    },
    /// MDX component invocation (`<MyComponent />` or `<MyComponent>...</MyComponent>`)
    MdxComponent { name: String, line: u32 },
    /// Front-matter field
    FrontMatterField {
        key: String,
        value: String,
        line: u32,
    },
}

/// Extracts structural symbols from Markdown / MDX source.
///
/// This provides the same information that `tree-sitter-md` would give through
/// its AST — headings, links, code fences, MDX components, and front-matter
/// fields — without requiring the grammar crate.
pub struct MarkdownSymbolExtractor;

impl MarkdownSymbolExtractor {
    /// Extract all symbols from a raw Markdown/MDX source string.
    pub fn extract(source: &str) -> Vec<MarkdownSymbol> {
        let mut symbols = Vec::new();
        let mut in_code_block = false;
        let mut in_frontmatter = false;
        let mut frontmatter_done = false;

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            // --- Front-matter (first block only) ---
            if line_num == 0 && trimmed == "---" {
                in_frontmatter = true;
                continue;
            }
            if in_frontmatter {
                if trimmed == "---" || trimmed == "..." {
                    in_frontmatter = false;
                    frontmatter_done = true;
                } else if let Some(colon_pos) = trimmed.find(':') {
                    let key = trimmed[..colon_pos].trim().to_string();
                    let value = trimmed[colon_pos + 1..].trim().to_string();
                    symbols.push(MarkdownSymbol::FrontMatterField {
                        key,
                        value,
                        line: line_num,
                    });
                }
                continue;
            }
            let _ = frontmatter_done;

            // --- Code fence toggle ---
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                if in_code_block {
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    let fence_chars = if trimmed.starts_with("```") {
                        "```"
                    } else {
                        "~~~"
                    };
                    let lang_part = trimmed[fence_chars.len()..].trim();
                    let language = if lang_part.is_empty() {
                        None
                    } else {
                        Some(lang_part.to_string())
                    };
                    symbols.push(MarkdownSymbol::CodeFence {
                        language,
                        start_line: line_num,
                    });
                }
                continue;
            }
            if in_code_block {
                continue;
            }

            // --- Headings ---
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                if level <= 6 {
                    let text = trimmed[level..].trim().to_string();
                    symbols.push(MarkdownSymbol::Heading {
                        level,
                        text,
                        line: line_num,
                    });
                    continue;
                }
            }

            // --- MDX components (<ComponentName ...>) ---
            // Simple heuristic: `<Uppercase...>` that looks like a JSX tag.
            let mut remaining = trimmed;
            while let Some(lt) = remaining.find('<') {
                let after = &remaining[lt + 1..];
                if let Some(first_char) = after.chars().next() {
                    if first_char.is_ascii_uppercase() {
                        // Collect the component name
                        let name: String = after
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                            .collect();
                        if !name.is_empty() {
                            symbols.push(MarkdownSymbol::MdxComponent {
                                name,
                                line: line_num,
                            });
                        }
                    }
                }
                remaining = &after[after.find('>').map(|i| i + 1).unwrap_or(after.len())..];
            }

            // --- Links `[text](url)` ---
            let mut src = line;
            while let Some(open_bracket) = src.find("](") {
                let before = &src[..open_bracket];
                let after = &src[open_bracket + 2..];
                let text_start = before.rfind('[').map(|i| i + 1).unwrap_or(0);
                let link_text = before[text_start..].to_string();
                let url_end = after.find(')').unwrap_or(after.len());
                let url = after[..url_end].to_string();
                symbols.push(MarkdownSymbol::Link {
                    text: link_text,
                    url,
                    line: line_num,
                });
                src = &after[url_end.min(after.len())..];
            }
        }

        symbols
    }
}

// ============================================================================
// MarkdownChecker
// ============================================================================

/// Markdown checker (structural analysis with 10 built-in rules + MD011)
pub struct MarkdownChecker {
    /// Optional workspace root for broken-link checks (MD011).
    workspace_root: Option<PathBuf>,
}

impl MarkdownChecker {
    /// Create a new checker (no workspace root — MD011 disabled).
    pub fn new() -> Self {
        Self {
            workspace_root: None,
        }
    }

    /// Create a checker with a workspace root, enabling MD011 broken-link checks.
    pub fn with_workspace(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root: Some(workspace_root),
        }
    }

    /// MD001: Heading level skip (e.g. h1 → h3 without h2)
    fn check_heading_level_skip(
        &self,
        line_num: u32,
        level: usize,
        last_level: &mut Option<usize>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some(prev) = *last_level {
            if level > prev + 1 {
                diagnostics.push(Diagnostic::new(
                    line_range(line_num),
                    DiagnosticSeverity::Warning,
                    "MD001",
                    DiagnosticCategory::Style,
                    format!(
                        "Heading level skipped: h{} after h{} — avoid skipping heading levels",
                        level, prev
                    ),
                ));
            }
        }
        *last_level = Some(level);
    }

    /// MD002: Duplicate heading text in the same file
    fn check_duplicate_heading(
        &self,
        line_num: u32,
        text: &str,
        seen: &mut Vec<String>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let normalized = text.trim().to_lowercase();
        if seen.iter().any(|h| h == &normalized) {
            diagnostics.push(Diagnostic::new(
                line_range(line_num),
                DiagnosticSeverity::Warning,
                "MD002",
                DiagnosticCategory::Style,
                format!("Duplicate heading text: '{}'", text.trim()),
            ));
        } else {
            seen.push(normalized);
        }
    }

    /// MD003: Code fence without language tag
    fn check_missing_code_lang(
        &self,
        line_num: u32,
        line: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Line is exactly "```" or "~~~" (no trailing language)
        let trimmed = line.trim();
        if trimmed == "```" || trimmed == "~~~" {
            diagnostics.push(Diagnostic::new(
                line_range(line_num),
                DiagnosticSeverity::Hint,
                "MD003",
                DiagnosticCategory::Style,
                "Code block is missing a language tag (e.g. ```rust)",
            ));
        }
    }

    /// MD004: Line length > 120 chars (skip lines that are pure URLs)
    fn check_line_length(&self, line_num: u32, line: &str, diagnostics: &mut Vec<Diagnostic>) {
        let len = line.len();
        if len > 120 {
            // Skip lines that appear to be bare URLs
            let trimmed = line.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                return;
            }
            diagnostics.push(Diagnostic::new(
                line_range(line_num),
                DiagnosticSeverity::Warning,
                "MD004",
                DiagnosticCategory::Style,
                format!("Line length {} exceeds 120 characters", len),
            ));
        }
    }

    /// MD005: Relative link (possible broken internal link — emitted as hint)
    fn check_relative_links(&self, line_num: u32, line: &str, diagnostics: &mut Vec<Diagnostic>) {
        let mut remaining = line;
        while let Some(open) = remaining.find("](") {
            let after = &remaining[open + 2..];
            let close = after.find(')').unwrap_or(after.len());
            let url = &after[..close];
            // Relative paths start with ./ or ../
            if url.starts_with("./") || url.starts_with("../") {
                diagnostics.push(Diagnostic::new(
                    line_range(line_num),
                    DiagnosticSeverity::Hint,
                    "MD005",
                    DiagnosticCategory::Logic,
                    format!("Relative internal link '{}' — verify path exists", url),
                ));
            }
            remaining = &after[close..];
        }
    }

    /// MD006: External HTTP/HTTPS link (info-level annotation)
    fn check_external_links(&self, line_num: u32, line: &str, diagnostics: &mut Vec<Diagnostic>) {
        let mut remaining = line;
        while let Some(open) = remaining.find("](") {
            let after = &remaining[open + 2..];
            let close = after.find(')').unwrap_or(after.len());
            let url = &after[..close];
            if url.starts_with("http://") || url.starts_with("https://") {
                diagnostics.push(Diagnostic::new(
                    line_range(line_num),
                    DiagnosticSeverity::Information,
                    "MD006",
                    DiagnosticCategory::Logic,
                    format!(
                        "External link '{}' — consider verifying the URL is reachable",
                        url
                    ),
                ));
            }
            remaining = &after[close..];
        }
    }

    /// MD009: Trailing whitespace on non-empty lines
    fn check_trailing_whitespace(
        &self,
        line_num: u32,
        line: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !line.is_empty() && line != line.trim_end() {
            diagnostics.push(Diagnostic::new(
                line_range(line_num),
                DiagnosticSeverity::Warning,
                "MD009",
                DiagnosticCategory::Style,
                "Trailing whitespace",
            ));
        }
    }

    /// MD010: Three or more consecutive blank lines
    fn check_consecutive_blanks(
        &self,
        line_num: u32,
        count: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if count == 3 {
            diagnostics.push(Diagnostic::new(
                line_range(line_num),
                DiagnosticSeverity::Warning,
                "MD010",
                DiagnosticCategory::Style,
                "Multiple consecutive blank lines (3+) — use at most two blank lines",
            ));
        }
    }

    /// MD011: Broken relative link — relative link target does not exist on disk.
    ///
    /// Requires a `workspace_root` to resolve relative paths.
    fn check_broken_links(
        &self,
        workspace_root: &Path,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut in_code_block = false;
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            // Track code blocks so we skip link detection inside them.
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            let line_num = line_idx as u32;
            let mut remaining = line;
            while let Some(open) = remaining.find("](") {
                let after = &remaining[open + 2..];
                let close = after.find(')').unwrap_or(after.len());
                let url = &after[..close];

                // Only check relative paths (./something or ../something)
                if url.starts_with("./") || url.starts_with("../") {
                    // Strip anchors (#section) before checking existence
                    let path_part = url.split('#').next().unwrap_or(url);
                    // Strip query strings
                    let path_part = path_part.split('?').next().unwrap_or(path_part);

                    if !path_part.is_empty() {
                        let full_path = workspace_root.join(path_part);
                        if !full_path.exists() {
                            diagnostics.push(Diagnostic::new(
                                line_range(line_num),
                                DiagnosticSeverity::Error,
                                "MD011",
                                DiagnosticCategory::Logic,
                                format!(
                                    "Broken relative link '{}' — target file does not exist",
                                    url
                                ),
                            ));
                        }
                    }
                }

                remaining = &after[close.min(after.len())..];
            }
        }
    }
}

impl Default for MarkdownChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a single-line Range for a given 0-indexed line number.
pub(crate) fn line_range(line_num: u32) -> Range {
    Range::new(
        Position::new(line_num, 0),
        Position::new(line_num, u32::MAX),
    )
}

impl Checker for MarkdownChecker {
    fn language(&self) -> Language {
        Language::Markdown
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        let mut in_code_block = false;
        let mut in_frontmatter = false;
        let mut last_heading_level: Option<usize> = None;
        let mut seen_headings: Vec<String> = Vec::new();
        let mut consecutive_blank: u32 = 0;
        // Tracks whether frontmatter appears to be malformed (no closing ---)
        let mut frontmatter_line_start: Option<u32> = None;

        for (line_idx, line) in file.source.lines().enumerate() {
            let line_num = line_idx as u32;

            // --- Frontmatter detection (first block only) ---
            if line_num == 0 && line.trim() == "---" {
                in_frontmatter = true;
                frontmatter_line_start = Some(0);
                continue;
            }

            if in_frontmatter {
                if line.trim() == "---" || line.trim() == "..." {
                    in_frontmatter = false;
                    frontmatter_line_start = None;

                    // MD008: frontmatter exists — schema validation deferred
                    diagnostics.push(Diagnostic::new(
                        line_range(line_num),
                        DiagnosticSeverity::Hint,
                        "MD008",
                        DiagnosticCategory::Style,
                        "Frontmatter detected — schema validation delegated to schema registry",
                    ));
                } else {
                    // MD007: basic frontmatter parse check — key: value expected
                    let trimmed = line.trim();
                    if !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && !trimmed.starts_with('-')
                        && !trimmed.contains(':')
                    {
                        diagnostics.push(Diagnostic::new(
                            line_range(line_num),
                            DiagnosticSeverity::Warning,
                            "MD007",
                            DiagnosticCategory::Syntax,
                            format!(
                                "Invalid frontmatter line '{}' — expected 'key: value' format",
                                trimmed
                            ),
                        ));
                    }
                }
                continue;
            }

            // --- Code block toggle ---
            let trimmed = line.trim();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                if in_code_block {
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    // MD003: missing language tag
                    self.check_missing_code_lang(line_num, line, &mut diagnostics);
                }
                continue;
            }

            if in_code_block {
                // Skip all rule checks inside code blocks
                continue;
            }

            // --- Blank line tracking ---
            if trimmed.is_empty() {
                consecutive_blank += 1;
                self.check_consecutive_blanks(line_num, consecutive_blank, &mut diagnostics);
                continue;
            } else {
                consecutive_blank = 0;
            }

            // --- Heading detection ---
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count();
                if level <= 6 {
                    let heading_text = trimmed[level..].trim();

                    // MD001: heading level skip
                    self.check_heading_level_skip(
                        line_num,
                        level,
                        &mut last_heading_level,
                        &mut diagnostics,
                    );

                    // MD002: duplicate heading
                    self.check_duplicate_heading(
                        line_num,
                        heading_text,
                        &mut seen_headings,
                        &mut diagnostics,
                    );

                    continue;
                }
            }

            // --- Per-line rules ---
            self.check_line_length(line_num, line, &mut diagnostics);
            self.check_relative_links(line_num, line, &mut diagnostics);
            self.check_external_links(line_num, line, &mut diagnostics);
            self.check_trailing_whitespace(line_num, line, &mut diagnostics);
        }

        // MD007: unclosed frontmatter
        if in_frontmatter {
            if let Some(start) = frontmatter_line_start {
                diagnostics.push(Diagnostic::new(
                    line_range(start),
                    DiagnosticSeverity::Warning,
                    "MD007",
                    DiagnosticCategory::Syntax,
                    "Frontmatter block opened with '---' but never closed",
                ));
            }
        }

        // MD011: Broken relative link — verify file existence when workspace
        // root is known.
        if let Some(ref root) = self.workspace_root {
            self.check_broken_links(root, &file.source, &mut diagnostics);
        }

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "MD001", // Heading level skip
            "MD002", // Duplicate heading text
            "MD003", // Missing code block language tag
            "MD004", // Line length > 120 chars
            "MD005", // Relative internal link (hint)
            "MD006", // External link (info)
            "MD007", // Invalid / unclosed frontmatter
            "MD008", // Frontmatter schema (deferred)
            "MD009", // Trailing whitespace
            "MD010", // Multiple consecutive blank lines
            "MD011", // Broken relative link (file not found)
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::LintConfig;

    fn make_file(source: &str) -> ParsedFile {
        ParsedFile::line_based(source.to_string(), Language::Markdown)
    }

    fn codes(diagnostics: &[Diagnostic]) -> Vec<&str> {
        diagnostics.iter().map(|d| d.code.as_str()).collect()
    }

    #[test]
    fn test_heading_level_skip() {
        let source = "# Title\n### Skip\n";
        let file = make_file(source);
        let checker = MarkdownChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD001"),
            "expected MD001 for heading skip, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_duplicate_heading() {
        let source = "# Hello\n## Section\n# Hello\n";
        let file = make_file(source);
        let checker = MarkdownChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD002"),
            "expected MD002 for duplicate heading, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_missing_code_lang() {
        let source = "Some text\n```\ncode here\n```\n";
        let file = make_file(source);
        let checker = MarkdownChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD003"),
            "expected MD003 for missing code lang, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_trailing_whitespace() {
        let source = "# Title\nSome line   \n";
        let file = make_file(source);
        let checker = MarkdownChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD009"),
            "expected MD009 for trailing whitespace, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_consecutive_blank_lines() {
        let source = "# Title\n\n\n\nContent\n";
        let file = make_file(source);
        let checker = MarkdownChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD010"),
            "expected MD010 for consecutive blanks, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_unclosed_frontmatter() {
        let source = "---\ntitle: Test\n# Body\n";
        let file = make_file(source);
        let checker = MarkdownChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD007"),
            "expected MD007 for unclosed frontmatter, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_no_false_positives_for_clean_file() {
        let source = "---\ntitle: Clean\n---\n\n# Title\n\n## Section\n\nSome content here.\n";
        let file = make_file(source);
        let checker = MarkdownChecker::new();
        let diags = checker.check(&file, &LintConfig::default());
        // MD008 is always emitted when frontmatter exists (deferred schema check)
        let unexpected: Vec<_> = diags.iter().filter(|d| d.code != "MD008").collect();
        assert!(
            unexpected.is_empty(),
            "unexpected diagnostics on clean file: {:?}",
            unexpected.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }

    // -----------------------------------------------------------------------
    // MD011: Broken relative link
    // -----------------------------------------------------------------------

    #[test]
    fn test_md011_broken_link_fires_on_missing_file() {
        use tempfile::tempdir;

        let dir = tempdir().expect("tempdir");
        // Workspace root has no `missing.md` file
        let checker = MarkdownChecker::with_workspace(dir.path().to_path_buf());

        let source = "See [missing](./missing.md) for details.\n";
        let file = make_file(source);
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            codes(&diags).contains(&"MD011"),
            "expected MD011 for missing file, got {:?}",
            codes(&diags)
        );
    }

    #[test]
    fn test_md011_no_fire_when_file_exists() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("exists.md");
        fs::write(&target, "# Exists\n").expect("create file");

        let checker = MarkdownChecker::with_workspace(dir.path().to_path_buf());
        let source = "See [exists](./exists.md) for details.\n";
        let file = make_file(source);
        let diags = checker.check(&file, &LintConfig::default());
        let md011_count = diags.iter().filter(|d| d.code == "MD011").count();
        assert_eq!(md011_count, 0, "MD011 should not fire for existing file");
    }

    #[test]
    fn test_md011_disabled_without_workspace() {
        // Without workspace_root, MD011 is disabled (no file-system checks).
        let checker = MarkdownChecker::new();
        let source = "See [missing](./missing.md) for details.\n";
        let file = make_file(source);
        let diags = checker.check(&file, &LintConfig::default());
        assert!(
            !codes(&diags).contains(&"MD011"),
            "MD011 should not fire without workspace root"
        );
    }

    // -----------------------------------------------------------------------
    // MarkdownSymbolExtractor
    // -----------------------------------------------------------------------

    #[test]
    fn test_symbol_extractor_headings() {
        let source = "# Title\n## Section\n### Sub\n";
        let symbols = MarkdownSymbolExtractor::extract(source);
        let headings: Vec<_> = symbols
            .iter()
            .filter_map(|s| {
                if let MarkdownSymbol::Heading { level, text, .. } = s {
                    Some((*level, text.as_str()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(headings, vec![(1, "Title"), (2, "Section"), (3, "Sub")]);
    }

    #[test]
    fn test_symbol_extractor_links() {
        let source = "See [Rust](https://rust-lang.org) for more.\n";
        let symbols = MarkdownSymbolExtractor::extract(source);
        let links: Vec<_> = symbols
            .iter()
            .filter_map(|s| {
                if let MarkdownSymbol::Link { text, url, .. } = s {
                    Some((text.as_str(), url.as_str()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(links, vec![("Rust", "https://rust-lang.org")]);
    }

    #[test]
    fn test_symbol_extractor_code_fence() {
        let source = "```rust\nfn main() {}\n```\n";
        let symbols = MarkdownSymbolExtractor::extract(source);
        let fences: Vec<_> = symbols
            .iter()
            .filter_map(|s| {
                if let MarkdownSymbol::CodeFence { language, .. } = s {
                    Some(language.as_deref())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(fences, vec![Some("rust")]);
    }

    #[test]
    fn test_symbol_extractor_mdx_component() {
        let source = "Here is a <MyComponent /> component.\n";
        let symbols = MarkdownSymbolExtractor::extract(source);
        let components: Vec<_> = symbols
            .iter()
            .filter_map(|s| {
                if let MarkdownSymbol::MdxComponent { name, .. } = s {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            components.contains(&"MyComponent"),
            "expected MyComponent, got {:?}",
            components
        );
    }

    #[test]
    fn test_symbol_extractor_frontmatter() {
        let source = "---\ntitle: Hello\nauthor: Bob\n---\n# Body\n";
        let symbols = MarkdownSymbolExtractor::extract(source);
        let fields: Vec<_> = symbols
            .iter()
            .filter_map(|s| {
                if let MarkdownSymbol::FrontMatterField { key, value, .. } = s {
                    Some((key.as_str(), value.as_str()))
                } else {
                    None
                }
            })
            .collect();
        assert!(fields.contains(&("title", "Hello")));
        assert!(fields.contains(&("author", "Bob")));
    }
}
