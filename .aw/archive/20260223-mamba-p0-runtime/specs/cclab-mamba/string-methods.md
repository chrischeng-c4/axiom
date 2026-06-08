---
id: string-methods
type: spec
title: "String Method Implementations"
version: 1
spec_type: utility
spec_group: cclab-mamba
created_at: 2026-02-15T17:30:51.143294+00:00
updated_at: 2026-02-15T17:30:51.143294+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
depends:
  - method-dispatch
changes:
  - file: crates/mamba/src/runtime/string_ops.rs
    action: MODIFY
    description: "Add string methods"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register string methods"
history:
  - timestamp: 2026-02-15T17:30:51.143294+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# String Method Implementations

## Overview

Implements Python string methods as extern C runtime functions in string_ops.rs. Each method takes an i64 MbValue receiver (ObjData::Str) and returns an i64 MbValue result. Covers: split, join, strip/lstrip/rstrip, replace, find, startswith, endswith, upper, lower, count, isdigit, isalpha. All registered in symbols.rs and wired into method-dispatch table.

## Requirements

### R1 - String split/join

```yaml
id: R1
priority: high
status: draft
```

mb_string_split(self, sep) -> list. mb_string_join(self, iterable) -> string.

### R2 - String strip methods

```yaml
id: R2
priority: high
status: draft
```

mb_string_strip/lstrip/rstrip(self) -> stripped string.

### R3 - String replace/find

```yaml
id: R3
priority: high
status: draft
```

mb_string_replace(self, old, new) -> new string. mb_string_find(self, sub) -> index or -1.

### R4 - String predicates

```yaml
id: R4
priority: medium
status: draft
```

startswith, endswith, isdigit, isalpha, count.

### R5 - String case methods

```yaml
id: R5
priority: medium
status: draft
```

mb_string_upper/lower(self) -> case-converted string.

### R6 - Symbol registration

```yaml
id: R6
priority: high
status: draft
```

Register all in symbols.rs, wire into dispatch table.

## Acceptance Criteria

### Scenario: Split

- **WHEN** 'a,b,c'.split(',')
- **THEN** Returns ['a','b','c']

### Scenario: Join

- **WHEN** ', '.join(['a','b'])
- **THEN** Returns 'a, b'

### Scenario: Strip

- **WHEN** '  hi  '.strip()
- **THEN** Returns 'hi'

### Scenario: Replace

- **WHEN** 'hello'.replace('l','r')
- **THEN** Returns 'herro'

</spec>
