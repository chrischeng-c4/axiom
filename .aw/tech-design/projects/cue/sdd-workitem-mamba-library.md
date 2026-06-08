---
id: cue-sdd-workitem-mamba-library
summary: Mamba library surface for Cue to consume SDD WorkItem lifecycle without shelling out to Score CLI.
fill_sections: [schema, logic, scenarios, changes, tests]
---

# SDD WorkItem Mamba Library

## Library Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/sdd-workitem-mamba-library/v0"
title: SDD WorkItem Mamba Library v0
type: object
additionalProperties: false
required: [operations, workitem_state]
properties:
  operations:
    type: array
    items:
      enum: [create_workitem, update_workitem, classify_prompt, list_workitems, get_workitem, create_prd_artifact, get_evidence]
  workitem_state:
    type: object
    required: [id, route, state, blockers, next_action]
    properties:
      id: { type: string }
      route: { enum: [prompt_to_workitem, prompt_to_prd, prompt_to_td, prompt_to_runtime] }
      state: { enum: [collecting, accepted, drafting, blocked, done] }
      blockers: { type: array, items: { type: string } }
      next_action: { type: string }
```

## Library Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-sdd-mamba-library-logic
entry: CueBackendCallsSdd
nodes:
  CueBackendCallsSdd: { kind: start, label: Cue backend calls SDD library }
  HasLibrarySurface: { kind: decision, label: Mamba library surface exists? }
  UseLibrary: { kind: terminal, label: use sdd-mamba }
  BridgeAllowed: { kind: decision, label: bridge allowed? }
  UseTemporaryBridge: { kind: terminal, label: use CPython bridge with migration link }
  BlockRuntime: { kind: terminal, label: block Cue backend slice }
edges:
  - { from: CueBackendCallsSdd, to: HasLibrarySurface, label: call needed }
  - { from: HasLibrarySurface, to: UseLibrary, label: yes }
  - { from: HasLibrarySurface, to: BridgeAllowed, label: no }
  - { from: BridgeAllowed, to: UseTemporaryBridge, label: yes }
  - { from: BridgeAllowed, to: BlockRuntime, label: no }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: cue_does_not_shell_out
    given: [Cue backend needs WorkItem state]
    when: [request is handled]
    then: [backend calls library API, not score CLI subprocess]
  - id: accepted_workitem_unlocks_prd
    given: [WorkItem state is accepted]
    when: [Cue requests PRD artifact]
    then: [library returns state that permits PRD creation]
  - id: blocked_workitem_surfaces_evidence
    given: [WorkItem has blockers]
    when: [Admin evidence is requested]
    then: [library returns blocker and lifecycle evidence]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/sdd/
    action: modify
    impl_mode: hand-written
    description: Expose stable WorkItem lifecycle APIs consumable from Mamba bindings.
  - path: crates/sdd-mamba/
    action: create
    impl_mode: hand-written
    description: Mamba-facing SDD WorkItem lifecycle wrapper.
  - path: projects/cue/backend/src/
    action: modify
    impl_mode: hand-written
    description: Consume SDD lifecycle as library API instead of shelling out to Score CLI.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  library_create_workitem:
    kind: integration
    verifies: [Mamba wrapper creates WorkItem state]
  library_get_evidence:
    kind: integration
    verifies: [Mamba wrapper exposes blockers and lifecycle evidence]
  cue_backend_no_score_subprocess:
    kind: review
    verifies: [Cue backend code does not shell out to score CLI]
```
