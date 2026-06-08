---
id: toolchain-reorg
change_id: toolchain-reorg
type: tasks
version: 1
created_at: 2026-01-28T08:24:33.001248+00:00
updated_at: 2026-01-28T08:24:33.001248+00:00
proposal_ref: toolchain-reorg
summary:
  total: 4
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 4
layers:
  logic:
    task_count: 2
    estimated_files: 2
  testing:
    task_count: 2
    estimated_files: 2
history:
  - timestamp: 2026-01-28T08:24:33.001248+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-01-28T08:24:33.001706+00:00
    agent: "task-generator"
    tool: "generate_tasks"
    action: "created"
    duration_secs: 0.06---

<tasks>

# Implementation Tasks

## Overview

This document outlines 4 implementation tasks for change `toolchain-reorg`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 2 | 🔲 Pending |
| Testing Layer | 2 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create state-machine-tools.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/state-machine-tools.rs
spec_ref: state-machine-tools:*
```

Implement State Machine MCP Tools covering:
- R1: State Machine Validation Tool
- R2: State Machine Generation Tool
- R3: Support Complex Machines

### Task 2.2: Create prism-mcp-refactor.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/prism-mcp-refactor.rs
spec_ref: prism-mcp-refactor:*
depends_on: [2.1]
```

Implement Prism MCP Refactor and Routing covering:
- R1: Filter Exposed MCP Tools
- R2: Daemon Tool Routing
- R3: Local Tool Routing

## 4. Testing Layer

### Task 4.1: Add tests for State Machine MCP Tools

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/state-machine-tools_test.rs
spec_ref: state-machine-tools:*
depends_on: [2.1]
```

Create unit tests for State Machine MCP Tools covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Prism MCP Refactor and Routing

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/prism-mcp-refactor_test.rs
spec_ref: prism-mcp-refactor:*
depends_on: [2.2]
```

Create unit tests for Prism MCP Refactor and Routing covering all requirements and acceptance scenarios

</tasks>
