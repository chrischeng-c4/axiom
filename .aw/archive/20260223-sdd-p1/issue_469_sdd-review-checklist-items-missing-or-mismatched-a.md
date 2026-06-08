---
number: 469
title: "SDD: Review checklist items missing or mismatched across all phases"
state: open
labels: [bug, P1, crate:sdd]
---

# #469 — SDD: Review checklist items missing or mismatched across all phases

## Summary

Review prompts in the Rust implementation contain fewer and different checklist items compared to what the spec defines. This directly affects review quality — agents won't check for required fields.

## Detailed Comparison

### Gap Analysis (all 3 gap phases)
- **Spec**: 6 checklist items including `action_needed` check, `repair_action` concreteness, gap type verification
- **Implementation**: 4 items — missing `action_needed`, `repair_action`, and type enum checks
- Files: `gap_codebase_spec.rs:89-94`, `gap_codebase_knowledge.rs:89-94`, `gap_spec_knowledge.rs:90-95`

### Proposal Review
- **Spec**: 7 structured items (DAG validity, gap coverage, context_refs, spec coherence, orphan specs, impact scope, scope_areas)
- **Implementation**: 5 vague criteria ("clarity, value, completeness, feasibility, impact accuracy")
- Files: `review-change-proposal.md:34-40` vs `proposal.rs:138-140`

### Post-Clarifications Review
- **Spec**: 5 items (cross-reference completeness, contradiction identification, resolution quality, unaddressed contradictions, resolution consistency)
- **Implementation**: 3 items — missing resolution quality and consistency checks
- Files: `review-spec-clarifications.md:33-38` vs `clarify.rs:426-429`

### Explore Context Reviews (three-way inconsistency)
- Knowledge context item 3: spec says "with source and description", create-phase checklist says "with name + source + description", review-phase checklist says "with examples"
- Codebase context item 2: spec says "Each file has path + symbols present", create-phase says "path + symbols + role", review-phase says "Each symbol has file path"
- Files: `explore_knowledge.rs:74,108`, `explore_codebase.rs:81,122`

### Spec Context Review
- **Spec**: 5 items
- **Implementation**: 6 items (extra: `codebase_paths`/`knowledge_refs` surfaced check)
- Minor: impl adds useful item not in spec

## Action

Align all review checklists — either update spec to match simpler impl, or (recommended) bring implementation checklists up to spec completeness.
