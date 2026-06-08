---
id: migration-architecture
type: spec
title: "Migration Architecture & Compatibility Matrix"
version: 1
spec_type: algorithm
tags: [logic]
spec_group: cclab-genesis
created_at: 2026-02-14T17:25:38.608119+00:00
updated_at: 2026-02-14T17:25:38.608119+00:00
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
      title: "Migration Logic Flow"
history:
  - timestamp: 2026-02-14T17:25:38.608119+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Migration Architecture & Compatibility Matrix

## Overview

Defines the migration strategy, compatibility matrix, and deprecation plan for moving from Aurora relay to YAML IR. Ensures dual-path support during transition and safe deprecation of legacy tools.

## Requirements

### R1 - Legacy Path Detection

```yaml
id: R1
priority: medium
status: draft
```

Identify if a change is using the legacy Aurora relay flow or the new YAML IR flow. New changes default to YAML IR. Legacy changes are detected by the absence of `spec_ir/` directory or explicit configuration.

### R2 - YAML Path Enforcement

```yaml
id: R2
priority: medium
status: draft
```

Enforce YAML IR flow for all new changes. Spec creation tools must generate YAML IR. Implementation tools must prefer YAML IR if present.

### R3 - Deprecation Warnings

```yaml
id: R3
priority: medium
status: draft
```

Emit deprecation warnings when legacy Aurora spec generation tools are used. Warning should include link to migration guide.

### R4 - Dual-Path Support

```yaml
id: R4
priority: medium
status: draft
```

Support both legacy and new flows simultaneously in the codebase, but scoped per-change. A single change cannot mix flows.

## Acceptance Criteria

### Scenario: Legacy Change Compatibility

- **WHEN** A change with no `spec_ir/` directory is processed
- **THEN** The system uses legacy Aurora relay pipeline and emits a deprecation warning

### Scenario: New Change Default

- **WHEN** A new change is created via genesis_create_spec
- **THEN** The system uses the new YAML IR pipeline

### Scenario: Mixed Flow Rejection

- **WHEN** A change attempts to use legacy tools while `spec_ir/` exists
- **THEN** The system returns an error stating mixing flows is not supported

## Diagrams

### Migration Logic Flow

```mermaid
flowchart TB
    Start((Start))
    CheckIR{Has spec_ir dir?} 
    UseNew[Use YAML Pipeline]
    CheckLegacy{Is Legacy?} 
    Warn[Emit Warning]
    UseLegacy[Use Aurora Relay]
    Error[Error: Invalid State]
    Start --> CheckIR
    CheckIR -->|Yes| UseNew
    CheckIR -->|No| CheckLegacy
    CheckLegacy -->|Yes| Warn
    Warn --> UseLegacy
    CheckLegacy -->|No| Error
```

</spec>
