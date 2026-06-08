---
id: titan-shield-integration
type: spec
title: "Titan-Shield Integration"
version: 1
spec_type: utility
created_at: 2026-02-02T06:44:38.666566+00:00
updated_at: 2026-02-02T06:44:38.666566+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-02T06:44:38.666566+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Titan-Shield Integration

## Overview

This spec defines the integration of `cclab-shield` into `cclab-titan` to replace the duplicated `pydantic_validation` module. This aligns `titan` with the ecosystem's validation strategy, reducing code duplication and ensuring consistent validation behavior.

## Requirements

### R1 - Add Shield Dependency

```yaml
id: R1
priority: medium
status: draft
```

Add `cclab-shield` as a workspace dependency in `crates/cclab-titan/Cargo.toml`.

### R2 - Remove Duplicated Code

```yaml
id: R2
priority: medium
status: draft
```

Remove `crates/cclab-titan/src/pydantic_validation.rs` to eliminate code duplication.

### R3 - Re-export Shield Types

```yaml
id: R3
priority: medium
status: draft
```

Update `crates/cclab-titan/src/lib.rs` to re-export validation types (`ValidationError`, `ValidationErrors`, `FieldValidator`, etc.) from `cclab-shield` to maintain API availability.

## Acceptance Criteria

### Scenario: Compilation Check

- **WHEN** `cargo build -p cclab-titan` is executed
- **THEN** The crate compiles successfully without `pydantic_validation.rs`.

### Scenario: API Availability

- **WHEN** Importing `cclab_titan::ValidationError` in client code
- **THEN** `cclab_titan::ValidationError` is resolved to `cclab_shield::ValidationError`.

### Scenario: Dependency Verification

- **WHEN** Checking `Cargo.toml` and build graph
- **THEN** `cclab-shield` is listed in `dependencies` and linked correctly.

</spec>
