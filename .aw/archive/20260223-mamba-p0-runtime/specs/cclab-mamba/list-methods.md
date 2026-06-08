---
id: list-methods
type: spec
title: "List Method Implementations"
version: 1
spec_type: utility
spec_group: cclab-mamba
created_at: 2026-02-15T17:30:59.614735+00:00
updated_at: 2026-02-15T17:30:59.614735+00:00
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
  - file: crates/mamba/src/runtime/list_ops.rs
    action: CREATE
    description: "New list methods module"
  - file: crates/mamba/src/runtime/mod.rs
    action: MODIFY
    description: "Add pub mod list_ops"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register list methods"
history:
  - timestamp: 2026-02-15T17:30:59.614735+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# List Method Implementations

## Overview

Implements Python list methods as extern C runtime functions in a new list_ops.rs module. Covers: append, pop, insert, remove, extend, sort, reverse, index, count, clear, copy. All mutate or query the underlying Vec in ObjData::List. Registered in symbols.rs and wired into method-dispatch table.

## Requirements

### R1 - List mutation methods

```yaml
id: R1
priority: high
status: draft
```

append, insert, remove, clear, extend — all mutate in-place, return None.

### R2 - List pop/index

```yaml
id: R2
priority: high
status: draft
```

pop(index?) -> removed item. index(item) -> position. Raise ValueError/IndexError on failure.

### R3 - List sort/reverse

```yaml
id: R3
priority: medium
status: draft
```

sort() and reverse() — in-place, return None.

### R4 - List copy/count

```yaml
id: R4
priority: medium
status: draft
```

copy() -> shallow copy. count(item) -> occurrences.

### R5 - Symbol registration

```yaml
id: R5
priority: high
status: draft
```

Register all in symbols.rs, add pub mod list_ops, wire into dispatch.

## Acceptance Criteria

### Scenario: Append and pop

- **WHEN** l=[1,2]; l.append(3); l.pop()
- **THEN** Returns 3, l is [1,2]

### Scenario: Sort

- **WHEN** [3,1,2].sort()
- **THEN** List becomes [1,2,3]

### Scenario: Remove ValueError

- **WHEN** [1,2].remove(3)
- **THEN** Raises ValueError

</spec>
