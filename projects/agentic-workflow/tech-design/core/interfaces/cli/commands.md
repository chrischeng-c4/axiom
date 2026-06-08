---
id: cli-commands-docs-phase
main_spec_ref: "crates/cclab-sdd/interfaces/cli/commands.md"
merge_strategy: new
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: cli-workflow-chain
    claim: cli-workflow-chain
    coverage: full
    rationale: "Command/root TDs support CLI workflow chain routing and root-runner dispatch."
---

# Cli Commands Docs Phase

## Overview
<!-- type: overview lang: markdown -->

Extend CLI commands spec with docs-phase action routing. No new top-level subcommands — docs-phase operations use the existing `cclab sdd workflow` and `cclab sdd artifact` dynamic dispatch.

**What**: Document 6 new action names routable through existing CLI dispatch and update the delegation guard mapping to permit docs-phase artifact actions.

**Why**: The artifact-tools-docs-phase spec defines 6 new MCP tools (`sdd_{workflow,artifact}_{create,review,revise}_change_docs`). These tools are invoked via CLI using `cclab sdd workflow <action>` / `cclab sdd artifact <action>` dynamic dispatch. The delegation guard function `is_artifact_permitted_under_guard` needs updating to map docs-phase workflow guards to their artifact counterparts.

| Aspect | Value |
|--------|-------|
| New subcommands | None — reuses existing `workflow` and `artifact` dispatch |
| New actions (workflow) | `create-change-docs`, `review-change-docs`, `revise-change-docs` |
| New actions (artifact) | `create-change-docs`, `review-change-docs`, `revise-change-docs` |
| Guard mapping | `create_change_docs` → `create_change_docs`, `revise_change_docs` → `create_change_docs` |
| Handler crate | `cclab-sdd-cli` (existing `commands.rs`) |
## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: cli-commands-docs-phase-requirements
title: CLI Commands Docs Phase Requirements
requirements:
  CR1:
    text: Update is_artifact_permitted_under_guard() to map docs-phase workflow guard actions to permitted artifact actions
    type: functional
    priority: high
    risk: medium
    verification: test
    notes: |
      Pattern: revise_change_docs delegates to create_change_docs artifact
      (same as implementation phase pattern).
  CR2:
    text: Map review_change_docs workflow guard to review_change_docs artifact action
    type: functional
    priority: high
    risk: low
    verification: test
  CR3:
    text: Update CLI to Logic Mapping table with 6 new action entries (3 workflow + 3 artifact) routing through ToolRegistry::call_tool()
    type: interface
    priority: high
    risk: low
    verification: inspection
  CR4:
    text: Update Command Tree documentation with new docs-phase action names under workflow and artifact subcommands
    type: interface
    priority: medium
    risk: low
    verification: inspection
  CR5:
    text: No new Commands enum variants needed — existing Workflow and Artifact variants handle all docs-phase actions via dynamic dispatch
    type: constraint
    priority: low
    risk: low
    verification: inspection
---
requirementDiagram
    requirement CR1 {
      id: CR1
      text: Map docs-phase workflow guards to permitted artifact actions
      risk: medium
      verifymethod: test
    }
    requirement CR2 {
      id: CR2
      text: Map review_change_docs workflow guard to review artifact
      risk: low
      verifymethod: test
    }
    requirement CR3 {
      id: CR3
      text: Update CLI to Logic Mapping table with 6 entries
      risk: low
      verifymethod: inspection
    }
    requirement CR4 {
      id: CR4
      text: Update Command Tree documentation
      risk: low
      verifymethod: inspection
    }
    requirement CR5 {
      id: CR5
      text: No new Commands enum variants
      risk: low
      verifymethod: inspection
    }
