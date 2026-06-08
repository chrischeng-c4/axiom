---
id: genesis-viewer
change_id: genesis-viewer
type: tasks
version: 1
created_at: 2026-01-24T14:58:13.128968+00:00
updated_at: 2026-01-24T14:58:13.128968+00:00
proposal_ref: genesis-viewer
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
    task_count: 2
    estimated_files: 0
  integration:
    task_count: 3
    estimated_files: 1
  testing:
    task_count: 1
    estimated_files: 1
history:
  - timestamp: 2026-01-24T14:58:13.128968+00:00
    agent: "mcp"
    tool: "create_tasks"
    action: "created"
  - timestamp: 2026-01-24T15:01:50.286622+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_tasks"
    action: "created"
    duration_secs: 134.60
  - timestamp: 2026-01-24T15:02:24.838258+00:00
    agent: "gpt-5.2-codex"
    tool: "review_tasks"
    action: "reviewed"
    duration_secs: 34.55---

<tasks>

# Implementation Tasks

## Overview

This document outlines 7 implementation tasks for change `genesis-viewer`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 1 | 🔲 Pending |
| Logic Layer | 2 | 🔲 Pending |
| Integration Layer | 3 | 🔲 Pending |
| Testing Layer | 1 | 🔲 Pending |

## 1. Data Layer

### Task 1.1: Refactor ViewerManager for project-level scanning

```yaml
id: 1.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/ui/viewer/manager.rs
spec_ref: plan-viewer:R1
```

Extend the FileInfo structure or add a FileNode structure to support hierarchical paths. Modify ViewerManager::new_project_manager(project_path) to scan the entire genesis/ directory.

## 2. Logic Layer

### Task 2.1: Add cclab-server routes for project view

```yaml
id: 2.1
action: MODIFY
status: pending
file: crates/cclab-server/src/http_server.rs
spec_ref: plan-viewer:R1
depends_on: [1.1]
```

Add the /{project}/genesis static HTML route and the corresponding API routes (e.g., /api/:project/genesis/tree) in crates/cclab-server/src/http_server.rs.

### Task 2.2: Enhance Markdown rendering engine for LaTeX

```yaml
id: 2.2
action: MODIFY
status: pending
file: crates/cclab-genesis/src/ui/viewer/render.rs
spec_ref: plan-viewer:R4
depends_on: [1.1]
```

Add pre-processing logic to render_markdown_to_html to wrap LaTeX syntax (e.g., $...$) in span tags with a math class for frontend KaTeX processing.

## 3. Integration Layer

### Task 3.1: Upgrade frontend interface (Tree View + Enhancements)

```yaml
id: 3.1
action: MODIFY
status: pending
file: crates/cclab-genesis/src/ui/viewer/assets/app.js
spec_ref: plan-viewer:R2,R3,R4,R5
depends_on: [2.1]
```

Refactor initFileNav to support recursive tree rendering. Add KaTeX initialization scripts. Implement table sorting logic on header click. Add hover preview logic.

### Task 3.2: Register new Skill for project viewing

```yaml
id: 3.2
action: CREATE
status: pending
file: crates/cclab-genesis/templates/mainthread/skills/genesis-view-project/SKILL.md
spec_ref: plan-viewer:R6
```

Create the genesis-view-project skill directory and SKILL.md.

### Task 3.3: Register command in CLAUDE.md

```yaml
id: 3.3
action: MODIFY
status: pending
file: crates/cclab-genesis/templates/mainthread/CLAUDE.md
spec_ref: plan-viewer:R6
depends_on: [3.2]
```

Register the new skill command and description in crates/cclab-genesis/templates/mainthread/CLAUDE.md.

## 4. Testing Layer

### Task 4.1: Functional verification tests

```yaml
id: 4.1
action: CREATE
status: pending
file: crates/cclab-genesis/tests/viewer_expansion_test.rs
spec_ref: plan-viewer:acceptance-criteria
depends_on: [3.1]
```

Write test cases to verify correct tree generation and LaTeX syntax marking.

</tasks>
