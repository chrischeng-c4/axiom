// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/marker.md#source
// CODEGEN-BEGIN
//! CODEGEN marker parser, replacer, and SPEC-REF emitter.
//!
//! Target files contain `// CODEGEN-BEGIN` / `// CODEGEN-END` block markers.
//! `score gen apply` updates only the content between them, preserving surrounding code.
//!
//! Marker format in Rust:
//! ```text
//! // SPEC-MANAGED: <spec-path>#<section-id>
//! // CODEGEN-BEGIN
//! <generated content>
//! // CODEGEN-END
//! ```
//!
//! SPEC-REF markers (inside CODEGEN blocks, for non-deterministic parts):
//! ```text
//! // SPEC-REF: <spec-path>#<section-id>
//! // TODO: <task description>
//! ```

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-markers.md#R1

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Comment syntax per language
// ---------------------------------------------------------------------------

/// Comment syntax for a target language.
#[derive(Debug, Clone, Copy, PartialEq)]
/// @spec projects/agentic-workflow/tech-design/core/generate/marker.md#source
pub enum Lang {
    Rust,
    Python,
    TypeScript,
    /// TOML / YAML — any file whose line-comment prefix is `#`.
    Toml,
    /// Markdown — HTML-comment wrapping `<!-- ... -->` so markers stay
    /// invisible in rendered output.
    Markdown,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/marker.md#source
impl Lang {
    /// Single-line comment prefix (e.g. `"// "` for Rust).
    pub fn line_comment(&self) -> &'static str {
        match self {
            Lang::Rust | Lang::TypeScript => "// ",
            Lang::Python | Lang::Toml => "# ",
            Lang::Markdown => "<!-- ",
        }
    }

    /// Closing syntax, only non-empty for langs with paired comment syntax
    /// (Markdown's `<!-- ... -->`). Lets the marker emitter produce correct
    /// block-comment pairs without duplicating per-language branches.
    pub fn line_comment_end(&self) -> &'static str {
        match self {
            Lang::Markdown => " -->",
            _ => "",
        }
    }
}

/// Check if a trimmed line is a CODEGEN-BEGIN marker in any comment style.
fn is_codegen_begin(line: &str) -> bool {
    line == "// CODEGEN-BEGIN" || line == "# CODEGEN-BEGIN" || line == "<!-- CODEGEN-BEGIN -->"
}

/// Check if a trimmed line is a CODEGEN-END marker in any comment style.
fn is_codegen_end(line: &str) -> bool {
    line == "// CODEGEN-END" || line == "# CODEGEN-END" || line == "<!-- CODEGEN-END -->"
}

// ---------------------------------------------------------------------------
// CODEGEN block representation
// ---------------------------------------------------------------------------

/// A parsed CODEGEN-BEGIN/END block from a target file.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-markers.md#R1
#[derive(Debug, Clone)]
pub struct CodegenBlock {
    /// Spec reference from the SPEC-MANAGED comment: `"spec-path#section-id"`.
    pub spec_ref: String,
    /// Content inside the CODEGEN-BEGIN/END markers.
    pub content: String,
    /// Line number of CODEGEN-BEGIN (0-indexed).
    pub begin_line: usize,
    /// Line number of CODEGEN-END (0-indexed).
    pub end_line: usize,
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse all CODEGEN-BEGIN/END blocks from a file's content.
///
/// Returns all blocks found, in order. Each block includes its spec_ref,
/// content, and line numbers.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-markers.md#R1
pub fn parse_codegen_blocks(file_content: &str) -> Vec<CodegenBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = file_content.lines().collect();
    let raw_string_lines = rust_raw_string_line_mask(file_content);
    let marker_fixture_lines = rust_marker_fixture_line_mask(file_content);

