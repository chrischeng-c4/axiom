---
id: builtin-types
type: spec
title: "Built-in Types"
version: 1
spec_type: utility
created_at: 2026-02-22T11:23:50.441228+00:00
updated_at: 2026-02-22T11:23:50.441228+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:23:50.441228+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Built-in Types

## Overview

Implement essential built-in types: `bytes`, `bytearray`, and `frozenset`.

## Requirements

### R1 - Bytes Type

```yaml
id: R1
priority: medium
status: draft
```

Implement the immutable `bytes` type with sequence operations.

### R2 - ByteArray Type

```yaml
id: R2
priority: medium
status: draft
```

Implement the mutable `bytearray` type with in-place modification.

### R3 - Frozenset Type

```yaml
id: R3
priority: medium
status: draft
```

Implement the immutable `frozenset` type with set operations.

## Acceptance Criteria

### Scenario: Bytes indexing

- **GIVEN** A bytes object b'abc'
- **WHEN** I access b[0]
- **THEN** It returns 97.

### Scenario: Bytearray modification

- **GIVEN** A bytearray object
- **WHEN** I set b[0] = 255
- **THEN** The first byte is 255.

### Scenario: Frozenset immutability

- **GIVEN** A frozenset {1, 2}
- **WHEN** I try to add an element
- **THEN** It raises AttributeError or TypeError.

</spec>
