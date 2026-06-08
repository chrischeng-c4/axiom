---
change_id: per-task-impl-loop
type: gap_spec_knowledge
created_at: 2026-02-09T09:41:47.107958+00:00
updated_at: 2026-02-09T09:41:47.107958+00:00
---

# Gap Analysis: Spec vs Knowledge

## 1. Responsibility Boundary Misalignments

- **Revision Limit Scope** (Severity: High)
  - **Observation:** `cclab/specs/cclab-genesis/run-change.md` (via `spec_context`) defines revision limits as change-wide, whereas the "4-stage Review Loop" pattern in knowledge architecture (referenced in `crates/cclab-genesis/src/mcp/tools/spec.rs`) and pitfall warnings in `knowledge_context` emphasize the need for enforcement at the granular "per task" level to prevent infinite loops during implementation.
  - **Reference:** `knowledge_context.md#patterns` (4-stage Review Loop), `spec_context.md#gaps`.

- **Review Artifact Granularity** (Severity: High)
  - **Observation:** The "Structured Review Artifacts" pattern (e.g., `REVIEW_SPEC_{id}.md` in `cclab/specs/cclab-genesis/run-change.md`) establishes a convention for granular review results. However, `impl-change/workflow` and `genesis_review_implementation` currently produce a monolithic `REVIEW_IMPL.md`, failing to align implementation reviews with the established architectural pattern for entity-specific artifacts.
  - **Reference:** `knowledge_context.md#patterns` (Structured Review Artifacts).

## 2. Knowledge Patterns Not Reflected in Specs

- **Dependency-Aware Orchestration** (Severity: Medium)
  - **Observation:** The `TaskGraph` utility for topological sorting (`crates/cclab-genesis/src/models/task_graph.rs`) is an established knowledge pattern, but the `impl-change/workflow` spec does not explicitly define its use as the mechanism for determining the "next pending task" in the implementation loop.
  - **Reference:** `knowledge_context.md#patterns` (TaskGraph for dependency ordering).

- **Resume Capability & State Persistence** (Severity: High)
  - **Observation:** Knowledge patterns for `STATE.yaml` (in `crates/cclab-genesis/src/mcp/tools/state_update.rs`) include auto-incrementing revision counters, but the current `state-management` spec lacks definitions for `current_task_id` and task-specific counters required to support the "State desynchronization" pitfall prevention documented in knowledge.
  - **Reference:** `knowledge_context.md#pitfalls` (State desynchronization on task interruption).

## 3. Implementation Contradictions

- **Loop Level Inconsistency** (Severity: High)
  - **Observation:** `impl-change/workflow` defines a sequential implementation loop at the change level, which contradicts the project's push towards per-task granularity as seen in `plan-change` (which generates discrete tasks) and the requirement for task-level revision tracking in the knowledge base.
  - **Reference:** `spec_context.md#relevant-specs` (impl-change/workflow), `knowledge_context.md#patterns` (4-stage Review Loop).