    let mut i = 0;
    while i < lines.len() {
        if marker_fixture_lines.get(i).copied().unwrap_or(false) {
            i += 1;
            continue;
        }
        let line = lines[i].trim();

        // Look for SPEC-MANAGED comment
        if let Some(spec_ref) = parse_spec_managed_comment(line) {
            // Next line should be CODEGEN-BEGIN
            if i + 1 < lines.len()
                && !marker_fixture_lines.get(i + 1).copied().unwrap_or(false)
                && is_codegen_begin(lines[i + 1].trim())
            {
                let begin_line = i + 1;
                let content_start = begin_line + 1;

                // Find CODEGEN-END
                let mut end_line = None;
                for j in content_start..lines.len() {
                    if raw_string_lines.get(j).copied().unwrap_or(false)
                        || marker_fixture_lines.get(j).copied().unwrap_or(false)
                    {
                        continue;
                    }
                    if is_codegen_end(lines[j].trim()) {
                        end_line = Some(j);
                        break;
                    }
                }

                if let Some(end) = end_line {
                    let content = lines[content_start..end].join("\n");
                    blocks.push(CodegenBlock {
                        spec_ref,
                        content,
                        begin_line,
                        end_line: end,
                    });
                    i = end + 1;
                    continue;
                }
            }
        }

        // Also look for bare CODEGEN-BEGIN without SPEC-MANAGED
        if is_codegen_begin(line) {
            let begin_line = i;
            let content_start = begin_line + 1;

            let mut end_line = None;
            for j in content_start..lines.len() {
                if raw_string_lines.get(j).copied().unwrap_or(false)
                    || marker_fixture_lines.get(j).copied().unwrap_or(false)
                {
                    continue;
                }
                if is_codegen_end(lines[j].trim()) {
                    end_line = Some(j);
                    break;
                }
            }

            if let Some(end) = end_line {
                let content = lines[content_start..end].join("\n");
                blocks.push(CodegenBlock {
                    spec_ref: String::new(),
                    content,
                    begin_line,
                    end_line: end,
                });
                i = end + 1;
                continue;
            }
        }

        i += 1;
    }

    blocks
}

/// Return a per-line mask for Rust raw-string literal bodies.
///
/// The marker scanner is intentionally line-oriented because target files may
/// be Rust, Python, TypeScript, TOML, or Markdown. Rust test fixtures often
/// embed literal CODEGEN markers in raw strings; those fixture lines must not
/// be interpreted as real target markers.
/// @spec projects/agentic-workflow/tech-design/core/generate/marker.md#source
pub fn rust_raw_string_line_mask(file_content: &str) -> Vec<bool> {
    rust_string_line_mask(file_content, false)
}

fn rust_marker_fixture_line_mask(file_content: &str) -> Vec<bool> {
    rust_string_line_mask(file_content, true)
}

fn rust_string_line_mask(file_content: &str, mask_regular_strings: bool) -> Vec<bool> {
    let mut mask = Vec::new();
    let mut raw_hashes: Option<usize> = None;
    let mut in_regular_string = false;
    let mut regular_string_escape = false;
    let mut pending_regular_string_lines: Vec<usize> = Vec::new();

    for line in file_content.lines() {
        let bytes = line.as_bytes();
        let line_idx = mask.len();
        let mut raw_line_masked = raw_hashes.is_some();
        let mut regular_line_masked = mask_regular_strings && in_regular_string;
        let mut i = 0usize;

        while i < bytes.len() {
            if let Some(hashes) = raw_hashes {
                raw_line_masked = true;
                if raw_string_end_at(bytes, i, hashes) {
                    i += hashes + 1;
                    raw_hashes = None;
                    continue;
                }
                i += 1;
                continue;
            }

            if in_regular_string {
                if mask_regular_strings {
                    regular_line_masked = true;
                }
                if regular_string_escape {
                    regular_string_escape = false;
                } else if bytes[i] == b'\\' {
                    regular_string_escape = true;
                } else if bytes[i] == b'"' {
                    in_regular_string = false;
                    pending_regular_string_lines.clear();
                }
                i += 1;
                continue;
            }

            if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
                break;
            }

            if bytes[i] == b'\'' {
                if let Some(end) = rust_char_literal_end_at(bytes, i) {
                    i = end + 1;
                    continue;
                }
            }

            if let Some((hashes, next)) = raw_string_start_at(bytes, i) {
                raw_line_masked = true;
                raw_hashes = Some(hashes);
                i = next;
                continue;
            }

            if bytes[i] == b'"' {
                in_regular_string = true;
                regular_string_escape = false;
                if mask_regular_strings {
                    regular_line_masked = true;
                }
                i += 1;
                continue;
            }

            i += 1;
        }

        if regular_line_masked && !pending_regular_string_lines.contains(&line_idx) {
            pending_regular_string_lines.push(line_idx);
        }
        mask.push(raw_line_masked || regular_line_masked);
    }

    // Unterminated regular-string fragments can appear inside marker-only
    // generated blocks. They are not valid Rust fixtures, so do not let them
    // hide every following real CODEGEN block from the scanner.
    if mask_regular_strings && in_regular_string {
        for idx in pending_regular_string_lines {
            if let Some(masked) = mask.get_mut(idx) {
                *masked = false;
            }
        }
    }

    mask
}

