---
change_id: per-task-impl-loop
type: spec_context
created_at: 2026-02-09T08:23:15.885094+00:00
updated_at: 2026-02-09T08:23:15.885094+00:00
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
  - reason: Orchestrates the 37-phase state machine including implement stage
  - key sections: Full State Machine, Implement Stage (4-stage), Action-Phase-Agent Mapping
- **impl-change/workflow** (group: cclab-genesis)
  - relevance: high
  - reason: Defines the current implementation workflow — needs refactoring from change-level to task-level loop
  - key sections: State Diagram, Sequential Implementation Loop
- **state-management** (group: cclab-genesis)
  - relevance: high
  - reason: STATE.yaml schema and StatePhase enum — needs per-task tracking fields
  - key sections: StatePhase Enum, StateManager API, Revision Counts
- **plan-change** (group: cclab-genesis)
  - relevance: medium
  - reason: Generates tasks.md which serves as input for the implementation loop
  - key sections: Task Generation, Acceptance Criteria
- **consolidate-read-tools** (group: cclab-genesis)
  - relevance: low
  - reason: Recently merged — uses genesis_read_file for reading tasks/specs during implementation
  - key sections: Scope prefix syntax

## Dependencies

- impl-change/workflow depends on tasks.md from plan-change
- run-change routes to implement.rs for implementation stage
- All workflows depend on state-management for persistence and revision tracking
- implement.rs uses genesis_get_task to read individual tasks

## Gaps

- STATE.yaml lacks fields for current_task_id and per-task revision counters
- implement.rs is change-level — needs task-level loop with next-pending-task detection
- genesis_review_implementation creates single REVIEW_IMPL.md — per-task loop needs per-task review artifacts or structured multi-review
- Max 2 revisions limit is change-wide — needs task-level enforcement
- tasks.md checkbox parsing needed to auto-detect completion
