---
id: 196
type: proposal
version: 1
created_at: 2026-02-12T08:04:23.653760+00:00
updated_at: 2026-02-12T08:04:23.653760+00:00
author: mcp
status: proposed
iteration: 1
summary: "Fix 4 known merge-change bugs: verdict mismatch, REVIEW_MERGE format, producer-less phases, filename casing"
history:
  - timestamp: 2026-02-12T08:04:23.653760+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 4
  new_files: 0
affected_specs:
  - id: merge-change-bugfixes
    path: specs/merge-change-bugfixes.md
    depends: []
---

<proposal>

# Change: 196

## Summary

Fix 4 known merge-change bugs: verdict mismatch, REVIEW_MERGE format, producer-less phases, filename casing

## Why

The merge-change workflow has 4 documented bugs that cause incorrect routing: (1) merge.rs still accepts NEEDS_REVISION alongside REVIEWED, (2) REVIEW_MERGE.md is written without YAML frontmatter causing extract_review_info to return None, (3) merged and merge_approved phases exist in routing but no tool produces them, (4) REVIEW_MERGE.md uses uppercase naming inconsistent with v2 convention. These bugs prevent the merge review cycle from working correctly in agent-delegated flows.

## What Changes

- Fix merge.rs to only accept APPROVED/REVIEWED/REJECTED verdicts (remove NEEDS_REVISION/NEEDS_FIX)
- Update implementation_service.rs create_merge_review to write YAML frontmatter before review content
- Document that merged phase is produced by genesis_review_merge (was already correct) and merge_approved is set manually
- Update merge-change.md spec to remove Implementation Notes (bugs fixed) and clarify phase producers
- Rename REVIEW_MERGE.md to review_merge.md in both code and spec for v2 naming consistency

## Impact

- **Scope**: minor
- **Affected Files**: ~4
- **New Files**: ~0
- Affected specs:
  - `merge-change-bugfixes` (no dependencies)
- Affected code: `crates/cclab-genesis/src/mcp/tools/run_change/merge.rs`, `crates/cclab-genesis/src/services/implementation_service.rs`, `crates/cclab-genesis/src/mcp/tools/run_change/helpers.rs`, `cclab/specs/cclab-genesis/merge-change.md`

</proposal>
