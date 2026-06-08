---
id: cue-backend-artifact-studio-workstream-api
summary: Fixture-backed Cue backend API for Artifact Studio Sessions, WorkItems, PRD, artifact, audit, and Admin evidence state.
fill_sections: [schema, rest-api, logic, scenarios, changes, tests]
---

# Backend Artifact Studio Workstream API

## Domain Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/backend-workstream-api/v0"
title: Cue Backend Workstream API v0
type: object
additionalProperties: false
required:
  - project
  - session
  - workitem
  - workitem_context
  - artifact
  - artifact_version
  - audit_event
  - admin_evidence
properties:
  project:
    type: object
    additionalProperties: false
    required: [id, name, owner_team, risk_tier, lifecycle_status, next_action]
    properties:
      id: { type: string, pattern: "^[a-z0-9][a-z0-9-]*$" }
      name: { type: string, minLength: 1 }
      owner_team: { type: string, minLength: 1 }
      owner_user: { type: ["string", "null"] }
      risk_tier: { enum: [tier_0, tier_1, tier_2, tier_3, tier_4] }
      lifecycle_status: { enum: [draft, blocked, sandbox, production, archived, retired] }
      summary: { type: string }
      next_action: { type: string, minLength: 1 }
      blocker_count: { type: integer, minimum: 0 }
      active_session_id: { type: string }
      sessions:
        type: array
        items: { $ref: "#/$defs/session" }
  session:
    $ref: "#/$defs/session"
  workitem:
    type: object
    additionalProperties: false
    required: [id, project_id, title, route, target_artifact_type, state, next_action, blockers]
    properties:
      id: { type: string, pattern: "^[a-z0-9][a-z0-9-]*$" }
      project_id: { type: string }
      title: { type: string, minLength: 1 }
      route: { enum: [prompt_to_workitem, prompt_to_prd, prompt_to_td, prompt_to_runtime] }
      target_artifact_type: { enum: [workitem, prd, td, app_spec, runtime] }
      state: { enum: [collecting, accepted, drafting, blocked, done] }
      prompt_summary: { type: string }
      next_action: { type: string }
      blockers:
        type: array
        items: { type: string }
      evidence_ids:
        type: array
        items: { type: string }
      workflow_plan:
        type: array
        items: { $ref: "#/$defs/workflow_step" }
      qc_status: { enum: [pass, needs_input, warning, blocked] }
      qc_checks:
        type: array
        items: { $ref: "#/$defs/qc_check" }
  workitem_context:
    type: object
    additionalProperties: false
    required: [type, project_id, workitem, workflow_plan, artifacts, blockers, qc_status, qc_checks, next_action]
    properties:
      type: { enum: [artifact, blockers, workflow_plan, project_overview] }
      project_id: { type: string }
      workitem: { $ref: "#/properties/workitem" }
      workflow_plan:
        type: array
        items: { $ref: "#/$defs/workflow_step" }
      artifacts:
        type: array
        items: { $ref: "#/properties/artifact" }
      blockers:
        type: array
        items: { type: string }
      qc_status: { enum: [pass, needs_input, warning, blocked] }
      qc_checks:
        type: array
        items: { $ref: "#/$defs/qc_check" }
      next_action: { type: string }
      next_artifact_kind: { enum: [prd, td, app_spec, runtime, null] }
  artifact:
    type: object
    additionalProperties: false
    required: [id, project_id, kind, title, state, latest_version_id]
    properties:
      id: { type: string }
      project_id: { type: string }
      workitem_id: { type: ["string", "null"] }
      kind: { enum: [prd, td, app_spec, runtime_manifest] }
      title: { type: string }
      state: { enum: [draft, needs_review, approved, blocked] }
      latest_version_id: { type: ["string", "null"] }
  artifact_version:
    type: object
    additionalProperties: false
    required: [id, artifact_id, version, created_at, content_summary]
    properties:
      id: { type: string }
      artifact_id: { type: string }
      version: { type: integer, minimum: 1 }
      created_at: { type: string, format: date-time }
      content_summary: { type: string }
      audit_event_id: { type: string }
  audit_event:
    type: object
    additionalProperties: false
    required: [id, entity_type, entity_id, transition, created_at]
    properties:
      id: { type: string }
      entity_type: { enum: [project, workitem, artifact, artifact_version, admin_review_ticket] }
      entity_id: { type: string }
      transition: { type: string }
      created_at: { type: string, format: date-time }
      actor: { type: string }
      summary: { type: string }
  admin_evidence:
    type: object
    additionalProperties: false
    required: [id, workitem_id, policy_state, blockers, diagnostics, audit_event_ids]
    properties:
      id: { type: string }
      workitem_id: { type: string }
      policy_state: { enum: [not_evaluated, pass, warning, blocked] }
      blockers:
        type: array
        items: { type: string }
      diagnostics:
        type: array
        items:
          type: object
          required: [source, status, detail]
          additionalProperties: false
          properties:
            source: { enum: [prompt_router, workitem_store, prd_generator, policy, backend, frontend] }
            status: { enum: [ok, warning, error] }
            detail: { type: string }
      audit_event_ids:
        type: array
        items: { type: string }
