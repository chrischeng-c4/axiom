---
id: generator-decoupling
type: spec
title: "Generator Decoupling and Legacy Removal"
version: 1
spec_type: integration
tags: [external]
spec_group: sdd
created_at: 2026-02-15T03:47:49.406174+00:00
updated_at: 2026-02-15T03:47:49.406174+00:00
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
    - type: sequence
      title: "Decoupled Generation Flow"
depends:
  - mcp-router-unification
changes:
  - file: crates/cclab-sdd/src/fillback/mod.rs
    action: MODIFY
  - file: crates/cclab-sdd/src/fillback/relay.rs
    action: DELETE
history:
  - timestamp: 2026-02-15T03:47:49.406174+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Generator Decoupling and Legacy Removal

## Overview

This spec enforces the 'Agnostic SpecIR pipeline' pattern by refactoring merged Aurora generators to consume YAML IR directly, removing the legacy `call_aurora_tool` relay logic. This ensures decoupling from the prompt/LLM layer and improves testability.

## Requirements

### R1 - Direct IR Consumption

```yaml
id: R1
priority: medium
status: draft
```

Refactor all generators to accept parsed `SpecIR` structs (from YAML) instead of raw JSON/LLM output.

### R2 - Remove Relay Logic

```yaml
id: R2
priority: medium
status: draft
```

Remove the `call_aurora_tool` function and related relay infrastructure that routed calls to the external Aurora crate.

### R3 - IR Validation

```yaml
id: R3
priority: medium
status: draft
```

Implement strict validation for input YAML IR before generation proceeds.

### R4 - Update Tests

```yaml
id: R4
priority: medium
status: draft
```

Update unit tests to use static YAML fixtures instead of mocked tool calls.

## Acceptance Criteria

### Scenario: Generate from YAML

- **WHEN** invoking a generator with a valid YAML IR file
- **THEN** the corresponding code files are written to disk without invoking external tools

### Scenario: Handle Invalid IR

- **WHEN** invoking a generator with malformed YAML
- **THEN** a validation error is returned gracefully

## Diagrams

### Decoupled Generation Flow

```mermaid
sequenceDiagram
    participant Orchestrator as Orchestrator
    participant Generator as Code Generator
    participant Parser as YAML Parser
    participant FileSystem as File System
    Orchestrator->>Generator: generate(ir_path)
    Generator->>Parser: parse(ir_path)
    Parser->>Generator: return SpecIR
    Generator->>FileSystem: write_files()
    Generator->>Orchestrator: return success
```

</spec>
