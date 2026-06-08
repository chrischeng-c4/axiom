---
id: cue-full-product-e2e-todo-app
summary: Local full-product e2e for create project to todo app delivery through WorkItem, PRD, TD, codebase, test, deployment, and operation dashboard stages.
fill_sections: [schema, logic, scenarios, changes, tests]
---

# Full Product E2E Todo App

This spec defines the first end-to-end Cue product scenario that spans project
creation, hidden repo provisioning, WorkItem creation, stage artifacts, local
CI/deployment evidence, and project operations dashboard state.

The e2e simulates GitLab and GCP locally. GitLab is represented by a hidden repo
folder inside a temporary e2e workspace. GCP is represented by deployment and
operation JSON evidence. The scenario must keep the owner-facing state free of
GitLab, GCP, Terraform, Kustomize, branch, commit, and CI implementation terms.

## Scenario Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/full-product-e2e-todo-app/v0"
title: Cue Full Product E2E Todo App v0
type: object
additionalProperties: false
required: [project, hidden_repo, workitem, stages, operations_dashboard]
properties:
  project:
    type: object
    additionalProperties: false
    required: [id, name, owner_visible_summary, admin_evidence]
    properties:
      id: { const: todo-app-project }
      name: { const: Todo App Project }
      owner_visible_summary:
        type: string
        not:
          pattern: "(GitLab|GCP|Terraform|Kustomize|CI|branch|commit)"
      admin_evidence:
        type: object
        additionalProperties: true
  hidden_repo:
    type: object
    additionalProperties: false
    required: [path, files]
    properties:
      path: { type: string }
      files:
        type: array
        items:
          enum:
            - prd.md
            - td.md
            - app-spec.json
            - src/todo-app.ts
            - tests/todo-app.test.json
            - .gitlab-ci.yml
            - deploy/kustomize/base/deployment.yaml
            - deploy/kustomize/overlays/sandbox/kustomization.yaml
            - deploy/kustomize/overlays/production/kustomization.yaml
            - infra/terraform/main.tf
            - releases/sandbox.json
            - operations/dashboard.json
  workitem:
    type: object
    additionalProperties: false
    required: [id, goal, status, current_stage_id]
    properties:
      id: { const: todo-app-workitem }
      goal: { const: Create a governed todo app }
      status: { enum: [ready, in_progress, done] }
      current_stage_id: { enum: [prd, td, codebase, test, deployment, operation] }
  stages:
    type: array
    minItems: 6
    items:
      type: object
      additionalProperties: false
      required: [id, status, artifact, agent_role, agent_label]
      properties:
        id: { enum: [prd, td, codebase, test, deployment, operation] }
        status: { const: done }
        artifact: { type: string }
        agent_role: { enum: [pm, architect, dev, qa_policy, release, data] }
        agent_label: { type: string }
  operations_dashboard:
    type: object
    additionalProperties: false
    required: [health, latest_release, ci_status, deployment_environment, open_blockers, runtime_metrics]
    properties:
      health: { const: ok }
      latest_release: { const: sandbox-v1 }
      ci_status: { const: passed }
      deployment_environment: { const: sandbox }
      open_blockers:
        type: array
        maxItems: 0
      runtime_metrics:
        type: object
        required: [todo_count, completed_count, error_rate]
        properties:
          todo_count: { type: integer, minimum: 0 }
          completed_count: { type: integer, minimum: 0 }
          error_rate: { type: number, maximum: 0 }
