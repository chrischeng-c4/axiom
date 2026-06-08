---
id: cue-mamba-readiness-gate
summary: Release gate for using Mamba as Cue backend runtime, including API, schema, agent, SDD work item, persistence, and test harness readiness.
fill_sections: [schema, logic, scenarios, changes, tests]
---

# Mamba Readiness Gate

## Readiness Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/mamba-readiness-gate/v0"
title: Cue Mamba Readiness Gate v0
type: object
additionalProperties: false
required: [capabilities, decision]
properties:
  capabilities:
    type: object
    additionalProperties: false
    required: [api, schema, sdd, agent, runtime, persistence, tests]
    properties:
      api: { enum: [ready, bridge_required, blocked] }
      schema: { enum: [ready, bridge_required, blocked] }
      sdd: { enum: [ready, bridge_required, blocked] }
      agent: { enum: [ready, bridge_required, blocked] }
      runtime: { enum: [ready, bridge_required, blocked] }
      persistence: { enum: [ready, bridge_required, blocked] }
      tests: { enum: [ready, bridge_required, blocked] }
  blockers:
    type: array
    items:
      type: object
      required: [capability, issue_ref, status]
      properties:
        capability: { enum: [api, schema, sdd, agent, runtime, persistence, tests] }
        issue_ref: { type: string }
        status: { enum: [open, in_progress, resolved] }
        bridge_allowed: { type: boolean }
  decision:
    type: object
    required: [backend_mode, release_allowed]
    properties:
      backend_mode: { enum: [mamba_native, cpython_bridge, blocked] }
      release_allowed: { type: boolean }
      required_migration_issue: { type: ["string", "null"] }
```

## Gate Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-mamba-readiness-gate-logic
entry: EvaluateCapabilityMatrix
nodes:
  EvaluateCapabilityMatrix: { kind: start, label: evaluate Mamba capabilities }
  AnyBlockedWithoutBridge: { kind: decision, label: blocked without bridge? }
  BlockRelease: { kind: terminal, label: block backend slice }
  AnyBridgeRequired: { kind: decision, label: bridge required? }
  RequireMigrationIssue: { kind: process, label: require linked Mamba migration issue }
  AllowBridgeSlice: { kind: terminal, label: allow CPython bridge slice }
  AllowMambaNative: { kind: terminal, label: allow Mamba-native slice }
edges:
  - { from: EvaluateCapabilityMatrix, to: AnyBlockedWithoutBridge, label: matrix ready }
  - { from: AnyBlockedWithoutBridge, to: BlockRelease, label: yes }
  - { from: AnyBlockedWithoutBridge, to: AnyBridgeRequired, label: no }
  - { from: AnyBridgeRequired, to: RequireMigrationIssue, label: yes }
  - { from: RequireMigrationIssue, to: AllowBridgeSlice, label: issue linked }
  - { from: AnyBridgeRequired, to: AllowMambaNative, label: no }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: all_capabilities_ready
    given: [api, schema, sdd, agent, runtime, persistence, tests are ready]
    when: [gate evaluates]
    then: [backend_mode is mamba_native, release_allowed is true]
  - id: bridge_allowed_with_migration
    given: [one capability is bridge_required, migration issue is linked]
    when: [gate evaluates]
    then: [backend_mode is cpython_bridge, release_allowed is true]
  - id: blocked_without_bridge
    given: [one capability is blocked and bridge_allowed is false]
    when: [gate evaluates]
    then: [backend_mode is blocked, release_allowed is false]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: .aw/tech-design/projects/cue/mamba-readiness-gate.md
    action: create
    impl_mode: hand-written
    description: Define Cue backend release gate for Mamba readiness and bridge policy.
  - path: projects/cue/backend/
    action: modify
    impl_mode: hand-written
    description: Add readiness status reporting once backend control plane exists.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  mamba_native_ready:
    kind: unit
    verifies: [all ready capabilities allow mamba_native]
  bridge_requires_migration_issue:
    kind: unit
    verifies: [bridge_required capability without migration issue is rejected]
  hard_block_prevents_release:
    kind: unit
    verifies: [blocked capability with no bridge blocks release]
```