```
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  S1:
    name: Workflow create-change-docs dispatches via dynamic routing
    verifies: [CR3, CR5]
    given: |
      User runs `cclab sdd workflow create-change-docs 1145`
    when: |
      CLI constructs tool name `sdd_workflow_create_change_docs`
    then: |
      - ToolRegistry::call_tool("sdd_workflow_create_change_docs", args) is invoked
      - Result JSON is printed to stdout
  S2:
    name: Artifact create-change-docs permitted under create_change_docs guard
    verifies: [CR1]
    given: |
      STATE.yaml has `delegation_guard.action = "create_change_docs"`
    when: |
      Agent calls `cclab sdd artifact create-change-docs 1145 payload.json`
    then: |
      - is_artifact_permitted_under_guard("create_change_docs", "create_change_docs") returns true
      - Artifact tool executes successfully
  S3:
    name: Artifact create-change-docs permitted under revise_change_docs guard
    verifies: [CR1]
    given: |
      STATE.yaml has `delegation_guard.action = "revise_change_docs"`
    when: |
      Agent calls `cclab sdd artifact create-change-docs 1145 payload.json`
    then: |
      - is_artifact_permitted_under_guard("create_change_docs", "revise_change_docs") returns true
      - Artifact tool executes (revise delegates to create artifact)
  S4:
    name: Artifact review-change-docs permitted under review_change_docs guard
    verifies: [CR2]
    given: |
      STATE.yaml has `delegation_guard.action = "review_change_docs"`
    when: |
      Agent calls `cclab sdd artifact review-change-docs 1145 payload.json`
    then: |
      - is_artifact_permitted_under_guard("review_change_docs", "review_change_docs") returns true
      - Artifact tool executes successfully
  S5:
    name: Unrelated artifact action blocked by docs guard
    verifies: [CR1]
    given: |
      STATE.yaml has `delegation_guard.action = "create_change_docs"`
    when: |
      Agent calls `cclab sdd artifact create-change-implementation 1145 payload.json`
    then: |
      - Guard check fails (neither name match nor permitted-under mapping)
      - CLI returns error- "Action 'create-change-implementation' blocked by delegation guard"
  S6:
    name: Kebab-case to snake_case normalization
    verifies: [CR3]
    given: |
      User runs `cclab sdd workflow review-change-docs 1145`
    when: |
      CLI normalizes action name
    then: |
      - Tool name becomes `sdd_workflow_review_change_docs` (hyphens to underscores)
      - The tool is found in ToolRegistry
```
## Diagrams
<!-- type: diagram lang: mermaid -->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- score-td-placeholder -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- score-td-placeholder -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- score-td-placeholder -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- score-td-placeholder -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- score-td-placeholder -->

## API Spec
<!-- type: api lang: yaml -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- score-td-placeholder -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->

### Schema
<!-- type: schema lang: yaml -->
<!-- score-td-placeholder -->

### Config
<!-- type: config lang: yaml -->
<!-- score-td-placeholder -->

## Test Plan
<!-- type: test-plan lang: mermaid -->