$defs:
  session:
    type: object
    additionalProperties: false
    required: [id, project_id, title, messages]
    properties:
      id: { type: string }
      project_id: { type: string }
      title: { type: string }
      active_workitem_id: { type: ["string", "null"] }
      messages:
        type: array
        items: { $ref: "#/$defs/message" }
  message:
    type: object
    additionalProperties: false
    required: [id, speaker, body]
    properties:
      id: { type: string }
      speaker: { enum: [owner, cue] }
      body: { type: string }
      action: { type: ["string", "null"] }
  workflow_step:
    type: object
    additionalProperties: false
    required: [id, label, state, depends_on, agent_role, agent_label]
    properties:
      id: { enum: [prd, td, website, codebase, test, deployment, operation] }
      label: { type: string }
      state: { enum: [not-started, ready, in-progress, blocked, done] }
      depends_on:
        type: array
        items: { type: string }
      agent_role: { enum: [pm, architect, designer, dev, data, qa_policy, release] }
      agent_label: { type: string }
      agent_task: { type: string }
  qc_check:
    type: object
    additionalProperties: false
    required: [id, label, status, summary]
    properties:
      id: { type: string }
      label: { type: string }
      status: { enum: [pass, needs_input, warning, blocked] }
      summary: { type: string }
```

## REST API
<!-- type: rest-api lang: yaml -->

```yaml
openapi: 3.1.0
info:
  title: Cue Backend Workstream API
  version: 0.1.0
paths:
  /api/health:
    get:
      operationId: getHealth
      responses:
        "200":
          description: Service health.
  /api/projects:
    get:
      operationId: listProjects
      responses:
        "200":
          description: Project list for Artifact Studio.
  /api/projects/{project_id}:
    get:
      operationId: getProject
      parameters:
        - { name: project_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: Project detail. }
        "404": { description: Project not found. }
  /api/projects/{project_id}/workitems:
    get:
      operationId: listProjectWorkitems
      parameters:
        - { name: project_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: Project WorkItems. }
  /api/projects/{project_id}/artifacts:
    get:
      operationId: listProjectArtifacts
      parameters:
        - { name: project_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: Project artifacts. }
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
        "200": { description: Prompt classification and WorkItem outcome. }
        "404": { description: Project not found. }
  /api/sessions/{session_id}/messages:
    post:
      operationId: submitSessionMessage
      parameters:
        - { name: session_id, in: path, required: true, schema: { type: string } }
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [content]
              properties:
                content: { type: string, minLength: 1 }
                title: { type: string }
      responses:
        "200": { description: Session message response with project, session, WorkItem, and context. }
        "404": { description: Session not found. }
  /api/workitems/{workitem_id}/context:
    get:
      operationId: getWorkitemContext
      parameters:
        - { name: workitem_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: WorkItem delivery context for the right pane. }
        "404": { description: WorkItem not found. }
  /api/workitems/{workitem_id}/prd:
    post:
      operationId: createPrdFromWorkitem
      parameters:
        - { name: workitem_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: PRD artifact created or updated. }
        "409": { description: WorkItem is not accepted. }
        "404": { description: WorkItem not found. }
  /api/admin/workitems:
    get:
      operationId: listAdminWorkitems
      responses:
        "200": { description: Dense Admin WorkItem queue. }
  /api/admin/workitems/{workitem_id}/evidence:
    get:
      operationId: getAdminWorkitemEvidence
      parameters:
        - { name: workitem_id, in: path, required: true, schema: { type: string } }
      responses:
        "200": { description: Admin evidence detail. }
        "404": { description: WorkItem evidence not found. }
