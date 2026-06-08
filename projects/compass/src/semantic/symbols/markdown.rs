//! Markdown symbol extraction (line-based)
//!
//! Extracts symbols from Markdown documents:
//! - Headings (h1–h6) as Label
//! - Links `[text](url)` as Resource
//! - Code blocks ` ```lang ` as Template
//! - Frontmatter fields (`key: value` between `---`) as Variable

use super::{SymbolKind, SymbolTableBuilder};
use crate::diagnostic::{Position, Range};

impl SymbolTableBuilder {
    pub(crate) fn visit_markdown_lines(&mut self, source: &str) {
        let mut in_code_block = false;
        let mut code_block_lang: Option<(String, u32)> = None; // (lang, start_line)
        let mut in_frontmatter = false;
        let mut frontmatter_done = false;
        let mut frontmatter_line: u32 = 0;

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx as u32;
            let trimmed = line.trim();

            // --- Frontmatter: between first pair of `---` at file start ---
            if !frontmatter_done && line_num == 0 && trimmed == "---" {
                in_frontmatter = true;
                frontmatter_line = line_num;
                continue;
            }
            if in_frontmatter {
                if trimmed == "---" || trimmed == "..." {
                    in_frontmatter = false;
                    frontmatter_done = true;
                    continue;
                }
                self.extract_markdown_frontmatter_field(trimmed, line_num);
                continue;
            }

            // --- Code block fence detection ---
            if trimmed.starts_with("```") {
                if in_code_block {
                    // Closing fence
                    in_code_block = false;
                    code_block_lang = None;
                } else {
                    // Opening fence — extract language tag
                    in_code_block = true;
                    let lang = trimmed.trim_start_matches('`').trim();
                    if !lang.is_empty() {
                        let col = line.find(lang).unwrap_or(0) as u32;
                        self.table.add_symbol(
                            lang.to_string(),
                            SymbolKind::Template,
                            mk_range(line_num, col, lang.len()),
                            None,
                            Some("code block language".to_string()),
                            self.current_scope,
                        );
                        code_block_lang = Some((lang.to_string(), line_num));
                    } else {
                        code_block_lang = None;
                    }
                }
                continue;
            }

            // Skip content inside code blocks
            if in_code_block {
                continue;
            }

            // --- ATX Headings: `# Title`, `## Title`, etc. ---
            if trimmed.starts_with('#') {
                self.extract_markdown_heading(line, trimmed, line_num);
                continue;
            }

            // --- Setext headings: line followed by `===` or `---` ---
            // (Handled implicitly; the heading text is the previous line — skip for simplicity)

            // --- Inline links: `[text](url)` ---
            self.extract_markdown_links(line, line_num);
        }

        let _ = (code_block_lang, frontmatter_line);
    }

    /// Extract ATX heading: `## Section Title`
    fn extract_markdown_heading(&mut self, raw_line: &str, trimmed: &str, line_num: u32) {
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level > 6 {
            return;
        }
        let text = trimmed[level..].trim();
        if text.is_empty() {
            return;
        }
        // Include the `##` prefix in the symbol name for context
        let hashes: String = "#".repeat(level);
        let name = format!("{} {}", hashes, text);
        // Column of the `#` in the raw line
        let col = raw_line.find('#').unwrap_or(0) as u32;
        self.table.add_symbol(
            name,
            SymbolKind::Label,
            mk_range(line_num, col, trimmed.len()),
            None,
            Some(format!("h{} heading", level)),
            self.current_scope,
        );
    }

    /// Extract frontmatter field: `key: value`
    fn extract_markdown_frontmatter_field(&mut self, trimmed: &str, line_num: u32) {
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            if key.is_empty() || key.starts_with('-') {
                return;
            }
            self.table.add_symbol(
                key.to_string(),
                SymbolKind::Variable,
                mk_range(line_num, 0, key.len()),
                None,
                Some("frontmatter field".to_string()),
                self.current_scope,
            );
        }
    }

    /// Extract inline links from a line: `[text](url)` patterns
    fn extract_markdown_links(&mut self, line: &str, line_num: u32) {
        let mut search = line;
        let mut col_offset = 0usize;

        while let Some(bracket_open) = search.find('[') {
            let after_bracket = &search[bracket_open + 1..];
            let Some(bracket_close) = after_bracket.find(']') else {
                break;
            };
            let after_close = &after_bracket[bracket_close + 1..];
            // Must be followed immediately by `(`
            if !after_close.starts_with('(') {
                // Advance past this `[` and keep looking
                let advance = bracket_open + 1;
                col_offset += advance;
                search = &search[advance..];
                continue;
            }
            let after_paren_open = &after_close[1..];
            let Some(paren_close) = after_paren_open.find(')') else {
                break;
            };
            let url = after_paren_open[..paren_close].trim();
            if !url.is_empty() {
                // Column of `(url` start
                let url_start = col_offset + bracket_open + 1 + bracket_close + 1 + 1; // after `(`
                let col = line.find(url).unwrap_or(url_start) as u32;
                self.table.add_symbol(
                    url.to_string(),
                    SymbolKind::Resource,
                    mk_range(line_num, col, url.len()),
                    None,
                    Some("markdown link".to_string()),
                    self.current_scope,
                );
            }
            // Advance past the full `[text](url)` match
            let advance = bracket_open + 1 + bracket_close + 1 + 1 + paren_close + 1;
            col_offset += advance;
            if advance >= search.len() {
                break;
            }
            search = &search[advance..];
        }
    }
}

