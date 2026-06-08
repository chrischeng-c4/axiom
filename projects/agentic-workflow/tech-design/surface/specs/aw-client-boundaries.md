---
id: aw-client-boundaries
summary: "Define aw CLI and Cue as separate clients over AW Core."
fill_sections: [overview, schema, scenarios, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "This spec defines aw CLI and Cue as separate clients over AW Core."
---

# AW Client Boundaries

## Overview
<!-- type: overview lang: markdown -->

AW Core is the shared workflow model. `aw CLI` and Cue are clients over that
model, not layers inside each other.

`aw CLI` is the standalone developer and agentic coding client. It is optimized
for repo-local commands, machine-readable JSON envelopes, TD/CB lifecycle
execution, hooks, branches, worktrees, and coding-agent prompts.

Cue is the enterprise team collaboration web frontend/backend client. It is
optimized for Project Sessions, WorkItem collaboration, artifact graphs,
approvals, registry state, audit, ownership, runtime governance, and hidden app
repo orchestration.

Cue is not an `aw CLI` wrapper. It can share AW Core semantics and may call
backend services that implement AW Core protocol, but its product architecture
must not be a shell-out layer around terminal commands. Likewise, `aw CLI`
must not absorb Cue-only team collaboration UX into repo-local command verbs.

## Schema
<!-- type: schema lang: yaml -->

```yaml
shared_layer:
  name: aw_core
  owns:
    - project
    - capability
    - work_item
    - artifact
    - gate
    - evidence
    - hitl
    - rollup
    - client_contract
  invariant: "Core semantics are client-independent."

clients:
  aw_cli:
    definition: "Standalone developer and agentic coding client over AW Core."
    primary_users:
      - standalone_developer
      - coding_agent
      - repo_maintainer
    owns_ux:
      - repo_local_commands
      - json_envelopes
      - invoke_commands
      - td_lifecycle
      - cb_lifecycle
      - worktrees
      - branches
      - hooks
      - scoped_lifecycle_commits
    must_not_own:
      - enterprise_team_workspace
      - runtime_registry_ui
      - approval_queue_ui
      - multi_user_artifact_studio
      - generated_app_control_plane

  cue:
    definition: "Enterprise team collaboration web frontend/backend client over AW Core."
    primary_users:
      - app_owner
      - project_owner
      - platform_admin
      - governed_agent_team
    owns_ux:
      - project_sessions
      - work_item_collaboration
      - artifact_graph_review
      - prd_and_td_artifacts
      - app_spec_and_runtime_artifacts
      - approvals
      - registry
      - audit
      - ownership
      - runtime_governance
      - hidden_app_repo_orchestration
    must_not_own:
      - terminal_first_end_user_experience
      - visible_git_workflow_for_business_users
      - shell_wrapper_as_product_architecture
      - cli_command_taxonomy_as_ui_model

boundary_invariants:
  - "Both clients preserve AW Core WorkItem-first artifact admission."
  - "Both clients preserve AW Core evidence-backed rollup."
  - "Client-specific UX may add fields or views, but must map back to AW Core concepts."
  - "Cue is not an aw CLI wrapper."
  - "aw CLI is not the enterprise collaboration surface."
  - "AW Core must not depend on CLI-only commands or Cue-only screens."
```

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-client-boundaries
scenarios:
  - id: S1
    title: "same WorkItem, different clients"
    given:
      - "a WorkItem is accepted under AW Core"
    when:
      - "aw CLI advances TD/CB from the repo"
      - "Cue displays the same WorkItem as an artifact graph"
    then:
      - "both clients preserve the same WorkItem identity and evidence rollup"
      - "CLI-specific branch/worktree state does not become a Cue UI requirement"
      - "Cue-specific collaboration state does not become a CLI command requirement"

  - id: S2
    title: "Cue is not an aw CLI wrapper"
    given:
      - "Cue needs to create or revise a PRD, TD, app spec, or runtime artifact"
    when:
      - "Cue invokes AW Core behavior"
    then:
      - "Cue uses backend services or protocol adapters for AW Core state"
      - "Cue does not model its product as shelling out to terminal commands"
      - "business users never need to understand Git, branches, worktrees, or CLI prompts"

  - id: S3
    title: "aw CLI remains repo-local"
    given:
      - "a coding agent works inside a project checkout"
    when:
      - "`aw run`, `aw wi`, `aw td`, or `aw cb` emits next steps"
    then:
      - "the CLI stays focused on repo-local lifecycle execution"
      - "enterprise approval queues, registry dashboards, and runtime ownership screens remain Cue surfaces"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/surface/specs/aw-client-boundaries.md
    action: create
    section: overview
    impl_mode: hand-written
    description: |
      Define aw CLI and Cue client boundaries over AW Core.
  - path: projects/agentic-workflow/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Point Agentic Workflow readers at the client boundary contract.
  - path: projects/cue/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: |
      Clarify that Cue is an AW Core web/backend client, not an aw CLI wrapper.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```