```

## Prompt And PRD Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-backend-workstream-logic
entry: ReceivePrompt
nodes:
  ReceivePrompt:
    kind: start
    label: POST project prompt
  ClassifyPrompt:
    kind: process
    label: classify route and risk hints
  IsGeneralChat:
    kind: decision
    label: general chat?
  RedirectChat:
    kind: terminal
    label: return out_of_scope redirect
  HasRequiredInputs:
    kind: decision
    label: enough details?
  AskClarification:
    kind: terminal
    label: return clarification blockers
  UpsertWorkItem:
    kind: process
    label: create or update WorkItem
  AppendSessionMessages:
    kind: process
    label: append owner and Cue messages
  BuildWorkItemContext:
    kind: process
    label: build workflow plan blockers QC and artifacts
  ReturnOwnerState:
    kind: terminal
    label: return workstream state
  ReceivePrdRequest:
    kind: start
    label: POST workitem PRD
  WorkItemAccepted:
    kind: decision
    label: accepted?
  RejectPrd:
    kind: terminal
    label: 409 not accepted
  CreatePrdArtifact:
    kind: process
    label: create PRD artifact version
  WriteAudit:
    kind: process
    label: write audit transition
  ReturnPrdState:
    kind: terminal
    label: return artifact state
edges:
  - from: ReceivePrompt
    to: ClassifyPrompt
    label: prompt submitted
  - from: ClassifyPrompt
    to: IsGeneralChat
    label: classification complete
  - from: IsGeneralChat
    to: RedirectChat
    label: yes
  - from: IsGeneralChat
    to: HasRequiredInputs
    label: no
  - from: HasRequiredInputs
    to: AskClarification
    label: no
  - from: HasRequiredInputs
    to: UpsertWorkItem
    label: yes
  - from: UpsertWorkItem
    to: AppendSessionMessages
    label: persisted
  - from: AppendSessionMessages
    to: BuildWorkItemContext
    label: session updated
  - from: BuildWorkItemContext
    to: ReturnOwnerState
    label: context ready
  - from: ReceivePrdRequest
    to: WorkItemAccepted
    label: workitem loaded
  - from: WorkItemAccepted
    to: RejectPrd
    label: no
  - from: WorkItemAccepted
    to: CreatePrdArtifact
    label: yes
  - from: CreatePrdArtifact
    to: WriteAudit
    label: version saved
  - from: WriteAudit
    to: ReturnPrdState
    label: audit saved
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: artifact_studio_loads_backend_state
    given:
      - backend has fixture project team-request-tracker
    when:
      - Artifact Studio requests projects, workitems, and artifacts
    then:
      - project list contains team-request-tracker
      - project list includes sessions and active_session_id
      - workitem pane is populated from API data
      - artifact pane is populated from API data
  - id: session_prompt_creates_workitem
    given:
      - Artifact Studio has an active project session
    when:
      - owner submits an actionable project prompt
    then:
      - owner message is appended to the session
      - Cue response message is appended to the session
      - WorkItem is created
      - response includes WorkItem context for the right pane
  - id: general_chat_redirects
    given:
      - project exists
    when:
      - prompt says "what is the weather today?"
    then:
      - response classification is general_chat
      - no WorkItem is created
      - response includes redirect guidance
  - id: actionable_prompt_creates_workitem
    given:
      - project exists
    when:
      - prompt requests an internal approval workflow
    then:
      - WorkItem is created or updated
      - route is prompt_to_workitem
      - next_action is owner-facing
  - id: prd_requires_accepted_workitem
    given:
      - WorkItem state is collecting
    when:
      - client posts to create PRD
    then:
      - response status is conflict
      - no PRD artifact version is created
  - id: accepted_workitem_creates_prd
    given:
      - WorkItem state is accepted
    when:
      - client posts to create PRD
    then:
      - PRD artifact version is created
      - audit event records the transition
      - Admin evidence references the audit event
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cue/backend/package.json
    action: create
    impl_mode: hand-written
    description: |
      Establish active Cue backend package commands and mark the package as
      the active backend path.
  - path: projects/cue/backend/src/main.py
    action: create
    impl_mode: hand-written
    description: |
      Define the mambalibs.http app, route registrations, static mount, backend
      startup entrypoint, Session message endpoint, and WorkItem context endpoint.
  - path: projects/cue/backend/src/models.py
    action: create
    impl_mode: hand-written
    description: |
      Define Project, WorkItem, Artifact, ArtifactVersion, AuditEvent, and
      AdminEvidence response shapes.
  - path: projects/cue/backend/src/store.py
    action: create
    impl_mode: hand-written
    description: |
      Provide fixture-backed or local-store-backed project, session, workitem,
      artifact, audit, evidence, workflow plan, and QC data. Mutating operations
      also emit structured pgkit commands so the temporary Python bridge keeps
      the future Postgres persistence boundary visible.
  - path: projects/cue/backend/src/services.py
    action: create
    impl_mode: hand-written
    description: |
      Implement prompt classification, Session message routing, WorkItem upsert,
      WorkItem context composition, PRD gating, PRD artifact creation, audit
      event write, and Admin evidence composition.
  - path: projects/cue/backend/src/mambalibs/agentkit.py
    action: create
    impl_mode: hand-written
    description: |
      Provide a Python-shaped preview of the future cclab-agent-mamba task and
      result boundary for prompt classification and later governed agent runs,
      including a Claude Code headless adapter that calls `claude -p` behind
      the same structured boundary.
  - path: projects/cue/backend/src/mambalibs/pgkit.py
    action: create
    impl_mode: hand-written
    description: |
      Provide a Python-shaped preview of the future Postgres persistence
      boundary under the pgkit name, with an explicit unavailable placeholder
      until the real Mamba-backed implementation is built in, plus a recording
      fixture implementation for contract tests.
  - path: projects/cue/backend/tests/test_workstream_api.py
    action: create
    impl_mode: hand-written
    description: |
      Add API contract and lifecycle tests for project reads, general-chat
      redirect, actionable prompt WorkItem creation, PRD gating, accepted
      WorkItem PRD creation, Admin evidence, Claude headless adapter command
      shape, and pgkit command emission.
  - path: projects/cue/scripts/test-all.mjs
    action: create
    impl_mode: hand-written
    description: |
      Run backend contract tests, Artifact Studio typecheck, and browser e2e
      from one local command.
  - path: projects/cue/package.json
    action: create
    impl_mode: hand-written
    description: |
      Expose the local all-in-one Cue validation command.
  - path: projects/cue/artifact-studio/src/api.ts
    action: create
    impl_mode: hand-written
    description: |
      Add Artifact Studio backend client functions for projects, workitems,
      artifacts, prompt submission, PRD creation, and Admin evidence fixtures.
  - path: projects/cue/artifact-studio/src/App.tsx
    action: modify
    impl_mode: hand-written
    description: |
      Replace hardcoded project/workitem/artifact arrays with backend client
      calls while preserving fixture fallback only for isolated frontend tests.
  - path: projects/cue/artifact-studio/e2e/workitems.e2e.mjs
    action: modify
    impl_mode: hand-written
    description: |
      Run Artifact Studio against backend fixture responses and assert the
      workstream panes render API-provided state.
  - path: projects/cue/be/README.md
    action: modify
    impl_mode: hand-written
    description: |
      Mark the older `be` scaffold as transitional once `projects/cue/backend`
      owns the endpoint contract and validation path.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  backend_api_contract:
    kind: api
    verifies:
      - health route returns service status
      - project routes return fixture-backed Project, WorkItem, and Artifact state
      - admin routes return AdminEvidence with blockers and diagnostics
  general_chat_redirect:
    kind: api
    verifies:
      - general chat prompt returns out_of_scope classification
      - WorkItem count is unchanged
  actionable_prompt_workitem:
    kind: api
    verifies:
      - actionable internal-work prompt creates or updates a WorkItem
      - response includes route, target artifact type, blockers, status, and next action
  session_message_workitem:
    kind: api
    verifies:
      - session message creates or updates a WorkItem for project work
      - response includes project, session, message, WorkItem, and context
      - general chat does not create a WorkItem
  workitem_context:
    kind: api
    verifies:
      - WorkItem context includes workflow plan, blockers, QC status, QC checks, and next artifact kind
  prd_gate_rejects_collecting_workitem:
    kind: api
    verifies:
      - non-accepted WorkItem returns conflict
      - no PRD artifact version is created
  accepted_workitem_creates_prd:
    kind: api
    verifies:
      - accepted WorkItem creates PRD artifact version
      - audit event records the transition
      - AdminEvidence references the audit event
  artifact_studio_backend_e2e:
    kind: browser
    verifies:
      - Artifact Studio renders project list from backend responses
      - WorkItem pane renders backend state
      - artifact pane renders backend PRD status
```

# Reviews

### Review 1
**Verdict:** approved

- [schema] Domain model covers the issue-required Project, WorkItem, Artifact, ArtifactVersion, AuditEvent, and AdminEvidence shapes with closed enums and required fields sufficient for fixture-backed implementation.
- [rest-api] Route list matches the #1985 required backend API surface, including project reads, prompt submission, PRD gating, and Admin evidence.
- [logic] Prompt-to-WorkItem and prompt-to-PRD transitions explicitly cover general-chat redirect, clarification, WorkItem upsert, accepted gate, PRD creation, and audit write.
- [changes] File scope keeps the implementation centered on `projects/cue/backend` and limits `projects/cue/be` to transitional preservation.
- [tests] Test entries cover the acceptance criteria, including general-chat redirect, WorkItem creation, PRD rejection for non-accepted WorkItems, accepted WorkItem PRD creation, and Artifact Studio backend consumption.
