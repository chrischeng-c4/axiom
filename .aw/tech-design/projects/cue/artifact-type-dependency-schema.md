---
id: cue-artifact-type-dependency-schema
summary: Artifact graph contract for WorkItem, PRD, TD, App Spec, runtime manifest, dependency ordering, status, and unlock gates.
fill_sections: [schema, state-machine, scenarios, changes, tests]
---

# Artifact Type Dependency Schema

Status: implemented

## Artifact Graph Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/artifact-graph/v0"
title: Cue Artifact Graph v0
type: object
additionalProperties: false
required: [artifact_types, dependencies]
properties:
  artifact_types:
    type: array
    items:
      type: object
      required: [kind, owner, review_required]
      properties:
        kind: { enum: [workitem, prd, td, app_spec, runtime_manifest] }
        owner: { enum: [project_owner, platform, agent_team] }
        review_required: { type: boolean }
  dependencies:
    type: array
    items:
      type: object
      required: [from, to, gate]
      properties:
        from: { enum: [workitem, prd, td, app_spec, runtime_manifest] }
        to: { enum: [workitem, prd, td, app_spec, runtime_manifest] }
        gate: { enum: [accepted, approved, validated, policy_passed] }
```

## Dependency State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: cue-artifact-dependency-state
initial: WorkItemDraft
nodes:
  WorkItemDraft: { kind: initial, label: WorkItemDraft }
  WorkItemAccepted: { kind: normal, label: WorkItemAccepted }
  PrdApproved: { kind: normal, label: PrdApproved }
  TdApproved: { kind: normal, label: TdApproved }
  AppSpecValidated: { kind: normal, label: AppSpecValidated }
  RuntimeReady: { kind: terminal, label: RuntimeReady }
edges:
  - { from: WorkItemDraft, to: WorkItemAccepted, event: accept_workitem }
  - { from: WorkItemAccepted, to: PrdApproved, event: approve_prd }
  - { from: PrdApproved, to: TdApproved, event: approve_td }
  - { from: TdApproved, to: AppSpecValidated, event: validate_app_spec }
  - { from: AppSpecValidated, to: RuntimeReady, event: policy_and_tests_pass }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: prd_locked_until_workitem_accepted
    given: [WorkItem state is collecting]
    when: [PRD route is requested]
    then: [PRD creation is rejected]
  - id: td_locked_until_prd_approved
    given: [PRD is draft]
    when: [TD route is requested]
    then: [TD creation is rejected]
  - id: runtime_locked_until_td_and_app_spec
    given: [TD approved, App Spec invalid]
    when: [runtime route is requested]
    then: [runtime artifact is blocked]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cue/schemas/
    action: modify
    impl_mode: hand-written
    description: Add artifact graph and dependency gate schema.
  - path: projects/cue/backend/src/
    action: modify
    impl_mode: hand-written
    description: Enforce artifact dependency unlock gates in Prompt-to-X routes.
  - path: projects/cue/artifact-studio/src/
    action: modify
    impl_mode: hand-written
    description: Render artifact dependency status and locked next actions.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  dependency_gate_prd:
    kind: unit
    verifies: [PRD locked until WorkItem accepted]
  dependency_gate_td:
    kind: unit
    verifies: [TD locked until PRD approved]
  dependency_gate_runtime:
    kind: unit
    verifies: [runtime locked until TD and App Spec are ready]
```
