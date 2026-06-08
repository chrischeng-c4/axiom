---
id: taipan-syntax
type: spec
title: "Taipan Language Syntax"
version: 1
spec_type: algorithm
created_at: 2026-02-12T07:45:22.244805+00:00
updated_at: 2026-02-12T07:45:22.244805+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Taipan Parsing Logic"
history:
  - timestamp: 2026-02-12T07:45:22.244805+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Taipan Language Syntax

## Overview

This specification defines the syntax and grammar of the Taipan programming language for version 0.1. It includes the definition of keywords, operators, and the structural rules for statements and expressions.

## Requirements

### R1 - Reserved Keywords

```yaml
id: R1
priority: high
status: draft
```

Define the set of reserved keywords: func, return, let, if, else, int, float.

### R2 - Numeric Literals

```yaml
id: R2
priority: high
status: draft
```

Support numeric literals including signed integers and 64-bit floating point numbers.

### R3 - Arithmetic Operators

```yaml
id: R3
priority: high
status: draft
```

Implement standard arithmetic operators (+, -, *, /) with standard precedence rules.

### R4 - Function Definitions

```yaml
id: R4
priority: high
status: draft
```

Define the syntax for function declarations including parameters, return types, and braced blocks.

### R5 - Statement Termination

```yaml
id: R5
priority: medium
status: draft
```

Require mandatory semicolons at the end of let statements and expression statements.

## Acceptance Criteria

### Scenario: Valid Function Definition

- **WHEN** Parsing 'func add(a: int, b: int) -> int { return a + b; }'
- **THEN** The parser should produce a FuncDef AST node with correct parameters and body.

### Scenario: Basic Arithmetic Expression

- **WHEN** Parsing '1 + 2;'
- **THEN** The parser should produce a BinaryExpr(+, Literal(1), Literal(2)) node.

### Scenario: Missing Semicolon Error

- **WHEN** Parsing 'let x = 10'
- **THEN** The parser should return a syntax error indicating the expected semicolon.

## Diagrams

### Taipan Parsing Logic

```mermaid
flowchart TB
    Source[Source String]
    Tokens([Token Stream])
    ASTNode{AST Node (Enum)} 
    LetStmt(Let Statement)
    FuncDef(Function Definition)
    BinaryExpr(Binary Expression)
    Literal(Literal (Int/Float))
    Source -->|Lexing| Tokens
    Tokens -->|Parsing| ASTNode
    ASTNode -->|Statement| LetStmt
    ASTNode -->|Statement| FuncDef
    ASTNode -->|Expression| BinaryExpr
    ASTNode -->|Expression| Literal
```

</spec>
