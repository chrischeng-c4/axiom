---
id: json-schema-core
type: spec
title: "JSON Schema Core Implementation"
version: 1
spec_type: data-model
created_at: 2026-02-02T13:49:43.559096+00:00
updated_at: 2026-02-02T13:49:43.559096+00:00
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
  has_api_spec: true
  has_semantic_diagrams: false
  api_spec_type: json-schema
  diagrams:
    - type: class
      title: "JSON Schema Core Class Diagram"
history:
  - timestamp: 2026-02-02T13:49:43.559096+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# JSON Schema Core Implementation

## Overview

Defines the core JSON Schema structures and parsing logic for cclab-aurora. This module is responsible for parsing JSON Schema strings into a strongly-typed Rust structure that can be used by validators and generators.

## Requirements

### R1 - Version Support

```yaml
id: R1
priority: medium
status: draft
```

The module must support parsing Draft 7 and Draft 2020-12 JSON Schemas.

### R2 - Typed Structure

```yaml
id: R2
priority: medium
status: draft
```

The module must provide a strongly-typed structure for schemas, including handling of recursive definitions ($ref).

### R3 - Serde Integration

```yaml
id: R3
priority: medium
status: draft
```

The module must handle serialization and deserialization using Serde.

## Acceptance Criteria

### Scenario: Parse Draft 7 Schema

- **GIVEN** A valid Draft 7 JSON Schema string
- **WHEN** The parse function is called
- **THEN** It is successfully parsed into a JsonSchema struct

### Scenario: Handle Recursion

- **GIVEN** A JSON Schema with a circular $ref
- **WHEN** The schema is traversed
- **THEN** The structure preserves the reference or resolves it lazily

## Diagrams

### JSON Schema Core Class Diagram

```mermaid
classDiagram
    class JsonSchemaCore {
    }
    JsonSchema *-- Schema : contains
    Schema --> SchemaType : has type
```

## API Specification (JSON Schema)

```yaml
properties:
  definitions:
    type: object
  schema_version:
    type: string
title: JsonSchema
type: object
```

</spec>
