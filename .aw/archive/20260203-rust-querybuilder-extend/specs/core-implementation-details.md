---
id: core-implementation-details
type: spec
title: "Core Engine Implementation Details: Advanced Query Extensions"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:18:54.421737+00:00
updated_at: 2026-01-31T10:18:54.421737+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Query Building Pipeline"
history:
  - timestamp: 2026-01-31T10:18:54.421737+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Core Engine Implementation Details: Advanced Query Extensions

## Overview

This specification details the implementation of advanced query features and a refined fluent API for the cclab-titan core engine. It focuses on achieving feature parity with the Python implementation while maintaining high performance and type safety in Rust. Key areas include aggregate convenience methods, window functions, Common Table Expressions (CTEs), JSONB operators, and a consistent RETURNING clause support across all mutation operations.

## Requirements

### R1 - Fluent API Refactor

```yaml
id: R1
priority: medium
status: draft
```

Refactor all QueryBuilder methods to consistently use 'mut self -> Result<Self>' or 'mut self -> Self' to enable clean method chaining and improve API ergonomics.

### R2 - Aggregate Convenience Methods

```yaml
id: R2
priority: medium
status: draft
```

Add convenience methods for common aggregate functions (count_agg, sum, avg, min, max, count_column, count_distinct) that simplify aggregate query construction.

### R3 - Window Function Convenience Methods

```yaml
id: R3
priority: medium
status: draft
```

Implement specialized methods for window functions (row_number, rank, dense_rank, lag, lead, etc.) to match the Python API capabilities.

### R4 - CTE Convenience Methods

```yaml
id: R4
priority: medium
status: draft
```

Extend CTE support with from_cte static method for direct entry and improve with_cte for better integration with the main query builder.

### R5 - JSONB Operator Helpers

```yaml
id: R5
priority: medium
status: draft
```

Integrate JSONB comparison operators (contains, contained_by, key_exists, etc.) into the query builder with dedicated helper methods.

### R6 - Unified RETURNING Clause Support

```yaml
id: R6
priority: medium
status: draft
```

Standardize RETURNING clause support across INSERT, UPDATE, UPSERT, and DELETE operations, allowing flexible column selection for mutation results.

## Acceptance Criteria

### Scenario: Complex Fluent Query Chaining

- **GIVEN** A QueryBuilder for the 'orders' table
- **WHEN** Chaining .with_cte('high_value', ...).select(vec!['user_id']).sum('amount', Some('total')).group_by(&['user_id']).having_sum('amount', Operator::Gt, 1000.into()).order_by('total', OrderDirection::Desc)
- **THEN** The resulting SQL should correctly combine all clauses with proper quoting and parameterization.

### Scenario: Update with specific RETURNING columns

- **GIVEN** A QueryBuilder for the 'users' table
- **WHEN** Calling .where_clause('id', Operator::Eq, 1.into()).returning(&['id', 'updated_at']).build_update(&[('status', 'active'.into())])
- **THEN** Generated SQL should be UPDATE "users" SET "status" = $1 WHERE "id" = $2 RETURNING "id", "updated_at"

### Scenario: JSONB containment check helper

- **GIVEN** A QueryBuilder for the 'products' table
- **WHEN** Calling .where_json_contains('attributes', '{"color": "red"}')
- **THEN** Generated SQL should include "attributes" @> '{"color": "red"}'::jsonb

### Scenario: Rank window function helper

- **GIVEN** A QueryBuilder for the 'events' table
- **WHEN** Calling .rank('event_rank', WindowSpec::new().partition_by(&['type']).order_by('timestamp', OrderDirection::Desc))
- **THEN** Generated SQL should include RANK() OVER (PARTITION BY type ORDER BY timestamp DESC) AS event_rank

### Scenario: Querying from CTE using from_cte helper

- **GIVEN** A high-level query builder entry point
- **WHEN** Calling QueryBuilder::from_cte('temp_res', subquery)
- **THEN** Generated SQL should be WITH "temp_res" AS (...) SELECT * FROM "temp_res"

### Scenario: Aggregate convenience methods (count_agg)

- **GIVEN** A QueryBuilder for the 'stats' table
- **WHEN** Calling .count_agg(Some("total_count"))
- **THEN** Generated SQL should include COUNT(*) AS total_count

## Diagrams

### Query Building Pipeline

```mermaid
flowchart TB
    start_build[Start build_select()]
    build_ctes[Build CTEs (WITH clause)]
    build_select_clause[Build SELECT (incl. Aggs & Windows)]
    build_from_clause[Build FROM table]
    build_joins[Build JOIN clauses]
    build_where_clause[Build WHERE conditions]
    build_group_by_having[Build GROUP BY & HAVING]
    build_order_limit_offset[Build ORDER, LIMIT, OFFSET]
    build_set_ops[Build Set Operations (UNION, etc.)]
    end_build[Return (SQL, Params)]
    start_build --> build_ctes
    build_ctes --> build_select_clause
    build_select_clause --> build_from_clause
    build_from_clause --> build_joins
    build_joins --> build_where_clause
    build_where_clause --> build_group_by_having
    build_group_by_having --> build_order_limit_offset
    build_order_limit_offset --> build_set_ops
    build_set_ops --> end_build
```

<semantic-data>

```json
{
  "edges": [],
  "metadata": null,
  "nodes": [
    {
      "id": "start_build",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "build_ctes",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "build_select_clause",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "build_from_clause",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "build_joins",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "build_where_clause",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "build_group_by_having",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "build_order_limit_offset",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "build_set_ops",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "end_build",
      "semantic": {
        "type": "return"
      }
    }
  ]
}
```

</semantic-data>

</spec>
