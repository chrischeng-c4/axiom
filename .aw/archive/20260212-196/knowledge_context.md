---
change_id: 196
type: knowledge_context
created_at: 2026-02-12T08:03:03.970576+00:00
updated_at: 2026-02-12T08:03:03.970576+00:00
iteration: 1
complexity: medium
stage: knowledge
scanned_categories:
  - genesis/specs/cclab-genesis
  - genesis/specs/genesis
---

# Knowledge Context

## Relevant Documents

- **cclab-genesis/merge-change.md**
  - summary: Defines the terminal phase of the genesis SDD workflow, covering spec merging, review quality assessment, and archiving. Highlighting issues with verdict parsing and terminology mismatch.
  - relevant sections: Phase Routing, Merge Logic, Implementation Notes
- **genesis/merge-change-tool.md**
  - summary: Describes the algorithm for the genesis_merge_change MCP tool, including the state machine transitions (Implemented -> Merging -> Archived) and the MergeAction variants.
  - relevant sections: R1 - MergeAction Enum, R2 - State Analysis, Diagrams
- **cclab-genesis/verdict-unification.md**
  - summary: The standard for unifying all genesis verdict enums to APPROVED/REVIEWED/REJECTED, which is a core part of the fixes for change 196.
  - relevant sections: R1 - Unify spec verdict names, R3 - Unify Rust verdict enums

## Patterns

- **Merge Phase State Machine** (source: genesis/merge-change-tool.md)
  - Phases should follow the Implemented -> Merging -> Archived flow, with intermediate review and fix stages.
- **Standardized Verdicts** (source: cclab-genesis/verdict-unification.md)
  - All review tools should produce standardized verdicts: APPROVED, REVIEWED, or REJECTED.
- **Consistent Review Artifacts** (source: cclab-genesis/merge-change.md)
  - Review artifacts like REVIEW_MERGE.md should have consistent casing and structured frontmatter for reliable parsing.

## Pitfalls

- Verdict terminology mismatch (e.g., NEEDS_FIX vs REVIEWED) causes routing loops.
- Missing frontmatter in tool-generated reviews prevents the parser from extracting verdicts.
- Producer-less phases (merged, merge_approved) create gaps in tool-orchestrated workflows.
