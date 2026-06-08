---
id: 196
change_id: 196
type: tasks
version: 1
created_at: 2026-02-12T08:04:50.333929+00:00
updated_at: 2026-02-12T08:04:50.333929+00:00
proposal_ref: 196
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
  - timestamp: 2026-02-12T08:04:50.333929+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 2 implementation tasks for change `196`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 1 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create merge-change-bugfixes.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/merge-change-bugfixes.rs
spec_ref: merge-change-bugfixes:*
```

Implement Merge-Change Bug Fixes covering:
- R1: Unify merge verdict routing
- R2: Add YAML frontmatter to REVIEW_MERGE.md
- R3: Document phase producers for merged/merge_approved

## 4. Testing Layer

### Task 4.1: Add tests for Merge-Change Bug Fixes

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/merge-change-bugfixes_test.rs
spec_ref: merge-change-bugfixes:*
depends_on: [2.1]
```

Create unit tests for Merge-Change Bug Fixes covering all requirements and acceptance scenarios

</tasks>
