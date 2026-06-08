---
id: genesis-325-329
change_id: genesis-325-329
type: tasks
version: 1
created_at: 2026-02-14T11:33:29.955948+00:00
updated_at: 2026-02-14T11:33:29.955948+00:00
proposal_ref: genesis-325-329
summary:
  total: 6
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 6
layers:
  logic:
    task_count: 2
    estimated_files: 2
  integration:
    task_count: 1
    estimated_files: 1
  testing:
    task_count: 3
    estimated_files: 3
history:
  - timestamp: 2026-02-14T11:33:29.955948+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 6 implementation tasks for change `genesis-325-329`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 2 | 🔲 Pending |
| Integration Layer | 1 | 🔲 Pending |
| Testing Layer | 3 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create spec-ir-contract.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/spec-ir-contract.rs
spec_ref: spec-ir-contract:*
```

Implement SpecIR Contract Definition covering:
- R1: SpecIR enum type
- R2: SpecIR metadata
- R3: SpecIR construction from Aurora types

### Task 2.2: Create prism-codegen-unification.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/prism-codegen-unification.rs
spec_ref: prism-codegen-unification:*
depends_on: [2.1]
```

Implement Prism Codegen Unification & Generator Migration covering:
- R1: Migrate Aurora generators to Prism
- R2: Unify CodeGenerator trait with SpecIR input
- R3: Framework generator SpecIR adaptation

## 3. Integration Layer

### Task 3.1: Create genesis-implement-integration.rs

```yaml
id: 3.1
action: CREATE
status: pending
file: src/api/genesis-implement-integration.rs
spec_ref: genesis-implement-integration:*
```

Implement Genesis Implement Phase Integration with Prism Codegen covering:
- R1: Detect codegen-eligible tasks
- R2: Structured codegen prompt
- R3: Fallback to manual implementation

## 4. Testing Layer

### Task 4.1: Add tests for SpecIR Contract Definition

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/spec-ir-contract_test.rs
spec_ref: spec-ir-contract:*
depends_on: [2.1]
```

Create unit tests for SpecIR Contract Definition covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Genesis Implement Phase Integration with Prism Codegen

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/genesis-implement-integration_test.rs
spec_ref: genesis-implement-integration:*
depends_on: [3.1]
```

Create unit tests for Genesis Implement Phase Integration with Prism Codegen covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Prism Codegen Unification & Generator Migration

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/prism-codegen-unification_test.rs
spec_ref: prism-codegen-unification:*
depends_on: [2.2]
```

Create unit tests for Prism Codegen Unification & Generator Migration covering all requirements and acceptance scenarios

</tasks>
