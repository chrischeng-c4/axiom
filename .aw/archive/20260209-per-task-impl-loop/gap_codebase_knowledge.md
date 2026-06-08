---
change_id: per-task-impl-loop
type: gap_codebase_knowledge
created_at: 2026-02-09T09:38:25.033787+00:00
updated_at: 2026-02-09T09:38:25.033787+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations

- **File: crates/cclab-genesis/src/mcp/tools/run_change/implement.rs**
  - **Violation:** Implementation phase utilizes a single `REVIEW_IMPL.md` file.
  - **Knowledge Ref:** `cclab/specs/cclab-genesis/run-change.md#Action ↔ Phase ↔ Agent Mapping`
  - **Gap:** The implementation stage produces a monolithic review artifact for the entire phase, unlike the planning stage which utilizes granular, per-item review artifacts (e.g., `REVIEW_SPEC_{id}.md`).
  - **Severity:** HIGH

- **File: crates/cclab-genesis/src/models/task_graph.rs & crates/cclab-genesis/src/models/verification.rs**
  - **Violation:** Redundant definitions of `TaskStatus` enum.
  - **Knowledge Ref:** `cclab/schemas/tasks.schema.json#definitions/task_block/properties/status`
  - **Gap:** Multiple internal model definitions for `TaskStatus` exist in `task_graph.rs` and `verification.rs`, creating redundant representations of the concept defined in the central task schema.
  - **Severity:** LOW

## Pattern Mismatches

- **File: crates/cclab-genesis/src/mcp/tools/run_change/implement.rs**
  - **Mismatch:** Handler lacks integration with `TaskGraph` for execution sequencing.
  - **Knowledge Ref:** `cclab/specs/cclab-genesis/impl-change/workflow.md#Sequential Implementation Loop`
  - **Gap:** Task execution in the handler follows a linear progression instead of utilizing the topological sort provided by the `TaskGraph::get_execution_order()` method for dependency-aware sequencing.
  - **Severity:** HIGH

- **File: crates/cclab-genesis/src/models/frontmatter.rs (State struct)**
  - **Mismatch:** Missing `current_task_id` field in the `State` model.
  - **Knowledge Ref:** `cclab/specs/cclab-genesis/state-management.md` (Pitfall: State desynchronization on task interruption)
  - **Gap:** The `State` struct tracks the overall phase but lacks a field for the active task ID, preventing precise state recovery and resumption of the implementation loop after an interruption.
  - **Severity:** HIGH

- **File: crates/cclab-genesis/src/mcp/tools/state_update.rs**
  - **Mismatch:** Revision counts are implemented at the phase level only.
  - **Knowledge Ref:** `cclab/specs/cclab-genesis/run-change.md#Revision Counters`
  - **Gap:** Currently, `revision_counts` only track global phase transitions, omitting the task-level counters required to monitor and limit iteration counts for individual tasks.
  - **Severity:** MEDIUM

- **File: crates/cclab-genesis/src/mcp/tools/run_change/implement.rs**
  - **Mismatch:** Handler utilizes a single-pass state machine instead of an iterative 4-stage loop.
  - **Knowledge Ref:** `cclab/specs/cclab-genesis/run-change.md#Full State Machine (37 Phases)`
  - **Gap:** The handler logic implements a linear Planned-to-Implemented transition, skipping the iterative (Implementing -> Testing -> Implemented -> Reviewed) stages defined in the unified workflow state machine.
  - **Severity:** MEDIUM
