---
id: aurora-codegen
change_id: aurora-codegen
type: tasks
version: 1
created_at: 2026-02-02T14:58:54.584889+00:00
updated_at: 2026-02-02T14:58:54.584889+00:00
proposal_ref: aurora-codegen
summary:
  total: 18
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 18
layers:
  data:
    task_count: 1
    estimated_files: 1
  logic:
    task_count: 6
    estimated_files: 6
  integration:
    task_count: 2
    estimated_files: 2
  testing:
    task_count: 9
    estimated_files: 9
history:
  - timestamp: 2026-02-02T14:58:54.584889+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 18 implementation tasks for change `aurora-codegen`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Data Layer | 1 | 🔲 Pending |
| Logic Layer | 6 | 🔲 Pending |
| Integration Layer | 2 | 🔲 Pending |
| Testing Layer | 9 | 🔲 Pending |

## 1. Data Layer

### Task 1.1: Create json-schema-core.rs

```yaml
id: 1.1
action: CREATE
status: pending
file: src/models/json-schema-core.rs
spec_ref: json-schema-core:*
depends_on: [2.1]
```

Implement JSON Schema Core Implementation covering:
- R1: Version Support
- R2: Typed Structure
- R3: Serde Integration

## 2. Logic Layer

### Task 2.1: Create aurora-codegen-system.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/aurora-codegen-system.rs
spec_ref: aurora-codegen-system:*
```

Implement Aurora Code Generation System Architecture covering:
- R1: Unified Internal Representation
- R2: Spec Validation
- R3: Template-Based Generation

### Task 2.2: Create task-generator-dedup.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/task-generator-dedup.rs
spec_ref: task-generator-dedup:*
depends_on: [2.1]
```

Implement Task Generator Nested Spec Deduplication covering:
- R1: Spec Path Normalization
- R2: Deduplication Strategy
- R3: Task File Path Uniqueness

### Task 2.3: Create spec-validator.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/logic/spec-validator.rs
spec_ref: spec-validator:*
depends_on: [1.1, 2.2]
```

Implement Spec Completeness Validator covering:
- R1: Type Validation
- R2: Reference Validation
- R3: Completeness Check

### Task 2.4: Create generator-axum.rs

```yaml
id: 2.4
action: CREATE
status: pending
file: src/logic/generator-axum.rs
spec_ref: generator-axum:*
depends_on: [2.3]
```

Implement Axum Code Generator covering:
- R1: Generator Interface
- R2: Context Transformation
- R3: Model Generation

### Task 2.5: Create generator-fastapi.rs

```yaml
id: 2.5
action: CREATE
status: pending
file: src/logic/generator-fastapi.rs
spec_ref: generator-fastapi:*
depends_on: [2.4]
```

Implement FastAPI Generator covering:
- R1: Input Mapping
- R2: Template Rendering
- R3: Project Layout

### Task 2.6: Create template-engine.rs

```yaml
id: 2.6
action: CREATE
status: pending
file: src/logic/template-engine.rs
spec_ref: template-engine:*
depends_on: [1.1, 2.5]
```

Implement Tera Template Engine Integration covering:
- R1: Tera Initialization
- R2: Template Rendering
- R3: String Manipulation Filters

## 3. Integration Layer

### Task 3.1: Create generator-express.rs

```yaml
id: 3.1
action: CREATE
status: pending
file: src/api/generator-express.rs
spec_ref: generator-express:*
```

Implement Express Generator covering:
- R1: Template Set Resolution
- R2: Context Construction
- R3: Deterministic Manifest

### Task 3.2: Create test-generation.rs

```yaml
id: 3.2
action: CREATE
status: pending
file: src/api/test-generation.rs
spec_ref: test-generation:*
depends_on: [2.5, 3.1, 2.4, 3.1]
```

Implement Test Generation Integration with cclab-probe covering:
- R1: Generator Test Artifacts
- R2: Probe Adapter Integration
- R3: Deterministic Outputs

## 4. Testing Layer

### Task 4.1: Add tests for Aurora Code Generation System Architecture

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/aurora-codegen-system_test.rs
spec_ref: aurora-codegen-system:*
depends_on: [2.1]
```

Create unit tests for Aurora Code Generation System Architecture covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Task Generator Nested Spec Deduplication

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/task-generator-dedup_test.rs
spec_ref: task-generator-dedup:*
depends_on: [2.2]
```

Create unit tests for Task Generator Nested Spec Deduplication covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Express Generator

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/generator-express_test.rs
spec_ref: generator-express:*
depends_on: [3.1]
```

Create unit tests for Express Generator covering all requirements and acceptance scenarios

### Task 4.4: Add tests for JSON Schema Core Implementation

```yaml
id: 4.4
action: CREATE
status: pending
file: tests/json-schema-core_test.rs
spec_ref: json-schema-core:*
depends_on: [1.1]
```

Create unit tests for JSON Schema Core Implementation covering all requirements and acceptance scenarios

### Task 4.5: Add tests for Spec Completeness Validator

```yaml
id: 4.5
action: CREATE
status: pending
file: tests/spec-validator_test.rs
spec_ref: spec-validator:*
depends_on: [2.3]
```

Create unit tests for Spec Completeness Validator covering all requirements and acceptance scenarios

### Task 4.6: Add tests for Axum Code Generator

```yaml
id: 4.6
action: CREATE
status: pending
file: tests/generator-axum_test.rs
spec_ref: generator-axum:*
depends_on: [2.4]
```

Create unit tests for Axum Code Generator covering all requirements and acceptance scenarios

### Task 4.7: Add tests for FastAPI Generator

```yaml
id: 4.7
action: CREATE
status: pending
file: tests/generator-fastapi_test.rs
spec_ref: generator-fastapi:*
depends_on: [2.5]
```

Create unit tests for FastAPI Generator covering all requirements and acceptance scenarios

### Task 4.8: Add tests for Tera Template Engine Integration

```yaml
id: 4.8
action: CREATE
status: pending
file: tests/template-engine_test.rs
spec_ref: template-engine:*
depends_on: [2.6]
```

Create unit tests for Tera Template Engine Integration covering all requirements and acceptance scenarios

### Task 4.9: Add tests for Test Generation Integration with cclab-probe

```yaml
id: 4.9
action: CREATE
status: pending
file: tests/test-generation_test.rs
spec_ref: test-generation:*
depends_on: [3.2]
```

Create unit tests for Test Generation Integration with cclab-probe covering all requirements and acceptance scenarios

</tasks>
