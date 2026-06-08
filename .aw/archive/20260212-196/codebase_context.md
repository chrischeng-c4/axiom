---
change_id: 196
type: codebase_context
created_at: 2026-02-12T08:03:28.212654+00:00
updated_at: 2026-02-12T08:03:28.212654+00:00
iteration: 1
complexity: medium
stage: codebase
prism_tools_used:
  - manual_inspection
---

# Codebase Context

## Analyzed Files

- **crates/cclab-genesis/src/mcp/tools/run_change/merge.rs** — Merge flow routing - checks verdict, routes to actions
  - symbols: `handle`, `Action`
- **crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs** — Verdict extraction from review files
  - symbols: `extract_review_info`
- **crates/cclab-genesis/src/services/implementation_service.rs** — Creates REVIEW_MERGE.md artifact
  - symbols: `create_merge_review`, `MergeReviewVerdict`
- **cclab/specs/cclab-genesis/merge-change.md** — Merge-change spec with known bugs documented in Implementation Notes

## Dependency Graph

- merge.rs uses helpers::extract_review_info to parse REVIEW_MERGE.md
- implementation_service.rs writes REVIEW_MERGE.md
- merge-change.md spec defines the expected behavior
