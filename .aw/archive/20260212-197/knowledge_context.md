---
change_id: 197
type: knowledge_context
created_at: 2026-02-12T08:16:36.710888+00:00
updated_at: 2026-02-12T08:16:36.710888+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - genesis
  - core-conventions
  - workflow-orchestration
---

# Knowledge Context

## Relevant Documents

- **cclab-genesis/delegate-agent.md**
  - summary: Defines verified agent dispatch with post-execution state/artifact checks.
  - relevant sections: Verification Table, Sequence Diagram, Behavior Flowchart
- **cclab-genesis/run-change/README.md**
  - summary: Orchestrates the unified workflow state machine with review/revise loops and retry limits.
  - relevant sections: Phase: Context Create/Review/Revise Lifecycle, Three-Verdict Routing, Post-Clarify Action Detail
- **cclab-genesis/implement-change.md**
  - summary: Defines the per-task implementation loop with revision limits and topological ordering.
  - relevant sections: Phase Routing Table (Per-Task Path), Sequence Diagram (Terminal Failure), Task Execution Order
- **cclab-genesis/verdict-unification.md**
  - summary: Standardizes verdicts to APPROVED/REVIEWED/REJECTED for consistent routing.
  - relevant sections: R1 - Unify spec verdict names

## Patterns

- **Hierarchical Task Loop** (source: cclab-genesis/implement-change.md)
  - Break down complex changes into smaller tasks with individual implement-review-revise cycles.
- **Revision Throttling (Retry Limits)** (source: cclab-genesis/run-change/README.md)
  - Limit automated revision attempts (usually 2 for tasks, 4 for context/specs) before escalating to terminal failure.
- **User-in-the-Loop Hooks** (source: cclab-genesis/run-change/README.md)
  - Agents must use AskUserQuestion to seek clarification, and specific 'post_clarify' phases allow resolving contradictions.
- **Deterministic Topological Ordering** (source: cclab-genesis/implement-change.md)
  - Use Kahn's algorithm with lexical tie-breaking to ensure a predictable and cycle-aware execution order.
- **Checkpoint-Based Resumption** (source: cclab-genesis/implement-change.md)
  - Store 'current_task_id' or similar state in STATE.yaml to allow resuming long-running workflows from the last checkpoint.

## Pitfalls

- Infinite recursion if agents are allowed to call delegate-agent (blocked by DisallowedMcpTools).
- Silent failures if verification logic in delegate-agent is not strictly aligned with actual phase outputs.
- Cyclic dependencies in tasks.md leading to unexecutable implementation plans.
