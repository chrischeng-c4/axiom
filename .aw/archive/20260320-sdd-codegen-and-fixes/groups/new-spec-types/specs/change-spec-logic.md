---
id: change-spec-logic
main_spec_ref: ~
merge_strategy: new
create_complete: true
filled_sections: [overview, requirements, scenarios, changes]
---

# Change Spec Logic

## Overview

<!-- type: overview lang: markdown -->

Fix file placement bug (#956) where payloads/, prompts/, and specs/ are written to the change root directory instead of under groups/{group-id}/ in multi-group changes. The spec_service.rs constructs specs paths without group awareness, and merge.rs reads only from the root specs/ directory. Prompts and payloads helpers already support group-scoped paths but callers in the change_spec and change_impl phases don't always pass group_id. This change ensures all artifact writes during grouped changes use groups/{group-id}/specs/, groups/{group-id}/prompts/, and groups/{group-id}/payloads/ paths, and updates the merge reader to iterate all groups.
## Requirements

<!-- type: requirements lang: markdown -->

### R1: Group-scoped spec path construction

spec_service.rs must construct specs directory as `groups/{group_id}/specs/` when the change has groups. Currently lines 467-472 use `change_dir.join("specs")` without group awareness. The `group_id` must be resolved from the current spec's group membership (via reference_context.md spec_plan) and passed through to the path builder.

**Priority**: high

### R2: Merge reader iterates all groups

merge.rs `find_specs_to_merge()` (line 32) must iterate `groups/*/specs/` directories instead of only reading from root `specs/`. For each group directory found, collect all spec .md files. Preserve group ordering from STATE.yaml `groups_progress.change_spec` list.

**Priority**: high

### R3: Callers pass group_id to prompt/payload helpers

workflow_common.rs `write_prompt_file` and helpers.rs `write_artifact_payload` already support `group_id: Option<&str>` parameter. Ensure all callers in change_spec phase (spec fill prompts) and change_impl phase (impl prompts) pass the correct group_id instead of None.

**Priority**: high

### R4: New changes only

Backward compatibility: only apply group-scoped layout to newly created changes. Detect layout by checking if `groups/` directory exists in the change directory. If no `groups/` dir, fall back to root-level paths.

**Priority**: medium

### R5: STATE.yaml active_group tracking

Add or use existing group context during spec/impl phases. The `groups_progress` HashMap already tracks completed groups. During spec creation, the current group being processed should be available to path construction code. No new STATE.yaml field needed if group_id is threaded through function parameters.

**Priority**: medium
## Scenarios

<!-- type: scenarios lang: markdown -->

### Scenario: Spec file placed under group directory
**GIVEN** a multi-group change with group_id "group-directory-fix"
**WHEN** spec_service writes a spec file for spec_id "change-spec-logic"
**THEN** the file is created at `cclab/changes/{change_id}/groups/group-directory-fix/specs/change-spec-logic.md`

### Scenario: Merge reads from all group spec directories
**GIVEN** a change with groups ["group-a", "group-b"] each having specs/
**WHEN** merge phase executes find_specs_to_merge()
**THEN** specs from both `groups/group-a/specs/` and `groups/group-b/specs/` are collected

### Scenario: Prompt files placed under group directory
**GIVEN** a multi-group change in change_spec phase for group "new-spec-types"
**WHEN** write_prompt_file is called for a fill_spec prompt
**THEN** the prompt is written to `groups/new-spec-types/prompts/fill_spec_*.md`

### Scenario: Backward compat with root-level layout
**GIVEN** an existing change created before this fix with specs at root `specs/`
**WHEN** merge phase runs
**THEN** it detects no `groups/` directory and falls back to reading from root `specs/`

### Scenario: Payload files placed under group directory
**GIVEN** a multi-group change in change_spec phase for group "specir-and-test-codegen"
**WHEN** write_artifact_payload is called
**THEN** the payload is written to `groups/specir-and-test-codegen/payloads/create-change-spec.json`
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

<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/cclab-sdd/src/services/spec_service.rs
    action: MODIFY
    desc: Update specs_dir path construction (lines 467-472) to use groups/{group_id}/specs/ when group_id is available. Thread group_id through from workflow callers.
  - path: crates/cclab-sdd/src/workflow/merge.rs
    action: MODIFY
    desc: Update find_specs_to_merge() to iterate groups/*/specs/ directories. Add fallback to root specs/ for backward compat. Preserve group ordering from STATE.yaml.
  - path: crates/cclab-sdd/src/workflow/helpers.rs
    action: MODIFY
    desc: No structural changes needed (already supports group_id). Verify all callers pass group_id correctly.
  - path: crates/cclab-sdd/src/tools/workflow_common.rs
    action: MODIFY
    desc: No structural changes needed (already supports group_id). Verify change_spec and change_impl phase callers pass group_id.
  - path: crates/cclab-sdd/src/workflow/change_spec.rs
    action: MODIFY
    desc: Pass current group_id to spec_service and prompt/payload write helpers during spec creation loop.
  - path: crates/cclab-sdd/src/workflow/change_impl.rs
    action: MODIFY
    desc: Pass current group_id to prompt write helpers during implementation prompt generation.
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

# Reviews