---
id: change-spec-logic
main_spec_ref: ~
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "This logic TD supports TD/CB artifact lifecycle state, authoring, review, validation, or merge behavior."
---

# Change Spec Logic

## Overview
<!-- type: overview lang: markdown -->

Fix file placement bug (#956) where payloads/, prompts/, and specs/ are written to the change root directory instead of under groups/{group-id}/ in multi-group changes. The spec_service.rs constructs specs paths without group awareness, and merge.rs reads only from the root specs/ directory. Prompts and payloads helpers already support group-scoped paths but callers in the change_spec and change_impl phases don't always pass group_id. This change ensures all artifact writes during grouped changes use groups/{group-id}/specs/, groups/{group-id}/prompts/, and groups/{group-id}/payloads/ paths, and updates the merge reader to iterate all groups.
## Requirements
<!-- type: doc lang: markdown -->

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
<!-- type: doc lang: markdown -->

### Scenario: Spec file placed under group directory
**GIVEN** a multi-group change with group_id "group-directory-fix"
**WHEN** spec_service writes a spec file for spec_id "change-spec-logic"
**THEN** the file is created at `.aw/changes/{change_id}/groups/group-directory-fix/specs/change-spec-logic.md`

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
<!-- type: doc lang: markdown -->

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
<!-- type: doc lang: markdown -->

### REST API
<!-- type: rest-api lang: yaml -->
<!-- score-td-placeholder -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- score-td-placeholder -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- score-td-placeholder -->

### CLI
<!-- type: cli lang: yaml -->
<!-- score-td-placeholder -->

### Schema
<!-- type: schema lang: json -->
<!-- score-td-placeholder -->

### Config
<!-- type: config lang: json -->
<!-- score-td-placeholder -->

## Test Plan
<!-- type: doc lang: markdown -->

<!-- TODO -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: crates/cclab-sdd/src/services/spec_service.rs
    action: modify
    section: logic
    impl_mode: codegen
    desc: Update specs_dir path construction (lines 467-472) to use groups/{group_id}/specs/ when group_id is available. Thread group_id through from workflow callers.
  - path: crates/cclab-sdd/src/workflow/merge.rs
    action: modify
    section: cli
    impl_mode: codegen
    desc: Update find_specs_to_merge() to iterate groups/*/specs/ directories. Add fallback to root specs/ for backward compat. Preserve group ordering from STATE.yaml.
  - path: crates/cclab-sdd/src/workflow/helpers.rs
    action: modify
    section: schema
    impl_mode: codegen
    desc: No structural changes needed (already supports group_id). Verify all callers pass group_id correctly.
  - path: crates/cclab-sdd/src/tools/workflow_common.rs
    action: modify
    section: rpc-api
    impl_mode: codegen
    desc: No structural changes needed (already supports group_id). Verify change_spec and change_impl phase callers pass group_id.
  - path: crates/cclab-sdd/src/workflow/change_spec.rs
    action: modify
    section: rest-api
    impl_mode: codegen
    desc: Pass current group_id to spec_service and prompt/payload write helpers during spec creation loop.
  - path: crates/cclab-sdd/src/workflow/change_impl.rs
    action: modify
    section: async-api
    impl_mode: codegen
    desc: Pass current group_id to prompt write helpers during implementation prompt generation.
  - action: annotate
    section: component
    impl_mode: hand-written
    description: "Traceability metadata edge for the component section."

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
    section: design-token
    impl_mode: hand-written
    description: "Traceability metadata edge for the design-token section."

  - action: annotate
    section: interaction
    impl_mode: hand-written
    description: "Traceability metadata edge for the interaction section."

  - action: annotate
    section: state-machine
    impl_mode: hand-written
    description: "Traceability metadata edge for the state-machine section."

  - action: annotate
    section: wireframe
    impl_mode: hand-written
    description: "Traceability metadata edge for the wireframe section."

```
## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
wireframes: []
```

## Component
<!-- type: component lang: yaml -->

```yaml
components: []
```

## Design Token
<!-- type: design-token lang: yaml -->

```yaml
tokens: []
```

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->
