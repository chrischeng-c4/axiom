---
id: cue-backend-control-plane-mvp
summary: Backend control plane MVP for Cue WorkItem, PRD artifact, Artifact Studio, Admin evidence, and Mamba-compatible bridge boundaries.
fill_sections: [schema, rest-api, logic, state-machine, scenarios, changes, tests]
---

# Backend Control Plane MVP

## Control Plane Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/backend-control-plane/v0"
title: Cue Backend Control Plane MVP
type: object
additionalProperties: false
required: [project, workitem, artifact, artifact_repository, admin_review_ticket, audit_event]
properties:
  project:
    type: object
    additionalProperties: false
    required: [id, name, owner_namespace, risk_tier, lifecycle_status, current_workstream_id]
    properties:
      id: { type: string }
      name: { type: string }
      goal: { type: string }
      owner_namespace: { enum: [personal, team, cross_team, platform] }
      owner_user: { type: ["string", "null"] }
      owner_team: { type: ["string", "null"] }
      risk_tier: { enum: [tier_0, tier_1, tier_2, tier_3, tier_4] }
      lifecycle_status: { enum: [draft, active, blocked, sandbox, production, archived, retired] }
      current_workstream_id: { type: ["string", "null"] }
  workitem:
    type: object
    additionalProperties: false
    required: [id, project_id, route, target_artifact_type, state, blockers, next_action]
    properties:
      id: { type: string }
      project_id: { type: string }
      route: { enum: [prompt_to_workitem, prompt_to_prd, prompt_to_td, prompt_to_runtime] }
      target_artifact_type: { enum: [workitem, prd, td, app_spec, runtime_artifact] }
      state: { enum: [collecting, accepted, drafting, blocked, done] }
      title: { type: string }
      prompt_summary: { type: string }
      blockers:
        type: array
        items: { type: string }
      next_action: { type: string }
      risk_hints:
        type: array
        items: { type: string }
  artifact:
    type: object
    additionalProperties: false
    required: [id, project_id, kind, state, dependency_ids, latest_version]
    properties:
      id: { type: string }
      project_id: { type: string }
      workitem_id: { type: ["string", "null"] }
      kind: { enum: [prd, td, app_spec, runtime_manifest] }
      state: { enum: [draft, needs_review, approved, blocked] }
      dependency_ids:
        type: array
        items: { type: string }
      latest_version: { type: integer, minimum: 0 }
  artifact_repository:
    type: object
    additionalProperties: false
    required: [id, project_id, backend, status]
    properties:
      id: { type: string }
      project_id: { type: string }
      backend: { enum: [fixture, local, gitlab] }
      status: { enum: [not_provisioned, fixture_backed, provisioned, blocked] }
      hidden_gitlab_project_id: { type: ["integer", "null"] }
      current_spec_ref: { type: ["string", "null"] }
      sandbox_ref: { type: ["string", "null"] }
      production_release_tag: { type: ["string", "null"] }
  admin_review_ticket:
    type: object
    additionalProperties: false
    required: [id, project_id, workitem_id, kind, state, evidence_ids]
    properties:
      id: { type: string }
      project_id: { type: string }
      workitem_id: { type: string }
      kind: { enum: [policy_exception, connector_grant, production_request, backend_blocker] }
      state: { enum: [open, approved, rejected, change_requested, closed] }
      evidence_ids:
        type: array
        items: { type: string }
  audit_event:
    type: object
    additionalProperties: false
    required: [id, entity_type, entity_id, transition, created_at]
    properties:
      id: { type: string }
      entity_type: { enum: [project, workitem, artifact, artifact_repository, admin_review_ticket] }
      entity_id: { type: string }
      transition: { type: string }
      created_at: { type: string, format: date-time }
      actor: { type: string }
      summary: { type: string }
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: Cue Backend Control Plane MVP
  version: 0.1.0
