---
id: sdd-parser-inline-yaml
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Inline YAML Parser Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/parser/inline_yaml.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `IssueBlockWrapper` | projects/agentic-workflow/src/parser/inline_yaml.rs | struct | pub | 44 |  |
| `RequirementBlockWrapper` | projects/agentic-workflow/src/parser/inline_yaml.rs | struct | pub | 36 |  |
| `TaskBlockWrapper` | projects/agentic-workflow/src/parser/inline_yaml.rs | struct | pub | 28 |  |
| `YamlBlock` | projects/agentic-workflow/src/parser/inline_yaml.rs | struct | pub | 16 |  |
| `extract_yaml_blocks` | projects/agentic-workflow/src/parser/inline_yaml.rs | function | pub | 56 | extract_yaml_blocks(content: &str) -> Vec<YamlBlock> |
| `extract_yaml_blocks_with_lines` | projects/agentic-workflow/src/parser/inline_yaml.rs | function | pub | 98 | extract_yaml_blocks_with_lines(content: &str) -> Vec<YamlBlock> |
| `parse_issue_blocks` | projects/agentic-workflow/src/parser/inline_yaml.rs | function | pub | 197 | parse_issue_blocks(     content: &str, ) -> Result<Vec<(crate::models::IssueBlock, Option<usize>)>> |
| `parse_requirement_blocks` | projects/agentic-workflow/src/parser/inline_yaml.rs | function | pub | 189 | parse_requirement_blocks(     content: &str, ) -> Result<Vec<(crate::models::RequirementBlock, Option<usize>)>> |
| `parse_task_blocks` | projects/agentic-workflow/src/parser/inline_yaml.rs | function | pub | 183 | parse_task_blocks(content: &str) -> Result<Vec<(crate::models::TaskBlock, Option<usize>)>> |
| `parse_typed_yaml_blocks` | projects/agentic-workflow/src/parser/inline_yaml.rs | function | pub | 140 | parse_typed_yaml_blocks(     content: &str,     block_type: &str, ) -> Result<Vec<(T, Option<usize>)>> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  YamlBlock:
    type: object
    required: [content, info_string, line_number]
    description: |
      Extracted YAML block with context.
    properties:
      content:
        type: string
        description: "Raw YAML content."
      info_string:
        type: string
        description: "Info string (e.g., \"yaml\", \"yml\", \"yaml {.task}\")."
      line_number:
        type: integer
        x-rust-type: "Option<usize>"
        description: "Line number in source (if available)."
    x-rust-struct:
      derive: [Debug, Clone]

  TaskBlockWrapper:
    type: object
    required: [task]
    description: |
      Wrapper struct for parsing task blocks.
    properties:
      task:
        type: object
        x-rust-type: "crate::models::TaskBlock"
        description: "The task block."
    x-rust-struct:
      derive: [Debug, Clone, "serde::Deserialize"]

  RequirementBlockWrapper:
    type: object
    required: [requirement]
    description: |
      Wrapper struct for parsing requirement blocks.
    properties:
      requirement:
        type: object
        x-rust-type: "crate::models::RequirementBlock"
        description: "The requirement block."
    x-rust-struct:
      derive: [Debug, Clone, "serde::Deserialize"]

  IssueBlockWrapper:
    type: object
    required: [issue]
    description: |
      Wrapper struct for parsing issue blocks.
    properties:
      issue:
        type: object
        x-rust-type: "crate::models::IssueBlock"
        description: "The issue block."
    x-rust-struct:
      derive: [Debug, Clone, "serde::Deserialize"]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/parser/inline_yaml.rs -->
````rust
//! Inline YAML Block Parser (AST-based)
//!
//! Extracts and parses YAML blocks embedded in Markdown documents.
//! Uses pulldown-cmark AST instead of regex for robust parsing.

/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#source
use anyhow::Result;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use serde::de::DeserializeOwned;

/// Extracted YAML block with context.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#schema
#[derive(Debug, Clone)]
pub struct YamlBlock {
    /// Raw YAML content.
    pub content: String,
    /// Info string (e.g., "yaml", "yml", "yaml {.task}").
    pub info_string: String,
    /// Line number in source (if available).
    pub line_number: Option<usize>,
}

