---
id: aurora-codegen-system
type: spec
title: "Taipan Compiler Core Architecture"
version: 1
spec_type: algorithm
main_spec_ref: cclab-aurora/aurora-codegen-system
merge_strategy: extend
created_at: 2026-02-12T07:44:50.954575+00:00
updated_at: 2026-02-12T07:44:50.954575+00:00
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
      title: "Taipan Compiler Pipeline Data Flow"
history:
  - timestamp: 2026-02-12T07:44:50.954575+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Taipan Compiler Core Architecture

## Overview

This specification defines the Taipan compiler core using the Aurora architectural pattern. It covers the transformation from source code to a unified intermediate representation (IR) and the subsequent generation of machine code via pluggable backends.

## Requirements

### R1 - Lexical Analysis and Parsing

```yaml
id: R1
priority: high
status: draft
```

Implement a parser to convert Taipan source code into an Abstract Syntax Tree (AST).

### R2 - Unified Internal Representation (IR)

```yaml
id: R2
priority: high
status: draft
```

Lower the AST into a Unified Intermediate Representation (Taipan IR) designed for optimization and codegen.

### R3 - Semantic Analysis Pipeline

```yaml
id: R3
priority: high
status: draft
```

Perform type checking and scope validation on the AST before IR lowering.

### R4 - Pluggable Backend System

```yaml
id: R4
priority: high
status: draft
```

Define a Backend trait to allow multiple targets (Cranelift, LLVM, WASM) while defaulting to Cranelift.

### R5 - Cranelift Implementation

```yaml
id: R5
priority: high
status: draft
```

Implement the Cranelift backend for v0.1 to produce native x86_64/AArch64 machine code.

## Acceptance Criteria

### Scenario: Parse Valid Source

- **WHEN** A syntactically correct Taipan program is parsed.
- **THEN** A valid AST should be generated representing the source program.

### Scenario: IR Lowering

- **WHEN** Semantic analysis passes and IR lowering is invoked.
- **THEN** The AST should be successfully lowered into Taipan IR in Static Single Assignment (SSA) form.

### Scenario: Generate Native Code

- **WHEN** The Taipan IR is passed to the Cranelift backend.
- **THEN** The Cranelift backend should emit a valid native binary for the target architecture.

## Diagrams

### Taipan Compiler Pipeline Data Flow

```mermaid
flowchart LR
    Source[Taipan Source (.tp)]
    AST(Abstract Syntax Tree)
    TypedAST(Typed AST)
    TaipanIR([Taipan IR (SSA)])
    BackendChoice{Backend (Cranelift)} 
    Binary[(Native Binary)]
    Source -->|Lexing/Parsing| AST
    AST -->|Semantic Analysis| TypedAST
    TypedAST -->|Lowering| TaipanIR
    TaipanIR -->|Codegen| BackendChoice
    BackendChoice -->|Emit| Binary
```

</spec>
