---
id: rust-querybuilder-extend
change_id: rust-querybuilder-extend
type: tasks
version: 1
created_at: 2026-01-31T10:20:53.531048+00:00
updated_at: 2026-01-31T10:20:53.531048+00:00
proposal_ref: rust-querybuilder-extend
summary:
  total: 8
  completed: 0
  in_progress: 0
  blocked: 0
  pending: 8
layers:
  logic:
    task_count: 4
    estimated_files: 4
  testing:
    task_count: 4
    estimated_files: 4
history:
  - timestamp: 2026-01-31T10:20:53.531048+00:00
    agent: "task-generator"
    tool: "create_tasks"
    action: "created"
---

<tasks>

# Implementation Tasks

## Overview

This document outlines 8 implementation tasks for change `rust-querybuilder-extend`.

| Layer | Tasks | Status |
|-------|-------|--------|
| Logic Layer | 4 | 🔲 Pending |
| Testing Layer | 4 | 🔲 Pending |

## 2. Logic Layer

### Task 2.1: Create core-implementation-details.rs

```yaml
id: 2.1
action: CREATE
status: pending
file: src/logic/core-implementation-details.rs
spec_ref: core-implementation-details:*
```

Implement Core Engine Implementation Details: Advanced Query Extensions covering:
- R1: Fluent API Refactor
- R2: Aggregate Convenience Methods
- R3: Window Function Convenience Methods

### Task 2.2: Create builder-refinement.rs

```yaml
id: 2.2
action: CREATE
status: pending
file: src/logic/builder-refinement.rs
spec_ref: builder-refinement:*
depends_on: [2.1]
```

Implement Builder Refinement: Fluent API and RETURNING Clause covering:
- R1: Consistent Method Chaining
- R4: Mutation SQL Integration
- R2: RETURNING Clause Support

### Task 2.3: Create query-extensions.rs

```yaml
id: 2.3
action: CREATE
status: pending
file: src/logic/query-extensions.rs
spec_ref: query-extensions:*
depends_on: [2.2]
```

Implement Query Extensions: Aggregates, Grouping, and JSONB covering:
- R1: COUNT(*) Aggregate
- R2: Column Aggregates
- R3: HAVING Clause Helpers

### Task 2.4: Create advanced-queries.rs

```yaml
id: 2.4
action: CREATE
status: pending
file: src/logic/advanced-queries.rs
spec_ref: advanced-queries:*
depends_on: [2.3]
```

Implement Advanced Queries: Window Functions and CTEs covering:
- R1: Rank-based Window Functions
- R2: Value-based Window Functions
- R3: CTE Entry Point (from_cte)

## 4. Testing Layer

### Task 4.1: Add tests for Core Engine Implementation Details: Advanced Query Extensions

```yaml
id: 4.1
action: CREATE
status: pending
file: tests/core-implementation-details_test.rs
spec_ref: core-implementation-details:*
depends_on: [2.1]
```

Create unit tests for Core Engine Implementation Details: Advanced Query Extensions covering all requirements and acceptance scenarios

### Task 4.2: Add tests for Builder Refinement: Fluent API and RETURNING Clause

```yaml
id: 4.2
action: CREATE
status: pending
file: tests/builder-refinement_test.rs
spec_ref: builder-refinement:*
depends_on: [2.2]
```

Create unit tests for Builder Refinement: Fluent API and RETURNING Clause covering all requirements and acceptance scenarios

### Task 4.3: Add tests for Query Extensions: Aggregates, Grouping, and JSONB

```yaml
id: 4.3
action: CREATE
status: pending
file: tests/query-extensions_test.rs
spec_ref: query-extensions:*
depends_on: [2.3]
```

Create unit tests for Query Extensions: Aggregates, Grouping, and JSONB covering all requirements and acceptance scenarios

### Task 4.4: Add tests for Advanced Queries: Window Functions and CTEs

```yaml
id: 4.4
action: CREATE
status: pending
file: tests/advanced-queries_test.rs
spec_ref: advanced-queries:*
depends_on: [2.4]
```

Create unit tests for Advanced Queries: Window Functions and CTEs covering all requirements and acceptance scenarios

</tasks>
