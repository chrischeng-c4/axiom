---
id: impl-merge-mcp-tools
change_id: impl-merge-mcp-tools
type: tasks
version: 1
created_at: 2026-02-05T16:03:56.108149+00:00
updated_at: 2026-02-05T16:03:56.108149+00:00
proposal_ref: impl-merge-mcp-tools
summary:
  total: 5
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 5
layers:
  logic:
    task_count: 3
    estimated_files: 3
  testing:
    task_count: 2
    estimated_files: 2
history:
  - timestamp: 2026-02-05T16:03:56.108149+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 5 implementation tasks for change `impl-merge-mcp-tools`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 3 | 🔲 Pending |
| Testing Layer | 2 | 🔲 Pending |

## 2. Logic Layer

### Task 2.0: Update mcp/tools/mod.rs

```yaml
id: 2.0
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/mod.rs
spec_ref: impl-change-tool:R3, merge-change-tool:R3
```

Register both tools in mod.rs:
- Add module declarations: `mod impl_change;` and `mod merge_change;`
- Add tool definitions to `all_tools_vec()`
- Add match arms to `call_tool()` for both tools

### Task 2.1: Create impl_change.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: crates/cclab-genesis/src/mcp/tools/impl_change.rs
spec_ref: impl-change-tool:*
```

Implement genesis_impl_change MCP Workflow Tool covering:
- R1: ImplAction Enum
- R2: State Analysis
- R3: Tool Definition
- R4: Security Validation
- R5: Response Format
- R6: Agent Config Integration

### Task 2.2: Create merge_change.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: crates/cclab-genesis/src/mcp/tools/merge_change.rs
spec_ref: merge-change-tool:*
depends_on: [2.1]
```

Implement genesis_merge_change MCP Workflow Tool covering:
- R1: MergeAction Enum
- R2: State Analysis
- R3: Tool Definition
- R4: Security Validation
- R5: Response Format
- R6: Auto Archive
- R7: Agent Config Integration

## 4. Testing Layer

### Task 4.1: Add tests for genesis_impl_change MCP Workflow Tool

```yaml
id: 4.1
action: CREATE
status: pending
file: crates/cclab-genesis/src/mcp/tools/impl_change.rs (inline tests)
spec_ref: impl-change-tool:*
depends_on: [2.1]
```

Add unit tests in impl_change.rs module covering all acceptance scenarios

### Task 4.2: Add tests for genesis_merge_change MCP Workflow Tool

```yaml
id: 4.2
action: CREATE
status: pending
file: crates/cclab-genesis/src/mcp/tools/merge_change.rs (inline tests)
spec_ref: merge-change-tool:*
depends_on: [2.2]
```

Add unit tests in merge_change.rs module covering all acceptance scenarios

</tasks>
