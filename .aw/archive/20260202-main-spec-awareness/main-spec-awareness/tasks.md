---
id: main-spec-awareness
change_id: main-spec-awareness
type: tasks
version: 1
created_at: 2026-02-02T10:32:03.301514+00:00
updated_at: 2026-02-02T10:32:03.301514+00:00
proposal_ref: main-spec-awareness
summary:
  total: 6
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 6
layers:
  logic:
    task_count: 4
    estimated_files: 4
  testing:
    task_count: 2
    estimated_files: 2
history:
  - timestamp: 2026-02-02T10:32:03.301514+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-02-02T10:32:03.302202+00:00
    agent: "task-generator"
    tool: "generate_tasks"
    action: "created"
    duration_secs: 0.10---

<tasks>

# Implementation Tasks

## Overview

This document outlines 6 implementation tasks for change `main-spec-awareness`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 4 | 🔲 Pending |
| Testing Layer | 2 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create read_main_spec.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/rpc/read_main_spec.rs
spec_ref: main-spec-integration:*
```

Implement Main Spec Integration covering:
- R1: List Main Specs Tool
- R2: Read Main Spec Tool
- R6: Update Planning Prompts

### Task 2.2: Create list_main_specs.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/rpc/list_main_specs.rs
spec_ref: main-spec-integration:*
depends_on: [2.1]
```

Implement Main Spec Integration covering:
- R1: List Main Specs Tool
- R2: Read Main Spec Tool
- R6: Update Planning Prompts

### Task 2.3: Create read_main_spec.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/rpc/read_main_spec.rs
spec_ref: main-spec-integration:*
depends_on: [2.2]
```

Implement Main Spec Integration covering:
- R1: List Main Specs Tool
- R2: Read Main Spec Tool
- R6: Update Planning Prompts

### Task 2.4: Create list_main_specs.rs

```yaml
id: 2.4
action: CREATE
status: pending
file: src/rpc/list_main_specs.rs
spec_ref: main-spec-integration:*
depends_on: [2.3]
```

Implement Main Spec Integration covering:
- R1: List Main Specs Tool
- R2: Read Main Spec Tool
- R6: Update Planning Prompts

## 4. Testing Layer

### Task 4.1: Add tests for Main Spec Integration

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/main-spec-integration_test.rs
spec_ref: main-spec-integration:*
depends_on: [2.3, 2.4]
```

Create unit tests for Main Spec Integration covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Main Spec Integration

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/main-spec-integration_test.rs
spec_ref: main-spec-integration:*
depends_on: [2.3, 2.4]
```

Create unit tests for Main Spec Integration covering all requirements and acceptance scenarios

</tasks>
