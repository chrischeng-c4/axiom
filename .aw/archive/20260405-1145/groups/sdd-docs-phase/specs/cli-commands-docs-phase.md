---
id: cli-commands-docs-phase
main_spec_ref: "crates/cclab-sdd/interfaces/cli/commands.md"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, cli, changes]
filled_sections: [overview, requirements, scenarios, cli, changes]
create_complete: true
---

# Cli Commands Docs Phase

## Overview

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

| ID | Requirement | Priority |
|----|-------------|----------|
| CR1 | Update `is_artifact_permitted_under_guard()` to map docs-phase workflow guard actions (`create_change_docs`, `revise_change_docs`) to their permitted artifact action (`create_change_docs`). Pattern: `revise_change_docs` delegates to `create_change_docs` artifact (same as implementation phase pattern). | high |
| CR2 | Update `is_artifact_permitted_under_guard()` to map `review_change_docs` workflow guard to `review_change_docs` artifact action. | high |
| CR3 | Update CLI → Logic Mapping table in main spec with 6 new action entries: 3 workflow actions and 3 artifact actions, all routing through `ToolRegistry::call_tool()`. | high |
| CR4 | Update Command Tree documentation in main spec to list the new docs-phase action names under `workflow` and `artifact` subcommands. | medium |
| CR5 | No new `Commands` enum variants needed — existing `Workflow` and `Artifact` variants handle all docs-phase actions via dynamic dispatch. | info |
## Scenarios

### Scenario: Workflow create-change-docs dispatches via dynamic routing
- **GIVEN** user runs `cclab sdd workflow create-change-docs 1145`
- **WHEN** the CLI constructs tool name `sdd_workflow_create_change_docs`
- **THEN** `ToolRegistry::call_tool("sdd_workflow_create_change_docs", args)` is invoked
- **AND** result JSON is printed to stdout

### Scenario: Artifact create-change-docs permitted under create_change_docs guard
- **GIVEN** STATE.yaml has `delegation_guard.action = "create_change_docs"`
- **WHEN** agent calls `cclab sdd artifact create-change-docs 1145 payload.json`
- **THEN** `is_artifact_permitted_under_guard("create_change_docs", "create_change_docs")` returns true
- **AND** artifact tool executes successfully

### Scenario: Artifact create-change-docs permitted under revise_change_docs guard
- **GIVEN** STATE.yaml has `delegation_guard.action = "revise_change_docs"`
- **WHEN** agent calls `cclab sdd artifact create-change-docs 1145 payload.json`
- **THEN** `is_artifact_permitted_under_guard("create_change_docs", "revise_change_docs")` returns true
- **AND** artifact tool executes (revise delegates to create artifact)

### Scenario: Artifact review-change-docs permitted under review_change_docs guard
- **GIVEN** STATE.yaml has `delegation_guard.action = "review_change_docs"`
- **WHEN** agent calls `cclab sdd artifact review-change-docs 1145 payload.json`
- **THEN** `is_artifact_permitted_under_guard("review_change_docs", "review_change_docs")` returns true
- **AND** artifact tool executes successfully

### Scenario: Unrelated artifact action blocked by docs guard
- **GIVEN** STATE.yaml has `delegation_guard.action = "create_change_docs"`
- **WHEN** agent calls `cclab sdd artifact create-change-implementation 1145 payload.json`
- **THEN** guard check fails (neither name match nor permitted-under mapping)
- **AND** CLI returns error: "Action 'create-change-implementation' blocked by delegation guard"

### Scenario: Kebab-case to snake_case normalization
- **GIVEN** user runs `cclab sdd workflow review-change-docs 1145`
- **WHEN** CLI normalizes action name
- **THEN** tool name becomes `sdd_workflow_review_change_docs` (hyphens → underscores)
- **AND** the tool is found in `ToolRegistry`
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

```yaml
changes:
  - path: crates/cclab-sdd-cli/src/commands.rs
    action: modify
    description: |
      Update is_artifact_permitted_under_guard() to add docs-phase guard mappings:
        "create_change_docs" => artifact_action == "create_change_docs",
        "revise_change_docs" => artifact_action == "create_change_docs",
        "review_change_docs" => artifact_action == "review_change_docs",
      Pattern: revise delegates to create artifact (same as implementation phase
      where begin_implementation/resume_implementation/implement_spec/implement_task
      all permit create_change_implementation artifact).

  - path: cclab/specs/crates/cclab-sdd/interfaces/cli/commands.md
    action: modify
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
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->


## CLI

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

# Reviews
