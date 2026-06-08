---
change_id: 198
type: spec_context
created_at: 2026-02-12T08:20:39.403700+00:00
updated_at: 2026-02-12T08:20:39.403700+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-genesis
---

# Spec Context

## Relevant Specs

- **run-change** (group: cclab-genesis)
  - relevance: high
  - reason: Target file — action enum needs sync
  - key sections: OpenRPC action enum, Prompt Sources table
- **implement-change** (group: cclab-genesis)
  - relevance: high
  - reason: Source of missing actions: review_task, revise_task, task_terminal_failure, all_tasks_done
  - key sections: Action enum, Per-Task Path
- **merge-change** (group: cclab-genesis)
  - relevance: medium
  - reason: Source of missing action: merge_complete
  - key sections: Phase Routing

## Gaps

- run-change/README.md action enum missing 5 actions from sub-specs
- run-change/README.md has orphan 'complete' action
