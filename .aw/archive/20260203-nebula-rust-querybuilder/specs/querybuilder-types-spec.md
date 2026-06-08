---
id: querybuilder-types-spec
type: spec
title: "Rust QueryBuilder Types Design"
version: 1
spec_type: utility
created_at: 2026-02-01T07:10:46.507018+00:00
updated_at: 2026-02-01T07:10:46.507018+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-01T07:10:46.507018+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Rust QueryBuilder Types Design

## Overview

This specification defines the internal Rust data structures for Nebula's QueryBuilder and QueryExpr. It focuses on a type-safe, clone-based design that allows for efficient MongoDB query construction and manipulation.

```mermaid
classDiagram
    class QueryBuilder {
        <<struct>>
        +collection: String
        +filters: Vec<QueryExpr>
        +sort: Option<Vec<(String, i32)>>
        +skip: Option<u64>
        +limit: Option<i64>
        +projection: Option<Vec<String>>
        +new(collection) Self
        +filter(expr) Self
        +sort(fields) Self
        +build_filter() Document
        +build_options() FindOptions
    }
    class QueryExpr {
        <<enum>>
        Eq(field, value)
        Ne(field, value)
        And(Vec<QueryExpr>)
        Or(Vec<QueryExpr>)
    }
    QueryBuilder *-- QueryExpr
```

## Requirements

### R1 - QueryExpr Enum Definition

```yaml
id: R1
priority: high
status: draft
```

Define a QueryExpr enum to represent MongoDB query operators (Eq, Ne, Gt, etc.) and logical operators (And, Or).

### R2 - QueryBuilder Struct Definition

```yaml
id: R2
priority: high
status: draft
```

Define a QueryBuilder struct to hold the state of a MongoDB query, including collection name, filters, sort order, and pagination parameters.

### R3 - Chainable API Methods

```yaml
id: R3
priority: high
status: draft
```

Implement clone-based chainable methods on QueryBuilder (filter, sort, skip, limit, projection) that return a new instance of the builder.

### R4 - Query Construction Logic

```yaml
id: R4
priority: high
status: draft
```

Provide methods to convert the QueryBuilder state into MongoDB-compatible BsonDocument and FindOptions.

## Acceptance Criteria

### Scenario: Basic Filter Creation

- **WHEN** The user creates an equality expression for a field.
- **THEN** A QueryExpr::Eq variant is created with the specified field and value.

### Scenario: Chaining Multiple Operations

- **WHEN** The user chains .filter().sort().limit() on a QueryBuilder instance.
- **THEN** Each step returns a new QueryBuilder instance containing the accumulated state.

### Scenario: BSON Conversion

- **WHEN** The to_bson() method is called on a QueryExpr.
- **THEN** A valid MongoDB filter document is generated representing the expression.

</spec>