/// Wrapper struct for parsing task blocks.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#schema
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TaskBlockWrapper {
    /// The task block.
    pub task: crate::models::TaskBlock,
}

/// Wrapper struct for parsing requirement blocks.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#schema
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RequirementBlockWrapper {
    /// The requirement block.
    pub requirement: crate::models::RequirementBlock,
}

/// Wrapper struct for parsing issue blocks.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#schema
#[derive(Debug, Clone, serde::Deserialize)]
pub struct IssueBlockWrapper {
    /// The issue block.
    pub issue: crate::models::IssueBlock,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#source
/// Extract all YAML/YML fenced code blocks using markdown AST
///
/// This approach is more robust than regex because:
/// - Properly handles nested code blocks
/// - Ignores YAML inside other code blocks (e.g., ```rust with yaml string)
/// - Handles various info string formats (yaml, yml, yaml {attrs})
pub fn extract_yaml_blocks(content: &str) -> Vec<YamlBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(content, options);
    let mut blocks = Vec::new();
    let mut current_block: Option<(String, String)> = None; // (info_string, content)

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                let info_str = info.to_string();
                // Match yaml, yml, yaml {attrs}, yml {attrs}
                if info_str.starts_with("yaml") || info_str.starts_with("yml") {
                    current_block = Some((info_str, String::new()));
                }
            }
            Event::Text(text) => {
                if let Some((_, ref mut block_content)) = current_block {
                    block_content.push_str(&text);
                }
            }
            Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                if let Some((info_string, block_content)) = current_block.take() {
                    blocks.push(YamlBlock {
                        content: block_content,
                        info_string,
                        line_number: None, // Can be enhanced with offset tracking
                    });
                }
            }
            _ => {}
        }
    }

    blocks
}

/// Extract YAML blocks with line numbers
///
/// More expensive but provides line number information for error reporting
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#source
pub fn extract_yaml_blocks_with_lines(content: &str) -> Vec<YamlBlock> {
    let mut blocks = extract_yaml_blocks(content);

    // Post-process to find line numbers
    let lines: Vec<&str> = content.lines().collect();
    for block in &mut blocks {
        // Find the opening fence for this block
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("```yaml") || trimmed.starts_with("```yml"))
                && block.line_number.is_none()
            {
                // Check if this is our block by looking at content on next lines
                let remaining: String = lines[idx + 1..]
                    .iter()
                    .take_while(|l| !l.trim().starts_with("```"))
                    .copied()
                    .collect::<Vec<_>>()
                    .join("\n");

                if remaining.trim() == block.content.trim() {
                    block.line_number = Some(idx + 1); // 1-indexed
                    break;
                }
            }
        }
    }

    blocks
}

/// Parse typed YAML blocks (task, issue, requirement)
///
/// Looks for YAML blocks that contain a specific type key at the root level.
///
/// # Arguments
/// * `content` - Markdown content
/// * `block_type` - Type to look for (e.g., "task", "issue", "requirement")
///
/// # Returns
/// * Vector of (parsed_value, line_number) tuples
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#source
pub fn parse_typed_yaml_blocks<T: DeserializeOwned>(
    content: &str,
    block_type: &str,
) -> Result<Vec<(T, Option<usize>)>> {
    let blocks = extract_yaml_blocks_with_lines(content);
    let mut results = Vec::new();

    for block in blocks {
        // Parse YAML first, then check if it contains the expected type key
        // This handles blocks with leading comments, whitespace, or document markers (---)
        match serde_yaml::from_str::<serde_yaml::Value>(&block.content) {
            Ok(value) => {
                // Check if the parsed YAML has the expected type key at root level
                if let Some(typed_value) = value.get(block_type) {
                    match serde_yaml::from_value::<T>(typed_value.clone()) {
                        Ok(parsed) => results.push((parsed, block.line_number)),
                        Err(e) => {
                            // Log warning but continue parsing other blocks
                            eprintln!("Warning: Failed to parse {} block: {}", block_type, e);
                        }
                    }
                }
                // If block doesn't have the type key, silently skip (not an error)
            }
            Err(e) => {
                // Only warn if the block looks like it should be our type
                // (contains the type key somewhere in the raw content)
                if block.content.contains(&format!("{}:", block_type)) {
                    eprintln!(
                        "Warning: Invalid YAML in potential {} block: {}",
                        block_type, e
                    );
                }
                // Otherwise silently skip - it's just a different kind of YAML block
            }
        }
    }

    Ok(results)
}