```mermaid
---
id: cli-commands-docs-phase-test-plan
title: CLI Commands Docs Phase Test Plan
tests:
  T1:
    type: test
    name: test_workflow_create_change_docs_dispatch
    file: crates/cclab-sdd-cli/src/commands.rs
    verifies: [CR3, CR5]
  T2:
    type: test
    name: test_guard_create_change_docs_permits_create_artifact
    file: crates/cclab-sdd-cli/src/commands.rs
    verifies: [CR1]
  T3:
    type: test
    name: test_guard_revise_change_docs_permits_create_artifact
    file: crates/cclab-sdd-cli/src/commands.rs
    verifies: [CR1]
  T4:
    type: test
    name: test_guard_review_change_docs_permits_review_artifact
    file: crates/cclab-sdd-cli/src/commands.rs
    verifies: [CR2]
  T5:
    type: test
    name: test_guard_blocks_unrelated_artifact_action
    file: crates/cclab-sdd-cli/src/commands.rs
    verifies: [CR1, CR2]
  T6:
    type: test
    name: test_kebab_to_snake_normalization
    file: crates/cclab-sdd-cli/src/commands.rs
    verifies: [CR3]
---
requirementDiagram
    element T1 { type: test }
    element T2 { type: test }
    element T3 { type: test }
    element T4 { type: test }
    element T5 { type: test }
    element T6 { type: test }

    T1 - verifies -> CR3
    T1 - verifies -> CR5
    T2 - verifies -> CR1
    T3 - verifies -> CR1
    T4 - verifies -> CR2
    T5 - verifies -> CR1
    T5 - verifies -> CR2
    T6 - verifies -> CR3
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/cclab-sdd-cli/src/commands.rs
    action: modify
    section: cli
    impl_mode: hand-written
    description: |
      Update is_artifact_permitted_under_guard() to add docs-phase guard mappings:
        "create_change_docs" => artifact_action == "create_change_docs",
        "revise_change_docs" => artifact_action == "create_change_docs",
        "review_change_docs" => artifact_action == "review_change_docs",
      Pattern: revise delegates to create artifact (same as implementation phase
      where begin_implementation/resume_implementation/implement_spec/implement_task
      all permit create_change_implementation artifact).

  - path: .aw/tech-design/crates/cclab-sdd/interfaces/cli/commands.md
    action: modify
    section: cli
    impl_mode: hand-written
    description: |
      1. Extend Command Tree YAML block to list docs-phase actions under
         workflow and artifact subcommands.
      2. Append 6 rows to CLI → Logic Mapping table:
         | sdd workflow create-change-docs <id> | ToolRegistry::call_tool() | sdd_workflow_create_change_docs |
         | sdd workflow review-change-docs <id> | ToolRegistry::call_tool() | sdd_workflow_review_change_docs |
         | sdd workflow revise-change-docs <id> | ToolRegistry::call_tool() | sdd_workflow_revise_change_docs |
         | sdd artifact create-change-docs <id> <payload> | ToolRegistry::call_tool() | sdd_artifact_create_change_docs |
         | sdd artifact review-change-docs <id> <payload> | ToolRegistry::call_tool() | sdd_artifact_review_change_docs |
         | sdd artifact revise-change-docs <id> <payload> | ToolRegistry::call_tool() | sdd_artifact_revise_change_docs |
  - action: annotate
    section: async-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the async-api section."

  - action: annotate
    section: config
    impl_mode: hand-written
    description: "Traceability metadata edge for the config section."

  - action: annotate
    section: db-model
    impl_mode: hand-written
    description: "Traceability metadata edge for the db-model section."

  - action: annotate
    section: dependency
    impl_mode: hand-written
    description: "Traceability metadata edge for the dependency section."

  - action: annotate
    section: interaction
    impl_mode: hand-written
    description: "Traceability metadata edge for the interaction section."

  - action: annotate
    section: logic
    impl_mode: hand-written
    description: "Traceability metadata edge for the logic section."

  - action: annotate
    section: requirements
    impl_mode: hand-written
    description: "Traceability metadata edge for the requirements section."

  - action: annotate
    section: rest-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rest-api section."

  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

  - action: annotate
    section: state-machine
    impl_mode: hand-written
    description: "Traceability metadata edge for the state-machine section."

  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```

## Doc
<!-- type: doc lang: markdown -->

N/A — covered by CLI help text and this spec.


## CLI
<!-- type: cli lang: yaml -->

```yaml
# Docs-phase actions routed through existing dynamic dispatch.
# No new Commands enum variants — uses Workflow/Artifact variants.

cclab sdd:
  workflow <action> <change_id> [extra_args_json]:
    # Existing command — new docs-phase actions:
    actions:
      create-change-docs:
        tool: sdd_workflow_create_change_docs
        description: "Resolve doc targets, build doc-writer prompt, dispatch agent"
      review-change-docs:
        tool: sdd_workflow_review_change_docs
        description: "Build doc-reviewer prompt with accuracy checklist, dispatch agent"
      revise-change-docs:
        tool: sdd_workflow_revise_change_docs
        description: "Build doc-writer prompt with review feedback, dispatch agent"

  artifact <action> <change_id> <payload_path>:
    # Existing command — new docs-phase actions:
    actions:
      create-change-docs:
        tool: sdd_artifact_create_change_docs
        description: "Write updated guide sections to output_dir"
        payload_schema:
          target_crate: string (required)
          guide_path: string (required)
          sections_content: object (required, map of section_name -> markdown)
          summary: string (required)
      review-change-docs:
        tool: sdd_artifact_review_change_docs
        description: "Write doc review verdict with CLI verification results"
        payload_schema:
          verdict: enum [APPROVED, REVIEWED, REJECTED] (required)
          review_notes: string (required)
          cli_verification_results: array (optional)
      revise-change-docs:
        tool: sdd_artifact_revise_change_docs
        description: "Write revised guide sections (delegates to create artifact)"
        payload_schema:
          target_crate: string (required)
          guide_path: string (required)
          sections_content: object (required)
          summary: string (required)

# Delegation guard mapping (is_artifact_permitted_under_guard)
guard_mapping:
  create_change_docs:
    permitted_artifacts: [create_change_docs]
  review_change_docs:
    permitted_artifacts: [review_change_docs]
  revise_change_docs:
    permitted_artifacts: [create_change_docs]
    note: "revise delegates to create artifact (same pattern as implementation phase)"
```
