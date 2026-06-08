---
id: hash-ops
type: spec
title: "Hash Operations"
version: 1
spec_type: algorithm
created_at: 2026-01-30T06:10:14.975900+00:00
updated_at: 2026-01-30T06:10:14.975900+00:00
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
      title: "HSET Flow"
history:
  - timestamp: 2026-01-30T06:10:14.975900+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Hash Operations

## Overview

Defines operations for the Hash data type (KvValue::Map). Provides commands to manipulate fields within a hash value stored at a key.

## Requirements

### R1 - Set Hash Fields

```yaml
id: R1
priority: medium
status: draft
```

Implement HSET and HMSET to set one or multiple fields in a hash. Creates the key if it doesn't exist. Returns number of fields added.

### R2 - Get Hash Fields

```yaml
id: R2
priority: medium
status: draft
```

Implement HGET, HMGET, and HGETALL to retrieve values. HGET returns single value, HMGET returns list, HGETALL returns all field-value pairs.

### R3 - Delete Hash Fields

```yaml
id: R3
priority: medium
status: draft
```

Implement HDEL to remove fields. Returns number of fields removed.

### R4 - Hash Metadata

```yaml
id: R4
priority: medium
status: draft
```

Implement HEXISTS to check field existence and HLEN to get number of fields.

## Acceptance Criteria

### Scenario: Set and Get Hash

- **WHEN** Client calls HSET 'myhash' 'f1' 'bar' 'f2' 'baz'
- **THEN** HGET returns 'bar', HGETALL returns {'f1': 'bar', 'f2': 'baz'}

### Scenario: Type Mismatch

- **WHEN** Client calls HSET on a key holding a String
- **THEN** Returns WRONGTYPE error

### Scenario: Delete Field

- **WHEN** Client calls HDEL 'myhash' 'f1' on a hash with one field
- **THEN** HGET 'f1' returns null, HLEN returns 0

## Diagrams

### HSET Flow

```mermaid
flowchart TB
    Start(HSET/HMSET)
    CheckType{Check Key Type} 
    Error[Return Type Mismatch]
    CreateMap[Create New Map Value]
    UpdateMap[Insert/Update Fields]
    Success(Return Count)
    Start -->|Command Received| CheckType
    CheckType -->|Key exists & not Map| Error
    CheckType -->|Key doesn't exist| CreateMap
    CheckType -->|Key is Map| UpdateMap
    CreateMap -->|| UpdateMap
    UpdateMap -->|Fields Updated| Success
```

</spec>
