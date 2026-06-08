---
id: sdd-p2
change_id: sdd-p2
type: tasks
version: 1
created_at: 2026-02-23T16:53:58.935995+00:00
updated_at: 2026-02-23T16:53:58.935995+00:00
proposal_ref: sdd-p2
summary:
  total: 12
  completed: 12
  in_progress: 0
  blocked: 0
  pending: 0
layers:
  logic:
    task_count: 6
    estimated_files: 6
  testing:
    task_count: 6
    estimated_files: 6
history:
  - timestamp: 2026-02-23T16:53:58.935995+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 12 implementation tasks for change `sdd-p2`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 6 | ✅ Complete |
| Testing Layer | 6 | ✅ Complete |

## 2. Logic Layer

### Task 2.1: Create implement-change.rs

```yaml
id: 2.1
action: CREATE
status: completed
file: src/logic/implement-change.rs
spec_ref: implement-change:*
```

Implement Update implement spec — document codegen action and agent/mainthread boundary covering:
- R1: Document implement_task_with_codegen action
- R2: Document agent-calls-advance pattern
- R3: Update executor field documentation

### Task 2.2: Create context-clarifications-create.rs

```yaml
id: 2.2
action: CREATE
status: completed
file: src/logic/context-clarifications-create.rs
spec_ref: context-clarifications-create:*
depends_on: [2.1]
```

Implement Update create-context-clarifications spec to match implementation covering:
- R1: Document issue fetch feature
- R2: Document scope collection
- R3: Document response fields

### Task 2.3: Create change-tasks.rs

```yaml
id: 2.3
action: CREATE
status: completed
file: src/logic/change-tasks.rs
spec_ref: change-tasks:*
depends_on: [2.2]
```

Implement Update tasks phase spec — sdd_write_artifact replaces sdd_generate_tasks covering:
- R1: Document sdd_write_artifact for task generation
- R2: Document generate action semantics

### Task 2.4: Create run-change-skill.rs

```yaml
id: 2.4
action: CREATE
status: completed
file: src/logic/run-change-skill.rs
spec_ref: run-change-skill:*
depends_on: [2.3]
```

Implement Update run-change spec — thresholds, phases, scopes, actions, executor covering:
- R1: Document auto-approve at REVIEWED threshold
- R2: Document actual StatePhase enum
- R3: Document actual scope names

### Task 2.5: Create merge-change.rs

```yaml
id: 2.5
action: CREATE
status: completed
file: src/logic/merge-change.rs
spec_ref: merge-change:*
depends_on: [2.4]
```

Implement Update merge spec — document codebase_paths/knowledge_refs enrichment covering:
- R1: Document codebase_paths enrichment
- R2: Document knowledge_refs enrichment

### Task 2.6: Create init-change.rs

```yaml
id: 2.6
action: CREATE
status: completed
file: src/logic/init-change.rs
spec_ref: init-change:*
depends_on: [2.5]
```

Implement Update init-change spec — git_workflow replaces branch_hint covering:
- R1: Document git_workflow parameter
- R2: Remove branch_hint references

## 4. Testing Layer

### Task 4.1: Add tests for Update implement spec — document codegen action and agent/mainthread boundary

```yaml
id: 4.1
action: CREATE
status: completed
file: tests/implement-change_test.rs
spec_ref: implement-change:*
depends_on: [2.1]
```

Create unit tests for Update implement spec — document codegen action and agent/mainthread boundary covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Update create-context-clarifications spec to match implementation

```yaml
id: 4.2
action: CREATE
status: completed
file: tests/context-clarifications-create_test.rs
spec_ref: context-clarifications-create:*
depends_on: [2.2]
```

Create unit tests for Update create-context-clarifications spec to match implementation covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Update tasks phase spec — sdd_write_artifact replaces sdd_generate_tasks

```yaml
id: 4.3
action: CREATE
status: completed
file: tests/change-tasks_test.rs
spec_ref: change-tasks:*
depends_on: [2.3]
```

Create unit tests for Update tasks phase spec — sdd_write_artifact replaces sdd_generate_tasks covering all requirements and acceptance scenarios

### Task 4.4: Add tests for Update run-change spec — thresholds, phases, scopes, actions, executor

```yaml
id: 4.4
action: CREATE
status: completed
file: tests/run-change-skill_test.rs
spec_ref: run-change-skill:*
depends_on: [2.4]
```

Create unit tests for Update run-change spec — thresholds, phases, scopes, actions, executor covering all requirements and acceptance scenarios

### Task 4.5: Add tests for Update merge spec — document codebase_paths/knowledge_refs enrichment

```yaml
id: 4.5
action: CREATE
status: completed
file: tests/merge-change_test.rs
spec_ref: merge-change:*
depends_on: [2.5]
```

Create unit tests for Update merge spec — document codebase_paths/knowledge_refs enrichment covering all requirements and acceptance scenarios

### Task 4.6: Add tests for Update init-change spec — git_workflow replaces branch_hint

```yaml
id: 4.6
action: CREATE
status: completed
file: tests/init-change_test.rs
spec_ref: init-change:*
depends_on: [2.6]
```

Create unit tests for Update init-change spec — git_workflow replaces branch_hint covering all requirements and acceptance scenarios

</tasks>
