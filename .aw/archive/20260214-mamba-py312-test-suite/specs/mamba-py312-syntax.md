---
id: mamba-py312-syntax
type: spec
title: "Python 3.12 Syntax Support"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-13T10:36:17.352154+00:00
updated_at: 2026-02-13T10:36:17.352154+00:00
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
      title: "Optional Type Parameters Parsing Flow"
history:
  - timestamp: 2026-02-13T10:36:17.352154+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Python 3.12 Syntax Support

## Overview

This specification defines the Python 3.12 syntax support for the Mamba compiler, focusing on PEP 695 (Type Parameter Syntax) and PEP 701 (f-strings). It details the parser changes required to handle generic function, class, and type alias definitions.

## Requirements

### R1 - Generic Function Definitions

```yaml
id: R1
priority: high
status: draft
```

The parser must support the new square bracket syntax for generic type parameters in function definitions: 'def func[T](...)'.

### R2 - Generic Class Definitions

```yaml
id: R2
priority: high
status: draft
```

The parser must support generic type parameters in class definitions: 'class Class[T]:'.

### R3 - Type Alias Statements

```yaml
id: R3
priority: high
status: draft
```

The parser must support the 'type' keyword for type alias definitions with optional generic parameters: 'type Alias[T] = List[T]'.

### R4 - PEP 701 f-strings

```yaml
id: R4
priority: medium
status: draft
```

The lexer and parser must support the improved f-string syntax (PEP 701), allowing nested f-strings, comments, and multi-line expressions within f-string interpolations.

## Acceptance Criteria

### Scenario: Parse Generic Function

- **WHEN** Input 'def f[T](x: T) -> T: pass' is parsed.
- **THEN** The parser should successfully produce a FnDef node containing 'T' as a type parameter.

### Scenario: Parse Generic Class

- **WHEN** Input 'class Stack[T]: pass' is parsed.
- **THEN** The parser should produce a ClassDef node with 'T' in type_params.

### Scenario: Parse Type Alias

- **WHEN** Input 'type Points[T] = List[T]' is parsed.
- **THEN** The parser should produce a TypeAlias node with 'T' in type_params and 'List[T]' as the value.

### Scenario: Parse Nested f-string (PEP 701)

- **WHEN** Input "f'outer {f\"inner {x}\"}'" is parsed.
- **THEN** The parser should correctly handle the nested interpolation.

## Diagrams

### Optional Type Parameters Parsing Flow

```mermaid
flowchart TB
    Start((Start parse_optional_type_params))
    PeekLBracket{Peek [?} 
    AdvanceLBracket[Advance ([)]
    ReturnEmpty([Return Empty Vec])
    ExpectIdent(Expect Ident)
    PushParam[Push to Params]
    PeekComma{Peek Comma?} 
    AdvanceComma[Advance (Comma)]
    ExpectRBracket(Expect ])
    ReturnParams([Return Params Vec])
    Start --> PeekLBracket
    PeekLBracket -->|Yes (LBracket found)| AdvanceLBracket
    PeekLBracket -->|No (LBracket not found)| ReturnEmpty
    AdvanceLBracket --> ExpectIdent
    ExpectIdent --> PushParam
    PushParam --> PeekComma
    PeekComma -->|Yes (Comma found)| AdvanceComma
    PeekComma -->|No (Comma not found)| ExpectRBracket
    AdvanceComma --> ExpectIdent
    ExpectRBracket --> ReturnParams
```

</spec>
