---
id: sdd-parser-review-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# ReviewBlock

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/parser/review.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReviewBlock` | projects/agentic-workflow/src/parser/review.rs | struct | pub | 75 |  |
| `parse_latest_review` | projects/agentic-workflow/src/parser/review.rs | function | pub | 84 | parse_latest_review(content: &str) -> Result<Option<ReviewBlock>> |
| `parse_review_verdict` | projects/agentic-workflow/src/parser/review.rs | function | pub | 14 | parse_review_verdict(review_path: &Path) -> Result<ReviewVerdict> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ReviewBlock:
    type: object
    description: ReviewBlock.
    properties:
      status: { type: string }
      iteration: { type: integer, x-rust-type: u32 }
      reviewer: { type: string }
      content: { type: string }
    required: [status, iteration, reviewer, content]
    x-rust-struct:
      derive: [Debug, Clone, PartialEq]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/parser/review.rs -->
```rust
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/review_types.md#source
use crate::models::ReviewVerdict;
use crate::Result;
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/review_types.md#source
/// Parse review verdict from REVIEW.md
///
/// Supports multiple formats:
/// 1. Checkbox format: `[x] APPROVED`, `[x] REVIEWED`, `[x] REJECTED`
/// 2. Plain text format: `## Verdict\nAPPROVED` or `## Verdict\n\nAPPROVED` (used by Codex)
pub fn parse_review_verdict(review_path: &Path) -> Result<ReviewVerdict> {
    if !review_path.exists() {
        return Ok(ReviewVerdict::Unknown);
    }

    let content = std::fs::read_to_string(review_path)?;
    let content_lower = content.to_lowercase();

    // Normalize whitespace: collapse multiple newlines to single newline after ## verdict
    let normalized = normalize_verdict_section(&content_lower);

    // Handle checkbox format
    if content_lower.contains("[x] approved") {
        return Ok(ReviewVerdict::Approved);
    }
    if content_lower.contains("[x] reviewed") {
        return Ok(ReviewVerdict::Reviewed);
    }
    if content_lower.contains("[x] rejected") {
        return Ok(ReviewVerdict::Rejected);
    }

    // Handle plain text format (with normalized whitespace)
    if normalized.contains("## verdict\napproved") {
        Ok(ReviewVerdict::Approved)
    } else if normalized.contains("## verdict\nreviewed") {
        Ok(ReviewVerdict::Reviewed)
    } else if normalized.contains("## verdict\nrejected") {
        Ok(ReviewVerdict::Rejected)
    } else {
        Ok(ReviewVerdict::Unknown)
    }
}

/// Normalize the verdict section by collapsing whitespace after "## verdict"
fn normalize_verdict_section(content: &str) -> String {
    // Find "## verdict" and normalize whitespace after it
    if let Some(verdict_idx) = content.find("## verdict") {
        let (before, after) = content.split_at(verdict_idx);
        let after_heading = &after[10..]; // Skip "## verdict"

        // Trim leading whitespace/newlines and get the verdict word
        let trimmed = after_heading.trim_start();
        if let Some(newline_idx) = trimmed.find('\n') {
            let verdict_word = &trimmed[..newline_idx].trim();
            return format!(
                "{}## verdict\n{}\n{}",
                before,
                verdict_word,
                &trimmed[newline_idx..]
            );
        } else {
            return format!("{}## verdict\n{}", before, trimmed.trim());
        }
    }
    content.to_string()
}

/// ReviewBlock.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/review_types.md#schema
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewBlock {
    pub status: String,
    pub iteration: u32,
    pub reviewer: String,
    pub content: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/review_types.md#source
/// Extract latest review block from content containing `<review>` XML blocks
pub fn parse_latest_review(content: &str) -> Result<Option<ReviewBlock>> {
    let reviews = crate::parser::extract_xml_blocks(content, "review")?;

    if reviews.is_empty() {
        return Ok(None);
    }

    let latest = reviews
        .iter()
        .max_by_key(|r| {
            r.attributes
                .get("iteration")
                .and_then(|i| i.parse::<u32>().ok())
                .unwrap_or(0)
        })
        .unwrap();

    Ok(Some(ReviewBlock {
        status: latest.attributes.get("status").cloned().unwrap_or_default(),
        iteration: latest
            .attributes
            .get("iteration")
            .and_then(|i| i.parse::<u32>().ok())
            .unwrap_or(1),
        reviewer: latest
            .attributes
            .get("reviewer")
            .cloned()
            .unwrap_or_default(),
        content: latest.content.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_verdict_single_newline() {
        let content = "## verdict\napproved";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\napproved"));
    }

    #[test]
    fn test_normalize_verdict_double_newline() {
        let content = "## verdict\n\nreviewed";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\nreviewed"));
    }

    #[test]
    fn test_normalize_verdict_multiple_newlines() {
        let content = "## verdict\n\n\nrejected";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\nrejected"));
    }

    #[test]
    fn test_normalize_verdict_with_prefix() {
        let content = "# Review\n\n## verdict\n\napproved\n\n## next steps";
        let normalized = normalize_verdict_section(content);
        assert!(normalized.contains("## verdict\napproved"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/parser/review.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete review parser module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