/// Parse all task blocks from tasks.md content
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#source
pub fn parse_task_blocks(content: &str) -> Result<Vec<(crate::models::TaskBlock, Option<usize>)>> {
    parse_typed_yaml_blocks::<crate::models::TaskBlock>(content, "task")
}

/// Parse all requirement blocks from spec content
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#source
pub fn parse_requirement_blocks(
    content: &str,
) -> Result<Vec<(crate::models::RequirementBlock, Option<usize>)>> {
    parse_typed_yaml_blocks::<crate::models::RequirementBlock>(content, "requirement")
}

/// Parse all issue blocks from CHALLENGE.md content
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/inline_yaml.md#source
pub fn parse_issue_blocks(
    content: &str,
) -> Result<Vec<(crate::models::IssueBlock, Option<usize>)>> {
    parse_typed_yaml_blocks::<crate::models::IssueBlock>(content, "issue")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml_blocks() {
        let content = r#"# Test Document

Some text here.

```yaml
key: value
nested:
  item: 1
```

More text.

```yml
another: block
```

```rust
// Not YAML
let yaml = "yaml: content";
```
"#;

        let blocks = extract_yaml_blocks(content);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].content.contains("key: value"));
        assert!(blocks[1].content.contains("another: block"));
    }

    #[test]
    fn test_extract_yaml_with_attrs() {
        let content = r#"
```yaml {.task}
task:
  id: "1.1"
  action: CREATE
```
"#;

        let blocks = extract_yaml_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].info_string.starts_with("yaml"));
    }

    #[test]
    fn test_parse_typed_task_block() {
        let content = r#"
### Task 1.1: Create Model

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/models/user.rs
```

Description of the task.
"#;

        let tasks = parse_task_blocks(content).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].0.id, "1.1");
        assert_eq!(tasks[0].0.action, crate::models::TaskAction::Create);
    }

    #[test]
    fn test_parse_typed_requirement_block() {
        let content = r#"
### R1: User Authentication

```yaml
requirement:
  id: R1
  priority: high
  status: draft
  scenarios: 3
```

The system SHALL support user authentication.
"#;

        let reqs = parse_requirement_blocks(content).unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].0.id, "R1");
        assert_eq!(reqs[0].0.priority, crate::models::RequirementPriority::High);
    }

    #[test]
    fn test_parse_typed_issue_block() {
        let content = r#"
### Issue 1: Missing Error Handling

```yaml
issue:
  id: 1
  severity: high
  category: security
  location:
    file: src/auth.rs
    line: 45
```

Description of the issue.
"#;

        let issues = parse_issue_blocks(content).unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].0.id, 1);
        assert_eq!(
            issues[0].0.severity,
            crate::models::frontmatter::IssueSeverity::High
        );
    }

    #[test]
    fn test_skip_non_typed_yaml() {
        let content = r#"
```yaml
# Just configuration, not a typed block
config:
  debug: true
```

```yaml
task:
  id: "1.1"
  action: CREATE
  status: pending
  file: src/test.rs
```
"#;

        let tasks = parse_task_blocks(content).unwrap();
        assert_eq!(tasks.len(), 1); // Only the task block, not the config block
    }

    #[test]
    fn test_yaml_with_comments() {
        let content = r#"
```yaml
# This is a comment
task:
  id: "2.1"
  action: MODIFY
  status: in_progress
  file: src/lib.rs
```
"#;

        let tasks = parse_task_blocks(content).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].0.id, "2.1");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/parser/inline_yaml.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete inline YAML parser module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Four data carriers; mix of plain Debug+Clone and path-style serde derive.
- [schema] All well-formed; foreign-type fields via x-rust-type; path-style derive.
- [changes] All four in `replaces`; hand-written boundary preserves parser fns.
