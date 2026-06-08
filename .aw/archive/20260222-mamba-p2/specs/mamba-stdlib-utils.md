---
id: mamba-stdlib-utils
type: spec
title: "Stdlib: contextlib, copy, operator, weakref"
version: 1
spec_type: utility
created_at: 2026-02-22T11:21:34.605630+00:00
updated_at: 2026-02-22T11:21:34.605630+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:21:34.605630+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: contextlib, copy, operator, weakref

## Overview

Implement four utility stdlib modules for Mamba: contextlib (context manager helpers: contextmanager decorator, suppress, nullcontext), copy (shallow/deep copy), operator (function versions of operators), and weakref (weak references with GC integration).

## Requirements

### R1 - contextlib module

```yaml
id: R1
priority: medium
status: draft
```

Create contextlib_mod.rs. mb_contextlib_suppress(exceptions): context manager that suppresses listed exception types. mb_contextlib_nullcontext(value): no-op context manager returning value. mb_contextlib_contextmanager: decorator that wraps generator as context manager (depends on context manager protocol #385).

### R2 - copy module

```yaml
id: R2
priority: high
status: draft
```

Create copy_mod.rs. mb_copy_copy(obj): shallow copy — clone top-level object, share nested refs. mb_copy_deepcopy(obj): recursive clone. Handle List, Dict, Set, Tuple, Instance. Support __copy__/__deepcopy__ dunder protocols.

### R3 - operator module

```yaml
id: R3
priority: medium
status: draft
```

Create operator_mod.rs. Thin wrappers: mb_operator_add, sub, mul, truediv, floordiv, mod_, pow, neg, pos, abs, eq, ne, lt, le, gt, ge, not_, and_, or_, xor. Also: mb_operator_itemgetter(key), mb_operator_attrgetter(attr).

### R4 - weakref module

```yaml
id: R4
priority: low
status: draft
```

Create weakref_mod.rs. mb_weakref_ref(obj): create weak reference that doesn't prevent GC collection. mb_weakref_deref(ref): dereference, return object or None if collected. Requires GC integration: weak refs are invalidated during sweep.

## Acceptance Criteria

### Scenario: copy shallow copies list

- **GIVEN** a = [[1, 2], [3, 4]]
- **WHEN** b = copy.copy(a); b[0].append(5)
- **THEN** a[0] == [1, 2, 5] (shared inner)

### Scenario: deepcopy is independent

- **GIVEN** a = [[1, 2], [3, 4]]
- **WHEN** b = copy.deepcopy(a); b[0].append(5)
- **THEN** a[0] == [1, 2] (independent)

### Scenario: suppress catches exception

- **WHEN** with suppress(ValueError): int('abc')
- **THEN** No exception propagated

### Scenario: operator.add wraps +

- **WHEN** operator.add(1, 2)
- **THEN** Returns 3

</spec>
