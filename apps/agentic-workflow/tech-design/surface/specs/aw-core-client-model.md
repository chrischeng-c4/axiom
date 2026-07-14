---
id: aw-core-client-model
summary: "Define the single agent-first AW project-iteration concept model."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "This spec defines AW project-iteration nouns, relationships, responsibilities, and invariants."
---

# AW Agent-First Project Iteration Model

## Overview
<!-- type: overview lang: markdown -->

Agentic Workflow (`aw`) is an agent-first project-iteration CLI for coding
agents. It owns next-action guidance, durable artifact skeletons, strict format
and phase validation, and code generation.

The public model contains Project, Capability, WorkItem, Artifact, Gate,
Evidence, and Rollup. The command-line surface is the product boundary. Each
bounded iteration starts from a WorkItem, consumes AW-produced artifact
skeletons, accepts agent-authored content only through declared fill slots, and
advances only after deterministic validation or explicit HITL.

Capability alignment: this spec covers the
`core-concept-model-and-invariants` gap under the stable
`aw-core-client-model-workitem-first-artifact-lifecycle` capability identity.
That identity is retained for traceability; it does not define a separate
product architecture.

## Schema
<!-- type: schema lang: yaml -->

```yaml
product:
  name: aw
  form: agent_first_project_iteration_cli
  primary_user: coding_agent
  owns:
    - next_action_guidance
    - durable_artifact_skeletons
    - strict_format_validation
    - strict_phase_transitions
    - code_generation

concepts:
  project:
    definition: "Repository-side product scope and parent rollup root."
    owns: [meta_docs, capability_map, work_inventory, verification_inventory]
    invariant: "Every capability, WorkItem, artifact, gate, and evidence item resolves to one Project context."

  capability:
    definition: "Verifiable product promise declared in the META-doc goal contract."
    owns: [gaps, claims, work_roots, verification_contract]
    invariant: "A Capability is verified only by closed work roots and required evidence."

  work_item:
    definition: "Bounded iteration root and durable loop state."
    owns: [problem, capability_alignment, scope, acceptance_criteria, reference_context]
    invariant: "Durable artifact work starts from an accepted, bounded WorkItem."

  artifact:
    definition: "Durable WI, EC, TD, generated-code, or evidence output produced from an AW skeleton."
    owns: [identity, skeleton, fill_slots, validation, generation, evidence, next_transition]
    invariant: "An agent fills only declared slots; it does not invent a competing durable format."

  gate:
    definition: "Deterministic verifier or explicit HITL condition controlling a transition."
    outcomes: [pass, fail, blocked, requires_hitl, not_applicable]
    invariant: "A phase advances only from an explicit gate result."

  evidence:
    definition: "Durable proof supporting a gate, claim, artifact, or rollup decision."
    examples: [command_output, ec_result, issue_comment, commit, generated_report]
    invariant: "Verified state cites evidence and never relies on agent memory."

  rollup:
    definition: "Propagation of child state to WorkItem, Capability, and Project roots."
    invariant: "A parent completes only when every required child is complete or explicitly blocked for HITL."

relationships:
  - "Project META-docs declare Capabilities."
  - "Capability gaps become bounded WorkItems."
  - "Accepted WorkItems admit AW-produced Artifact skeletons."
  - "Agents fill declared Artifact slots."
  - "Gates evaluate Artifacts and record Evidence."
  - "Evidence enables Rollup."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-agent-first-project-iteration-model
scenarios:
  - id: S1
    title: "AW owns the next action"
    given:
      - "a coding agent invokes a root runner for a bounded WorkItem"
    when:
      - "the current lifecycle state is evaluated"
    then:
      - "stdout contains exactly one runnable next command or an explicit terminal/HITL marker"
      - "the agent does not invent a hidden transition"

  - id: S2
    title: "AW owns durable artifact shape"
    given:
      - "a WorkItem needs an EC or TD artifact"
    when:
      - "the agent starts authoring"
    then:
      - "AW creates the durable skeleton first"
      - "stdout names the bounded fill slot and payload contract"
      - "out-of-slot content fails before phase advancement"

  - id: S3
    title: "verification drives rollup"
    given:
      - "an implementation artifact has been generated or filled"
    when:
      - "its required gate runs"
    then:
      - "pass records evidence and permits parent rollup"
      - "fail returns to a bounded implementation action"
      - "ambiguity emits HITL instead of guessing"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tech-design/surface/specs/aw-core-client-model.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Replace the multi-surface ontology with the single agent-first CLI model
      while preserving the stable spec and capability identities for traceability.
  - path: apps/agentic-workflow/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Publish the canonical product definition and seven public nouns.
  - path: apps/agentic-workflow/CAPABILITIES.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Register #1496 as the verified agent-first CLI product-model work root.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."
```
