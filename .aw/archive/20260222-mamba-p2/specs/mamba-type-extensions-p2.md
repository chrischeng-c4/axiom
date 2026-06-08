---
id: mamba-type-extensions-p2
type: spec
title: "Type extensions: frozenset, enum, dataclasses"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-22T11:20:03.360298+00:00
updated_at: 2026-02-22T11:20:03.360298+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Dataclass Decorator Flow"
history:
  - timestamp: 2026-02-22T11:20:03.360298+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Type extensions: frozenset, enum, dataclasses

## Overview

Add three new type system features to Mamba: frozenset (immutable set with ObjData::FrozenSet variant), enum module (Enum base class, auto() values, IntEnum/StrEnum), and dataclasses decorator (@dataclass auto-generating __init__, __repr__, __eq__ from class fields).

## Requirements

### R1 - FrozenSet ObjData variant

```yaml
id: R1
priority: high
status: draft
```

Add ObjData::FrozenSet(Vec<MbValue>) to rc.rs. Implement frozenset_ops.rs with immutable set operations: union, intersection, difference, symmetric_difference, issubset, issuperset, contains. No mutation methods (add/remove/discard). Update GC mark_object, json_mod, and all ObjData match sites.

### R2 - frozenset() constructor and dispatch

```yaml
id: R2
priority: high
status: draft
```

Add mb_frozenset_new(iterable) builtin. Register dispatch_frozenset_method in builtins method dispatch. Register frozenset symbols in symbols.rs.

### R3 - Enum base class

```yaml
id: R3
priority: medium
status: draft
```

Create enum_mod.rs in stdlib. Enum metaclass stores members dict. Support class MyEnum(Enum): A = 1; B = 2 syntax. Members accessible by name (MyEnum.A) and value. Iteration over members.

### R4 - auto() value generation

```yaml
id: R4
priority: medium
status: draft
```

Implement auto() that returns incrementing int values starting from 1. Used inside Enum class bodies.

### R5 - IntEnum and StrEnum

```yaml
id: R5
priority: low
status: draft
```

IntEnum members compare equal to their int values. StrEnum members compare equal to their string values. Implemented as Enum subclasses with type coercion.

### R6 - @dataclass decorator

```yaml
id: R6
priority: high
status: draft
```

Create dataclasses_mod.rs. @dataclass(cls) inspects class fields and auto-generates: __init__ (from field names/defaults), __repr__ (ClassName(field=value, ...)), __eq__ (field-by-field comparison). Support field() for defaults and metadata.

### R7 - Dataclass frozen and order

```yaml
id: R7
priority: low
status: draft
```

Support @dataclass(frozen=True) which makes instances immutable (setattr raises). @dataclass(order=True) generates __lt__, __le__, __gt__, __ge__.

## Acceptance Criteria

### Scenario: frozenset creation and membership

- **WHEN** fs = frozenset([1, 2, 3])
- **THEN** 2 in fs == True, len(fs) == 3

### Scenario: frozenset is immutable

- **GIVEN** fs = frozenset([1, 2])
- **WHEN** fs.add(3)
- **THEN** AttributeError raised

### Scenario: Enum member access

- **GIVEN** class Color(Enum): RED=1; GREEN=2
- **WHEN** Color.RED
- **THEN** Returns member with value 1

### Scenario: dataclass auto-generates init

- **GIVEN** @dataclass class Point: x: int; y: int
- **WHEN** p = Point(1, 2)
- **THEN** p.x == 1, p.y == 2, repr(p) == 'Point(x=1, y=2)'

## Diagrams

### Dataclass Decorator Flow

```mermaid
flowchart TB
    dec[@dataclass applied to class]
    inspect[Inspect class fields and annotations]
    init[Generate __init__ method]
    repr[Generate __repr__ method]
    eq[Generate __eq__ method]
    reg[Register modified class]
    dec --> inspect
    inspect --> init
    inspect --> repr
    inspect --> eq
    init --> reg
    repr --> reg
    eq --> reg
```

</spec>
