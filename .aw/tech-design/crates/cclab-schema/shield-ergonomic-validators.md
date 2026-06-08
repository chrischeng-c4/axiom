---
id: shield-ergonomic-validators
type: spec
title: "Shield Ergonomic Validators"
version: 1
spec_type: algorithm
created_at: 2026-01-28T07:42:01.008364+00:00
updated_at: 2026-01-28T07:42:01.008364+00:00
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
      title: "Validator Execution Order"
history:
  - timestamp: 2026-01-28T07:42:01.008364+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Shield Ergonomic Validators

## Overview

This specification defines the ergonomic decorator-based validation system for cclab-shield, allowing users to define custom validation logic in Python that runs alongside the Rust core engine.

## Requirements

### R1 - Field Validators

```yaml
id: R1
priority: medium
status: draft
```

Users can define field-level validators using the @field_validator decorator.

### R2 - Model Validators

```yaml
id: R2
priority: medium
status: draft
```

Users can define model-level validators using the @model_validator decorator.

### R3 - Validator Modes

```yaml
id: R3
priority: medium
status: draft
```

Validators should support different modes: before (raw input) and after (typed value).

### R4 - Cross-field Access

```yaml
id: R4
priority: medium
status: draft
```

Validators must receive a context object allowing access to other validated fields.

## Acceptance Criteria

### Scenario: Happy path field validator

- **GIVEN** A validator for field age that checks if age < 150.
- **WHEN** Validating a model with age=30.
- **THEN** The value 30 should pass.

### Scenario: Root validator for cross-field validation

- **GIVEN** A model validator that checks if password == confirm_password.
- **WHEN** Validating a model where passwords do not match.
- **THEN** A ValidationError should be raised for the confirm_password field.

### Scenario: Mode before validator

- **GIVEN** A before validator that strips whitespace from a string before it's passed to Rust.
- **WHEN** Input data has space-padded string.
- **THEN** Rust should receive trimmed and validation should pass.

### Scenario: Context access for missing field

- **GIVEN** A validator that attempts to access a field that was not provided in input.
- **WHEN** Validating a partial model.
- **THEN** The validator should handle None or missing field gracefully.

## Diagrams

### Validator Execution Order

```mermaid
flowchart TB
    init[Input Data]
    mode_before[Before Validators]
    rust_val[Rust Core Validation]
    mode_after_field[After Field Validators]
    mode_after_model[After Model Validators]
    done[Validated Instance]
    init --> mode_before
    mode_before -->|Run @field_validator(mode=before)| rust_val
    rust_val -->|Type Coercion & Constraints| mode_after_field
    mode_after_field -->|Run @field_validator(mode=after)| mode_after_model
    mode_after_model -->|Run @model_validator(mode=after)| done
```

</spec>