fn rust_char_literal_end_at(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'\'') {
        return None;
    }
    let mut i = start + 1;
    if i >= bytes.len() {
        return None;
    }
    if bytes[i] == b'\\' {
        i += 1;
        if i >= bytes.len() {
            return None;
        }
    }
    i += 1;
    if bytes.get(i) == Some(&b'\'') {
        Some(i)
    } else {
        None
    }
}

fn raw_string_start_at(bytes: &[u8], i: usize) -> Option<(usize, usize)> {
    if i > 0 {
        let prev = bytes[i - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'"' || prev == b'\'' {
            return None;
        }
    }

    let mut start = i;
    if bytes.get(start) == Some(&b'b') && bytes.get(start + 1) == Some(&b'r') {
        start += 1;
    }
    if bytes.get(start) != Some(&b'r') {
        return None;
    }

    let mut quote = start + 1;
    while bytes.get(quote) == Some(&b'#') {
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return None;
    }

    Some((quote - start - 1, quote + 1))
}

fn raw_string_end_at(bytes: &[u8], i: usize, hashes: usize) -> bool {
    if bytes.get(i) != Some(&b'"') {
        return false;
    }
    (0..hashes).all(|offset| bytes.get(i + 1 + offset) == Some(&b'#'))
}

fn parse_spec_managed_comment(line: &str) -> Option<String> {
    // Matches: `// SPEC-MANAGED: path#section` (any comment prefix)
    let stripped = line
        .strip_prefix("// SPEC-MANAGED: ")
        .or_else(|| line.strip_prefix("# SPEC-MANAGED: "))
        .or_else(|| {
            line.strip_prefix("<!-- SPEC-MANAGED: ")
                .and_then(|s| s.strip_suffix(" -->"))
        });
    stripped.map(|s| s.trim().to_string())
}

// ---------------------------------------------------------------------------
// Replacer
// ---------------------------------------------------------------------------

/// Replace the content inside a CODEGEN-BEGIN/END block identified by spec_ref.
///
/// Finds the block with the matching `spec_ref` in `SPEC-MANAGED` comment,
/// replaces its content with `new_content`, and returns the updated file string.
/// If no matching block is found, returns the original file unchanged.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-markers.md#R2
pub fn replace_codegen_block(file_content: &str, spec_ref: &str, new_content: &str) -> String {
    let lines: Vec<&str> = file_content.lines().collect();
    let blocks = parse_codegen_blocks(file_content);

    // Find block with matching spec_ref
    let block = blocks.iter().find(|b| b.spec_ref == spec_ref);
    let block = match block {
        Some(b) => b,
        None => return file_content.to_string(),
    };

    // The SPEC-MANAGED comment is on the line before CODEGEN-BEGIN
    let managed_line = if block.begin_line > 0 {
        let candidate = lines[block.begin_line - 1].trim();
        if candidate.contains("SPEC-MANAGED:") {
            block.begin_line - 1
        } else {
            block.begin_line
        }
    } else {
        block.begin_line
    };

    let duplicate_ranges: Vec<(usize, usize)> = blocks
        .iter()
        .filter(|b| b.spec_ref == spec_ref && b.begin_line != block.begin_line)
        .map(|b| {
            let managed = if b.begin_line > 0 {
                let candidate = lines[b.begin_line - 1].trim();
                if candidate.contains("SPEC-MANAGED:") {
                    b.begin_line - 1
                } else {
                    b.begin_line
                }
            } else {
                b.begin_line
            };
            (managed, b.end_line)
        })
        .collect();

    let mut result_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if duplicate_ranges
            .iter()
            .any(|(start, end)| i >= *start && i <= *end)
        {
            continue;
        }

        if i < managed_line {
            result_lines.push(*line);
        } else if i == managed_line && managed_line < block.begin_line {
            // Keep SPEC-MANAGED comment
            result_lines.push(*line);
        } else if i == block.begin_line {
            // Keep CODEGEN-BEGIN
            result_lines.push(*line);
            // Insert new content
            if !new_content.is_empty() {
                result_lines.push(new_content);
            }
        } else if i > block.begin_line && i < block.end_line {
            // Skip old content
        } else if i == block.end_line {
            // Keep CODEGEN-END
            result_lines.push(*line);
        } else {
            result_lines.push(*line);
        }
    }

    let mut rendered = result_lines.join("\n");
    if !rendered.is_empty() {
        rendered.push('\n');
    }
    rendered
}

