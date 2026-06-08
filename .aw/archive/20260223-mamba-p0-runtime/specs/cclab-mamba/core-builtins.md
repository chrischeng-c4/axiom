---
id: core-builtins
type: spec
title: "Core Built-in Functions"
version: 1
spec_type: utility
spec_group: cclab-mamba
created_at: 2026-02-15T17:31:16.764683+00:00
updated_at: 2026-02-15T17:31:16.764683+00:00
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
  - file: crates/mamba/src/runtime/builtins.rs
    action: MODIFY
    description: "Add new builtin functions"
  - file: crates/mamba/src/runtime/iter.rs
    action: MODIFY
    description: "Add EnumerateIterator, ZipIterator, ReversedIterator variants"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register new builtins"
history:
  - timestamp: 2026-02-15T17:31:16.764683+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Core Built-in Functions

## Overview

Extends builtins.rs with commonly used Python built-in functions: enumerate, zip, min, max, sum, sorted, reversed, isinstance, input, hash, id, repr, hex, oct, bin, chr, ord. Iterator-returning builtins (enumerate, zip, reversed) create new ObjData::Iterator variants. All registered in symbols.rs.

## Requirements

### R1 - Iterator builtins

```yaml
id: R1
priority: high
status: draft
```

enumerate(iterable, start=0), zip(iter_a, iter_b), reversed(seq) — return new iterator objects.

### R2 - Aggregation builtins

```yaml
id: R2
priority: high
status: draft
```

min(iterable), max(iterable), sum(iterable, start=0), sorted(iterable) — iterate and return result.

### R3 - Type checking builtins

```yaml
id: R3
priority: high
status: draft
```

isinstance(obj, classinfo) -> bool. Uses exception hierarchy for class matching.

### R4 - Conversion builtins

```yaml
id: R4
priority: medium
status: draft
```

repr(obj), hash(obj), id(obj), input(prompt?), chr(i), ord(c), hex(n), oct(n), bin(n).

### R5 - Symbol registration

```yaml
id: R5
priority: high
status: draft
```

Register all new builtins in symbols.rs.

## Acceptance Criteria

### Scenario: Enumerate

- **WHEN** list(enumerate(['a','b']))
- **THEN** Returns [(0,'a'),(1,'b')]

### Scenario: Min/max

- **WHEN** min([3,1,2]), max([3,1,2])
- **THEN** Returns 1 and 3

### Scenario: Isinstance

- **WHEN** isinstance(ValueError('x'), Exception)
- **THEN** Returns True

### Scenario: Sorted

- **WHEN** sorted([3,1,2])
- **THEN** Returns [1,2,3]

</spec>
