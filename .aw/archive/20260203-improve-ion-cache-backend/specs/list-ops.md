---
id: list-ops
type: spec
title: "List Operations"
version: 1
spec_type: algorithm
created_at: 2026-01-30T06:11:02.059472+00:00
updated_at: 2026-01-30T06:11:02.059472+00:00
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
      title: "Blocking Pop Flow"
history:
  - timestamp: 2026-01-30T06:11:02.059472+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# List Operations

## Overview

Defines operations for the List data type (KvValue::List), including both non-blocking (LPUSH, LPOP) and blocking (BLPOP, BRPOP) commands.

## Requirements

### R1 - Push Elements

```yaml
id: R1
priority: medium
status: draft
```

Implement LPUSH and RPUSH to add elements to the head or tail of a list. Creates key if not exists. Returns list length.

### R2 - Pop Elements

```yaml
id: R2
priority: medium
status: draft
```

Implement LPOP and RPOP to remove and return elements from head or tail. Returns null if key doesn't exist.

### R3 - Read List

```yaml
id: R3
priority: medium
status: draft
```

Implement LRANGE to retrieve a range of elements and LLEN to get the length.

### R4 - Blocking Pop

```yaml
id: R4
priority: medium
status: draft
```

Implement BLPOP and BRPOP to block connection until an element is available or timeout occurs.

## Acceptance Criteria

### Scenario: Queue Operation

- **WHEN** Client calls RPUSH 'q' 'first' 'second', then LPOP 'q'
- **THEN** LPOP returns 'first', then 'second'

### Scenario: Blocking Pop Success

- **WHEN** Client A calls BLPOP 'q' 5, Client B calls RPUSH 'q' 'data'
- **THEN** Client A returns 'data' immediately after Client B's push

### Scenario: Blocking Pop Timeout

- **WHEN** Client calls BLPOP 'q' 1 and no data is pushed
- **THEN** Returns null

## Diagrams

### Blocking Pop Flow

```mermaid
flowchart TB
    Start(BLPOP)
    Check{Check List} 
    Pop[Pop Element]
    Register[Register Waiter]
    Wait([Wait/Sleep])
    RetNull(Return Null)
    End(Return Value)
    Start -->|BLPOP key timeout| Check
    Check -->|Exists & Not Empty| Pop
    Check -->|Empty/None| Register
    Register -->|| Wait
    Wait -->|Timeout| RetNull
    Wait -->|Data Pushed| Pop
    Pop -->|Return element| End
```

</spec>
