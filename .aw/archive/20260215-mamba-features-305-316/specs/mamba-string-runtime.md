---
id: mamba-string-runtime
type: spec
title: "String Operations and f-string Interpolation (#312)"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-14T09:31:18.907644+00:00
updated_at: 2026-02-14T09:31:18.907644+00:00
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
      title: "f-string Interpolation Flow"
history:
  - timestamp: 2026-02-14T09:31:18.907644+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# String Operations and f-string Interpolation (#312)

## Overview

This specification defines the runtime support and codegen for string operations and PEP 701 f-string interpolation. It covers the logic for parsing nested expressions within f-strings and the runtime routines for string formatting and manipulation.

## Requirements

### R1 - f-string Syntax Support (PEP 701)

```yaml
id: R1
priority: high
status: draft
```

Support lexing and parsing of PEP 701 f-strings, including nested f-strings and multi-line expressions.

### R2 - Runtime String Formatting

```yaml
id: R2
priority: high
status: draft
```

Provide a runtime routine `mb_string_format` that handles the efficient construction of strings from literal and expression parts.

### R3 - String Operations/Methods

```yaml
id: R3
priority: medium
status: draft
```

Implement common string methods (e.g., upper, lower, split, join) as runtime functions.

## Acceptance Criteria

### Scenario: Nested f-string interpolation

- **GIVEN** A nested f-string literal: f'a {f\"b {x}\"}'
- **WHEN** The f-string is executed.
- **THEN** The runtime should correctly evaluate the expression and produce 'a b value_of_x'.

### Scenario: String Concatenation

- **GIVEN** Two strings s1 and s2.
- **WHEN** s1 + s2 is executed.
- **THEN** The runtime should produce a new string object containing the combined content.

## Diagrams

### f-string Interpolation Flow

```mermaid
flowchart TB
    Start(f-string literal encountered)
    LexFString[Lex f-string parts (literal vs expressions)]
    ParseInterpolations[Parse expressions inside interpolations]
    LowerToFormat[Lower to 'mb_string_format' call with args]
    RuntimeJoin[Runtime: Join parts and convert to string]
    End(Final string object created)
    Start --> LexFString
    LexFString --> ParseInterpolations
    ParseInterpolations --> LowerToFormat
    LowerToFormat --> RuntimeJoin
    RuntimeJoin --> End
```

</spec>
