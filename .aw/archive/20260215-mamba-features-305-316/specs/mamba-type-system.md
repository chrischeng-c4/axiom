---
id: mamba-type-system
type: spec
title: "Generics and Protocol Types (#314)"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-14T09:31:54.344953+00:00
updated_at: 2026-02-14T09:31:54.344953+00:00
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
      title: "Generic and Protocol Type Checking Flow"
history:
  - timestamp: 2026-02-14T09:31:54.344953+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Generics and Protocol Types (#314)

## Overview

This specification defines the Mamba type system enhancements for Generics (PEP 695) and Protocol types (structural subtyping). It covers how the compiler tracks type parameters and verifies that objects satisfy specific interface requirements (Protocols) regardless of their inheritance hierarchy.

## Requirements

### R1 - PEP 695 Generics Support

```yaml
id: R1
priority: high
status: draft
```

Implement support for generic classes and functions using the square bracket syntax (e.g., List[T], def f[T]).

### R2 - Protocol Type Verification

```yaml
id: R2
priority: high
status: draft
```

Provide a mechanism for structural subtyping (Protocols) where an object is considered a subtype of a protocol if it implements all required methods.

### R3 - Generic Type Resolution

```yaml
id: R3
priority: high
status: draft
```

Correctly resolve and substitute generic type parameters during type checking and MIR lowering.

## Acceptance Criteria

### Scenario: Protocol Matching

- **GIVEN** A protocol 'Drawable' with a 'draw' method, and a class 'Circle' that implements 'draw' but does NOT inherit from 'Drawable'.
- **WHEN** A Circle instance is passed to a function taking Drawable.
- **THEN** The type checker should accept Circle where Drawable is expected.

### Scenario: Generic Type Constraint

- **GIVEN** A generic class Box[T] and a variable typed as Box[int].
- **WHEN** A string is added to the Box[int].
- **THEN** The type checker should raise an error.

## Diagrams

### Generic and Protocol Type Checking Flow

```mermaid
flowchart TB
    Start(Type Checking Generic Usage)
    IdentifyGenerics[Identify Generic Classes/Functions]
    SubstituteTypeParams[Substitute Concrete Types for TypeParams]
    CheckProtocolMatch[Verify Protocol (Structural) Matching]
    End(Type Checking Complete)
    Start --> IdentifyGenerics
    IdentifyGenerics --> SubstituteTypeParams
    SubstituteTypeParams --> CheckProtocolMatch
    CheckProtocolMatch --> End
```

</spec>
