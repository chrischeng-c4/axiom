---
id: artifact-tools-phase3
main_spec_ref: "crates/cclab-sdd/interfaces/tools/artifact-tools.md"
merge_strategy: new
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Artifact Tools Phase3

## Overview

Extend artifact tools with post-write alignment validation. After `sdd_artifact_create_change_spec` (and `sdd_artifact_revise_change_spec`) writes a section to disk, call `spec_alignment::check()` on the entire spec file. Format violations (Phase 1 rules: missing section annotation, duplicate section, format priority violation) block the write — return error, revert the file. Coverage gaps (Phase 2 rules: orphan requirements, uncovered requirements) produce warnings in the response JSON but do not block.

**Scope**: This spec covers only the artifact tool integration point. Same `check()` function, different strictness than CLI or merge callers.

| Caller | Format violations | Coverage gaps |
|--------|-------------------|---------------|
| Artifact tools (this spec) | Error — block write | Warning — allow write |
| CLI post-hoc | Report all | Report all |
| Merge workflow | Warning | Warning |
## Requirements
<!-- type: requirements lang: markdown -->

<!-- TODO -->

## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

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
  - path: crates/cclab-sdd/src/tools/create_change_spec.rs
    action: modify
    description: |
      In `execute_artifact()`, after writing section content to disk (after `std::fs::write`),
      call `spec_alignment::check(&spec_path)`. If result contains format violations
      (ViolationKind matches Phase 1: MissingSectionAnnotation, DuplicateSection,
      FormatPriorityViolation, DuplicateDefinition, DefinitionConflict*,
      RpcFieldConsistency, NestedSchemaConflict*), revert the file to `current`
      (the pre-write content) and return error JSON with violations list.
      If result contains only Phase 2 warnings (OrphanRequirement,
      UncoveredRequirement), include them in response as `alignment_warnings`
      array and allow the write to succeed.

  - path: crates/cclab-sdd/src/tools/revise_change_spec.rs
    action: modify
    description: |
      Same pattern as create_change_spec: after writing revised section,
      call `spec_alignment::check()` on the entire file. Block on format
      violations (revert + error), warn on coverage gaps.

  - path: crates/cclab-sdd/src/tools/create_change_spec.rs
    action: modify
    description: |
      Extend artifact response JSON schema: add optional `alignment_warnings`
      field (array of Violation objects) to the success response.
      Schema: `{ status, artifacts_written, alignment_warnings?: Violation[], next_actions }`.

  - path: crates/cclab-sdd/src/tools/revise_change_spec.rs
    action: modify
    description: |
      Extend revise artifact response JSON: add optional `alignment_warnings`
      field matching create_change_spec pattern.
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
