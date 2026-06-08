---
id: prompt-registry
change_id: prompt-registry
type: tasks
version: 1
created_at: 2026-02-08T15:46:05.426586+00:00
updated_at: 2026-02-08T15:46:05.426586+00:00
proposal_ref: prompt-registry
summary:
  total: 2
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 2
layers:
  logic:
    task_count: 1
    estimated_files: 1
  testing:
    task_count: 1
    estimated_files: 1
history:
  - timestamp: 2026-02-08T15:46:05.426586+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 2 implementation tasks for change `prompt-registry`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 1 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create prompt-registry.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/prompt-registry.rs
spec_ref: prompt-registry:*
```

Implement Prompt Registry - Inline Agent Prompts for run_change covering:
- R1: Restructure run_change into folder module
- R2: Populate agent_prompt for all agent-delegated actions
- R3: Delete src/prompts/ directory

## 4. Testing Layer

### Task 4.1: Add tests for Prompt Registry - Inline Agent Prompts for run_change

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/prompt-registry_test.rs
spec_ref: prompt-registry:*
depends_on: [2.1]
```

Create unit tests for Prompt Registry - Inline Agent Prompts for run_change covering all requirements and acceptance scenarios

</tasks>
