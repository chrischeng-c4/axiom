---
change: 196
date: 2026-02-12
---

# Clarifications

## Q1: Change Type
- **Question**: Is this a spec-only change or code+spec?
- **Answer**: Both spec and code changes. merge-change.md spec needs updates, and merge.rs + review_merge tool code needs bug fixes.
- **Rationale**: All 4 bugs require both spec corrections and corresponding code fixes.

## Q2: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place — work on current branch
- **Rationale**: All 12 genesis specs consistency issues (#193-#204) use in_place workflow.

## Q3: Bug Priority
- **Question**: Which bugs to fix in this change?
- **Answer**: All 4 bugs: (1) Verdict mismatch NEEDS_FIX vs NEEDS_REVISION — unify to REVIEWED per #193 verdict unification, (2) REVIEW_MERGE.md frontmatter parsing, (3) Producer-less phases merged/merge_approved, (4) Filename casing REVIEW_MERGE.md → review_merge.md.
- **Rationale**: These are all documented in the same issue and are closely related merge-change bugs.

## Q4: Verdict Alignment
- **Question**: What verdict values should merge review use?
- **Answer**: APPROVED/REVIEWED/REJECTED per #193 verdict unification. NEEDS_FIX and NEEDS_REVISION should both map to REVIEWED.
- **Rationale**: Issue #193 established the three-verdict standard across the entire genesis workflow.

