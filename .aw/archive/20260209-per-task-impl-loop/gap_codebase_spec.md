---
change_id: per-task-impl-loop
type: gap_codebase_spec
created_at: 2026-02-09T08:48:10.526184+00:00
updated_at: 2026-02-09T08:48:10.526184+00:00
---

# Gap Analysis: Codebase vs Spec for 'per-task-impl-loop'

## Code without Specification

### 1. TaskGraph Model
- **File**: `crates/cclab-genesis/src/models/task_graph.rs`
- **Gap**: High-complexity logic for parsing `tasks.md` and calculating execution order (topological sort) exists in code but is not described in `plan-change` or `impl-change` specs.
- **Severity**: Medium

### 2. Verification Mapping
- **File**: `crates/cclab-genesis/src/models/verification.rs`
- **Gap**: Utility mapping of implementation status to markdown symbols for `tasks.md` integration is implemented but undocumented in specs.
- **Severity**: Low

## Specification without Implementation

### 1. Per-Task Implementation Loop
- **Spec**: `impl-change/workflow`
- **Gap**: Specification requires a refactored loop that iterates through tasks. Current `implement.rs` uses a change-level state machine (Planned -> Implementing -> Implemented).
- **Severity**: High

### 2. State Task Tracking
- **Spec**: `state-management`
- **Gap**: Requires `current_task_id` and task-specific revision counters in `STATE.yaml`. `StateManager` and `frontmatter.rs` currently lack these fields.
- **Severity**: High

### 3. Task Execution Sequencing
- **Spec**: `impl-change/workflow`
- **Gap**: Requires `implement.rs` to detect the next pending task using `TaskGraph`. Currently, `implement.rs` does not load or use `TaskGraph`.
- **Severity**: High

### 4. Per-Task Review Artifacts
- **Spec**: `impl-change/workflow`
- **Gap**: Requires granular reviews for each task. `genesis_review_implementation` and `implementation_service.rs` only support a single `REVIEW_IMPL.md` for the entire change.
- **Severity**: Medium

### 5. Task-Level Revision Limits
- **Spec**: `state-management` / `impl-change/workflow`
- **Gap**: Requires enforcing revision limits at the task level. Current implementation uses `revision_counts` at the change level in `StateManager`.
- **Severity**: Medium
