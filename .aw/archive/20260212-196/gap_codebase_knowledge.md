---
change_id: 196
type: gap_codebase_knowledge
created_at: 2026-02-12T08:03:56.275216+00:00
updated_at: 2026-02-12T08:03:56.275216+00:00
---

# Gap Analysis: Codebase vs Knowledge (merge-change bugs)

## Gaps Found

### Gap 1: extract_review_info doesn't handle checkbox format
- **Severity**: high
- **Source A**: `helpers.rs` extract_review_info parses YAML frontmatter only
- **Source B**: Knowledge pattern says review artifacts should have frontmatter
- **Details**: Either the tool needs to write frontmatter, or the parser needs to handle checkbox format

## Summary

1 gap found (1 high, 0 medium, 0 low).