```

## Delivery Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-full-product-e2e-todo-app-logic
entry: CreateProject
nodes:
  CreateProject: { kind: start, label: create Cue project }
  ProvisionHiddenRepo: { kind: process, label: provision local hidden repo folder }
  CreateWorkItem: { kind: process, label: create todo app WorkItem }
  CreatePrd: { kind: process, label: PM agent writes PRD artifact }
  CreateTd: { kind: process, label: Architect agent writes TD artifact }
  CreateCodebase: { kind: process, label: Dev agent writes app code CI Kustomize Terraform }
  RunTests: { kind: process, label: QA policy agent writes local CI and test evidence }
  DeploySandbox: { kind: process, label: Release agent writes sandbox release evidence }
  UpdateOperations: { kind: process, label: Data agent writes operations dashboard }
  VerifyOwnerSurface: { kind: decision, label: owner copy hides infrastructure terms }
  Done: { kind: terminal, label: e2e passed }
edges:
  - { from: CreateProject, to: ProvisionHiddenRepo, label: project_id }
  - { from: ProvisionHiddenRepo, to: CreateWorkItem, label: repo_ready }
  - { from: CreateWorkItem, to: CreatePrd, label: workitem_ready }
  - { from: CreatePrd, to: CreateTd, label: prd_done }
  - { from: CreateTd, to: CreateCodebase, label: td_done }
  - { from: CreateCodebase, to: RunTests, label: codebase_done }
  - { from: RunTests, to: DeploySandbox, label: tests_passed }
  - { from: DeploySandbox, to: UpdateOperations, label: sandbox_ready }
  - { from: UpdateOperations, to: VerifyOwnerSurface, label: dashboard_ready }
  - { from: VerifyOwnerSurface, to: Done, label: pass }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: full_product_todo_app_happy_path
    given:
      - Cue full product e2e starts with an empty temporary workspace
    when:
      - project Todo App Project is created
      - hidden repo is provisioned as a local folder
      - owner prompt asks for a todo app
      - Cue advances PRD TD codebase test deployment and operation stages
    then:
      - local hidden repo contains PRD TD app spec source tests CI Kustomize Terraform release and dashboard files
      - every WorkItem stage is done in dependency order
      - active workflow is rendered as a DAG graph with node cards and dependency edges
      - desktop layout gives the right context pane more width than the conversation column
      - every WorkItem stage declares a distinct node agent role
      - PRD uses PM agent, TD uses Architect agent, Codebase uses Dev agent, Test uses QA/policy agent, Deployment uses Release agent, and Operation uses Data agent
      - operations dashboard reports health ok ci passed sandbox release and zero open blockers
      - owner-facing project summary hides GitLab GCP Terraform Kustomize CI branch and commit terms
      - admin evidence exposes the hidden repo path and deployment evidence
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cue/artifact-studio/src/api.ts
    action: modify
    impl_mode: hand-written
    description: |
      Add the create-project client call and operations dashboard response
      fields needed by the browser product path, including per-node workflow
      agent role metadata.
  - path: projects/cue/artifact-studio/src/App.tsx
    action: modify
    impl_mode: hand-written
    description: |
      Wire the New Project button to create a Todo App Project and show the
      project operations dashboard after the operation stage completes. Show
      each workflow node's responsible agent in an Airflow-like workflow graph
      and keep the stage list as a compact progress summary.
  - path: projects/cue/artifact-studio/e2e/workitems.e2e.mjs
    action: modify
    impl_mode: hand-written
    description: |
      Add a browser e2e that clicks through New Project, WorkItem creation,
      PRD, TD, codebase, test, deployment, operation, operations dashboard, and
      debug evidence checks. Assert that every workflow node has a distinct
      agent role.
  - path: projects/cue/e2e/full-product.e2e.mjs
    action: create
    impl_mode: hand-written
    description: |
      Add a deterministic local harness for the same full-product artifact
      sequence and node-agent contract; browser e2e remains the primary product
      acceptance path.
  - path: projects/cue/package.json
    action: modify
    impl_mode: hand-written
    description: |
      Add a script for the full-product e2e and keep it available through the
      all-in-one Cue validation command.
  - path: projects/cue/scripts/test-all.mjs
    action: modify
    impl_mode: hand-written
    description: |
      Run the full-product e2e after backend contract tests, typecheck, and
      browser e2e.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  full_product_e2e:
    kind: browser
    command: cd projects/cue/artifact-studio && npm run test:e2e
    verifies:
      - user clicks New Project and creates Todo App Project
      - user prompt creates a WorkItem before downstream artifacts
      - user clicks PRD TD codebase test deployment and operation actions in order
      - workflow graph displays six node cards and five dependency edges
      - desktop viewport renders the right context pane wider than the conversation column
      - workflow plan displays a different agent for every node
      - local hidden repo folder is created
      - PRD TD codebase test deployment and operation artifacts exist
      - CI Kustomize and Terraform files are produced as codebase artifacts
      - operations dashboard is part of project state
      - owner-facing copy hides infrastructure internals
      - debug evidence exposes hidden repo files for platform inspection
      - debug evidence includes six distinct workflow node agent roles
  all_in_one_validation:
    kind: local
    command: cd projects/cue && npm run test:all
    verifies:
      - backend contracts
      - Artifact Studio typecheck
      - Artifact Studio browser e2e
      - full product e2e
```
