---
id: 484
change_id: 484
type: tasks
version: 1
created_at: 2026-02-24T02:59:06.059869+00:00
updated_at: 2026-02-24T02:59:06.059869+00:00
proposal_ref: 484
summary:
  total: 6
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 6
layers:
  logic:
    task_count: 3
    estimated_files: 3
  testing:
    task_count: 3
    estimated_files: 3
history:
  - timestamp: 2026-02-24T02:59:06.059869+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 6 implementation tasks for change `484`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 3 | 🔲 Pending |
| Testing Layer | 3 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create dynamic-tool-schema.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/dynamic-tool-schema.rs
spec_ref: dynamic-tool-schema:*
```

Implement Dynamic tools/list Schema Based on Session covering:
- R1: Dynamic required field removal
- R2: No-op for unbound sessions
- R3: Tool call injection

### Task 2.2: Create init-mcp-json.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/init-mcp-json.rs
spec_ref: init-mcp-json:*
depends_on: [2.1]
```

Implement Generate .mcp.json with Project Header in cclab init covering:
- R1: Generate .mcp.json with header
- R2: Add .mcp.json to .gitignore
- R3: Track .mcp.json.example

### Task 2.3: Create mcp-session-binding.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/logic/mcp-session-binding.rs
spec_ref: mcp-session-binding:*
depends_on: [2.2]
```

Implement MCP Session-based Project Binding covering:
- R1: Read X-Cclab-Project header on initialize
- R2: Session-bound project_path resolution
- R3: Backwards compatibility

## 4. Testing Layer

### Task 4.1: Add tests for Dynamic tools/list Schema Based on Session

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/dynamic-tool-schema_test.rs
spec_ref: dynamic-tool-schema:*
depends_on: [2.1]
```

Create unit tests for Dynamic tools/list Schema Based on Session covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Generate .mcp.json with Project Header in cclab init

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/init-mcp-json_test.rs
spec_ref: init-mcp-json:*
depends_on: [2.2]
```

Create unit tests for Generate .mcp.json with Project Header in cclab init covering all requirements and acceptance scenarios

### Task 4.3: Add tests for MCP Session-based Project Binding

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/mcp-session-binding_test.rs
spec_ref: mcp-session-binding:*
depends_on: [2.3]
```

Create unit tests for MCP Session-based Project Binding covering all requirements and acceptance scenarios

</tasks>
