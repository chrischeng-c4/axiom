---
id: genesis-372
change_id: genesis-372
type: tasks
version: 1
created_at: 2026-02-14T17:27:25.756965+00:00
updated_at: 2026-02-14T17:27:25.756965+00:00
proposal_ref: genesis-372
summary:
  total: 10
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 10
layers:
  data:
    task_count: 1
    estimated_files: 1
  logic:
    task_count: 2
    estimated_files: 2
  integration:
    task_count: 2
    estimated_files: 2
  testing:
    task_count: 5
    estimated_files: 5
history:
  - timestamp: 2026-02-14T17:27:25.756965+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 10 implementation tasks for change `genesis-372`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 1 | 🔲 Pending |
| Logic Layer | 2 | 🔲 Pending |
| Integration Layer | 2 | 🔲 Pending |
| Testing Layer | 5 | 🔲 Pending |

## 1. Data Layer

### Task 1.1: Create spec-ir-yaml-schema.rs

```yaml
id: 1.1
action: CREATE
status: pending
file: src/models/spec-ir-yaml-schema.rs
spec_ref: spec-ir-yaml-schema:*
```

Implement SpecIR YAML Manifest Schema covering:
- R1: Standard Envelope
- R2: Kind Registry
- R3: Strict Serialization

## 2. Logic Layer

### Task 2.1: Create codegen-orchestration.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/workflows/codegen-orchestration.rs
spec_ref: genesis-codegen-orchestration:*
```

Implement Genesis Codegen Orchestration covering:
- R1: YAML Detection
- R2: Prism Invocation
- R3: Fallback Logic

### Task 2.2: Create migration-architecture.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/migration-architecture.rs
spec_ref: migration-architecture:*
depends_on: [2.1]
```

Implement Migration Architecture & Compatibility Matrix covering:
- R1: Legacy Path Detection
- R2: YAML Path Enforcement
- R3: Deprecation Warnings

## 3. Integration Layer

### Task 3.1: Create genesis-spec-generation.rs

```yaml
id: 3.1
action: CREATE
status: pending
file: src/api/genesis-spec-generation.rs
spec_ref: genesis-spec-generation:*
```

Implement Genesis Spec Generation Logic covering:
- R1: YAML Generation
- R2: File Naming
- R3: Content Mapping

### Task 3.2: Create prism-yaml-codegen.rs

```yaml
id: 3.2
action: CREATE
status: pending
file: src/api/prism-yaml-codegen.rs
spec_ref: prism-yaml-codegen:*
depends_on: [3.1]
```

Implement Prism YAML-Based Code Generation covering:
- R1: YAML Reader
- R2: Generic Generator Input
- R3: Generator Dispatch

## 4. Testing Layer

### Task 4.1: Add tests for SpecIR YAML Manifest Schema

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/spec-ir-yaml-schema_test.rs
spec_ref: spec-ir-yaml-schema:*
depends_on: [1.1]
```

Create unit tests for SpecIR YAML Manifest Schema covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Genesis Spec Generation Logic

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/genesis-spec-generation_test.rs
spec_ref: genesis-spec-generation:*
depends_on: [3.1]
```

Create unit tests for Genesis Spec Generation Logic covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Prism YAML-Based Code Generation

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/prism-yaml-codegen_test.rs
spec_ref: prism-yaml-codegen:*
depends_on: [3.2]
```

Create unit tests for Prism YAML-Based Code Generation covering all requirements and acceptance scenarios

### Task 4.4: Add tests for Genesis Codegen Orchestration

```yaml
id: 4.4
action: CREATE
status: pending
file: tests/genesis-codegen-orchestration_test.rs
spec_ref: genesis-codegen-orchestration:*
depends_on: [2.1]
```

Create unit tests for Genesis Codegen Orchestration covering all requirements and acceptance scenarios

### Task 4.5: Add tests for Migration Architecture & Compatibility Matrix

```yaml
id: 4.5
action: CREATE
status: pending
file: tests/migration-architecture_test.rs
spec_ref: migration-architecture:*
depends_on: [2.2]
```

Create unit tests for Migration Architecture & Compatibility Matrix covering all requirements and acceptance scenarios

</tasks>