/// Insert a new CODEGEN-BEGIN/END block at a given position in the file.
///
/// Inserts after the line containing `anchor_text` (or at the end if not found).
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-markers.md#R6
pub fn insert_codegen_block(
    file_content: &str,
    spec_ref: &str,
    initial_content: &str,
    anchor_text: Option<&str>,
    lang: Lang,
) -> String {
    let lines: Vec<&str> = file_content.lines().collect();
    let pfx = lang.line_comment();
    let suf = lang.line_comment_end();

    let block = format!(
        "{pfx}SPEC-MANAGED: {spec_ref}{suf}\n{pfx}CODEGEN-BEGIN{suf}\n{initial_content}\n{pfx}CODEGEN-END{suf}",
    );

    let insert_after = anchor_text.and_then(|anchor| lines.iter().position(|l| l.contains(anchor)));

    let insert_pos = insert_after.map(|p| p + 1).unwrap_or(lines.len());

    let mut result = lines[..insert_pos].join("\n");
    if !result.is_empty() {
        result.push('\n');
    }
    result.push_str(&block);
    result.push('\n');
    if insert_pos < lines.len() {
        result.push_str(&lines[insert_pos..].join("\n"));
    }
    result
}

// ---------------------------------------------------------------------------
// SPEC-REF emitter
// ---------------------------------------------------------------------------

/// Emit a SPEC-REF marker for a non-deterministic part.
///
/// Format (Rust): `// SPEC-REF: <spec-path>#<section>\n// TODO: <task>`
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-markers.md#R4
pub fn emit_spec_ref(spec_path: &str, section: &str, task: &str, lang: Lang) -> String {
    let pfx = lang.line_comment();
    format!("{pfx}SPEC-REF: {spec_path}#{section}\n{pfx}TODO: {task}")
}

// ---------------------------------------------------------------------------
// Marker tracking
// ---------------------------------------------------------------------------

/// A single SPEC-REF marker entry for the tracking file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// @spec projects/agentic-workflow/tech-design/core/generate/marker.md#source
pub struct MarkerEntry {
    pub spec_path: String,
    pub section: String,
    pub file: String,
    pub line: usize,
    pub task: String,
}

/// Collect all SPEC-REF markers from a file's content.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-markers.md#R5
pub fn collect_spec_refs(file_path: &str, file_content: &str) -> Vec<MarkerEntry> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = file_content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        // Match: // SPEC-REF: path#section or # SPEC-REF: path#section
        if let Some(ref_part) = line
            .strip_prefix("// SPEC-REF: ")
            .or_else(|| line.strip_prefix("# SPEC-REF: "))
        {
            let (spec_path, section) = if let Some(pos) = ref_part.rfind('#') {
                (&ref_part[..pos], &ref_part[pos + 1..])
            } else {
                (ref_part, "")
            };

            // Look for TODO on next line
            let task = if i + 1 < lines.len() {
                let next = lines[i + 1].trim();
                next.strip_prefix("// TODO: ")
                    .or_else(|| next.strip_prefix("# TODO: "))
                    .unwrap_or("")
                    .to_string()
            } else {
                String::new()
            };

            entries.push(MarkerEntry {
                spec_path: spec_path.to_string(),
                section: section.to_string(),
                file: file_path.to_string(),
                line: i + 1,
                task,
            });
        }
        i += 1;
    }
    entries
}

