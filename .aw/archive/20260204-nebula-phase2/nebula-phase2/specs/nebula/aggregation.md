---
id: aggregation
type: spec
title: "Rust Aggregation Pipeline Helper"
version: 1
spec_type: algorithm
spec_group: nebula
created_at: 2026-02-04T06:52:00.628027+00:00
updated_at: 2026-02-04T06:52:00.628027+00:00
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
      title: "Aggregation Helper Flow"
history:
  - timestamp: 2026-02-04T06:52:00.628027+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-04T06:52:05.219292+00:00
    agent: "codex:deep"
    tool: "revise_spec"
    action: "revised"
  - timestamp: 2026-02-04T06:52:39.083416+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"---

<spec>

# Rust Aggregation Pipeline Helper

## Overview

Move single-value aggregation pipeline construction and execution (avg/sum/min/max/count) into Rust, exposing a thin PyO3 wrapper so Python only passes parameters and receives a scalar result.

## Requirements

### R1 - Rust Pipeline Builder

```yaml
id: R1
priority: high
status: draft
```

Build MongoDB aggregation pipelines in Rust for avg/sum/min/max/count using an optional match stage and a group stage producing a single scalar output. Count must be implemented as a document count after match and must not depend on a field.

### R2 - Scalar Result Mapping

```yaml
id: R2
priority: high
status: draft
```

Execute the pipeline in Rust and return a single scalar result to Python (None when no documents match). Numeric types must map to Python int/float consistently with the existing Python behavior.

### R3 - Thin PyO3 Wrapper

```yaml
id: R3
priority: high
status: draft
```

Expose a PyO3 binding so Python `_engine` and `QueryBuilder` pass only parameters; Python must not construct aggregation pipelines for these operations.

### R4 - Error Propagation

```yaml
id: R4
priority: medium
status: draft
```

Aggregation failures (invalid field, type mismatch, execution error) must surface as Python exceptions with the underlying Rust error context preserved.

### R5 - Regression Coverage

```yaml
id: R5
priority: medium
status: draft
```

Tests must cover all five aggregation functions, no-match (None) behavior, numeric type mapping, and non-numeric field handling on the Rust path.

## Acceptance Criteria

### Scenario: Average Aggregation Returns Float

- **GIVEN** a collection with numeric `score` values
- **WHEN** Python calls the aggregation helper with op `avg`, field `score`, and no match filter
- **THEN** the helper returns the numeric average as a Python float

### Scenario: Count Aggregation With Filter

- **GIVEN** a collection with mixed documents and a match filter that selects some of them
- **WHEN** Python calls the aggregation helper with op `count` and the filter
- **THEN** the helper returns the number of matching documents as a Python int

### Scenario: Numeric Type Mapping Preserved

- **GIVEN** a collection where sum/min/max produce integer and floating results depending on stored types
- **WHEN** Python calls the aggregation helper with op `sum`, `min`, or `max`
- **THEN** the helper maps integer results to Python int and floating results to Python float, matching existing behavior

### Scenario: No Match Returns None

- **GIVEN** a match filter that selects zero documents
- **WHEN** any aggregation operation is executed
- **THEN** the helper returns `None` to Python

### Scenario: Non-Numeric Field Error

- **GIVEN** a field containing non-numeric values
- **WHEN** an aggregation operation that requires numeric input is executed
- **THEN** the helper raises a Python exception with the Rust error context

### Scenario: Thin Wrapper Enforcement

- **GIVEN** the Python `QueryBuilder` aggregation call
- **WHEN** the operation is invoked
- **THEN** Python passes parameters to the PyO3 binding without constructing a MongoDB pipeline

## Diagrams

### Aggregation Helper Flow

```mermaid
flowchart LR
    start[Start]
    validate[Validate Inputs\n(op, field, match)]
    build[Build Pipeline\n(match + group)]
    execute[Execute Pipeline]
    parse[Parse Scalar\n(or None)]
    return[Return to Python]
    start --> validate
    validate --> build
    build --> execute
    execute --> parse
    parse --> return
```

</spec>
