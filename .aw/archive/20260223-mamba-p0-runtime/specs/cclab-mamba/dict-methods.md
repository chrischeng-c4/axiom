---
id: dict-methods
type: spec
title: "Dict Method Implementations"
version: 1
spec_type: utility
spec_group: cclab-mamba
created_at: 2026-02-15T17:31:07.009218+00:00
updated_at: 2026-02-15T17:31:07.009218+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
depends:
  - method-dispatch
changes:
  - file: crates/mamba/src/runtime/dict_ops.rs
    action: CREATE
    description: "New dict methods module"
  - file: crates/mamba/src/runtime/mod.rs
    action: MODIFY
    description: "Add pub mod dict_ops"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register dict methods"
history:
  - timestamp: 2026-02-15T17:31:07.009218+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Dict Method Implementations

## Overview

Implements Python dict methods as extern C runtime functions in a new dict_ops.rs module. Covers: get, keys, values, items, update, pop, setdefault, clear, copy. Views return materialized lists for MVP. Registered in symbols.rs and wired into method-dispatch table.

## Requirements

### R1 - Dict lookup methods

```yaml
id: R1
priority: high
status: draft
```

get(key, default?) -> value or default. setdefault(key, default?) -> value.

### R2 - Dict view methods

```yaml
id: R2
priority: high
status: draft
```

keys(), values(), items() -> materialized lists.

### R3 - Dict mutation methods

```yaml
id: R3
priority: high
status: draft
```

update(other), pop(key, default?), clear(). Raise KeyError when appropriate.

### R4 - Dict copy

```yaml
id: R4
priority: medium
status: draft
```

copy() -> shallow copy.

### R5 - Symbol registration

```yaml
id: R5
priority: high
status: draft
```

Register all, add pub mod dict_ops, wire into dispatch.

## Acceptance Criteria

### Scenario: Get with default

- **WHEN** {'a':1}.get('b',0)
- **THEN** Returns 0

### Scenario: Keys

- **WHEN** {'a':1,'b':2}.keys()
- **THEN** Returns ['a','b']

### Scenario: Pop KeyError

- **WHEN** {'a':1}.pop('b')
- **THEN** Raises KeyError

### Scenario: Update

- **WHEN** d={'a':1}; d.update({'b':2})
- **THEN** d is {'a':1,'b':2}

</spec>
