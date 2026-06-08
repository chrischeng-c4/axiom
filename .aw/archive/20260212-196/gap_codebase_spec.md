---
change_id: 196
type: gap_codebase_spec
created_at: 2026-02-12T08:03:47.930143+00:00
updated_at: 2026-02-12T08:03:47.930143+00:00
---

# Gap Analysis: Codebase vs Spec (merge-change bugs)

## Gaps Found

### Gap 1: merge.rs checks NEEDS_REVISION but spec says REVIEWED
- **Severity**: high
- **Source A**: `merge.rs:47` checks `Some("REVIEWED") | Some("NEEDS_REVISION")`
- **Source B**: merge-change.md spec says verdict should be REVIEWED per unified standard
- **Details**: Code still accepts legacy NEEDS_REVISION alongside REVIEWED. Should only accept REVIEWED.

### Gap 2: REVIEW_MERGE.md written without frontmatter
- **Severity**: high
- **Source A**: `implementation_service.rs` writes checkbox format without `---` frontmatter
- **Source B**: merge-change.md spec defines artifact format with YAML frontmatter + checkbox
- **Details**: `extract_review_info` in helpers.rs parses frontmatter, returns None for checkbox-only format

### Gap 3: merged/merge_approved phases have no producer
- **Severity**: medium
- **Source A**: `merge.rs:61,71` routes `Merged` and `MergeApproved` phases
- **Source B**: No tool writes these phases — genesis_review_merge writes `merge_reviewed`, not `merged`
- **Details**: These phases exist in routing but no tool produces them

### Gap 4: REVIEW_MERGE.md uppercase naming
- **Severity**: low
- **Source A**: Code uses `REVIEW_MERGE.md` (uppercase)
- **Source B**: v2 convention uses lowercase: `review_proposal.md`, `review_spec_{id}.md`
- **Details**: Inconsistent with v2 naming convention

## Summary

4 gaps found (2 high, 1 medium, 1 low).