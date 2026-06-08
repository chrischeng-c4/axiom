---
id: spec-validator
type: spec
title: "Spec Completeness Validator"
version: 1
spec_type: algorithm
created_at: 2026-02-02T13:49:55.191928+00:00
updated_at: 2026-02-02T13:49:55.191928+00:00
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
      title: "Spec Validation Logic"
history:
  - timestamp: 2026-02-02T13:49:55.191928+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Spec Completeness Validator

## Overview

Defines the validation logic for JSON Schemas within the Aurora Codegen System. It ensures that schemas are structurally sound, have valid references, and contain sufficient metadata (like descriptions) for high-quality code generation.

## Requirements

### R1 - Type Validation

```yaml
id: R1
priority: medium
status: draft
```

The validator must check that all properties have a defined type (or $ref).

### R2 - Reference Validation

```yaml
id: R2
priority: medium
status: draft
```

The validator must ensure all $ref pointers resolve to existing definitions.

### R3 - Completeness Check

```yaml
id: R3
priority: medium
status: draft
```

The validator should warn if descriptions are missing for public fields.

## Acceptance Criteria

### Scenario: Missing Type

- **GIVEN** A schema with a missing type for 'age'
- **WHEN** The validator is run
- **THEN** The validator returns an error indicating 'age' has no type

### Scenario: Broken Reference

- **GIVEN** A schema with a $ref to '#/definitions/Unknown'
- **WHEN** The validator is run
- **THEN** The validator returns an error for the broken reference

## Diagrams

### Spec Validation Logic

```mermaid
flowchart TB
    Start((Start))
    CheckStructure{Check Structure (Types)} 
    CheckRefs{Check References ($ref)} 
    CheckCompleteness{Check Completeness (Desc)} 
    Success(Validation Passed)
    Error[Validation Error]
    Warning[Validation Warning]
    Start -->|Input Schema| CheckStructure
    CheckStructure -->|Valid| CheckRefs
    CheckStructure -->|Invalid| Error
    CheckRefs -->|All Found| CheckCompleteness
    CheckRefs -->|Missing Ref| Error
    CheckCompleteness -->|Complete| Success
    CheckCompleteness -->|Missing Fields| Warning
```

</spec>
