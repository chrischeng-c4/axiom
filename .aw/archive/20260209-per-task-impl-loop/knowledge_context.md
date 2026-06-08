---
change_id: per-task-impl-loop
type: knowledge_context
created_at: 2026-02-09T08:35:22.894372+00:00
updated_at: 2026-02-09T08:35:22.894372+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - architecture
  - patterns
  - mcp
  - workflow
  - testing
---

# Knowledge Context

## Relevant Documents

- **cclab/knowledge/40-mcp/dynamic-config.md**
  - summary: Dynamic tool loading to reduce LLM overhead per workflow stage
  - relevant sections: Tool Filtering by Stage, Implementation Strategy
- **cclab/knowledge/40-mcp/index.md**
  - summary: MCP architecture overview — tool registration, stage filtering
  - relevant sections: Tool Registry, Stage Filtering
- **cclab/knowledge/30-claude/skills.md**
  - summary: Claude skill definitions including genesis:run-change and genesis:agent
  - relevant sections: Skill Format, Agent Invocation

## Patterns

- **4-stage Review Loop** (source: crates/cclab-genesis/src/mcp/tools/spec.rs)
  - Standardized Create-Review-Revise-Approve cycle with max 2 revisions. Used for proposals, specs, impl, merge. Per-task loop should reuse this pattern.
- **Structured Review Artifacts** (source: cclab/specs/cclab-genesis/run-change.md)
  - REVIEW_SPEC_{id}.md pattern for granular review verdicts. Per-task needs REVIEW_TASK_{id}.md or equivalent.
- **Revision Counter in STATE.yaml** (source: crates/cclab-genesis/src/mcp/tools/state_update.rs)
  - Auto-increment revision_counts on *Revised transitions. Per-task needs task-level counters e.g. task:{id} keys.
- **TaskGraph for dependency ordering** (source: crates/cclab-genesis/src/models/task_graph.rs)
  - Topological sort of tasks respecting depends_on. Per-task loop must pick next task in dependency order.

## Pitfalls

- Infinite review loops without revision limits — must enforce max 2 per task
- State desynchronization on task interruption — current_task_id in STATE.yaml needed for resume
- Dependency violations in task execution order — must use TaskGraph topological sort
- tasks.md checkbox parsing must handle edge cases (partial checks, nested lists)
