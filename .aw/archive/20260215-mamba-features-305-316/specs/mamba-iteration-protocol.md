---
id: mamba-iteration-protocol
type: spec
title: "For-loop Iteration Protocol (#311)"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-14T09:31:27.125633+00:00
updated_at: 2026-02-14T09:31:27.125633+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Iteration Protocol Flow"
history:
  - timestamp: 2026-02-14T09:31:27.125633+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# For-loop Iteration Protocol (#311)

## Overview

This specification defines the for-loop iteration protocol for Mamba, which matches the standard Python protocol involving __iter__ and __next__ methods. It includes the codegen for for-loops and the runtime handling of iterator objects and StopIteration exceptions.

## Requirements

### R1 - Obtain Iterator via __iter__

```yaml
id: R1
priority: high
status: draft
```

For-loops must call the `__iter__` method on the target object to obtain an iterator.

### R2 - Advance Iterator via __next__

```yaml
id: R2
priority: high
status: draft
```

The loop must repeatedly call `__next__` on the iterator until a `StopIteration` exception is raised.

### R3 - Built-in Iterators

```yaml
id: R3
priority: high
status: draft
```

Provide runtime support for built-in iterators for lists, dicts, and tuples.

## Acceptance Criteria

### Scenario: Iterate over List

- **GIVEN** A list [1, 2, 3].
- **WHEN** A for-loop iterates over the list.
- **THEN** The loop body should execute 3 times with values 1, 2, and 3.

### Scenario: Non-iterable Object Error

- **GIVEN** An object without an __iter__ method.
- **WHEN** A for-loop attempts to iterate over the object.
- **THEN** The runtime should raise a TypeError.

## Diagrams

### Iteration Protocol Flow

```mermaid
flowchart TB
    Start(For-loop Start)
    GetIter[Call __iter__ on object]
    CallNext[Call __next__ on iterator]
    CheckExhausted{StopIteration raised?} 
    ExecBody[Execute loop body with value]
    End(For-loop End)
    Start --> GetIter
    GetIter --> CallNext
    CallNext --> CheckExhausted
    CheckExhausted -->|No| ExecBody
    ExecBody --> CallNext
    CheckExhausted -->|Yes| End
```

</spec>
