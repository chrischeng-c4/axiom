---
number: 478
title: "SDD: implement_task_with_codegen action undocumented in spec"
state: open
labels: [enhancement, P2, crate:sdd]
---

# #478 — SDD: implement_task_with_codegen action undocumented in spec

## Summary

The implementation has a complete codegen path (`implement_task_with_codegen`) that is not documented in the spec at all.

## Spec (implement-change.md:83)

Action enum lists:
> `begin_implementation`, `implement_task`, `review_task`, `revise_task`, `task_terminal_failure`, `all_tasks_done`, `resume_implementation`, `review_implementation`, `resolve_implementation`, `implementation_complete`

No `implement_task_with_codegen`.

## Implementation (implement.rs:374-431)

Complete codegen path:
- `Action::ImplementTaskWithCodegen { task_id, spec_ref }`
- `is_codegen_eligible()` helper determines eligibility
- Uses `prism_generate_from_spec` pipeline for SpecIR-eligible tasks
- Returns `action: "implement_task_with_codegen"` with `spec_ref` field

## Additional Undocumented Implementation Behaviors

- `impl_reviewed` + APPROVED always returns `all_tasks_done` instead of `implement_task` for next task (`implement.rs:206-209`)
- `LegacyReview` sets `next_phase` to unparseable string `"impl_approved or impl_reviewed"` (`implement.rs:551`)
- Fallback from per-task to legacy when `current_task_id` is None (`implement.rs:227-234`)

## Action

Document the codegen path in `implement-change.md` and add `implement_task_with_codegen` to the action enum.
