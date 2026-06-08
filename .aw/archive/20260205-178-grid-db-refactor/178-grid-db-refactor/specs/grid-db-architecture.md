---
id: grid-db-architecture
type: spec
title: "Grid DB Architecture"
version: 1
spec_type: algorithm
created_at: 2026-02-05T04:45:54.311421+00:00
updated_at: 2026-02-05T04:45:54.311421+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-05T04:45:54.311421+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Grid DB Architecture

## Overview

Defines the core storage and query architecture for `cclab-grid-db`, centered on Morton encoding, WAL-backed cell persistence, and rectangular range queries over Morton-ordered keys.

## Requirements

### R1 - Morton Encoding

```yaml
id: R1
priority: must
status: draft
```

The system must encode `(x, y)` integer coordinates into a deterministic Morton (Z-order) key and provide a decode operation such that decoding the encoded key returns the original `(x, y)` within the supported coordinate bounds.

### R2 - Cell Persistence

```yaml
id: R2
priority: must
status: draft
```

The CellStore must support upsert/get/delete of cell records keyed by Morton key, persist updates by appending to the WAL before applying to the in-memory/on-disk store, and recover to the latest durable state by replaying the WAL idempotently.

### R3 - Range Queries

```yaml
id: R3
priority: must
status: draft
```

The query engine must accept an axis-aligned rectangular bounds input, map it to one or more Morton key ranges, scan the CellStore in Morton order, filter by the original rectangle, and return results ordered by Morton key.

## Acceptance Criteria

### Scenario: Morton Round-Trip

- **GIVEN** valid integer coordinates `(x, y)` within supported bounds
- **WHEN** the system encodes `(x, y)` to a Morton key and then decodes it
- **THEN** the decoded coordinates equal the original `(x, y)`

### Scenario: Durable Upsert

- **GIVEN** a new or existing cell payload and its `(x, y)` coordinates
- **WHEN** the CellStore performs an upsert
- **THEN** the update is appended to the WAL before applying to the store, and a subsequent recovery replays the WAL to the same final value

### Scenario: Range Query Over Rectangle

- **GIVEN** a rectangle defined by `min_x, min_y, max_x, max_y`
- **WHEN** the query engine executes a range query
- **THEN** the engine computes Morton ranges, scans the store, filters cells outside the rectangle, and returns the remaining cells ordered by Morton key

## Flow Diagram

```mermaid
flowchart LR
  subgraph Write_Path[Write Path]
    A[Request] -->|write| B[Encode (x,y) -> Morton Key]
    B --> C[Append WAL Entry]
    C --> D[Apply to CellStore]
    D --> E[Commit/Ack]
  end
  subgraph Query_Path[Query Path]
    Q[Rectangle Bounds] -->|query| R[Map to Morton Ranges]
    R --> S[Scan CellStore by Morton Key]
    S --> T[Filter by Rectangle]
    T --> U[Return Ordered Cells]
  end
```

</spec>
