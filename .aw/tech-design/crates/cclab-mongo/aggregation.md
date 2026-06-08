---
id: aggregation
type: spec
title: "Nebula Aggregation Migration"
version: 1
spec_type: integration
spec_group: cclab-nebula
merge_strategy: new
created_at: 2026-02-03T09:20:23.553944+00:00
updated_at: 2026-02-03T09:20:23.553944+00:00
requirements:
  total: 2
  ids:
    - R1
    - R2
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: sequence
      title: "Aggregation Execution Flow"
history:
  - timestamp: 2026-02-03T09:20:23.553944+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Nebula Aggregation Migration

## Overview

Ensure Python `AggregationBuilder` delegates execution to the Rust `PyDocument.aggregate` method, bypassing any legacy Python engine logic.

## Requirements

### R1 - Delegate Execution

```yaml
id: R1
priority: medium
status: draft
```

Python `AggregationBuilder.to_list` must call `PyDocument.aggregate` via the `_engine` bridge.

### R2 - Security Validation

```yaml
id: R2
priority: medium
status: draft
```

The Rust implementation must reject dangerous operators (like `$function`, `$accumulator`, `$where`) for security.

## Acceptance Criteria

### Scenario: Valid Aggregation

- **GIVEN** A valid aggregation pipeline built in Python
- **WHEN** User calls `User.aggregate(...).to_list()`
- **THEN** The pipeline is executed in Rust and results returned.

### Scenario: Unsafe Aggregation

- **GIVEN** A pipeline containing `$where` operator
- **WHEN** User attempts to execute the pipeline
- **THEN** A `ValueError` is raised by Rust validation.

## Diagrams

### Aggregation Execution Flow

```mermaid
sequenceDiagram
    participant Python as Python Layer
    participant Rust as Rust Layer
    participant MongoDB as MongoDB
    Python->>Python: Build pipeline list (dicts)
    Python->>Rust: Call PyDocument.aggregate(pipeline)
    Rust->>Rust: Validate and Convert pipeline to BSON
    Rust->>MongoDB: Execute aggregation on MongoDB
    MongoDB->>Rust: Return results
    Rust->>Python: Return list of dicts
```

</spec>
