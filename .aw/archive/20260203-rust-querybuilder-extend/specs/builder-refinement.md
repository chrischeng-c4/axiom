---
id: builder-refinement
type: spec
title: "Builder Refinement: Fluent API and RETURNING Clause"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:12:38.192076+00:00
updated_at: 2026-01-31T10:12:38.192076+00:00
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
      title: "Update with RETURNING Flow"
history:
  - timestamp: 2026-01-31T10:12:38.192076+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Builder Refinement: Fluent API and RETURNING Clause

## Overview

Refines the QueryBuilder fluent API in crates/cclab-titan/src/query/builder.rs and adds RETURNING clause support in crates/cclab-titan/src/query/modify.rs.

## Requirements

### R1 - Consistent Method Chaining

```yaml
id: R1
priority: high
status: draft
```

Ensure all QueryBuilder methods consistently use 'mut self -> Self' or 'mut self -> Result<Self>' to enable clean method chaining. Refactor select, where_clause, etc. if needed.

### R2 - RETURNING Clause Support

```yaml
id: R2
priority: medium
status: draft
```

Implement returning(*columns) in crates/cclab-titan/src/query/builder.rs to specify columns to return.

### R3 - RETURNING ALL Support

```yaml
id: R3
priority: medium
status: draft
```

Implement returning_all() in crates/cclab-titan/src/query/builder.rs as a shorthand for RETURNING *.

### R4 - Mutation SQL Integration

```yaml
id: R4
priority: high
status: draft
```

Update SQL generation in modify.rs (build_insert, build_update, build_delete) to include the RETURNING clause if specified.

## Acceptance Criteria

### Scenario: Update with RETURNING clause

- **GIVEN** A new QueryBuilder for 'users' table with a WHERE condition
- **WHEN** Calling .where_clause('id', Operator::Eq, 42).returning(&['id', 'name']).build_update(&[('name', 'Bob')])
- **THEN** Generated SQL should be UPDATE "users" SET "name" = $1 WHERE "id" = $2 RETURNING "id", "name"

### Scenario: Insert with RETURNING ALL clause

- **GIVEN** A new QueryBuilder for 'orders' table
- **WHEN** Calling .returning_all().build_insert(&[('total', 100)])
- **THEN** Generated SQL should be INSERT INTO "orders" ("total") VALUES ($1) RETURNING *

### Scenario: Method chaining with Results

- **GIVEN** A new QueryBuilder for 'products' table
- **WHEN** Calling QueryBuilder::new('products')?.select(vec!['id'.to_string()])?.where_clause('price', Operator::Gt, 10)?
- **THEN** Should allow chaining with ? operator and return the final configured builder

## Diagrams

### Update with RETURNING Flow

```mermaid
flowchart TB
    start_update(Create QueryBuilder for 'users')
    where_id[Add WHERE id = 123]
    returning_cols[Add RETURNING id, name]
    build_update[Build UPDATE SQL with RETURNING clause]
    end_update(Generated SQL contains RETURNING #quot;id#quot;, #quot;name#quot;)
    start_update --> where_id
    where_id --> returning_cols
    returning_cols --> build_update
    build_update --> end_update
```

<semantic-data>

```json
{
  "edges": [],
  "metadata": null,
  "nodes": [
    {
      "id": "start_update",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "where_id",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "returning_cols",
      "semantic": {
        "type": "assign"
      }
    },
    {
      "id": "build_update",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "end_update",
      "semantic": {
        "type": "return"
      }
    }
  ]
}
```

</semantic-data>

</spec>
