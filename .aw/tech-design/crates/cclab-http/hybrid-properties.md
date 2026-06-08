---
id: hybrid-properties
type: spec
title: "Hybrid Properties"
version: 1
spec_type: algorithm
created_at: 2026-01-28T08:03:14.139755+00:00
updated_at: 2026-01-28T08:03:14.139755+00:00
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
    - type: flowchart
      title: "Hybrid Property Resolution Flow"
history:
  - timestamp: 2026-01-28T08:03:14.139755+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Hybrid Properties

## Overview

This specification defines the implementation of Hybrid Properties in QueryBuilder. Hybrid Properties allow users to define virtual columns that are backed by SQL expressions, enabling complex computed values to be treated as regular attributes during selection and filtering.

## Requirements

### R1 - Property Registration

```yaml
id: R1
priority: medium
status: draft
```

Allow registering SQL expressions as virtual columns (Hybrid Properties) for a table.

### R2 - Select Expansion

```yaml
id: R2
priority: medium
status: draft
```

Automatically expand Hybrid Properties into their underlying SQL expressions during SELECT query building.

### R3 - Clause Integration

```yaml
id: R3
priority: medium
status: draft
```

Support using Hybrid Properties in WHERE, ORDER BY, and HAVING clauses.

### R4 - Aliasing Support

```yaml
id: R4
priority: medium
status: draft
```

Ensure proper aliasing of expanded expressions to match the property name.

## Acceptance Criteria

### Scenario: Selection Scenario

- **GIVEN** A Hybrid Property 'full_name' defined as 'first_name || \" \" || last_name'
- **WHEN** QueryBuilder.select(['full_name']) is called
- **THEN** The generated SQL SELECT clause includes 'first_name || \" \" || last_name AS full_name'.

### Scenario: Filtering Scenario

- **GIVEN** A Hybrid Property 'is_active' defined as 'status = \"active\"'
- **WHEN** QueryBuilder.where_clause('is_active', Eq, True) is called
- **THEN** The generated SQL WHERE clause includes '(status = \"active\") = TRUE'.

## Diagrams

### Hybrid Property Resolution Flow

```mermaid
flowchart LR
    QueryBuilder[QueryBuilder]
    HybridRegistry[Hybrid Property Registry]
    SQLExpression[SQL Expression (e.g., first_name || ' ' || last_name)]
    QueryBuilder_select[select(['full_name'])]
    QueryBuilder_select_output[SELECT first_name || ' ' || last_name AS full_name]
    QueryBuilder --> HybridRegistry
    HybridRegistry --> SQLExpression
    QueryBuilder_select --> HybridRegistry
    HybridRegistry --> QueryBuilder_select_output
```

</spec>
