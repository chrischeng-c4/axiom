---
id: cue-artifact-studio-admin-layout
summary: Product layout contract separating owner-facing Artifact Studio, platform-operator Admin, backend, shared contracts, schemas, examples, and legacy notes.
fill_sections: [schema, logic, scenarios, changes, tests]
---

# Artifact Studio Admin Layout

Status: implemented

## Layout Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/product-layout/v0"
title: Cue Product Layout v0
type: object
additionalProperties: false
required: [workspaces]
properties:
  workspaces:
    type: array
    items:
      type: object
      required: [name, path, audience, role]
      properties:
        name: { enum: [artifact_studio, admin, backend, shared, schemas, examples, legacy_docs] }
        path:
          enum:
            - projects/cue/artifact-studio
            - projects/cue/admin
            - projects/cue/backend
            - projects/cue/shared
            - projects/cue/schemas
            - projects/cue/examples
            - projects/cue/docs/legacy
        audience: { enum: [project_owner, platform_operator, developer, generated_app_runtime] }
        role: { enum: [frontend_site, api_service, shared_contracts, contract_store, fixture_store, history_only] }
```

## Routing Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-layout-routing-logic
entry: RouteCueWork
nodes:
  RouteCueWork: { kind: start, label: route work by audience }
  OwnerFacing: { kind: decision, label: project owner? }
  ArtifactStudio: { kind: terminal, label: projects/cue/artifact-studio }
  PlatformOperator: { kind: decision, label: platform operator? }
  Admin: { kind: terminal, label: projects/cue/admin }
  BackendNeed: { kind: decision, label: API or service? }
  Backend: { kind: terminal, label: projects/cue/backend }
  ContractNeed: { kind: decision, label: shared contract? }
  SharedOrSchemas: { kind: terminal, label: shared/schemas/examples }
  LegacyOnly: { kind: terminal, label: docs/legacy only }
edges:
  - { from: RouteCueWork, to: OwnerFacing, label: inspect audience }
  - { from: OwnerFacing, to: ArtifactStudio, label: yes }
  - { from: OwnerFacing, to: PlatformOperator, label: no }
  - { from: PlatformOperator, to: Admin, label: yes }
  - { from: PlatformOperator, to: BackendNeed, label: no }
  - { from: BackendNeed, to: Backend, label: yes }
  - { from: BackendNeed, to: ContractNeed, label: no }
  - { from: ContractNeed, to: SharedOrSchemas, label: yes }
  - { from: ContractNeed, to: LegacyOnly, label: no }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: owner_work_goes_to_artifact_studio
    given: [feature targets prompt, WorkItem, PRD, artifact graph, sandbox request]
    when: [route is resolved]
    then: [workspace is projects/cue/artifact-studio]
  - id: desktop_layout_prioritizes_work_context
    given: [Artifact Studio is viewed on a desktop width]
    when: [active WorkItem workflow graph is visible]
    then:
      - project and session navigation remains the narrow left rail
      - conversation uses a narrower middle column and bounded message width
      - right context pane is wider than the conversation column
      - right context pane owns the workflow graph, artifacts, gates, blockers, and next action
  - id: operator_work_goes_to_admin
    given: [feature targets evidence, policy, grants, diagnostics, release controls]
    when: [route is resolved]
    then: [workspace is projects/cue/admin]
  - id: backend_work_goes_to_backend
    given: [feature targets API, jobs, adapter, registry, audit, policy]
    when: [route is resolved]
    then: [workspace is projects/cue/backend]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cue/artifact-studio/
    action: preserve
    impl_mode: hand-written
    description: |
      Owner-facing Jet frontend site. On desktop, keep project/session
      navigation narrow, keep conversation bounded, and make the right context
      pane wider because WorkItem workflow state is the primary work surface.
  - path: projects/cue/admin/
    action: create
    impl_mode: hand-written
    description: Platform-operator Admin Jet frontend site.
  - path: projects/cue/backend/
    action: create
    impl_mode: hand-written
    description: Cue API, services, adapters, policy, registry, audit, and orchestration.
  - path: projects/cue/shared/
    action: create
    impl_mode: hand-written
    description: Shared domain types, API client, and status mapping only.
  - path: projects/cue/fe/
    action: retire
    impl_mode: hand-written
    description: Transitional frontend scaffold retired after Artifact Studio/Admin split lands.
  - path: projects/cue/be/
    action: retire
    impl_mode: hand-written
    description: Transitional backend scaffold retired after backend path owns API contract.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  workspace_routing:
    kind: review
    verifies: [owner features, admin features, backend features route to correct paths]
  desktop_layout_ratio:
    kind: browser
    verifies: [right context pane is wider than conversation column on desktop]
  legacy_not_extended:
    kind: review
    verifies: [docs/legacy remains history only]
  no_rust_product_crate:
    kind: inspection
    verifies: [projects/cue has no Cargo.toml product crate]
```