paths:
  /api/health:
    get:
      operationId: getHealth
      responses: { "200": { description: Service health } }
  /api/projects:
    get:
      operationId: listProjects
      responses: { "200": { description: Project list } }
  /api/projects/{project_id}:
    get:
      operationId: getProject
      parameters:
        - { name: project_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: Project detail }
        "404": { description: Project not found }
  /api/projects/{project_id}/workitems:
    get:
      operationId: listProjectWorkitems
      parameters:
        - { name: project_id, in: path, required: true, schema: { type: string } }
      responses: { "200": { description: Project workstream WorkItems } }
  /api/projects/{project_id}/artifacts:
    get:
      operationId: listProjectArtifacts
      parameters:
        - { name: project_id, in: path, required: true, schema: { type: string } }
      responses: { "200": { description: Project artifact graph } }
  /api/projects/{project_id}/prompts:
    post:
      operationId: submitProjectPrompt
      parameters:
        - { name: project_id, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [prompt]
              properties:
                prompt: { type: string, minLength: 1 }
      responses:
        "200": { description: WorkItem classification or redirect result }
        "404": { description: Project not found }
  /api/workitems/{workitem_id}/prd:
    post:
      operationId: createPrdArtifact
      parameters:
        - { name: workitem_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: PRD artifact created or updated }
        "409": { description: WorkItem not accepted }
        "404": { description: WorkItem not found }
  /api/admin/workitems:
    get:
      operationId: listAdminWorkitemQueue
      responses: { "200": { description: Dense Admin queue } }
  /api/admin/workitems/{workitem_id}/evidence:
    get:
      operationId: getAdminWorkitemEvidence
      parameters:
        - { name: workitem_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: Evidence, blockers, policy, diagnostics, tickets, audit }
        "404": { description: Evidence not found }
```

## WorkItem And PRD Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-backend-control-plane-workitem-prd
entry: PromptSubmitted
nodes:
  PromptSubmitted: { kind: start, label: prompt submitted }
  ClassifyIntent: { kind: process, label: classify project intent }
  GeneralChat: { kind: decision, label: general chat? }
  RedirectOutOfCue: { kind: terminal, label: redirect without artifact }
  MissingInputs: { kind: decision, label: missing required governance inputs? }
  ReturnClarification: { kind: terminal, label: ask for minimum clarification }
  UpsertWorkItem: { kind: process, label: create or update WorkItem }
  WorkItemAccepted: { kind: decision, label: accepted? }
  ReturnWorkItemState: { kind: terminal, label: return owner state }
  PrdRequested: { kind: start, label: prompt-to-PRD requested }
  LoadWorkItem: { kind: process, label: load WorkItem }
  AcceptedForPrd: { kind: decision, label: accepted? }
  RejectPrd: { kind: terminal, label: reject with 409 }
  CreatePrdVersion: { kind: process, label: create PRD artifact version }
  WriteAuditEvent: { kind: process, label: persist audit event }
  ReturnArtifactState: { kind: terminal, label: return PRD state }
edges:
  - { from: PromptSubmitted, to: ClassifyIntent, label: prompt }
  - { from: ClassifyIntent, to: GeneralChat, label: classified }
  - { from: GeneralChat, to: RedirectOutOfCue, label: yes }
  - { from: GeneralChat, to: MissingInputs, label: no }
  - { from: MissingInputs, to: ReturnClarification, label: yes }
  - { from: MissingInputs, to: UpsertWorkItem, label: no }
  - { from: UpsertWorkItem, to: WorkItemAccepted, label: persisted }
  - { from: WorkItemAccepted, to: ReturnWorkItemState, label: any }
  - { from: PrdRequested, to: LoadWorkItem, label: workitem id }
  - { from: LoadWorkItem, to: AcceptedForPrd, label: loaded }
  - { from: AcceptedForPrd, to: RejectPrd, label: no }
  - { from: AcceptedForPrd, to: CreatePrdVersion, label: yes }
  - { from: CreatePrdVersion, to: WriteAuditEvent, label: artifact saved }
  - { from: WriteAuditEvent, to: ReturnArtifactState, label: audit saved }
---
```

## Lifecycle State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
---
id: cue-backend-control-plane-lifecycle
initial: PromptDraft
nodes:
  PromptDraft: { kind: initial, label: PromptDraft }
  Clarifying: { kind: normal, label: Clarifying }
  WorkItemAccepted: { kind: normal, label: WorkItemAccepted }
  PrdDrafted: { kind: normal, label: PrdDrafted }
  AdminVisible: { kind: normal, label: AdminVisible }
  Blocked: { kind: normal, label: Blocked }
  Done: { kind: terminal, label: Done }
edges:
  - { from: PromptDraft, to: Clarifying, event: missing_inputs }
  - { from: PromptDraft, to: WorkItemAccepted, event: actionable_prompt }
  - { from: Clarifying, to: WorkItemAccepted, event: clarification_answered }
  - { from: WorkItemAccepted, to: PrdDrafted, event: prd_requested }
  - { from: PrdDrafted, to: AdminVisible, event: evidence_written }
  - { from: AdminVisible, to: Blocked, event: blocker_detected }
  - { from: AdminVisible, to: Done, event: owner_review_ready }
  - { from: Blocked, to: AdminVisible, event: blocker_resolved }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: owner_loads_workstream
    given: [fixture-backed project exists]
    when: [Artifact Studio requests project, workitems, and artifacts]
    then: [owner sees current state, blockers, and next action]
  - id: general_chat_redirect
    given: [project exists]
    when: [prompt is unrelated general chat]
    then: [response redirects, no WorkItem is created, audit is optional]
  - id: actionable_prompt_to_workitem
    given: [project exists]
    when: [prompt describes internal work artifact]
    then: [WorkItem is created or updated, route is prompt_to_workitem]
  - id: prompt_to_prd_gate
    given: [WorkItem is collecting]
    when: [PRD route is requested]
    then: [request is rejected, PRD artifact is absent]
  - id: prompt_to_prd_success
    given: [WorkItem is accepted]
    when: [PRD route is requested]
    then: [PRD artifact version is created, audit event is written]
  - id: admin_evidence_view
    given: [WorkItem has blockers or policy state]
    when: [Admin requests evidence]
    then: [response includes diagnostics, blockers, tickets, and audit ids]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cue/backend/
    action: create
    impl_mode: hand-written
    description: Active backend package for Cue API, services, fixture/local store, and static mount.
  - path: projects/cue/backend/src/
    action: create
    impl_mode: hand-written
    description: Domain models, API routes, prompt router, WorkItem service, PRD artifact service, Admin evidence service, audit writer, and repository adapter interfaces.
  - path: projects/cue/backend/tests/
    action: create
    impl_mode: hand-written
    description: Contract tests covering project reads, prompt classification, WorkItem creation, PRD gating, PRD creation, and Admin evidence.
  - path: projects/cue/artifact-studio/src/
    action: modify
    impl_mode: hand-written
    description: Load project/workitem/artifact state from backend APIs instead of hardcoded frontend arrays.
  - path: projects/cue/README.md
    action: modify
    impl_mode: hand-written
    description: Document backend control plane MVP command path and transitional be/fe retirement state.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  api_project_state:
    kind: api
    verifies: [health, project list, project detail, workitems, artifacts]
  prompt_general_chat:
    kind: api
    verifies: [classification out_of_scope, no WorkItem write]
  prompt_actionable_work:
    kind: api
    verifies: [WorkItem upsert, route, target artifact type, next action]
  prd_requires_accepted_workitem:
    kind: api
    verifies: [conflict response, no artifact version]
  prd_creates_artifact_and_audit:
    kind: api
    verifies: [PRD artifact version, audit event, Admin evidence link]
  artifact_studio_backend_e2e:
    kind: browser
    verifies: [frontend renders backend project, workitem, artifact, blockers]
  mamba_bridge_contract:
    kind: contract
    verifies: [CPython bridge preserves Mamba-shaped request and response DTOs]
```

# Reviews

### Review 1
**Verdict:** approved

- [schema] Control-plane entities cover the epic scope: Project, WorkItem, Artifact, ArtifactRepository, AdminReviewTicket, and AuditEvent.
- [rest-api] API surface matches #1984 and cleanly separates owner-facing Artifact Studio routes from Admin evidence routes.
- [logic] Prompt-to-WorkItem and prompt-to-PRD flow covers redirect, clarification, accepted WorkItem gating, PRD artifact creation, and audit persistence.
- [state-machine] Lifecycle states model the MVP path from prompt draft through clarification, accepted WorkItem, PRD draft, Admin visibility, blocker handling, and owner-ready completion.
- [changes] Implementation remains centered on `projects/cue/backend` and explicitly avoids Tracker runtime deployment or final GitLab provisioner work.
- [tests] Test contract covers API, lifecycle, frontend e2e consumption, and Mamba bridge compatibility.
