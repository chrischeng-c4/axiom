---
change_id: 196
type: spec_context
created_at: 2026-02-12T08:00:53.021676+00:00
updated_at: 2026-02-12T08:00:53.021676+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-genesis
---

# Spec Context

## Relevant Specs

- **merge-change** (group: cclab-genesis)
  - relevance: high
  - reason: Primary target for bug fixes (routing, verdict mismatch, artifact format)
  - key sections: Phase Routing, Implementation Notes, Artifact Format: REVIEW_MERGE.md
- **verdict-unification** (group: cclab-genesis)
  - relevance: high
  - reason: Provides the standard for verdict enums (APPROVED/REVIEWED/REJECTED)
  - key sections: R1 - Unify spec verdict names, R5 - Update routing logic
- **review-context** (group: cclab-genesis)
  - relevance: medium
  - reason: Provides the pattern for structured review artifacts with YAML frontmatter
  - key sections: Artifact Format (review)

## Dependencies

- cclab-genesis/verdict-unification
- cclab-genesis/review-context

## Gaps

- Verdict terminology mismatch: 'REVIEWED' (enum) vs 'NEEDS_FIX'/'NEEDS_REVISION' (spec/routing)
- REVIEW_MERGE.md missing YAML frontmatter (parser expects it, tool doesn't write it)
- Producer-less phases: 'merged' and 'merge_approved' exist in routing but aren't produced by tools
- Filename casing inconsistency: REVIEW_MERGE.md vs potential ReviewMerge conventions
