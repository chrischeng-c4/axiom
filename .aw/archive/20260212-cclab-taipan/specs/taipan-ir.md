---
id: taipan-ir
type: spec
title: "Taipan Intermediate Representation (IR)"
version: 1
spec_type: algorithm
created_at: 2026-02-12T07:46:08.085322+00:00
updated_at: 2026-02-12T07:46:08.085322+00:00
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
    - type: class
      title: "Taipan IR Class Diagram"
    - type: flowchart
      title: "Lowering Process (TypedAST -> IR)"
history:
  - timestamp: 2026-02-12T07:46:08.085322+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Taipan Intermediate Representation (IR)

## Overview

This specification defines the Intermediate Representation (IR) used by the Taipan compiler. The IR is structured in Static Single Assignment (SSA) form and acts as the bridge between the high-level Typed AST and the low-level machine code backends.

## Requirements

### R1 - IR Hierarchy Structure

```yaml
id: R1
priority: high
status: draft
```

Define the IR hierarchy: Module contains Functions, which contain BasicBlocks, which contain Instructions.

### R2 - Static Single Assignment (SSA)

```yaml
id: R2
priority: high
status: draft
```

Every instruction that produces a value must assign it to a unique, immutable virtual register (SSA).

### R3 - Instruction Set Architecture (ISA) Core

```yaml
id: R3
priority: high
status: draft
```

Support essential opcodes: Add, Sub, Mul, Div, LoadConst, Call, Return.

### R4 - Typed IR Representation

```yaml
id: R4
priority: high
status: draft
```

Each instruction and value in the IR must be explicitly typed (Int, Float).

### R5 - Lowering Algorithm

```yaml
id: R5
priority: high
status: draft
```

Provide a transformation algorithm to convert the Typed AST into the Taipan IR.

## Acceptance Criteria

### Scenario: Lower Simple Addition

- **WHEN** Lowering the expression '1 + 2'
- **THEN** The IR should contain a LoadConst for '1', a LoadConst for '2', and an Add instruction using their results.

### Scenario: Lower Function Call

- **WHEN** Lowering the expression 'print(x)'
- **THEN** The IR should contain a Call instruction with the target function name and argument values.

### Scenario: Maintain SSA Invariants

- **WHEN** Lowering complex nested expressions.
- **THEN** Each virtual register should be defined exactly once.

## Diagrams

### Taipan IR Class Diagram

```mermaid
classDiagram
    class Module {
        +String name
    }
    class Function {
        +String name
        +Vec<Type> params
        +Type return_type
    }
    class BasicBlock {
        +u32 id
    }
    class Instruction {
        +OpCode op
        +Type result_type
    }
    Module *-- Function
    Function *-- BasicBlock
    BasicBlock *-- Instruction
```

### Lowering Process (TypedAST -> IR)

```mermaid
flowchart TB
    ASTNode{Typed AST Node} 
    LowerStmt(Lower Statement)
    LowerExpr(Lower Expression)
    BasicBlockBuilder([Basic Block Builder])
    IRModule[(Taipan IR Module)]
    ASTNode -->|Dispatch| LowerStmt
    ASTNode -->|Dispatch| LowerExpr
    LowerStmt -->|Create Instruction| BasicBlockBuilder
    LowerExpr -->|Create Value/Instruction| BasicBlockBuilder
    BasicBlockBuilder -->|Append| IRModule
```

</spec>
