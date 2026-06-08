---
id: magic-methods
type: spec
title: "Magic Method Dispatch"
version: 1
spec_type: utility
spec_group: cclab-mamba
created_at: 2026-02-15T17:31:49.366660+00:00
updated_at: 2026-02-15T17:31:49.366660+00:00
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
  - exception-hierarchy
changes:
  - file: crates/mamba/src/runtime/class.rs
    action: MODIFY
    description: "Add mb_binop, mb_compare, mb_call_str/repr/bool/len/iter/next/contains"
  - file: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    description: "Lower operators to mb_binop/mb_compare CallExtern"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register dunder dispatch functions"
history:
  - timestamp: 2026-02-15T17:31:49.366660+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Magic Method Dispatch

## Overview

Implements magic method (dunder) dispatch for operators and protocol methods. Binary operators (+, -, *, /, //, %, **) route to __add__/__sub__/__mul__ etc. Comparison operators route to __eq__/__lt__/__gt__ etc. Conversion functions str()/repr()/bool() route to __str__/__repr__/__bool__. Protocol methods __len__/__iter__/__next__/__contains__ support len(), for-loops, and in-operator. For built-in types, dispatch goes directly to optimized runtime functions; for user classes, falls back to MRO attribute lookup.

## Requirements

### R1 - Arithmetic operator dispatch

```yaml
id: R1
priority: high
status: draft
```

mb_binop(left, right, op) -> result. Maps op enum to __add__/__sub__/__mul__/__truediv__/__floordiv__/__mod__/__pow__. For int+int, call optimized path. For user classes, look up dunder via MRO.

### R2 - Comparison operator dispatch

```yaml
id: R2
priority: high
status: draft
```

mb_compare(left, right, op) -> bool. Maps to __eq__/__ne__/__lt__/__le__/__gt__/__ge__. Returns MbValue bool.

### R3 - Conversion dunder dispatch

```yaml
id: R3
priority: high
status: draft
```

mb_call_str(obj), mb_call_repr(obj), mb_call_bool(obj) — invoke __str__/__repr__/__bool__ on obj.

### R4 - Protocol dunder dispatch

```yaml
id: R4
priority: high
status: draft
```

mb_call_len(obj) -> __len__. mb_call_iter(obj) -> __iter__. mb_call_next(obj) -> __next__. mb_call_contains(container, item) -> __contains__.

### R5 - Symbol registration and lowering

```yaml
id: R5
priority: high
status: draft
```

Register all dunder dispatch functions in symbols.rs. Update hir_to_mir.rs to emit CallExtern for operators/conversions/protocols.

## Acceptance Criteria

### Scenario: Int addition

- **WHEN** 3 + 4 evaluated
- **THEN** mb_binop routes to optimized int add, returns 7

### Scenario: User class __add__

- **GIVEN** class Vec with __add__ method
- **WHEN** Vec(1,2) + Vec(3,4)
- **THEN** MRO lookup finds __add__, calls it

### Scenario: Str conversion

- **WHEN** str(42) called
- **THEN** mb_call_str routes to int __str__, returns '42'

### Scenario: Len protocol

- **WHEN** len([1,2,3]) called
- **THEN** mb_call_len routes to list __len__, returns 3

</spec>
