---
id: advanced-queries
type: spec
title: "Advanced Queries: Window Functions and CTEs"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:12:30.507904+00:00
updated_at: 2026-01-31T10:12:30.507904+00:00
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
      title: "CTE Definition Flow"
history:
  - timestamp: 2026-01-31T10:12:30.507904+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Advanced Queries: Window Functions and CTEs

## Overview

Implements advanced query features in crates/cclab-titan/src/query/select.rs including Window Functions and Common Table Expressions (CTEs). Matching the rich API of the Python implementation.

## Requirements

### R1 - Rank-based Window Functions

```yaml
id: R1
priority: high
status: draft
```

Implement row_number, rank, dense_rank window function convenience methods in select.rs.

### R2 - Value-based Window Functions

```yaml
id: R2
priority: high
status: draft
```

Implement window_sum, window_avg, lag, and lead window function convenience methods in select.rs.

### R3 - CTE Entry Point (from_cte)

```yaml
id: R3
priority: medium
status: draft
```

Add a static from_cte(name, query) method to QueryBuilder in builder.rs or select.rs.

### R4 - CTE Integration Logic

```yaml
id: R4
priority: medium
status: draft
```

Ensure with_cte in select.rs correctly integrates with the WITH clause in build_select.

## Acceptance Criteria

### Scenario: Row number window function

- **GIVEN** A new QueryBuilder for 'orders' table
- **WHEN** Calling .row_number('rank', WindowSpec::new().partition_by(&['user_id']).order_by('amount', OrderDirection::Desc))
- **THEN** Generated SQL should include ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY amount DESC) AS rank

### Scenario: Querying from a CTE using from_cte

- **GIVEN** An existing high_value_orders CTE query
- **WHEN** Calling QueryBuilder::from_cte('high_value', high_value_query)
- **THEN** Generated SQL should include WITH high_value AS (...) SELECT * FROM high_value

### Scenario: Lag window function to find previous price

- **GIVEN** A new QueryBuilder for 'stock_prices' table
- **WHEN** Calling .lag('price', 1, Some(ExtractedValue::Int(0)), WindowSpec::new().order_by('timestamp', OrderDirection::Asc), 'prev_price')
- **THEN** Generated SQL should include LAG("price", 1, 0) OVER (ORDER BY timestamp ASC) AS prev_price

## Diagrams

### CTE Definition Flow

```mermaid
flowchart TB
    start(Call with_cte(name, subquery))
    validate_name{Validate CTE name identifier} 
    build_sq_sql[Build subquery SQL and params]
    add_cte_def[Add to ctes vector]
    end(Return Result<Self>)
    start --> validate_name
    validate_name -->|Valid Name| build_sq_sql
    build_sq_sql --> add_cte_def
    add_cte_def --> end
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
      "id": "validate_name",
      "semantic": {
        "error": {
          "code": 400,
          "message": "Invalid CTE name"
        },
        "type": "validation"
      }
    },
    {
      "id": "build_sq_sql",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "add_cte_def",
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
