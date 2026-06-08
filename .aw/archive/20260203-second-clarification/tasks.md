---
id: second-clarification
change_id: second-clarification
type: tasks
version: 1
created_at: 2026-01-27T03:40:32.495563+00:00
updated_at: 2026-01-27T03:40:32.495563+00:00
proposal_ref: second-clarification
summary:
  total: 7
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 7
layers:
  data:
    task_count: 1
    estimated_files: 0
  logic:
    task_count: 1
    estimated_files: 0
  integration:
    task_count: 4
    estimated_files: 0
  testing:
    task_count: 1
    estimated_files: 1
history:
  - timestamp: 2026-01-27T03:40:32.495563+00:00
    agent: "mcp"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-01-27T03:44:47.302271+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_tasks"
    action: "created"
    duration_secs: 305.35
  - timestamp: 2026-01-27T03:45:27.728252+00:00
    agent: "gpt-5.2-codex"
    tool: "review_tasks"
    action: "reviewed"
    duration_secs: 40.42---

<tasks>

# Implementation Tasks

## Overview

This document outlines 7 implementation tasks for change `second-clarification`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 1 | 🔲 Pending |
| Logic Layer | 1 | 🔲 Pending |
| Integration Layer | 4 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 1. Data Layer

### Task 1.1: Add NeedsSecondClarification state phase

```yaml
id: 1.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/models/frontmatter.rs
spec_ref: second-clarification-mechanism:R1
```

Add NeedsSecondClarification variant to StatePhase enum and update Serialize/Deserialize implementations.

## 2. Logic Layer

### Task 2.1: Implement append_clarifications logic

```yaml
id: 2.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/services/clarifications_service.rs
spec_ref: second-clarification-mechanism:R3
depends_on: [1.1]
```

Implement append_clarifications function that appends Q&A to clarifications.md with a phase header.

## 3. Integration Layer

### Task 3.1: Update exploration tool for conditional transition

```yaml
id: 3.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/exploration.rs
spec_ref: second-clarification-mechanism:R2
depends_on: [1.1]
```

Update execute function to set phase to needs_second_clarification when needs_clarification is true.

### Task 3.2: Add append_clarifications MCP tool

```yaml
id: 3.2
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/clarifications.rs
spec_ref: second-clarification-mechanism:R3
depends_on: [2.1]
```

Add genesis_append_clarifications tool definition and execution logic.

### Task 3.3: Register append_clarifications tool

```yaml
id: 3.3
action: MODIFY
status: pending
file: crates/cclab-genesis/src/mcp/tools/mod.rs
spec_ref: second-clarification-mechanism:R3
depends_on: [3.2]
```

Register the new genesis_append_clarifications tool in the MCP registry.

### Task 3.4: Update CLI status display

```yaml
id: 3.4
action: MODIFY
status: pending
file: crates/cclab-genesis/src/cli/status.rs
spec_ref: second-clarification-mechanism:R5
depends_on: [1.1]
```

Update status command to show icon and color for NeedsSecondClarification phase.

## 4. Testing Layer

### Task 4.1: Test second clarification mechanism

```yaml
id: 4.1
action: CREATE
status: pending
file: crates/cclab-genesis/tests/test_second_clarification.rs
spec_ref: second-clarification-mechanism:acceptance-criteria
depends_on: [3.1, 3.2, 3.3, 3.4]
```

Add unit tests for append_clarifications and the exploration transition.

</tasks>
