---
id: taipan-jit-backend
type: spec
title: "Taipan JIT Backend and Symbol Wiring"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-13T07:27:24.904958+00:00
updated_at: 2026-02-13T07:27:24.904958+00:00
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
      title: "JIT Compilation Flow"
changes:
  - file: crates/cclab-taipan/src/codegen/cranelift/mod.rs
    action: MODIFY
    description: "Add JIT module support and symbol wiring logic."
  - file: crates/cclab-taipan/src/driver/config.rs
    action: MODIFY
    description: "Add Backend::CraneliftJit variant."
history:
  - timestamp: 2026-02-13T07:27:24.904958+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Taipan JIT Backend and Symbol Wiring

## Overview

This specification defines the transition of the Cranelift backend from a pure AOT ObjectModule approach to a dual-mode system that supports JIT execution via Cranelift's JITModule. It includes the logic for wiring runtime symbols (tp_*) into the JIT execution environment.

## Requirements

### R1 - JIT Module Initialization

```yaml
id: R1
priority: medium
status: draft
```

Initialize Cranelift JITModule with appropriate settings for the host platform.

### R2 - Runtime Symbol Wiring

```yaml
id: R2
priority: medium
status: draft
```

Map all runtime 'tp_*' functions to their physical memory addresses in the JIT symbol table.

### R3 - Callable Entry Point Logic

```yaml
id: R3
priority: medium
status: draft
```

Implement logic to finalize JIT compilation and retrieve a callable entry point address for compiled functions.

### R4 - Memory Management for Executable Code

```yaml
id: R4
priority: medium
status: draft
```

Ensure executable memory used by JIT functions is properly managed and freed during session shutdown.

## Acceptance Criteria

### Scenario: Successful JIT Module Init

- **GIVEN** A JIT-enabled Cranelift backend.
- **WHEN** The backend is instantiated with Backend::CraneliftJit.
- **THEN** The backend should successfully initialize a JITModule targeting the current host architecture.

### Scenario: Call Runtime Function from JIT

- **GIVEN** A compiled JIT module.
- **WHEN** A function calling 'tp_print' is executed in the JIT environment.
- **THEN** The JIT-compiled code should successfully resolve and call the runtime function at the correct address.

### Scenario: Safe Memory Cleanup

- **GIVEN** A finished JIT session.
- **WHEN** The JIT backend is dropped.
- **THEN** All executable memory regions associated with the session should be released without leaks.

## Diagrams

### JIT Compilation Flow

```mermaid
flowchart TB
    Start((Start JIT Backend))
    InitJIT[Initialize JITModule]
    WireSymbols[Wire tp_* Runtime Symbols]
    CompileMIR[Compile MIR to JIT Code]
    FinalizeJIT[Finalize JIT Memory]
    GetEntryPoint(Get entry point pointer)
    Start --> InitJIT
    InitJIT --> WireSymbols
    WireSymbols --> CompileMIR
    CompileMIR --> FinalizeJIT
    FinalizeJIT --> GetEntryPoint
```

</spec>
