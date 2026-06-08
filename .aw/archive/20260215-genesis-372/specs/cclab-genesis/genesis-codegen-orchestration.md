---
id: genesis-codegen-orchestration
type: spec
title: "Genesis Codegen Orchestration"
version: 1
spec_type: workflow
tags: [state, logic]
spec_group: cclab-genesis
created_at: 2026-02-14T17:27:19.968801+00:00
updated_at: 2026-02-14T17:27:19.968801+00:00
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
  has_api_spec: true
  has_semantic_diagrams: false
  api_spec_type: serverless-workflow-0.8
  diagrams:
    - type: flowchart
      title: "Implementation Orchestration"
history:
  - timestamp: 2026-02-14T17:27:19.968801+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Genesis Codegen Orchestration

## Overview

Orchestrates the implementation phase to preferentially use the new YAML-based SpecIR pipeline. It detects the presence of YAML manifests and invokes Prism accordingly, while respecting the migration strategy for legacy fallback.

## Requirements

### R1 - YAML Detection

```yaml
id: R1
priority: medium
status: draft
```

The orchestrator must check for the existence of `spec_ir/*.yaml` files for the current change before deciding on the implementation strategy.

### R2 - Prism Invocation

```yaml
id: R2
priority: medium
status: draft
```

If YAML IR files are present, the orchestrator must invoke Prism's code generation interface passing the paths to these files.

### R3 - Fallback Logic

```yaml
id: R3
priority: medium
status: draft
```

If no YAML IR is found, the system must consult the `migration-architecture` configuration to determine if legacy fallback is allowed or if an error should be raised.

## Acceptance Criteria

### Scenario: YAML Flow (Default)

- **WHEN** YAML IR files exist in `spec_ir/`
- **THEN** Prism is invoked with the YAML file paths

### Scenario: Legacy Flow (Fallback)

- **WHEN** No YAML IR exists and legacy is allowed
- **THEN** The legacy Aurora/Agent pipeline is used (if enabled)

### Scenario: Missing IR Error

- **WHEN** No YAML IR exists and legacy is disabled
- **THEN** An error is raised indicating missing spec artifacts

## Diagrams

### Implementation Orchestration

```mermaid
flowchart TB
    Start((Start Task))
    CheckSpecs{Specs exist?} 
    CheckIR{YAML IR exists?} 
    InvokePrism[Invoke Prism (YAML)]
    CheckLegacy{Legacy allowed?} 
    InvokeLegacy[Invoke Agent/Aurora]
    Error[Error]
    Start --> CheckSpecs
    CheckSpecs -->|Yes| CheckIR
    CheckIR -->|Yes| InvokePrism
    CheckIR -->|No| CheckLegacy
    CheckLegacy -->|Yes| InvokeLegacy
    CheckLegacy -->|No| Error
    CheckSpecs -->|No| Error
```

## API Specification (Serverless Workflow 0.8)

```yaml
description: Orchestration workflow for code generation
id: codegen-orchestration
name: Codegen Orchestration
specVersion: '0.8'
start: CheckSpecs
states:
- name: CheckSpecs
  type: switch
- name: CheckIR
  type: switch
- name: InvokePrism
  type: operation
- name: CheckLegacy
  type: switch
- name: InvokeLegacy
  type: operation
- name: Error
  type: end
version: 1.0.0
```

</spec>
