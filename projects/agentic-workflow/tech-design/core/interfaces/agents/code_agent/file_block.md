---
id: sdd-agents-code-agent-file-block
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Agent-facing public interfaces are part of the AW Core client-independent workflow protocol surface."
---

# FileBlock

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/agents/code_agent/parser.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FileBlock` | projects/agentic-workflow/src/agents/code_agent/parser.rs | struct | pub | 21 |  |
| `parse_file_blocks` | projects/agentic-workflow/src/agents/code_agent/parser.rs | function | pub | 38 | parse_file_blocks(input: &str) -> NovaResult<Vec<FileBlock>> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FileBlock:
    type: object
    required: [path, content]
    description: A parsed file block extracted from an LLM response.
    properties:
      path:
        type: string
        description: "Repository-relative path declared in the `path` attribute."
      content:
        type: string
        description: "Raw file content between the tags (leading/trailing newlines stripped)."
    x-rust-struct:
      derive: [Debug, Clone, PartialEq]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/agents/code_agent/parser.rs -->
````rust
//! XML block parser for LLM code generation responses.
//!
//! The LLM is instructed to wrap generated files in:
//!
//! ```xml
//! <file path="src/lib.rs">
//! // file contents here
//! </file>
//! ```
//!
//! This module extracts those blocks reliably without a full XML parser.

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/file_block.md#source
use agent::error::{NovaError, NovaResult};

/// A parsed file block extracted from an LLM response.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/file_block.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct FileBlock {
    /// Repository-relative path declared in the `path` attribute.
    pub path: String,
    /// Raw file content between the tags (leading/trailing newlines stripped).
    pub content: String,
}

/// Parse all `<file path="...">...</file>` blocks from `input`.
///
/// # Errors
///
/// Returns [`NovaError::MalformedLLMResponse`] when:
/// - An opening `<file` tag is missing the `path` attribute.
/// - An opening tag is never closed with `>`.
/// - A `</file>` closing tag is missing for an opened block.
/// - No `<file>` blocks are found at all.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/code_agent/file_block.md#source
pub fn parse_file_blocks(input: &str) -> NovaResult<Vec<FileBlock>> {
    let mut blocks = Vec::new();
    let mut remaining = input;

    loop {
        match remaining.find("<file ") {
            None => break,
            Some(tag_start) => {
                let after_open = &remaining[tag_start..];

                // Find closing `>` of the opening tag.
                let close_bracket = after_open.find('>').ok_or_else(|| {
                    NovaError::MalformedLLMResponse(
                        "Unclosed <file> opening tag — missing '>'".to_string(),
                    )
                })?;

                let tag_content = &after_open[..close_bracket];
                let path = extract_path_attr(tag_content)?;

                // Content starts immediately after `>`.
                let content_area = &after_open[close_bracket + 1..];

                let end_tag = "</file>";
                let end_pos = content_area.find(end_tag).ok_or_else(|| {
                    NovaError::MalformedLLMResponse(format!(
                        "Missing </file> closing tag for path: {}",
                        path
                    ))
                })?;

                let content = content_area[..end_pos]
                    .trim_start_matches('\n')
                    .trim_end_matches('\n')
                    .to_string();

                blocks.push(FileBlock { path, content });

                // Advance past this entire block.
                let consumed = tag_start + (close_bracket + 1) + end_pos + end_tag.len();
                remaining = &remaining[consumed..];
            }
        }
    }

    if blocks.is_empty() {
        return Err(NovaError::MalformedLLMResponse(
            "No <file path=\"...\">...</file> blocks found in LLM response".to_string(),
        ));
    }

    Ok(blocks)
}

// ---- helpers ----

fn extract_path_attr(tag: &str) -> NovaResult<String> {
    // Support both double and single quotes: path="..." or path='...'
    for &quote in &['"', '\''] {
        let attr = format!("path={}", quote);
        if let Some(start) = tag.find(&attr) {
            let after = &tag[start + attr.len()..];
            if let Some(end) = after.find(quote) {
                let path = after[..end].to_string();
                if path.is_empty() {
                    return Err(NovaError::MalformedLLMResponse(
                        "Empty path attribute in <file> tag".to_string(),
                    ));
                }
                return Ok(path);
            }
        }
    }
    Err(NovaError::MalformedLLMResponse(
        "Missing or malformed 'path' attribute in <file> tag".to_string(),
    ))
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_file_block() {
        let input = r#"<file path="src/lib.rs">
fn hello() {}
</file>"#;
        let blocks = parse_file_blocks(input).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].path, "src/lib.rs");
        assert_eq!(blocks[0].content, "fn hello() {}");
    }

    #[test]
    fn test_parse_multiple_file_blocks() {
        let input = r#"Some preamble text.
<file path="src/main.rs">
fn main() {}
</file>
Some middle text.
<file path="src/lib.rs">
fn helper() {}
</file>
Trailing text."#;

        let blocks = parse_file_blocks(input).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].path, "src/main.rs");
        assert_eq!(blocks[0].content, "fn main() {}");
        assert_eq!(blocks[1].path, "src/lib.rs");
        assert_eq!(blocks[1].content, "fn helper() {}");
    }

    #[test]
    fn test_parse_single_quotes() {
        let input = "<file path='src/lib.rs'>\nfn x() {}\n</file>";
        let blocks = parse_file_blocks(input).unwrap();
        assert_eq!(blocks[0].path, "src/lib.rs");
    }

    #[test]
    fn test_error_no_blocks() {
        let err = parse_file_blocks("no blocks here").unwrap_err();
        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
        assert!(err.to_string().contains("No <file"));
    }

    #[test]
    fn test_error_missing_path_attr() {
        let err = parse_file_blocks("<file >content</file>").unwrap_err();
        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
    }

    #[test]
    fn test_error_missing_close_tag() {
        let err = parse_file_blocks("<file path=\"x.rs\">content without close").unwrap_err();
        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
        assert!(err.to_string().contains("</file>"));
    }

    #[test]
    fn test_error_unclosed_open_tag() {
        let err = parse_file_blocks("<file path=\"x.rs\" content</file>").unwrap_err();
        assert!(matches!(err, NovaError::MalformedLLMResponse(_)));
    }

    #[test]
    fn test_multiline_content_preserved() {
        let input =
            "<file path=\"config.toml\">\n[package]\nname = \"foo\"\nversion = \"0.1.0\"\n</file>";
        let blocks = parse_file_blocks(input).unwrap();
        let content = &blocks[0].content;
        assert!(content.contains("[package]"));
        assert!(content.contains("name = \"foo\""));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/agents/code_agent/parser.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete file-block parser module, including
      the FileBlock data shape, XML-ish parser, path extraction helpers, and
      tests.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Single-struct scope clean.
- [schema] Partial-derive (no serde) matches source.
- [changes] Two-entry split correct.

## Review 2
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Promotes parser logic and tests into source-template ownership while retaining FileBlock schema as the data contract.
- [source] Uses `strip-managed-markers` to preserve existing parser behavior and remove mixed CODEGEN/HANDWRITE boundaries.
- [changes] Correctly routes the target file through the `source` section with `impl_mode: codegen`.
