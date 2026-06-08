---
id: sdd-merge
change_id: sdd-merge
type: tasks
version: 1
created_at: 2026-02-15T03:49:30.503904+00:00
updated_at: 2026-02-15T03:49:30.503904+00:00
proposal_ref: sdd-merge
summary:
  total: 12
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 12
layers:
  logic:
    task_count: 3
    estimated_files: 3
  integration:
    task_count: 3
    estimated_files: 3
  testing:
    task_count: 6
    estimated_files: 6
history:
  - timestamp: 2026-02-15T03:49:30.503904+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 12 implementation tasks for change `sdd-merge`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 3 | 🔲 Pending |
| Integration Layer | 3 | 🔲 Pending |
| Testing Layer | 6 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create crate-unification.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/crate-unification.rs
spec_ref: crate-unification:*
```

Implement Crate Unification and Rename covering:
- R1: Rename Genesis Crate
- R2: Merge Aurora Code
- R3: Remove Aurora Crate

### Task 2.2: Create prompt-template-update.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/prompt-template-update.rs
spec_ref: prompt-template-update:*
depends_on: [2.1]
```

Implement Prompt Template Updates covering:
- R1: Rename in Code
- R2: Update Documentation
- R3: Cleanup Legacy References

### Task 2.3: Create review-verdict-unification.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/logic/review-verdict-unification.rs
spec_ref: review-verdict-unification:*
depends_on: [2.2]
```

Implement Unified Review Verdicts covering:
- R1: Define Verdict Enum
- R2: Update Review Model
- R3: Backward Compatibility

## 3. Integration Layer

### Task 3.1: Create manifest-handling.rs

```yaml
id: 3.1
action: CREATE
status: pending
file: src/api/manifest-handling.rs
spec_ref: manifest-handling:*
```

Implement Manifest Handling in Merge Logic covering:
- R1: Include Spec IR in Archive
- R2: Validate IR Manifests
- R3: Sync IR on Merge

### Task 3.2: Create generator-decoupling.rs

```yaml
id: 3.2
action: CREATE
status: pending
file: src/api/generator-decoupling.rs
spec_ref: generator-decoupling:*
depends_on: [3.1]
```

Implement Generator Decoupling and Legacy Removal covering:
- R1: Direct IR Consumption
- R2: Remove Relay Logic
- R3: IR Validation

### Task 3.3: Create mcp-router-unification.rs

```yaml
id: 3.3
action: CREATE
status: pending
file: src/api/mcp-router-unification.rs
spec_ref: mcp-router-unification:*
depends_on: [3.2]
```

Implement Unified MCP Router and Registry covering:
- R1: Unified Router Implementation
- R2: Add PDG Tools
- R3: Unify Registry Logic

## 4. Testing Layer

### Task 4.1: Add tests for Manifest Handling in Merge Logic

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/manifest-handling_test.rs
spec_ref: manifest-handling:*
depends_on: [3.1]
```

Create unit tests for Manifest Handling in Merge Logic covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Crate Unification and Rename

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/crate-unification_test.rs
spec_ref: crate-unification:*
depends_on: [2.1]
```

Create unit tests for Crate Unification and Rename covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Generator Decoupling and Legacy Removal

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/generator-decoupling_test.rs
spec_ref: generator-decoupling:*
depends_on: [3.2]
```

Create unit tests for Generator Decoupling and Legacy Removal covering all requirements and acceptance scenarios

### Task 4.4: Add tests for Prompt Template Updates

```yaml
id: 4.4
action: CREATE
status: pending
file: tests/prompt-template-update_test.rs
spec_ref: prompt-template-update:*
depends_on: [2.2]
```

Create unit tests for Prompt Template Updates covering all requirements and acceptance scenarios

### Task 4.5: Add tests for Unified MCP Router and Registry

```yaml
id: 4.5
action: CREATE
status: pending
file: tests/mcp-router-unification_test.rs
spec_ref: mcp-router-unification:*
depends_on: [3.3]
```

Create unit tests for Unified MCP Router and Registry covering all requirements and acceptance scenarios

### Task 4.6: Add tests for Unified Review Verdicts

```yaml
id: 4.6
action: CREATE
status: pending
file: tests/review-verdict-unification_test.rs
spec_ref: review-verdict-unification:*
depends_on: [2.3]
```

Create unit tests for Unified Review Verdicts covering all requirements and acceptance scenarios

</tasks>
