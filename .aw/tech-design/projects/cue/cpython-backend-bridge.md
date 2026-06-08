---
id: cue-cpython-backend-bridge
summary: Temporary CPython bridge contract for Cue backend slices while preserving future Mamba API and migration semantics.
fill_sections: [schema, logic, scenarios, changes, tests]
---

# CPython Backend Bridge

## Bridge Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/cpython-backend-bridge/v0"
title: Cue CPython Backend Bridge v0
type: object
additionalProperties: false
required: [module, api_shape, migration]
properties:
  module:
    type: object
    required: [path, status]
    properties:
      path: { type: string, pattern: "^projects/cue/backend/" }
      status: { enum: [temporary, migrating, retired] }
  api_shape:
    type: object
    required: [request_dtos, response_dtos, error_shape]
    properties:
      request_dtos: { type: array, items: { type: string } }
      response_dtos: { type: array, items: { type: string } }
      error_shape: { enum: [mamba_compatible] }
  migration:
    type: object
    required: [mamba_issue_ref, removal_condition]
    properties:
      mamba_issue_ref: { type: string }
      removal_condition: { type: string }
      bridge_owner: { type: string }
```

## Bridge Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-cpython-bridge-logic
entry: NeedBackendSlice
nodes:
  NeedBackendSlice: { kind: start, label: backend slice required }
  MambaPathReady: { kind: decision, label: Mamba path ready? }
  UseMamba: { kind: terminal, label: implement on Mamba }
  ReferencePathWorks: { kind: decision, label: CPython reference works? }
  FileMambaIssue: { kind: process, label: file or link Mamba issue }
  ImplementBridge: { kind: process, label: implement bridge with Mamba DTOs }
  PinMigration: { kind: process, label: record removal condition }
  BridgeReady: { kind: terminal, label: bridge accepted }
  BlockSlice: { kind: terminal, label: block slice }
edges:
  - { from: NeedBackendSlice, to: MambaPathReady, label: evaluate }
  - { from: MambaPathReady, to: UseMamba, label: yes }
  - { from: MambaPathReady, to: ReferencePathWorks, label: no }
  - { from: ReferencePathWorks, to: BlockSlice, label: no }
  - { from: ReferencePathWorks, to: FileMambaIssue, label: yes }
  - { from: FileMambaIssue, to: ImplementBridge, label: issue linked }
  - { from: ImplementBridge, to: PinMigration, label: dto shape pinned }
  - { from: PinMigration, to: BridgeReady, label: migration rule saved }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: mamba_ready_no_bridge
    given: [Mamba supports the target API]
    when: [backend slice starts]
    then: [bridge is not created]
  - id: bridge_with_linked_migration
    given: [Mamba is blocked, CPython reference works]
    when: [bridge lands]
    then: [migration issue is linked, DTOs are Mamba-compatible]
  - id: bridge_without_migration_rejected
    given: [bridge code exists without migration issue]
    when: [review runs]
    then: [review blocks the slice]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/projects/cue/cpython-backend-bridge.md
    action: create
    impl_mode: hand-written
    description: Define temporary bridge rules for Cue backend slices blocked by Mamba readiness.
  - path: projects/cue/backend/
    action: modify
    impl_mode: hand-written
    description: Keep bridge modules narrow, Mamba-shaped, and linked to migration issues.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  bridge_requires_mamba_issue:
    kind: review
    verifies: [temporary bridge has migration issue]
  bridge_preserves_dtos:
    kind: contract
    verifies: [request and response shapes remain Mamba-compatible]
  bridge_retirement_condition:
    kind: review
    verifies: [removal condition exists before bridge lands]
```
