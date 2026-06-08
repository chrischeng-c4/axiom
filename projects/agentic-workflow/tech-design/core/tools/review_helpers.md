---
id: projects-sdd-src-tools-review-helpers-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# Standardized projects/agentic-workflow/src/tools/review_helpers.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/review_helpers.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `build_review_section` | projects/agentic-workflow/src/tools/review_helpers.rs | function | pub | 94 | build_review_section(     verdict: &str,     summary: &str,     checklist: &[Value],     issues: &[Value],     iteration: u64,     display_name: &str,     change_id: &str, ) -> String |
| `extract_review_section` | projects/agentic-workflow/src/tools/review_helpers.rs | function | pub | 23 | extract_review_section(content: &str) -> Option<String> |
| `remove_frontmatter_field` | projects/agentic-workflow/src/tools/review_helpers.rs | function | pub | 72 | remove_frontmatter_field(content: &str, key: &str) -> String |
| `strip_review_section` | projects/agentic-workflow/src/tools/review_helpers.rs | function | pub | 11 | strip_review_section(content: &str) -> String |
| `upsert_frontmatter_field` | projects/agentic-workflow/src/tools/review_helpers.rs | function | pub | 36 | upsert_frontmatter_field(content: &str, key: &str, value: &str) -> String |
## Source
<!-- type: source lang: rust -->

````rust
//! Shared helpers for inline review sections in artifact files.
//!
//! Used by `review.rs` (writing inline reviews) and `file_service.rs` (reading them).

use serde_json::Value;

/// Strip the `# Reviews` section (and everything after) from content.
pub fn strip_review_section(content: &str) -> String {
    if let Some(idx) = content.find("\n# Reviews") {
        content[..idx].trim_end().to_string()
    } else if content.starts_with("# Reviews") {
        String::new()
    } else {
        content.trim_end().to_string()
    }
}

/// Extract the `# Reviews` section content from an artifact file.
pub fn extract_review_section(content: &str) -> Option<String> {
    let idx = if let Some(idx) = content.find("\n# Reviews") {
        idx + 1
    } else if content.starts_with("# Reviews") {
        0
    } else {
        return None;
    };
    Some(content[idx..].to_string())
}

/// Add or update a field in YAML frontmatter.
pub fn upsert_frontmatter_field(content: &str, key: &str, value: &str) -> String {
    if !content.starts_with("---\n") {
        return format!("---\n{}: {}\n---\n\n{}", key, value, content);
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return content.to_string(),
    };
    let fm_text = &content[4..closing];
    let after = &content[closing + 4..]; // skip "\n---"

    let key_prefix = format!("{}:", key);
    let new_line = format!("{}: {}", key, value);
    let mut found = false;
    let new_lines: Vec<String> = fm_text
        .lines()
        .map(|line| {
            if line.starts_with(&key_prefix) {
                found = true;
                new_line.clone()
            } else {
                line.to_string()
            }
        })
        .collect();

    let mut fm = new_lines.join("\n");
    if !found {
        fm.push('\n');
        fm.push_str(&new_line);
    }
    format!("---\n{}\n---{}", fm, after)
}

/// Remove a field from YAML frontmatter.
pub fn remove_frontmatter_field(content: &str, key: &str) -> String {
    if !content.starts_with("---\n") {
        return content.to_string();
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return content.to_string(),
    };
    let fm_text = &content[4..closing];
    let after = &content[closing + 4..];

    let key_prefix = format!("{}:", key);
    let new_lines: Vec<&str> = fm_text
        .lines()
        .filter(|line| !line.starts_with(&key_prefix))
        .collect();

    format!("---\n{}\n---{}", new_lines.join("\n"), after)
}

