---
id: shield-basemodel-api-enhancement
type: spec
title: "Shield BaseModel API Enhancement"
version: 1
spec_type: utility
created_at: 2026-01-28T07:42:17.614370+00:00
updated_at: 2026-01-28T07:42:17.614370+00:00
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
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "JSON Operations Flow"
history:
  - timestamp: 2026-01-28T07:42:17.614370+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Shield BaseModel API Enhancement

## Overview

This specification enhances the BaseModel API to provide Pydantic-compatible methods for JSON serialization/deserialization and improved error reporting.

## Requirements

### R1 - JSON Serialization

```yaml
id: R1
priority: medium
status: draft
```

Provide model_dump_json() for high-performance serialization.

### R2 - JSON Deserialization

```yaml
id: R2
priority: medium
status: draft
```

Provide model_validate_json() for direct JSON-to-Model validation.

### R3 - Structured Errors

```yaml
id: R3
priority: medium
status: draft
```

ValidationError should include structured data: loc (tuple), msg (str), type (str), and input (any).

### R4 - Field Aliases

```yaml
id: R4
priority: medium
status: draft
```

Support 'alias' in Field for mapping different external keys to internal attributes.

## Acceptance Criteria

### Scenario: JSON Round-trip

- **GIVEN** A model instance with some data.
- **WHEN** Dumping to JSON and validating back.
- **THEN** The new instance should be equal to the original.

### Scenario: Alias validation

- **GIVEN** A field userName with alias=user_name.
- **WHEN** Validating input {'user_name': 'John'}.
- **THEN** The model should be populated correctly with John.

### Scenario: Error formatting depth

- **GIVEN** Nested objects with validation errors.
- **WHEN** Validating invalid zip_code.
- **THEN** The error loc should be ('user', 'address', 'zip_code').

### Scenario: Alias serialization

- **GIVEN** A field with alias='user_id' and name='id'.
- **WHEN** Dumping model to JSON.
- **THEN** The output JSON should contain 'user_id' instead of 'id'.

## Diagrams

### JSON Operations Flow

```mermaid
flowchart LR
    dump_json[model_dump_json()]
    rust_ser[Rust Serialization]
    json_out[JSON String]
    val_json[model_validate_json()]
    rust_de[Rust Deserialization & Validation]
    model_inst[Model Instance]
    dump_json --> rust_ser
    rust_ser --> json_out
    val_json --> rust_de
    rust_de --> model_inst
```

</spec>
