---
id: consolidate-read-tools
change_id: consolidate-read-tools
type: tasks
version: 1
created_at: 2026-02-09T07:18:07.173867+00:00
updated_at: 2026-02-09T07:18:07.173867+00:00
proposal_ref: consolidate-read-tools
summary:
  total: 2
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 2
layers:
  integration:
    task_count: 1
    estimated_files: 1
  testing:
    task_count: 1
    estimated_files: 1
history:
  - timestamp: 2026-02-09T07:18:07.173867+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 2 implementation tasks for change `consolidate-read-tools`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Integration Layer | 1 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 3. Integration Layer

### Task 3.1: Create genesis_read_file.rs

```yaml
id: 3.1
action: CREATE
status: pending
file: src/rpc/genesis_read_file.rs
spec_ref: consolidate-read-tools:*
```

Implement Consolidated genesis_read_file MCP Tool covering:
- R1: Scope prefix syntax for file parameter
- R2: Remove 6 standalone tool registrations
- R3: Update tool definition schema

## 4. Testing Layer

### Task 4.1: Add tests for Consolidated genesis_read_file MCP Tool

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/consolidate-read-tools_test.rs
spec_ref: consolidate-read-tools:*
depends_on: [3.1]
```

Create unit tests for Consolidated genesis_read_file MCP Tool covering all requirements and acceptance scenarios

</tasks>
