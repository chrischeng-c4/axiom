---
id: aw-core-client-model
summary: "Define the client-independent AW Core concept model shared by aw CLI, Cue, and future clients."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This spec defines the AW Core nouns, relationships, and client-independent invariants."
---

# AW Core Client Model

## Overview
<!-- type: overview lang: markdown -->

AW Core is the client-independent workflow model under Agentic Workflow clients.
It defines the durable nouns, relationships, and invariants that every client
must preserve. `aw CLI`, Cue, and future clients can expose different UX, but
they share the same core protocol.

Capability alignment: this spec covers the `aw-core-concept-model-and-invariants`
gap under the `aw-core-client-model-workitem-first-artifact-lifecycle`
capability root.

The core model is intentionally not a CLI command taxonomy and not a Cue product
taxonomy. CLI commands, JSON envelopes, branch/worktree mechanics, and terminal
prompts are client adapters over AW Core. Cue screens, team collaboration,
approval queues, registries, and runtime governance are also client adapters
over AW Core. Neither client surface may redefine the core nouns.

## Schema
<!-- type: schema lang: yaml -->

```yaml
concepts:
  project:
    definition: "Governance and rollup root for one product or repo-side workflow scope."
    owns:
      - capability_map
      - issue_backend_projection
      - repo_owned_scopes
      - verification_inventory
    invariant: "Every capability, WorkItem, artifact, gate, and evidence item rolls up to exactly one Project context."

  capability:
    definition: "Verifiable product promise inside a Project."
    owns:
      - gaps
      - claims
      - verification_contract
      - work_roots
    invariant: "A Capability is verified only through closed/non-deferred work roots plus required gates or accepted evidence."

  work_item:
    definition: "Accepted pre-plan and admission root for artifact work."
    owns:
      - problem
      - capability_alignment
      - requirements
      - acceptance_criteria
      - agent_estimate
      - target_artifacts
    invariant: "Artifact work starts from an accepted WorkItem, not from an unbounded raw prompt."

  artifact:
    definition: "Durable output created or revised under a WorkItem."
    examples:
      - prd
      - tech_design
      - code_artifact
      - app_spec
      - workflow_spec
      - policy
      - test_plan
      - release_package
    invariant: "Every Artifact records its governing WorkItem, target route, gates, and evidence."

  gate:
    definition: "Executable or manual condition that must pass before rollup may advance."
    examples:
      - spec_validation
      - tests
      - capability_check
      - review
      - cold_verify
      - human_approval
    invariant: "Gate outcome is explicit: pass, fail, blocked, skipped_by_policy, or not_applicable."

  evidence:
    definition: "Durable proof that a gate, artifact, claim, or rollup decision is valid."
    examples:
      - command_output
      - review_verdict
      - issue_comment
      - commit
      - generated_report
      - fixture_result
    invariant: "Verified state cannot rely on agent memory; it must cite evidence."

  hitl:
    definition: "Human-in-the-loop decision point for ambiguous, risky, or policy-sensitive workflow transitions."
    invariant: "HITL blocks rollup until the human decision is captured as evidence."

  rollup:
    definition: "Propagation of child state to parent WorkItem, Capability, and Project roots."
    invariant: "A parent can complete only when every required child is complete, deferred by policy, or blocked with explicit HITL/evidence."

  client:
    definition: "UX or automation surface that reads and writes AW Core state through the shared protocol."
    examples:
      - aw_cli
      - cue
      - future_api_client
    invariant: "Clients may specialize interaction and persistence adapters, but must not fork AW Core semantics."

relationships:
  - "Project contains Capabilities."
  - "Capability gaps become WorkItems or parent epics."
  - "Accepted WorkItems admit Artifacts through target artifact routes."
  - "Artifacts declare Gates."
  - "Gate results produce Evidence."
  - "Evidence enables Rollup."
  - "Clients operate on the same AW Core state and cannot redefine it."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-core-client-model
scenarios:
  - id: S1
    title: "CLI and Cue share core concepts"
    given:
      - "a WorkItem has accepted capability alignment and target artifacts"
    when:
      - "aw CLI emits a JSON envelope"
      - "Cue renders the same WorkItem in a collaborative web view"
    then:
      - "both clients use the same Project, Capability, WorkItem, Artifact, Gate, Evidence, HITL, Rollup, and Client meanings"
      - "client-specific UX does not alter the AW Core state model"

  - id: S2
    title: "project rollup stays client-independent"
    given:
      - "an Artifact passes its gates and records evidence"
    when:
      - "the workflow rolls up through WorkItem and Capability"
    then:
      - "Project readiness is computed from core state"
      - "the result does not depend on whether the action came from aw CLI or Cue"

  - id: S3
    title: "raw prompt cannot become core artifact state"
    given:
      - "a user asks a client to create an artifact from an unbounded prompt"
    when:
      - "no accepted WorkItem exists"
    then:
      - "AW Core has no Artifact admission root"
      - "the client must create or select a WorkItem before artifact work proceeds"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md
    action: create
    section: overview
    impl_mode: hand-written
    description: |
      Define the canonical AW Core concept model and client-independent invariants.
  - path: projects/agentic-workflow/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Point the project README at the AW Core concept model so clients share one vocabulary.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
