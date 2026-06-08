---
id: pulsar-frame-core
type: spec
title: "Pulsar Frame Core"
version: 1
spec_type: utility
created_at: 2026-01-30T06:31:29.025527+00:00
updated_at: 2026-01-30T06:31:29.025527+00:00
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
    - type: class
      title: "Core Classes"
history:
  - timestamp: 2026-01-30T06:31:29.025527+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Frame Core

## Overview

Defines the core structures (Series, DataFrame, Index) and indexing logic.

## Requirements

### R1 - Series Structure

```yaml
id: R1
priority: medium
status: draft
```

Implement Series struct.

### R2 - Index Structure

```yaml
id: R2
priority: medium
status: draft
```

Implement Index struct.

### R3 - DataFrame Structure

```yaml
id: R3
priority: medium
status: draft
```

Implement DataFrame struct.

### R4 - Indexing

```yaml
id: R4
priority: medium
status: draft
```

Implement loc/iloc indexing.

## Acceptance Criteria

### Scenario: Select Column

- **GIVEN** DataFrame
- **WHEN** Select col
- **THEN** Series returned

### Scenario: Loc Indexing

- **GIVEN** DataFrame
- **WHEN** Call loc
- **THEN** Row/Col returned

### Scenario: Iloc Indexing

- **GIVEN** DataFrame
- **WHEN** Call iloc
- **THEN** Row/Col returned

## Diagrams

### Core Classes

```mermaid
classDiagram
    class Series {
        +NdArray values
        +String name
        +new(NdArray data) Self
    }
    class Index {
        +Vec<Value> values
        +new(Vec<Value> values) Self
    }
    class DataFrame {
        +Vec<Series> columns
        +loc(Value label) Series
    }
    DataFrame *-- Series : contains
```

## Data Model

```json
{
  "properties": {
    "columns": {
      "items": {
        "$ref": "#/definitions/Series"
      },
      "type": "array"
    },
    "index": {
      "$ref": "#/definitions/Index"
    }
  },
  "required": [
    "columns",
    "index"
  ],
  "type": "object"
}
```

</spec>
