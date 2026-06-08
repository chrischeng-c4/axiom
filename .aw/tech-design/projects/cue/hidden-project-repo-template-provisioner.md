---
id: cue-hidden-project-repo-template-provisioner
summary: Hidden GitLab project template and provisioner contract for generated app artifact repositories.
fill_sections: [schema, logic, scenarios, changes, tests]
---

# Hidden Project Repo Template Provisioner

Status: implemented

## Template Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "https://cclab.dev/cue/hidden-project-repo-template/v0"
title: Cue Hidden Project Repo Template v0
type: object
additionalProperties: false
required: [template_files, provision_request, provision_result]
properties:
  template_files:
    type: array
    items:
      enum: [app-spec.json, policy.json, permissions.json, connectors.json, tests, generated, releases, gitlab-ci]
  provision_request:
    type: object
    required: [app_id, namespace, owner, visibility]
    properties:
      app_id: { type: string }
      namespace: { enum: [personal, team, cross_team, platform] }
      owner: { type: string }
      visibility: { const: hidden }
  provision_result:
    type: object
    required: [gitlab_project_id, full_path, default_branch, current_spec_ref]
    properties:
      gitlab_project_id: { type: integer }
      full_path: { type: string }
      default_branch: { const: main }
      current_spec_ref: { type: string }
```

## Provision Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cue-hidden-repo-provisioner-logic
entry: ProvisionRequested
nodes:
  ProvisionRequested: { kind: start, label: provision requested }
  DerivePath: { kind: process, label: derive GitLab path from ownership namespace }
  CheckCollision: { kind: decision, label: path exists? }
  ResolveCollision: { kind: process, label: append short hash or reject immutable id }
  CreateProject: { kind: process, label: create hidden GitLab project }
  ApplyTemplate: { kind: process, label: write template files }
  CommitInitialSpec: { kind: process, label: commit app-spec and policy }
  UpdateRegistry: { kind: process, label: store project mapping }
  Done: { kind: terminal, label: repo provisioned }
edges:
  - { from: ProvisionRequested, to: DerivePath, label: request }
  - { from: DerivePath, to: CheckCollision, label: path }
  - { from: CheckCollision, to: ResolveCollision, label: yes }
  - { from: CheckCollision, to: CreateProject, label: no }
  - { from: ResolveCollision, to: CreateProject, label: resolved }
  - { from: CreateProject, to: ApplyTemplate, label: project id }
  - { from: ApplyTemplate, to: CommitInitialSpec, label: files written }
  - { from: CommitInitialSpec, to: UpdateRegistry, label: ref }
  - { from: UpdateRegistry, to: Done, label: registry saved }
---
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: fixture_provisioner_contract
    given: [GitLab adapter is fixture-backed]
    when: [provision request is submitted]
    then: [registry stores hidden project mapping and template refs]
  - id: path_collision
    given: [derived path already exists before sandbox deploy]
    when: [provision request is submitted]
    then: [short hash collision policy resolves or rejects deterministically]
  - id: user_never_sees_gitlab
    given: [project owner loads Artifact Studio]
    when: [repo is provisioned]
    then: [UI shows version and health, not GitLab project internals]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cue/app-repo-template/
    action: create
    impl_mode: hand-written
    description: Hidden generated-app repo template with app spec, policy, permissions, connectors, tests, generated, releases, and CI metadata.
  - path: projects/cue/backend/src/
    action: modify
    impl_mode: hand-written
    description: Add provisioner interface, fixture adapter, GitLab adapter boundary, and registry mapping writer.
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  fixture_provisioner_creates_mapping:
    kind: unit
    verifies: [fixture project id, path, default branch, current spec ref]
  template_contains_required_files:
    kind: inspection
    verifies: [app-spec, policy, permissions, connectors, tests, generated, releases, ci]
  owner_ui_hides_gitlab:
    kind: browser
    verifies: [Artifact Studio does not expose GitLab project internals]
```