/// Group marker entries by spec_path for the tracking YAML.
/// @spec projects/agentic-workflow/tech-design/core/generate/marker.md#source
pub fn group_markers(entries: Vec<MarkerEntry>) -> HashMap<String, Vec<MarkerEntry>> {
    let mut map: HashMap<String, Vec<MarkerEntry>> = HashMap::new();
    for entry in entries {
        map.entry(entry.spec_path.clone()).or_default().push(entry);
    }
    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FILE: &str = r#"use serde::{Deserialize, Serialize};

// SPEC-MANAGED: projects/agentic-workflow/logic/state.md#state-phase
// CODEGEN-BEGIN
pub enum StatePhase {
    ChangeInited,
    ChangeArchived,
}

// CODEGEN-END

// Hand-written code below
impl StatePhase {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::ChangeArchived)
    }
}
"#;

    #[test]
    fn test_parse_codegen_blocks() {
        let blocks = parse_codegen_blocks(SAMPLE_FILE);
        assert_eq!(blocks.len(), 1);
        let block = &blocks[0];
        assert_eq!(
            block.spec_ref,
            "projects/agentic-workflow/logic/state.md#state-phase"
        );
        assert!(block.content.contains("StatePhase"));
        assert!(block.content.contains("ChangeInited"));
    }

    #[test]
    fn test_replace_codegen_block() {
        let new_content =
            "pub enum StatePhase {\n    ChangeInited,\n    ChangeArchived,\n    ChangeRejected,\n}";
        let result = replace_codegen_block(
            SAMPLE_FILE,
            "projects/agentic-workflow/logic/state.md#state-phase",
            new_content,
        );

        assert!(result.contains("ChangeRejected"));
        // Hand-written code preserved
        assert!(result.contains("impl StatePhase"));
        assert!(result.contains("is_terminal"));
    }

    #[test]
    fn test_replace_codegen_block_preserves_final_newline() {
        let result = replace_codegen_block(
            SAMPLE_FILE,
            "projects/agentic-workflow/logic/state.md#state-phase",
            "pub enum StatePhase {}",
        );

        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_replace_missing_spec_ref_unchanged() {
        let result = replace_codegen_block(SAMPLE_FILE, "nonexistent#ref", "new content");
        assert_eq!(result, SAMPLE_FILE);
    }

    #[test]
    fn test_emit_spec_ref() {
        let marker = emit_spec_ref(
            "projects/agentic-workflow/logic/state.md",
            "routing",
            "Implement routing logic",
            Lang::Rust,
        );
        assert_eq!(
            marker,
            "// SPEC-REF: projects/agentic-workflow/logic/state.md#routing\n// TODO: Implement routing logic"
        );
    }

    #[test]
    fn test_collect_spec_refs() {
        let content = r#"
// SPEC-REF: path/to/spec.md#section-a
// TODO: Implement this
let x = 1;
"#;
        let entries = collect_spec_refs("src/lib.rs", content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].spec_path, "path/to/spec.md");
        assert_eq!(entries[0].section, "section-a");
        assert_eq!(entries[0].task, "Implement this");
    }

    #[test]
    fn test_insert_codegen_block() {
        let file = "fn foo() {}\n\nfn bar() {}\n";
        let result = insert_codegen_block(
            file,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#foo",
            "// generated",
            Some("fn foo()"),
            Lang::Rust,
        );
        assert!(result.contains(
            "SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#foo"
        ));
        assert!(result.contains("CODEGEN-BEGIN"));
        assert!(result.contains("// generated"));
        assert!(result.contains("CODEGEN-END"));
        assert!(result.contains("fn bar()"));
    }

    #[test]
    fn test_multiple_codegen_blocks() {
        let file = r#"
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#block-a
// CODEGEN-BEGIN
content a
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#block-b
// CODEGEN-BEGIN
content b
// CODEGEN-END
"#;
        let blocks = parse_codegen_blocks(file);
        assert_eq!(blocks.len(), 2);
        assert_eq!(
            blocks[0].spec_ref,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#block-a"
        );
        assert_eq!(
            blocks[1].spec_ref,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#block-b"
        );
    }

    #[test]
    fn test_parse_codegen_blocks_ignores_rust_raw_string_fixtures() {
        let file = r##"
const SAMPLE: &str = r#"
// SPEC-MANAGED: fixture.md#source
// CODEGEN-BEGIN
fixture
// CODEGEN-END
"#;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#source
// CODEGEN-BEGIN
pub fn generated() {}
// CODEGEN-END
"##;
        let blocks = parse_codegen_blocks(file);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].spec_ref,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#source"
        );
        assert!(blocks[0].content.contains("generated"));
    }

    #[test]
    fn test_parse_codegen_blocks_ignores_rust_regular_multiline_string_fixtures() {
        let file = r#"
fn fixture() {
    let content = "\
// SPEC-MANAGED: fixture.md#source
// CODEGEN-BEGIN
fixture
// CODEGEN-END
";
}

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#source
// CODEGEN-BEGIN
pub fn generated() {}
// CODEGEN-END
"#;
        let blocks = parse_codegen_blocks(file);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].spec_ref,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#source"
        );
        assert!(blocks[0].content.contains("generated"));
    }

    #[test]
    fn test_parse_codegen_blocks_regular_string_inside_outer_block_does_not_close_early() {
        let file = r#"
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source
// CODEGEN-BEGIN
fn fixture() {
    let content = "\
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/lib.md#source
// CODEGEN-BEGIN
// CODEGEN-END
";
}
pub fn after_fixture() {}
// CODEGEN-END
"#;
        let blocks = parse_codegen_blocks(file);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].spec_ref,
            "projects/agentic-workflow/tech-design/surface/interfaces/src/cb.md#source"
        );
        assert!(blocks[0].content.contains("pub fn after_fixture() {}"));
    }

    #[test]
    fn test_parse_codegen_blocks_unclosed_string_in_block_does_not_hide_following_block() {
        let file = r#"
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#source
// CODEGEN-BEGIN
pub fn generated() {
    let content = "\
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#schema
// CODEGEN-BEGIN
pub struct Generated;
// CODEGEN-END
"#;
        let blocks = parse_codegen_blocks(file);
        assert_eq!(blocks.len(), 2);
        assert_eq!(
            blocks[0].spec_ref,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#source"
        );
        assert_eq!(
            blocks[1].spec_ref,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#schema"
        );
    }

    #[test]
    fn test_replace_codegen_block_collapses_duplicate_spec_refs() {
        let file = r#"
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#source
// CODEGEN-BEGIN
old first
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#source
// CODEGEN-BEGIN
old duplicate
// CODEGEN-END
"#;
        let result = replace_codegen_block(
            file,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#source",
            "new content",
        );

        assert!(result.contains("new content"));
        assert!(!result.contains("old first"));
        assert!(!result.contains("old duplicate"));
        assert_eq!(result.matches("SPEC-MANAGED").count(), 1);
        assert_eq!(result.matches("CODEGEN-BEGIN").count(), 1);
        assert_eq!(result.matches("CODEGEN-END").count(), 1);
    }

    // -----------------------------------------------------------------------
    // Gap 3 regression tests: language-appropriate comment wrapping
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_codegen_blocks_python_comment_style() {
        let file = r#"
# SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#schema
# CODEGEN-BEGIN
class MyModel:
    id: str
    name: str
# CODEGEN-END
"#;
        let blocks = parse_codegen_blocks(file);
        assert_eq!(
            blocks.len(),
            1,
            "Should parse Python-style # CODEGEN markers"
        );
        assert_eq!(
            blocks[0].spec_ref,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#schema"
        );
        assert!(blocks[0].content.contains("class MyModel"));
    }

    #[test]
    fn test_parse_codegen_blocks_bare_python_markers() {
        let file = "# CODEGEN-BEGIN\nx = 1\n# CODEGEN-END\n";
        let blocks = parse_codegen_blocks(file);
        assert_eq!(blocks.len(), 1, "Should parse bare Python CODEGEN markers");
        assert!(blocks[0].content.contains("x = 1"));
    }

    #[test]
    fn test_insert_codegen_block_python_lang() {
        let file = "def foo():\n    pass\n";
        let result = insert_codegen_block(
            file,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#foo",
            "# generated",
            None,
            Lang::Python,
        );
        assert!(
            result.contains(
                "# SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#foo"
            ),
            "Python markers should use #"
        );
        assert!(
            result.contains("# CODEGEN-BEGIN"),
            "Python CODEGEN-BEGIN should use #"
        );
        assert!(
            result.contains("# CODEGEN-END"),
            "Python CODEGEN-END should use #"
        );
        assert!(
            !result.contains("// CODEGEN-BEGIN"),
            "Python markers should NOT use //"
        );
    }

    #[test]
    fn test_replace_codegen_block_python_style() {
        let file = "# SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#model\n# CODEGEN-BEGIN\nold_content\n# CODEGEN-END\n";
        let result = replace_codegen_block(
            file,
            "projects/agentic-workflow/tech-design/core/tools/spec.md#model",
            "new_content",
        );
        assert!(
            result.contains("new_content"),
            "Should replace content in Python-style blocks"
        );
        assert!(
            !result.contains("old_content"),
            "Old content should be replaced"
        );
        assert!(
            result.contains("# CODEGEN-BEGIN"),
            "Should preserve Python markers"
        );
    }

    #[test]
    fn test_emit_spec_ref_python() {
        let marker = emit_spec_ref(
            "projects/agentic-workflow/tech-design/core/tools/spec.md",
            "section",
            "Implement logic",
            Lang::Python,
        );
        assert_eq!(
            marker,
            "# SPEC-REF: projects/agentic-workflow/tech-design/core/tools/spec.md#section\n# TODO: Implement logic"
        );
    }
}

// CODEGEN-END
