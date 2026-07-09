---
id: preview-ci-template-lifecycle
summary: >
  Add copyable local-first CI/CD lifecycle templates for Preview. The templates
  cover MR open/update/rerun and close cleanup for GitHub Actions, GitLab CI,
  and a local kind script while keeping GKE-specific values parameterized.
capability_refs:
  - id: "preview-external-contracts"
    role: primary
    gap: "ci-template-lifecycle"
    claim: "ci-template-lifecycle"
    coverage: partial
    rationale: >
      Work item #1112 makes the proven local lifecycle copyable before teams
      start a real GKE pilot.
fill_sections: [logic, schema, cli, unit-test, e2e-test, changes]
---

# TD: Preview CI Template Lifecycle

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: preview-ci-template-lifecycle-flow
entry: preview_ci_templates
nodes:
  open_update: { kind: start, label: "MR opened/updated/manual rerun" }
  render_apply: { kind: process, label: "discover-base -> render -> apply plan/dry-run/apply" }
  validate: { kind: process, label: "rollout -> router resolve -> comment" }
  close: { kind: start, label: "MR closed/merged" }
  cleanup: { kind: process, label: "cleanup plan -> cleanup apply" }
edges:
  - { from: open_update, to: render_apply }
  - { from: render_apply, to: validate }
  - { from: close, to: cleanup }
---
flowchart TD
    open_update([Open/update/rerun]) --> render_apply[Discover, render, plan, dry-run, apply]
    render_apply --> validate[Rollout, router resolve, comment]
    close([Close/merge]) --> cleanup[Cleanup plan and apply]
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
required_variables:
  - PREVIEW_MR
  - PREVIEW_SHA
  - PREVIEW_IMAGE
  - PREVIEW_APP
  - PREVIEW_HOST
  - PREVIEW_BASE_NAMESPACE
  - PREVIEW_CONTEXT
  - PREVIEW_TTL_HOURS
templates:
  - docs/ci-templates/github-actions-preview.yaml
  - docs/ci-templates/gitlab-ci-preview.yml
  - docs/ci-templates/local-kind-lifecycle.sh
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  open_update_rerun:
    - preview discover-base
    - preview render
    - preview apply --plan-only
    - preview apply --dry-run
    - preview apply
    - kubectl rollout status
    - preview router resolve
    - preview comment
  close_merge:
    - preview cleanup plan
    - preview cleanup apply
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: preview-ci-template-lifecycle-unit-tests
requirements:
  template_variables:
    id: R1
    text: "All templates define required PREVIEW_* variables."
    kind: behavior
    risk: medium
    verify: "cargo test -p preview --test local_cicd_contract ci_templates_document_required_variables_and_command_order"
  template_order:
    id: R2
    text: "All templates preserve open/update and close cleanup command order."
    kind: behavior
    risk: medium
    verify: "cargo test -p preview --test local_cicd_contract ci_templates_document_required_variables_and_command_order"
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "required variables"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "command order"
      risk: medium
      verifymethod: test
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: preview-kind-template-path
    command: "PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture"
    assertions:
      - "kind lifecycle covers the documented local sequence from base discovery through cleanup apply."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "apps/preview/docs/ci-templates/README.md"
    action: add
    section: logic
    description: "Document the open/update/rerun and close cleanup lifecycle flow."
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/README.md"
    action: add
    section: schema
    description: "Document required variables and local-first lifecycle order."
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/github-actions-preview.yaml"
    action: add
    section: cli
    description: "Provide copyable GitHub Actions preview lifecycle."
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/gitlab-ci-preview.yml"
    action: add
    section: cli
    description: "Provide copyable GitLab CI preview lifecycle."
    impl_mode: hand-written
  - path: "apps/preview/docs/ci-templates/local-kind-lifecycle.sh"
    action: add
    section: cli
    description: "Provide local kind lifecycle script matching the tested path."
    impl_mode: hand-written
  - path: "apps/preview/tests/local_cicd_contract.rs"
    action: modify
    section: unit-test
    description: "Validate required variables and command order across templates."
    impl_mode: hand-written
```
