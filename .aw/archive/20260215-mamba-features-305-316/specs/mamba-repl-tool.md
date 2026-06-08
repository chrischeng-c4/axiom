---
id: mamba-repl-tool
type: spec
title: "REPL and Interactive Mode (#316)"
version: 1
spec_type: utility
created_at: 2026-02-14T09:32:30.802811+00:00
updated_at: 2026-02-14T09:32:30.802811+00:00
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
  - timestamp: 2026-02-14T09:32:30.802811+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# REPL and Interactive Mode (#316)

## Overview

This specification defines the Read-Eval-Print Loop (REPL) and interactive mode for Mamba. it covers the user interface for interactive command entry, incremental compilation using the JIT backend, and persistent state across REPL iterations.

## Requirements

### R1 - REPL Interface

```yaml
id: R1
priority: high
status: draft
```

Implement an interactive command-line interface for entering and executing Mamba code line-by-line.

### R2 - Incremental JIT Compilation

```yaml
id: R2
priority: high
status: draft
```

Use the Cranelift JIT backend to compile and execute snippets of code incrementally.

### R3 - Persistent Global State

```yaml
id: R3
priority: high
status: draft
```

Maintain global state (variables, classes, functions) across different REPL iterations.

### R4 - Multi-line Input Handling

```yaml
id: R4
priority: medium
status: draft
```

Support multi-line input for compound statements like class and def.

## Acceptance Criteria

### Scenario: Simple Variable Assignment

- **GIVEN** The REPL is running.
- **WHEN** The user enters 'x = 10'.
- **THEN** Entering 'x' should print 10.

### Scenario: Call Defined Function

- **GIVEN** A function 'def f(): return 42' defined in a previous iteration.
- **WHEN** The user enters 'f()'.
- **THEN** Entering 'f()' should print 42.

</spec>
