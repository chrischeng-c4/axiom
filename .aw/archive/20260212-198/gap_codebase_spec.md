---
change_id: 198
type: gap_codebase_spec
created_at: 2026-02-12T08:21:07.211451+00:00
updated_at: 2026-02-12T08:21:07.211451+00:00
---

# Gap Analysis: Codebase vs Spec

## Gaps Found

### Gap 1: Action enum missing 5 sub-spec actions
- **Severity**: high
- **Source A**: run-change/README.md action enum (lines 40-55)
- **Source B**: implement-change.md and merge-change.md action definitions
- **Details**: Missing: review_task, revise_task, task_terminal_failure, all_tasks_done, merge_complete

### Gap 2: Orphan 'complete' action
- **Severity**: medium
- **Source A**: run-change/README.md:54 lists 'complete'
- **Source B**: No sub-spec defines behavior for 'complete'
- **Details**: Should be removed or replaced with merge_complete

## Summary
2 gaps found (1 high, 1 medium, 0 low)."