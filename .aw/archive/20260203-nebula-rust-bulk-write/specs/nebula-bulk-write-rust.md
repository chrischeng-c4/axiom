---
id: nebula-bulk-write-rust
type: spec
title: "Nebula Rust Bulk Write Implementation"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:43:27.255835+00:00
updated_at: 2026-01-31T10:43:27.255835+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: true
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Bulk Write Execution Flowchart"
history:
  - timestamp: 2026-01-31T10:43:27.255835+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Nebula Rust Bulk Write Implementation

## Overview

This specification defines the migration of the bulk write core logic from Python to Rust. It leverages Rust's type safety and performance by implementing a dedicated BulkOperation enum and utilizing PyO3's automatic conversion capabilities.

## Requirements

### R1 - BulkOperation Enum Design

```yaml
id: R1
priority: medium
status: draft
```

Implement a Rust enum `BulkOperation` that encapsulates all supported MongoDB bulk write operations.

### R2 - Automatic Python-to-Rust Conversion

```yaml
id: R2
priority: medium
status: draft
```

Implement the `FromPyObject` trait for `BulkOperation` to allow automatic conversion from Python dictionaries (as produced by the existing `to_dict()` methods).

### R3 - Rust bulk_write Method

```yaml
id: R3
priority: medium
status: draft
```

Add a static method `bulk_write` to `RustDocument` in `cclab-nucleus` that accepts the collection name, a list of operations, and the ordered flag.

### R4 - Bulk Write Result Mapping

```yaml
id: R4
priority: medium
status: draft
```

Return a structured `BulkWriteResult` (as a Python dictionary) containing counts for inserted, matched, modified, deleted, and upserted documents.

## Acceptance Criteria

### Scenario: Happy Path - Mixed Operations

- **GIVEN** A list of valid InsertOne and UpdateOne operations from Python.
- **WHEN** User calls `bulk_write` with the list.
- **THEN** The operations are correctly executed in Rust, and a dictionary with correct counts is returned to Python.

### Scenario: Ordered Execution - Stop on Error

- **GIVEN** A list of operations where one operation fails (e.g., unique constraint violation).
- **WHEN** User calls `bulk_write` with `ordered=True`.
- **THEN** Execution stops at the first error, and an exception is raised in Python. Successes before the error are persisted.

### Scenario: Edge Case - Empty List

- **GIVEN** An empty list of operations.
- **WHEN** User calls `bulk_write` with an empty list.
- **THEN** Rust returns a result with all counts set to zero without contacting the database.

## Diagrams

### Bulk Write Execution Flowchart

```mermaid
flowchart TB
    Start[Python call bulk_write]
    ExtractOps[PyO3 Extract operations (FromPyObject)]
    ConvertToBson[Convert to BSON (GIL released)]
    ExecuteBulkWrite[Execute MongoDB bulk_write]
    FormatResult[Format BulkWriteResult for Python]
    End[Return to Python]
    Start --> ExtractOps
    ExtractOps --> ConvertToBson
    ConvertToBson --> ExecuteBulkWrite
    ExecuteBulkWrite --> FormatResult
    FormatResult --> End
```

## Data Model

```json
{
  "properties": {
    "operations": {
      "items": {
        "anyOf": [
          {
            "properties": {
              "document": {
                "type": "object"
              },
              "op": {
                "const": "insert_one"
              }
            },
            "required": [
              "op",
              "document"
            ],
            "type": "object"
          },
          {
            "properties": {
              "filter": {
                "type": "object"
              },
              "op": {
                "const": "update_one"
              },
              "update": {
                "type": "object"
              },
              "upsert": {
                "type": "boolean"
              }
            },
            "required": [
              "op",
              "filter",
              "update"
            ],
            "type": "object"
          },
          {
            "properties": {
              "filter": {
                "type": "object"
              },
              "op": {
                "const": "update_many"
              },
              "update": {
                "type": "object"
              },
              "upsert": {
                "type": "boolean"
              }
            },
            "required": [
              "op",
              "filter",
              "update"
            ],
            "type": "object"
          },
          {
            "properties": {
              "filter": {
                "type": "object"
              },
              "op": {
                "const": "delete_one"
              }
            },
            "required": [
              "op",
              "filter"
            ],
            "type": "object"
          },
          {
            "properties": {
              "filter": {
                "type": "object"
              },
              "op": {
                "const": "delete_many"
              }
            },
            "required": [
              "op",
              "filter"
            ],
            "type": "object"
          },
          {
            "properties": {
              "filter": {
                "type": "object"
              },
              "op": {
                "const": "replace_one"
              },
              "replacement": {
                "type": "object"
              },
              "upsert": {
                "type": "boolean"
              }
            },
            "required": [
              "op",
              "filter",
              "replacement"
            ],
            "type": "object"
          }
        ]
      },
      "type": "array"
    },
    "ordered": {
      "default": true,
      "type": "boolean"
    }
  },
  "required": [
    "operations"
  ],
  "type": "object"
}
```

</spec>
