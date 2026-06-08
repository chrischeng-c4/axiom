---
id: query-builder
type: spec
title: "Rust-Backed Query Builder Parity"
version: 1
spec_type: utility
created_at: 2026-02-03T10:07:44.816783+00:00
updated_at: 2026-02-03T10:07:44.816783+00:00
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
  has_semantic_diagrams: false
  diagrams:
    - type: sequence
      title: "Query Builder Delegation (Read + Link Fetch)"
history:
  - timestamp: 2026-02-03T10:07:44.816783+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-03T10:07:55.883977+00:00
    agent: "codex:deep"
    tool: "revise_spec"
    action: "revised"
  - timestamp: 2026-02-03T10:08:11.751158+00:00
    agent: "codex:max"
    tool: "review_spec"
    action: "reviewed"---

<spec>

# Rust-Backed Query Builder Parity

## Overview

Define the Rust-backed QueryBuilder integration to preserve Python query semantics while delegating read and write operations to PyQueryBuilder for performance and parity. The spec covers query construction, execution, write operations (update/delete/upsert), fluent update operators, and link-fetch handling expectations.

## Requirements

### R1 - Query Execution Delegation

```yaml
id: R1
priority: medium
status: draft
```

Python QueryBuilder must delegate read operations (to_list, first, count, exists) to Rust PyQueryBuilder while preserving filters, sort, skip, limit, projection, and inheritance filtering (with_children/_class_id). Returned BSON docs must be converted into Document instances using existing _from_db behavior, with the original model class determining polymorphic loading.

### R2 - Write Operations Parity

```yaml
id: R2
priority: medium
status: draft
```

Rust PyQueryBuilder must expose delete, update, and upsert operations compatible with Python QueryBuilder. delete returns deleted_count (int), update returns modified_count (int) with optional upsert flag, and upsert returns a dict containing matched_count, modified_count, and upserted_id (or None).

### R3 - Fluent Update Operators

```yaml
id: R3
priority: medium
status: draft
```

Python QueryBuilder fluent update helpers (set/inc/push/pull/add_to_set/unset) must map to the corresponding MongoDB update operators and route through the Rust-backed update path. FieldProxy keys are converted to field names before the update doc is sent.

### R4 - Fetch Links Behavior

```yaml
id: R4
priority: medium
status: draft
```

When fetch_links is enabled, QueryBuilder must perform batched link fetching after the Rust query results are converted into Document instances, preserving depth semantics and existing BackLink handling.

### R5 - Input Validation and Errors

```yaml
id: R5
priority: medium
status: draft
```

Rust PyQueryBuilder must validate collection names and queries using existing validation helpers and raise Python errors on invalid inputs (e.g., negative limit, malformed sort tuples, or non-dict update documents), matching current behavior where applicable.

### R6 - Non-Regression in Filters

```yaml
id: R6
priority: medium
status: draft
```

Filter construction must remain functionally identical to the current Python implementation, including merging multiple filters and applying _class_id constraints for root/child models before Rust execution.

## Acceptance Criteria

### Scenario: Read With Inheritance Filter

- **GIVEN** a root model with child classes and with_children set to false
- **WHEN** QueryBuilder.to_list is called
- **THEN** the filter sent to PyQueryBuilder includes the root _class_id constraint and results are returned as Document instances of the correct class.

### Scenario: Read With Sort Skip Limit

- **GIVEN** a QueryBuilder with sort, skip, and limit set
- **WHEN** QueryBuilder.first is called
- **THEN** PyQueryBuilder executes the query with the expected options and returns the first matching document or None.

### Scenario: Update With Upsert

- **GIVEN** a QueryBuilder and an update document with upsert enabled
- **WHEN** QueryBuilder.update is invoked with upsert=True
- **THEN** PyQueryBuilder performs update_many with upsert and returns the modified_count from the Rust result.

### Scenario: Upsert Returns Counts

- **GIVEN** a QueryBuilder and an update document
- **WHEN** QueryBuilder.upsert is called
- **THEN** the result contains matched_count, modified_count, and upserted_id fields consistent with MongoDB semantics.

### Scenario: Fluent Operator Mapping

- **GIVEN** a QueryBuilder using set/inc/push/pull/add_to_set/unset with FieldProxy keys
- **WHEN** the method is called
- **THEN** the update document sent to Rust uses the correct MongoDB operator and field names derived from FieldProxy.

### Scenario: Fetch Links After Query

- **GIVEN** fetch_links is enabled with depth=2 and query results include Link fields
- **WHEN** QueryBuilder.to_list completes
- **THEN** linked documents are batched and hydrated to the requested depth using the existing link fetching utilities.

## Diagrams

### Query Builder Delegation (Read + Link Fetch)

```mermaid
sequenceDiagram
    participant User as User Code
    participant Python as Python QueryBuilder
    participant Rust as Rust PyQueryBuilder
    participant Mongo as MongoDB
    participant Links as Python Link Fetcher
    User->>Python: find(filter).sort().limit()
    Python->>Rust: build filter/sort/limit
    User->>Python: to_list(fetch_links=True)
    Python->>Rust: to_list()
    Rust->>Mongo: execute query
    Mongo->>Rust: documents
    Rust->>Python: BSON dicts
    Python->>Python: hydrate Document instances
    Python->>Links: fetch_links_batched(docs, depth)
    Links->>Python: hydrated links
    Python->>User: return documents
```

</spec>
