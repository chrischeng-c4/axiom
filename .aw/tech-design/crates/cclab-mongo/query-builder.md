---
id: query-builder
type: spec
title: "Nebula Query Builder Migration"
version: 1
spec_type: integration
spec_group: cclab-nebula
merge_strategy: new
created_at: 2026-02-03T09:20:35.789507+00:00
updated_at: 2026-02-03T09:20:35.789507+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: sequence
      title: "Query Builder Delegation Flow"
history:
  - timestamp: 2026-02-03T09:20:35.789507+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Nebula Query Builder Migration

## Overview

Refactor Python `QueryBuilder` to wrap the Rust `PyQueryBuilder`. This moves query construction and execution logic to Rust, while Python handles high-level object hydration.

## Requirements

### R1 - Wrap Rust Builder

```yaml
id: R1
priority: medium
status: draft
```

Python `QueryBuilder` must initialize and maintain a `PyQueryBuilder` instance.

### R2 - Delegate Construction

```yaml
id: R2
priority: medium
status: draft
```

Fluent methods (`filter`, `sort`, `skip`, `limit`, `project`) must delegate to the underlying `PyQueryBuilder`.

### R3 - Delegate Execution

```yaml
id: R3
priority: medium
status: draft
```

Execution methods (`to_list`, `count`, `first`) must call the corresponding `PyQueryBuilder` methods.

### R4 - Inheritance Handling

```yaml
id: R4
priority: medium
status: draft
```

Python side must still handle `_with_children` inheritance logic by adding appropriate filters before passing to Rust.

## Acceptance Criteria

### Scenario: Build and Execute

- **GIVEN** A `QueryBuilder` chain defined in Python
- **WHEN** User executes the query
- **THEN** The Rust builder is constructed incrementally and executed, returning raw dicts which Python hydrates.

### Scenario: Inheritance Query

- **GIVEN** A query on a child class in a polymorphic collection
- **WHEN** User queries the child model
- **THEN** Python adds the `_class_id` filter before delegating to Rust.

## Diagrams

### Query Builder Delegation Flow

```mermaid
sequenceDiagram
    participant User as User Code
    participant Python as Python QueryBuilder
    participant Rust as Rust PyQueryBuilder
    participant MongoDB as MongoDB
    User->>Python: User.find(filter).sort(s).limit(n)
    Python->>Rust: PyQueryBuilder.new().filter().sort().limit()
    User->>Python: .to_list()
    Python->>Rust: PyQueryBuilder.to_list()
    Rust->>MongoDB: Execute Query
    MongoDB->>Rust: Return Documents
    Rust->>Python: Return Python Dicts
    Python->>User: Hydrate Document Objects
```

</spec>
