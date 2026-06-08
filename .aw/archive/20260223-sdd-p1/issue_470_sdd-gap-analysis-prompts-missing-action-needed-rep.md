---
number: 470
title: "SDD: Gap analysis prompts missing action_needed, repair_action, and type enum values"
state: open
labels: [bug, P1, crate:sdd]
---

# #470 — SDD: Gap analysis prompts missing action_needed, repair_action, and type enum values

## Summary

All three gap analysis create prompts are significantly shallower than the spec requires. Agents following the Rust prompts will produce artifacts missing required schema fields.

## Missing from Implementation Prompts

| Field | Spec requires | Implementation provides |
|-------|--------------|----------------------|
| `action_needed` | Decide if gap should be fixed in this change | Not mentioned |
| `repair_action` | Describe concrete repair action | Not mentioned |
| Gap type enum | `code_without_spec`, `spec_without_code`, `convention_violation`, `pattern_mismatch`, `undocumented_pattern`, `spec_contradicts_knowledge`, etc. | Natural language only, no enum values |
| Relevance map step | "If spec-issue relevance map exists, focus on those" | Absent |

## Affected Files

- `gap_codebase_spec.rs:67-74` — missing `action_needed`, `repair_action`, type classification
- `gap_codebase_knowledge.rs:67-74` — also missing `undocumented_pattern` type entirely
- `gap_spec_knowledge.rs:67-74` — same pattern

## Spec References

- `create-gap-codebase-spec.md:34-46` (payload schema) and lines 65-80 (prompt template)
- `create-gap-codebase-knowledge.md:40` (type enum including `undocumented_pattern`)
- `create-gap-spec-knowledge.md:65-75` (prompt template)

## Impact

Downstream phases that read gap artifacts expect these fields. Missing data degrades proposal and spec creation quality.
