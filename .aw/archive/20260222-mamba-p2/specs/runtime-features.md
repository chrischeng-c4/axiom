---
id: runtime-features
type: spec
title: "Runtime Features"
version: 1
spec_type: utility
created_at: 2026-02-22T11:23:10.366102+00:00
updated_at: 2026-02-22T11:23:10.366102+00:00
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
history:
  - timestamp: 2026-02-22T11:23:10.366102+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Runtime Features

## Overview

Implement key runtime features required for Python compatibility, including context managers, unpacking, exception groups, and object model enhancements.

## Requirements

### R1 - Context Managers

```yaml
id: R1
priority: medium
status: draft
```

Support the `with` statement and the context manager protocol (`__enter__`, `__exit__`).

### R2 - Unpacking

```yaml
id: R2
priority: medium
status: draft
```

Support iterable unpacking (`*args`) and dictionary unpacking (`**kwargs`) in function calls and literals.

### R3 - Exception Groups

```yaml
id: R3
priority: medium
status: draft
```

Support `except*` syntax and `ExceptionGroup` for handling multiple exceptions (PEP 654).

### R4 - Slots

```yaml
id: R4
priority: medium
status: draft
```

Support `__slots__` in class definitions to restrict attribute creation.

### R5 - Finalizers

```yaml
id: R5
priority: medium
status: draft
```

Support the `__del__` finalizer method for object cleanup.

### R6 - Format Protocol

```yaml
id: R6
priority: medium
status: draft
```

Support the `__format__` protocol and f-string debug syntax.

## Acceptance Criteria

### Scenario: With statement execution

- **GIVEN** A class with __enter__ and __exit__
- **WHEN** I use it in a with statement
- **THEN** __enter__ runs before body, __exit__ runs after.

### Scenario: List unpacking

- **GIVEN** A list [1, 2]
- **WHEN** I call f(*[1, 2])
- **THEN** The function receives 1 and 2 as separate arguments.

### Scenario: Slots restriction

- **GIVEN** A class with __slots__ = ['x']
- **WHEN** I try to assign to y
- **THEN** It raises AttributeError.

</spec>