/// Build the `# Reviews` markdown section for inline reviews.
pub fn build_review_section(
    verdict: &str,
    summary: &str,
    checklist: &[Value],
    issues: &[Value],
    iteration: u64,
    display_name: &str,
    change_id: &str,
) -> String {
    let mut md = String::new();
    md.push_str("# Reviews\n\n");
    md.push_str(&format!(
        "## Review: {} (Iteration {})\n\n",
        display_name, iteration
    ));
    md.push_str(&format!("**Change ID**: {}\n\n", change_id));
    md.push_str(&format!("**Verdict**: {}\n\n", verdict));

    md.push_str("### Summary\n\n");
    md.push_str(summary);
    md.push_str("\n\n");

    if !checklist.is_empty() {
        md.push_str("### Checklist\n\n");
        for item in checklist {
            let name = item.get("item").and_then(|v| v.as_str()).unwrap_or("");
            let passed = item
                .get("passed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let note = item.get("note").and_then(|v| v.as_str());
            let icon = if passed { "✅" } else { "❌" };
            md.push_str(&format!("- {} {}\n", icon, name));
            if let Some(n) = note {
                md.push_str(&format!("  - {}\n", n));
            }
        }
        md.push('\n');
    }

    md.push_str("### Issues\n\n");
    if issues.is_empty() {
        md.push_str("No issues found.\n");
    } else {
        for issue in issues {
            let severity = issue
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("MEDIUM");
            let desc = issue
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let rec = issue.get("recommendation").and_then(|v| v.as_str());
            md.push_str(&format!("- **[{}]** {}\n", severity, desc));
            if let Some(r) = rec {
                md.push_str(&format!("  - *Recommendation*: {}\n", r));
            }
        }
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_review_section() {
        let content =
            "---\nid: test\n---\n\n# Proposal\n\nContent.\n\n# Reviews\n\n## Review\n\nOld.\n";
        let stripped = strip_review_section(content);
        assert!(stripped.contains("# Proposal"));
        assert!(stripped.contains("Content."));
        assert!(!stripped.contains("# Reviews"));
    }

    #[test]
    fn test_strip_review_section_no_reviews() {
        let content = "---\nid: test\n---\n\n# Proposal\n\nContent.\n";
        let stripped = strip_review_section(content);
        assert!(stripped.contains("# Proposal"));
        assert!(stripped.contains("Content."));
    }

    #[test]
    fn test_extract_review_section() {
        let content =
            "---\nid: test\n---\n\n# Proposal\n\nContent.\n\n# Reviews\n\n## Review\n\nOld.\n";
        let section = extract_review_section(content).unwrap();
        assert!(section.starts_with("# Reviews"));
        assert!(section.contains("Old."));
    }

    #[test]
    fn test_extract_review_section_none() {
        let content = "---\nid: test\n---\n\n# Proposal\n\nContent.\n";
        assert!(extract_review_section(content).is_none());
    }

    #[test]
    fn test_upsert_frontmatter_field_add() {
        let content = "---\nid: test\ntype: proposal\n---\n\n# Body\n";
        let result = upsert_frontmatter_field(content, "review_verdict", "REVIEWED");
        assert!(result.contains("review_verdict: REVIEWED"));
        assert!(result.contains("id: test"));
        assert!(result.contains("type: proposal"));
    }

    #[test]
    fn test_upsert_frontmatter_field_update() {
        let content = "---\nid: test\nreview_verdict: REVIEWED\n---\n\n# Body\n";
        let result = upsert_frontmatter_field(content, "review_verdict", "REJECTED");
        assert!(result.contains("review_verdict: REJECTED"));
        assert!(!result.contains("review_verdict: REVIEWED"));
    }

    #[test]
    fn test_upsert_frontmatter_field_no_frontmatter() {
        let content = "# No frontmatter\n\nBody.\n";
        let result = upsert_frontmatter_field(content, "review_verdict", "REVIEWED");
        assert!(result.contains("---\nreview_verdict: REVIEWED\n---"));
        assert!(result.contains("# No frontmatter"));
    }

    #[test]
    fn test_remove_frontmatter_field() {
        let content = "---\nid: test\nreview_verdict: REVIEWED\ntype: proposal\n---\n\n# Body\n";
        let result = remove_frontmatter_field(content, "review_verdict");
        assert!(!result.contains("review_verdict"));
        assert!(result.contains("id: test"));
        assert!(result.contains("type: proposal"));
    }

    #[test]
    fn test_remove_frontmatter_field_not_present() {
        let content = "---\nid: test\ntype: proposal\n---\n\n# Body\n";
        let result = remove_frontmatter_field(content, "review_verdict");
        assert_eq!(result, content);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/review_helpers.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-review-helpers-rs>"
    description: "Inline review section helper functions."
```
