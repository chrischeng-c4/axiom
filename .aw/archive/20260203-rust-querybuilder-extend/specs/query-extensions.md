---
id: query-extensions
type: spec
title: "Query Extensions: Aggregates, Grouping, and JSONB"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:12:22.724405+00:00
updated_at: 2026-01-31T10:12:22.724405+00:00
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
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Aggregate Method Flow"
history:
  - timestamp: 2026-01-31T10:12:22.724405+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Query Extensions: Aggregates, Grouping, and JSONB

## Overview

Extends QueryBuilder in crates/cclab-titan/src/query/select.rs with aggregate functions, grouping, and HAVING clause convenience methods. Matching Python cclab.titan.query functionality.

## Requirements

### R1 - COUNT(*) Aggregate

```yaml
id: R1
priority: high
status: draft
```

Implement count_agg(alias) in select.rs returning COUNT(*).

### R2 - Column Aggregates

```yaml
id: R2
priority: high
status: draft
```

Implement sum, avg, min, max, count_column, and count_distinct in select.rs.

### R3 - HAVING Clause Helpers

```yaml
id: R3
priority: high
status: draft
```

Implement having_sum, having_avg, having_min, having_max, and having_count in select.rs.

### R4 - JSONB Operators

```yaml
id: R4
priority: medium
status: draft
```

Add support for JSONB operators in select.rs: contains (@>), contained_by (<@), has_key (?), has_any_key (?|), and has_all_keys (?&).

## Acceptance Criteria

### Scenario: Sum aggregate with group by and having

- **GIVEN** A new QueryBuilder for 'orders' table
- **WHEN** Calling .sum('amount', Some('total')).group_by(&['user_id']).having_sum('amount', Operator::Gt, ExtractedValue::Int(1000))
- **THEN** Generated SQL should include SUM(amount) AS total, GROUP BY user_id, and HAVING SUM(amount) > 1000

### Scenario: JSONB containment check

- **GIVEN** A new QueryBuilder for 'users' table
- **WHEN** Calling .where_json_contains('metadata', '{"role": "admin"}')
- **THEN** Generated SQL should include WHERE "metadata" @> '{"role": "admin"}'::jsonb

### Scenario: COUNT(DISTINCT) aggregate

- **GIVEN** A new QueryBuilder for 'events' table
- **WHEN** Calling .count_distinct('user_id', Some('unique_users'))
- **THEN** Generated SQL should include COUNT(DISTINCT "user_id") AS unique_users

## Diagrams

### Aggregate Method Flow

```mermaid
flowchart TB
    start(Call sum(col, alias))
    validate_col{Validate Column Identifier} 
    validate_alias{Validate Alias (if present)} 
    add_agg[Add to aggregates vector]
    end(Return Result<Self>)
    start --> validate_col
    validate_col -->|Valid Column| validate_alias
    validate_alias -->|Valid Alias| add_agg
    add_agg --> end
```

<semantic-data>

```json
{
  "edges": [],
  "metadata": null,
  "nodes": [
    {
      "id": "start",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "validate_col",
      "semantic": {
        "error": {
          "code": 400,
          "message": "Invalid column identifier"
        },
        "type": "validation"
      }
    },
    {
      "id": "validate_alias",
      "semantic": {
        "error": {
          "code": 400,
          "message": "Invalid alias identifier"
        },
        "type": "validation"
      }
    },
    {
      "id": "add_agg",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "end",
      "semantic": {
        "type": "return"
      }
    }
  ]
}
```

</semantic-data>

</spec>