fn mk_range(line: u32, col: u32, len: usize) -> Range {
    Range::new(
        Position::new(line, col),
        Position::new(line, col + len as u32),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{SymbolKind, SymbolTableBuilder};

    fn build(source: &str) -> super::super::SymbolTable {
        SymbolTableBuilder::new().build_markdown_from_source(source)
    }

    #[test]
    fn test_headings() {
        let src = "# Title\n## Section\n### Sub\n";
        let table = build(src);
        let headings: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Label)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            headings.iter().any(|h| h.contains("Title")),
            "missing h1, got: {:?}",
            headings
        );
        assert!(
            headings.iter().any(|h| h.contains("Section")),
            "missing h2, got: {:?}",
            headings
        );
        assert!(
            headings.iter().any(|h| h.contains("Sub")),
            "missing h3, got: {:?}",
            headings
        );
    }

    #[test]
    fn test_heading_level_prefix() {
        let src = "## My Heading\n";
        let table = build(src);
        let heading = table
            .all_symbols()
            .iter()
            .find(|s| s.kind == SymbolKind::Label)
            .map(|s| s.name.as_str());
        assert_eq!(heading, Some("## My Heading"), "got: {:?}", heading);
    }

    #[test]
    fn test_links() {
        let src = "Check [docs](https://example.com) and [repo](https://github.com/org/repo).\n";
        let table = build(src);
        let urls: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Resource)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            urls.contains(&"https://example.com"),
            "missing url, got: {:?}",
            urls
        );
        assert!(
            urls.contains(&"https://github.com/org/repo"),
            "missing url, got: {:?}",
            urls
        );
    }

    #[test]
    fn test_code_blocks() {
        let src = "# Title\n\n```rust\nfn main() {}\n```\n\n```python\nprint('hi')\n```\n";
        let table = build(src);
        let langs: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Template)
            .map(|s| s.name.as_str())
            .collect();
        assert!(langs.contains(&"rust"), "missing 'rust', got: {:?}", langs);
        assert!(
            langs.contains(&"python"),
            "missing 'python', got: {:?}",
            langs
        );
    }

    #[test]
    fn test_code_block_no_language() {
        let src = "```\nsome code\n```\n";
        let table = build(src);
        let templates: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Template)
            .map(|s| s.name.as_str())
            .collect();
        // No language tag — no Template symbol
        assert!(
            templates.is_empty(),
            "unexpected template symbols: {:?}",
            templates
        );
    }

    #[test]
    fn test_frontmatter() {
        let src = "---\ntitle: My Doc\nauthor: Alice\ndate: 2024-01-01\n---\n# Content\n";
        let table = build(src);
        let vars: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable)
            .map(|s| s.name.as_str())
            .collect();
        assert!(vars.contains(&"title"), "missing 'title', got: {:?}", vars);
        assert!(
            vars.contains(&"author"),
            "missing 'author', got: {:?}",
            vars
        );
        assert!(vars.contains(&"date"), "missing 'date', got: {:?}", vars);
    }

    #[test]
    fn test_links_not_extracted_inside_code_blocks() {
        let src = "```\n[not a link](http://skip.me)\n```\n";
        let table = build(src);
        let links: Vec<&str> = table
            .all_symbols()
            .iter()
            .filter(|s| s.kind == SymbolKind::Resource)
            .map(|s| s.name.as_str())
            .collect();
        assert!(
            links.is_empty(),
            "links should be skipped in code blocks, got: {:?}",
            links
        );
    }
}
