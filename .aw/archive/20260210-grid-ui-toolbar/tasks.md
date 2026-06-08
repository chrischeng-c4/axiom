---
id: grid-ui-toolbar
change_id: grid-ui-toolbar
type: tasks
version: 1
created_at: 2026-02-10T02:53:49.671411+00:00
updated_at: 2026-02-10T02:53:49.671411+00:00
proposal_ref: grid-ui-toolbar
summary:
  total: 8
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 8
layers:
  logic:
    task_count: 3
    estimated_files: 3
  integration:
    task_count: 1
    estimated_files: 1
  testing:
    task_count: 4
    estimated_files: 4
history:
  - timestamp: 2026-02-10T02:53:49.671411+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 8 implementation tasks for change `grid-ui-toolbar`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 3 | 🔲 Pending |
| Integration Layer | 1 | 🔲 Pending |
| Testing Layer | 4 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create formula-bar-redesign.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/formula-bar-redesign.rs
spec_ref: formula-bar-redesign:*
```

Implement Formula Bar Redesign covering:
- R1: Cell address display
- R3: Formula input field
- R4: Selection change sync

### Task 2.2: Create menu-bar-dropdowns.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/menu-bar-dropdowns.rs
spec_ref: menu-bar-dropdowns:*
depends_on: [2.1]
```

Implement Menu Bar with Dropdown Menus covering:
- R1: Menu bar with 6 menus
- R2: File menu items
- R3: Edit menu items

### Task 2.3: Create header-gridlines.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/logic/header-gridlines.rs
spec_ref: header-gridlines:*
depends_on: [2.2]
```

Implement Header Gridline Separators covering:
- R1: Column header vertical separators
- R2: Row header horizontal separators
- R3: Scroll-aware rendering

## 3. Integration Layer

### Task 3.1: Create toolbar-formatting.rs

```yaml
id: 3.1
action: CREATE
status: pending
file: src/api/toolbar-formatting.rs
spec_ref: toolbar-formatting:*
depends_on: [2.3]
```

Implement Formatting Toolbar covering:
- R1: Toolbar layout with button groups
- R2: Formatting buttons wire to WASM
- R3: Selection state sync

## 4. Testing Layer

### Task 4.1: Add tests for Formula Bar Redesign

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/formula-bar-redesign_test.rs
spec_ref: formula-bar-redesign:*
depends_on: [2.1]
```

Create unit tests for Formula Bar Redesign covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Menu Bar with Dropdown Menus

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/menu-bar-dropdowns_test.rs
spec_ref: menu-bar-dropdowns:*
depends_on: [2.2]
```

Create unit tests for Menu Bar with Dropdown Menus covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Header Gridline Separators

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/header-gridlines_test.rs
spec_ref: header-gridlines:*
depends_on: [2.3]
```

Create unit tests for Header Gridline Separators covering all requirements and acceptance scenarios

### Task 4.4: Add tests for Formatting Toolbar

```yaml
id: 4.4
action: CREATE
status: pending
file: tests/toolbar-formatting_test.rs
spec_ref: toolbar-formatting:*
depends_on: [3.1]
```

Create unit tests for Formatting Toolbar covering all requirements and acceptance scenarios

</tasks>
