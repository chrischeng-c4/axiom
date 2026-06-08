---
id: aw-workitem-artifact-gate
summary: "Define accepted WorkItem admission and artifact routes for AW Core artifact creation."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: workitem-artifact-admission-gate
    claim: workitem-artifact-admission-gate
    coverage: full
    rationale: "This spec defines the WorkItem-first artifact admission gate and route model."
---

# AW WorkItem Artifact Gate

## Overview
<!-- type: overview lang: markdown -->

AW Core artifact work is WorkItem-first. A client may help a user draft or
select a WorkItem, but it must not create a durable artifact from an unbounded
raw prompt. The invariant is:

> No artifact before accepted WorkItem.

An accepted WorkItem is the admission root for artifact creation. It carries the
bounded problem, capability alignment, requirements, acceptance criteria, agent
estimate, unresolved HITL state, and target artifact routes. The requested
artifact type must be allowed by one of those routes before a client can create
or revise that artifact.

The existing `aw wi -> aw td -> aw cb -> aw td merge` path is the code route
inside this generalized model. It is not a special exception; it is one route
where WorkItem admission permits TD authoring and code artifact generation.

## Schema
<!-- type: schema lang: yaml -->

```yaml
accepted_work_item:
  required_fields:
    - problem
    - capability_alignment
    - requirements
    - acceptance_criteria
    - agent_estimate
    - target_artifact_routes
  required_state:
    bounded: true
    split_required: false
    unresolved_hitl: false
    accepted_by_policy: true
  canonical_root:
    issue_platform: "canonical WorkItem identity and accepted state"
    local_artifacts: "draft and planning intermediates before publication"

admission_rule:
  invariant: "No artifact before accepted WorkItem."
  request_checks:
    - "A WorkItem exists and is accepted."
    - "The requested artifact type is allowed by target_artifact_routes."
    - "The route-specific gate can run or returns HITL/blocked with evidence."
  blocked_cases:
    raw_prompt_without_work_item: "client must create or select a WorkItem first"
    unaccepted_work_item: "client must complete acceptance gates first"
    route_mismatch: "client must revise WorkItem target routes or request an allowed artifact"

artifact_routes:
  prd:
    artifacts: [prd]
    admission: "accepted WorkItem targets product requirement output"
    first_gate: "prd_scope_review"
  td:
    artifacts: [tech_design]
    admission: "accepted WorkItem targets technical design"
    first_gate: "td_create_or_validate"
  cb_code:
    artifacts: [code_artifact]
    admission: "accepted WorkItem has a TD-approved code route"
    first_gate: "cb_gen_or_claim"
  app_spec:
    artifacts: [app_spec]
    admission: "accepted WorkItem targets generated or governed app specification"
    first_gate: "app_spec_validation"
  workflow_spec:
    artifacts: [workflow_spec]
    admission: "accepted WorkItem targets workflow or automation specification"
    first_gate: "workflow_spec_validation"
  policy:
    artifacts: [policy]
    admission: "accepted WorkItem targets governance or operational policy"
    first_gate: "policy_review"
  test_plan:
    artifacts: [test_plan]
    admission: "accepted WorkItem targets validation inventory or test plan"
    first_gate: "test_plan_review"
  release_package:
    artifacts: [release_package]
    admission: "accepted WorkItem targets release packaging or rollout evidence"
    first_gate: "release_gate_review"

route_invariants:
  - "A route is declared on the WorkItem before artifact creation."
  - "A route owns its artifact type vocabulary and first gate."
  - "Clients may present route-specific UX, but route admission is AW Core state."
  - "TD/CB remains the code route: WorkItem admission -> TD -> CB/code -> merge evidence."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-workitem-artifact-gate
scenarios:
  - id: S1
    title: "raw prompt artifact request is blocked"
    given:
      - "a user asks a client to create a TD from a raw prompt"
      - "no accepted WorkItem exists"
    when:
      - "the client asks AW Core for artifact admission"
    then:
      - "AW Core denies artifact creation"
      - "the next action is create_or_select_work_item"
      - "no TD, PRD, CB, App Spec, Workflow Spec, Policy, Test Plan, or Release Package is persisted"

  - id: S2
    title: "requested artifact must match WorkItem route"
    given:
      - "an accepted WorkItem targets only the test_plan route"
    when:
      - "a client requests code_artifact creation"
    then:
      - "AW Core denies route admission"
      - "the WorkItem must be revised before code artifact work proceeds"

  - id: S3
    title: "TD and CB stay one generalized route"
    given:
      - "an accepted WorkItem targets the td and cb_code routes"
    when:
      - "`aw CLI` starts TD and then CB"
    then:
      - "TD creation is admitted by the WorkItem"
      - "CB/code creation is admitted by the TD-approved code route"
      - "merge evidence rolls up through the same WorkItem root"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/surface/specs/aw-workitem-artifact-gate.md
    action: create
    section: overview
    impl_mode: hand-written
    description: |
      Define the accepted WorkItem admission gate and artifact route model.
  - path: projects/agentic-workflow/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Document the WorkItem-first artifact admission invariant in the README.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
