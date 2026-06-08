---
id: grid-merge-cells
change_id: grid-merge-cells
type: tasks
version: 1
created_at: 2026-02-10T03:44:00.547130+00:00
updated_at: 2026-02-10T03:44:00.547130+00:00
proposal_ref: grid-merge-cells
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
  - timestamp: 2026-02-10T03:44:00.547130+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 6 implementation tasks for change `grid-merge-cells`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 3 | 🔲 Pending |
| Testing Layer | 3 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create merge-row-col-shift.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/merge-row-col-shift.rs
spec_ref: merge-row-col-shift:*
```

Implement Merge Region Adjustment on Row/Column Operations covering:
- R1: Insert rows shifts merge ranges down
- R2: Delete rows adjusts merge ranges
- R3: Insert columns shifts merge ranges right

### Task 2.2: Create merge-ui-controls.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/merge-ui-controls.rs
spec_ref: merge-ui-controls:*
depends_on: [2.1]
```

Implement Merge/Unmerge UI Controls covering:
- R1: Toolbar merge button toggles
- R2: Merge button state syncs on selection change
- R3: Sort protection feedback

### Task 2.3: Create merge-selection-navigation.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/logic/merge-selection-navigation.rs
spec_ref: merge-selection-navigation:*
depends_on: [2.1, 2.2]
```

Implement Merge-Aware Selection and Keyboard Navigation covering:
- R1: Click on slave cell selects merge region
- R2: Arrow key navigation skips slave cells
- R3: Arrow key exits merged region correctly

## 4. Testing Layer

### Task 4.1: Add tests for Merge Region Adjustment on Row/Column Operations

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/merge-row-col-shift_test.rs
spec_ref: merge-row-col-shift:*
depends_on: [2.1]
```

Create unit tests for Merge Region Adjustment on Row/Column Operations covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Merge/Unmerge UI Controls

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/merge-ui-controls_test.rs
spec_ref: merge-ui-controls:*
depends_on: [2.2]
```

Create unit tests for Merge/Unmerge UI Controls covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Merge-Aware Selection and Keyboard Navigation

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/merge-selection-navigation_test.rs
spec_ref: merge-selection-navigation:*
depends_on: [2.3]
```

Create unit tests for Merge-Aware Selection and Keyboard Navigation covering all requirements and acceptance scenarios

</tasks>
