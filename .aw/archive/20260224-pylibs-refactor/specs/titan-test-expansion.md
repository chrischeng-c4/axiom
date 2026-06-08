---
id: titan-test-expansion
type: spec
title: "Expand cclab-titan Integration Tests"
version: 1
spec_type: utility
created_at: 2026-02-24T10:37:20.884830+00:00
updated_at: 2026-02-24T10:37:20.884830+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Titan Test Architecture"
history:
  - timestamp: 2026-02-24T10:37:20.884830+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Expand cclab-titan Integration Tests

## Overview

Implement P0 integration tests for the cclab-pg (Titan) ORM to ensure reliability of core database operations. The expansion covers connection pooling, constraint handling, referential integrity (cascade), and upsert operations, addressing critical testing gaps identified in the current implementation.

## Requirements

### R1 - Connection Pool Tests

```yaml
id: R1
priority: high
status: draft
```

Implement connection pool integration tests in crates/cclab-pg/tests/pool_tests.rs. Verify min/max connections, timeouts, and retry logic with a real/mocked PostgreSQL instance.

### R2 - Constraint Violation Tests

```yaml
id: R2
priority: high
status: draft
```

Implement constraint violation tests in crates/cclab-pg/tests/constraint_tests.rs. Verify Conflict (Unique), ForeignKey, and Validation (Not Null, Check) errors are correctly mapped from SQLSTATE.

### R3 - Cascade Behavior Tests

```yaml
id: R3
priority: medium
status: draft
```

Implement cascade operation tests in crates/cclab-pg/tests/cascade_tests.rs. Verify Cascade, Restrict, SetNull, and SetDefault behaviors on delete/update.

### R4 - Upsert Operation Tests

```yaml
id: R4
priority: high
status: draft
```

Implement upsert integration tests in crates/cclab-pg/tests/upsert_tests.rs. Verify ON CONFLICT behavior for single and bulk operations.

### R5 - Test Integration and CI Parity

```yaml
id: R5
priority: medium
status: draft
```

Ensure all tests are integrated into the crates/cclab-pg/tests directory and compatible with the project CI.

## Acceptance Criteria

### Scenario: Connection Resilience

- **GIVEN** A PostgreSQL instance that is temporarily unavailable.
- **WHEN** Initializing a connection with retry configuration.
- **THEN** The client should successfully connect after the instance becomes available within the retry window.

### Scenario: Constraint Mapping

- **GIVEN** A table with a unique constraint on 'email'.
- **WHEN** Inserting a duplicate email.
- **THEN** The operation must return DataBridgeError::Conflict with a descriptive message.

### Scenario: Cascade Delete

- **GIVEN** Parent and child tables with ON DELETE CASCADE.
- **WHEN** Deleting a parent row.
- **THEN** All associated child rows must be automatically deleted by the database.

### Scenario: Upsert Functional Parity

- **GIVEN** An existing row and a new row with the same unique key.
- **WHEN** Performing an upsert operation.
- **THEN** The existing row should be updated with the new values instead of failing.

## Diagrams

### Titan Test Architecture

```mermaid
flowchart TB
    test_runner[Test Runner (cargo test)]
    pool_tests[pool_tests.rs]
    constraint_tests,label:constraint_tests.rs[constraint_tests.rs]
    cascade_tests[cascade_tests.rs]
    upsert_tests[upsert_tests.rs]
    connection_rs[connection.rs (Pooling)]
    error_rs[error.rs (Constraints)]
    schema_rs[schema.rs (Cascade)]
    query_rs[query.rs (Upsert)]
    test_runner -->|executes| pool_tests
    test_runner -->|executes| constraint_tests
    test_runner -->|executes| cascade_tests
    test_runner -->|executes| upsert_tests
    pool_tests -->|verifies| connection_rs
    constraint_tests -->|verifies| error_rs
    cascade_tests -->|verifies| schema_rs
    upsert_tests -->|verifies| query_rs
```

</spec>
