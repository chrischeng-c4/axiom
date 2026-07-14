---
id: aw-client-boundaries
summary: "Define the single agent-first AW CLI product boundary."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: agent-first-cli-product-model
    claim: agent-first-cli-product-model
    coverage: full
    rationale: "This spec defines one coding-agent CLI product boundary and its four owned responsibilities."
---

# AW Agent-First CLI Product Boundary

## Overview
<!-- type: overview lang: markdown -->

Agentic Workflow (`aw`) is an agent-first project-iteration CLI for coding
agents. It owns next-action guidance, durable artifact skeletons, strict format
and phase validation, and code generation.

The CLI is the product. Repo-local commands, machine-readable envelopes,
project META-docs, WorkItem state, EC verification, TD/codegen, scoped lifecycle
commits, and evidence rollup all belong to this one boundary. AW does not own a
parallel collaboration product, a general-purpose UI, or an alternate protocol
whose state can diverge from CLI-observable workflow state.

## Schema
<!-- type: schema lang: yaml -->

```yaml
product_boundary:
  name: aw_cli
  primary_user: coding_agent
  environment: repository_checkout
  owns:
    - next_action_guidance
    - artifact_skeleton_and_fill_contracts
    - format_and_phase_validation
    - td_codegen
    - ec_verification
    - evidence_rollup
  observable_contract:
    - stdout_next_command_or_terminal_marker
    - aw_cli_v1_envelope
    - repo_and_project_meta_docs
    - work_item_state
    - generated_and_handwritten_ownership_markers
  excluded_product_surfaces:
    - parallel_collaboration_application
    - general_purpose_repo_ui
    - alternate_workflow_protocol
    - hidden_lifecycle_state

boundary_invariants:
  - "Every mutating workflow transition is observable through the AW CLI contract."
  - "AW creates each supported durable artifact skeleton before an agent fills it."
  - "An invalid format or phase fails before durable state advances."
  - "Generated implementation remains traceable to TD and EC evidence."
  - "A new product surface requires its own product and must not redefine AW workflow semantics."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-agent-first-cli-product-boundary
scenarios:
  - id: S1
    title: "coding agent follows one CLI contract"
    given:
      - "a coding agent is iterating on a registered project"
    when:
      - "it invokes a root or worker command"
    then:
      - "stdout names the only next action"
      - "all durable state remains inspectable through repository artifacts and configured issue state"

  - id: S2
    title: "artifact creation stays CLI-owned"
    given:
      - "a supported durable artifact does not exist"
    when:
      - "the workflow reaches its authoring phase"
    then:
      - "AW emits the canonical skeleton"
      - "the agent receives a bounded fill contract"
      - "freehand competing formats are rejected"

  - id: S3
    title: "unowned product surface is excluded"
    given:
      - "a feature does not participate in META-doc to WI to EC to TD/codegen iteration"
    when:
      - "its ownership is evaluated"
    then:
      - "it is not advertised as an AW product capability"
      - "its implementation is removed or moved under a separately owned product"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Rewrite the historical boundary spec as the single agent-first CLI
      product boundary while retaining its stable path and id for traceability.
  - path: apps/agentic-workflow/src/cli/llm.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Add a binary-owned product-model topic and a regression that rejects the
      removed architecture from active product contracts.
  - path: apps/agentic-workflow/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Keep the canonical product definition aligned with binary orientation.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."
```
