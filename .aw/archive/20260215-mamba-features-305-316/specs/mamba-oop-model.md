---
id: mamba-oop-model
type: spec
title: "Complete OOP Model: Inheritance, super(), and Dunder Methods (#307)"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-14T09:31:45.876005+00:00
updated_at: 2026-02-14T09:31:45.876005+00:00
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
      title: "OOP Attribute Lookup (MRO) Flow"
history:
  - timestamp: 2026-02-14T09:31:45.876005+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Complete OOP Model: Inheritance, super(), and Dunder Methods (#307)

## Overview

This specification defines the complete Object-Oriented Programming (OOP) model for Mamba, including single and multiple inheritance, the C3 Method Resolution Order (MRO), instance creation, and standard magic methods (dunder methods). It also covers the implementation of super() for dynamic method dispatch.

## Requirements

### R1 - C3 Method Resolution Order

```yaml
id: R1
priority: high
status: draft
```

Implement C3 linearization to compute a stable and consistent Method Resolution Order for all classes.

### R2 - super() Support

```yaml
id: R2
priority: high
status: draft
```

Provide a built-in `super()` function that correctly identifies the next class in the MRO relative to the current method's class and instance.

### R3 - Magic Method Dispatch (Operator Overloading)

```yaml
id: R3
priority: high
status: draft
```

Support operator overloading by dispatching binary and unary operations to the appropriate dunder methods (e.g., __add__, __neg__).

### R4 - Attribute Access Model

```yaml
id: R4
priority: high
status: draft
```

Manage class-level and instance-level attribute access via `__getattr__`, `__setattr__`, and `__delattr__` equivalent logic in the runtime.

## Acceptance Criteria

### Scenario: super() Dispatch

- **GIVEN** Class B inherits from Class A, both implementing 'speak'.
- **WHEN** B.speak() calls super().speak().
- **THEN** The result should be 'A's version of speak' concatenated with 'B's version'.

### Scenario: Operator Overloading Dispatch

- **GIVEN** An instance X of Class C with __add__ implemented.
- **WHEN** X + 5 is executed.
- **THEN** The __add__ method of C should be called.

## Diagrams

### OOP Attribute Lookup (MRO) Flow

```mermaid
flowchart TB
    Start(Attribute Access Start)
    CheckLocal{Check Instance __dict__} 
    WalkMRO[Get Next Class in MRO]
    CheckClassLocal{Check Class __dict__} 
    RaiseError[Raise AttributeError]
    End(Return Value)
    Start --> CheckLocal
    CheckLocal -->|Found| End
    CheckLocal -->|Not Found| WalkMRO
    WalkMRO --> CheckClassLocal
    CheckClassLocal -->|Found| End
    CheckClassLocal -->|Next Base| WalkMRO
    CheckClassLocal -->|End of MRO| RaiseError
    RaiseError --> End
```

</spec>
