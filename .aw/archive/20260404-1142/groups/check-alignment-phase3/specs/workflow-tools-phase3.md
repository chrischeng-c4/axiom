---
id: workflow-tools-phase3
main_spec_ref: "crates/cclab-sdd/interfaces/tools/workflow-tools.md"
merge_strategy: new
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Workflow Tools Phase3

## Overview

Extend workflow tool interfaces with alignment validation integration. Two changes to the OpenRPC definitions in `workflow-tools.md`:

1. **`sdd_run_change` response extension**: Add `alignment_warnings` field (array of Violation objects or null) to the result schema. Populated for alignment-eligible phases (`ChangeSpecCreated`, `ChangeSpecReviewed`, `ChangeImplementationCreated`, `ChangeImplementationReviewed`); null for all other phases.

2. **Review tool prompt injection**: `sdd_workflow_review_change_implementation` and `sdd_workflow_review_change_spec` inject an `## Alignment Report` section into the review prompt. Not a schema change — behavioral change documented via `x-alignment-injection` extension field.

**Scope**: Interface-level changes only. Implementation logic (how violations are collected, error isolation) is specified in `check-alignment-phase3`. This spec updates the OpenRPC contracts to reflect the new response shape and behavioral extension.

| Tool | Change | Schema impact |
|------|--------|---------------|
| `sdd_run_change` | Add `alignment_warnings` to result | New optional field in result schema |
| `sdd_workflow_review_change_implementation` | Inject alignment report into prompt | Behavioral — `x-alignment-injection` extension |
| `sdd_workflow_review_change_spec` | Inject alignment report into prompt | Behavioral — `x-alignment-injection` extension |
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
  - path: cclab/specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md
    action: modify
    description: |
      In `sdd_run_change` OpenRPC definition, add `alignment_warnings` field
      to result schema properties:

      "alignment_warnings": {
        "type": ["array", "null"],
        "items": { "$ref": "#/definitions/Violation" },
        "description": "Alignment violations for current group specs. Populated for phases: ChangeSpecCreated, ChangeSpecReviewed, ChangeImplementationCreated, ChangeImplementationReviewed. Null for other phases, clean specs, or check errors."
      }

      Add shared `definitions` block at end of the sdd_run_change JSON:

      "definitions": {
        "Violation": {
          "type": "object",
          "required": ["kind", "message", "file"],
          "properties": {
            "kind": { "type": "string" },
            "message": { "type": "string" },
            "heading": { "type": ["string", "null"] },
            "line": { "type": ["integer", "null"] },
            "file": { "type": "string" }
          }
        }
      }

      Add `x-alignment-eligible-phases` extension:

      "x-alignment-eligible-phases": [
        "ChangeSpecCreated",
        "ChangeSpecReviewed",
        "ChangeImplementationCreated",
        "ChangeImplementationReviewed"
      ]

  - path: cclab/specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md
    action: modify
    description: |
      In `sdd_workflow_review_change_implementation` OpenRPC definition,
      add `x-alignment-injection` extension field:

      "x-alignment-injection": {
        "enabled": true,
        "check_target": "change-spec file for the spec being reviewed",
        "injection_point": "between Pre-Review Step and Instructions sections",
        "format": "## Alignment Report\n\n| File | Kind | Message |\n...",
        "on_clean": "No alignment violations found.",
        "on_error": "skip injection silently, tracing::warn!"
      }

      This documents the behavioral change — review prompts now include
      alignment check results. Ref: check-alignment-phase3 R23.

  - path: cclab/specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md
    action: modify
    description: |
      In `sdd_workflow_review_change_spec` OpenRPC definition,
      add identical `x-alignment-injection` extension field:

      "x-alignment-injection": {
        "enabled": true,
        "check_target": "change-spec file being reviewed",
        "injection_point": "between Pre-Review Step and Instructions sections",
        "format": "## Alignment Report\n\n| File | Kind | Message |\n...",
        "on_clean": "No alignment violations found.",
        "on_error": "skip injection silently, tracing::warn!"
      }

      Same pattern as implementation review. Ref: check-alignment-phase3 R24.
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
