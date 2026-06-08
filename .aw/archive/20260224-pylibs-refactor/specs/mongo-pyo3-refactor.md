---
id: mongo-pyo3-refactor
type: spec
title: "Refactor cclab-mongo PyO3 Bindings"
version: 1
spec_type: utility
created_at: 2026-02-24T10:35:24.099590+00:00
updated_at: 2026-02-24T10:35:24.099590+00:00
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
      title: "File Structure Decomposition"
history:
  - timestamp: 2026-02-24T10:35:24.099590+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Refactor cclab-mongo PyO3 Bindings

## Overview

Refactor cclab-mongo PyO3 bindings to improve maintainability by splitting oversized files. document.rs (728 lines) and query.rs (480 lines) will be decomposed into smaller submodules, ensuring each file is well under the 500-line soft limit while preserving the public API and internal logic accessibility.

## Requirements

### R1 - Decompose document.rs

```yaml
id: R1
priority: high
status: draft
```

Split document.rs into document.rs (core struct and basic methods), document_ops.rs (instance CRUD methods), and document_static.rs (static convenience methods).

### R2 - Decompose query.rs

```yaml
id: R2
priority: high
status: draft
```

Split query.rs into query_expr.rs (PyQueryExpr class) and query_builder.rs (PyQueryBuilder class).

### R3 - API Preservation

```yaml
id: R3
priority: high
status: draft
```

Ensure all public API symbols (PyDocument, PyQueryExpr, PyQueryBuilder) and module registration logic remain unchanged for downstream consumers.

### R4 - File Size Constraint

```yaml
id: R4
priority: medium
status: draft
```

All resulting source files in pyo3_bindings must be under 500 lines of code.

### R5 - Internal Visibility and Helper Access

```yaml
id: R5
priority: medium
status: draft
```

Ensure PyDocument fields and internal helpers (config, connection, conversion) are accessible across the new module boundaries using pub(crate) visibility where necessary.

## Acceptance Criteria

### Scenario: Modularization Success

- **GIVEN** Large document.rs (728 lines) and query.rs (480 lines) in pyo3_bindings.
- **WHEN** The code is split into submodules and compiled.
- **THEN** The crate should build successfully, and all resulting files (document.rs, document_ops.rs, document_static.rs, query_expr.rs, query_builder.rs) must be under 500 lines.

### Scenario: API Consistency and Parity

- **GIVEN** Refactored submodules and unchanged registration logic in mod.rs.
- **WHEN** The compiled extension is imported and used in Python.
- **THEN** The Python module 'cclab._nebula' must still export 'Document', 'QueryExpr', and 'QueryBuilder' with all existing methods functional.

### Scenario: Internal State Integrity

- **GIVEN** PyDocument fields marked as pub(crate) to allow cross-module access.
- **WHEN** An instance method in document_ops.rs (like save) is called.
- **THEN** Methods in document_ops.rs and document_static.rs must be able to read and modify PyDocument instance state without restriction.

### Scenario: Regression Testing

- **GIVEN** Existing cclab-mongo integration tests.
- **WHEN** The standard test suite is executed.
- **THEN** All tests must pass, confirming that no behavioral changes were introduced during the structural refactor.

## Diagrams

### File Structure Decomposition

```mermaid
flowchart TB
    mod_rs[mod.rs (Registry)]
    document_rs[document.rs (Core)]
    document_ops_rs[document_ops.rs (CRUD)]
    document_static_rs[document_static.rs (Static)]
    query_expr_rs[query_expr.rs]
    query_builder_rs[query_builder.rs]
    mod_rs -->|exports| document_rs
    mod_rs -->|exports| query_expr_rs
    mod_rs -->|exports| query_builder_rs
    document_rs -->|includes pymethods from| document_ops_rs
    document_rs -->|includes pymethods from| document_static_rs
```

</spec>
