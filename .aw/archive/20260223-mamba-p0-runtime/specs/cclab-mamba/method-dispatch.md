---
id: method-dispatch
type: spec
title: "Type-Tagged Method Dispatch for Built-in Types"
version: 1
spec_type: algorithm
tags: [logic]
spec_group: cclab-mamba
created_at: 2026-02-15T17:30:31.966813+00:00
updated_at: 2026-02-15T17:30:31.966813+00:00
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
      title: "Dispatch Flow"
changes:
  - file: crates/mamba/src/runtime/class.rs
    action: MODIFY
    description: "Add mb_call_method"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register mb_call_method"
  - file: crates/mamba/src/lower/hir_to_mir.rs
    action: MODIFY
    description: "Lower method calls"
history:
  - timestamp: 2026-02-15T17:30:31.966813+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Type-Tagged Method Dispatch for Built-in Types

## Overview

Implements a type-tagged method dispatch mechanism for built-in types (str, list, dict, int, float, bool). When a method call like x.split() is encountered, the runtime extracts the ObjData tag from the receiver, looks up the method name in a dispatch table, and calls the corresponding Rust runtime function. For user-defined classes, falls back to MRO-based attribute lookup. This is the foundation for all P0 method implementations.

## Requirements

### R1 - Dispatch table for built-in types

```yaml
id: R1
priority: high
status: draft
```

Create mb_call_method(receiver: i64, method_name: i64, args: *const i64, argc: i64) -> i64 that extracts ObjData variant, matches (variant, method_name_str), calls target runtime function, returns i64. Raises AttributeError for unknown methods.

### R2 - Primitive type dispatch

```yaml
id: R2
priority: high
status: draft
```

For NaN-boxed primitives (TAG_INT, TAG_BOOL, TAG_NONE), dispatch methods without heap allocation.

### R3 - MRO fallback for user classes

```yaml
id: R3
priority: high
status: draft
```

When receiver is ObjData::Instance, fall back to MRO-based attribute lookup.

### R4 - Symbol registration

```yaml
id: R4
priority: high
status: draft
```

Register mb_call_method in symbols.rs as MirExtern for JIT/AOT.

## Acceptance Criteria

### Scenario: String method dispatch

- **WHEN** s.split(' ') called on string
- **THEN** Routes to mb_string_split

### Scenario: Primitive dispatch

- **WHEN** str(42) invokes __str__ on int
- **THEN** Handles TAG_INT without heap lookup

### Scenario: MRO fallback

- **WHEN** Foo().bar() on user class
- **THEN** Falls back to MRO

### Scenario: AttributeError

- **WHEN** 42.nonexistent() called
- **THEN** Raises AttributeError

## Diagrams

### Dispatch Flow

```mermaid
flowchart TB
    s([mb_call_method])
    t{Check tag} 
    p[Primitive dispatch]
    d[Deref ObjData]
    m{Match variant} 
    b[Builtin table]
    i[MRO fallback]
    e([AttributeError])
    r([Return])
    s --> t
    t -->|prim| p
    t -->|PTR| d
    p --> r
    d --> m
    m -->|builtin| b
    m -->|Instance| i
    m -->|other| e
    b --> r
    i --> r
```

</spec>
