// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/parser/review_types.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
