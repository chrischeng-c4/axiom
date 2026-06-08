---
id: workflow-tools-docs-phase
main_spec_ref: "crates/cclab-sdd/interfaces/tools/workflow-tools.md"
merge_strategy: new
fill_sections: [overview, rpc-api, changes]
filled_sections: [overview, rpc-api, changes]
create_complete: true
---

# Workflow Tools Docs Phase

## Overview

Extend `sdd_run_change` workflow tool interface with docs-phase dispatch routing. Four new phase entries are added to the orchestrator's routing table, integrating the docs generation CRR cycle between `ChangeImplementationReviewed` and `ChangeMergeCreated`:

1. **`DocsCheck`** — transient phase, resolved inline (no agent dispatch). Evaluates `[sdd.docs]` config presence + crate match; advances to `DocsCreated` or skips to `ChangeMergeCreated`.
2. **`DocsCreated`** — dispatches `sdd-doc-reviewer` agent via `sdd_workflow_review_change_docs`
3. **`DocsReviewed`** — verdict-dependent: APPROVED → `ChangeMergeCreated`; REVIEWED/REJECTED → dispatches `sdd-doc-writer` via `sdd_workflow_revise_change_docs`
4. **`DocsRevised`** — dispatches `sdd-doc-reviewer` via `sdd_workflow_review_change_docs`

**Response extension**: `docs_target_progress` field added to `sdd_run_change` result schema, tracking per-target generation state for multi-target docs configs.

**Scope**: Interface changes to `sdd_run_change` routing/result only. Individual docs tool definitions: `artifact-tools-docs-phase`. State machine variants: `state-machine-docs-phase`. Orchestration logic: `docs-phase-logic`.

| Phase | Transient | Agent dispatched | Tool dispatched |
|-------|-----------|------------------|-----------------|
| `DocsCheck` | yes | — (inline) | — |
| `DocsCreated` | no | `sdd-doc-reviewer` | `sdd_workflow_review_change_docs` |
| `DocsReviewed` | no | `sdd-doc-writer` (if revise) | `sdd_workflow_revise_change_docs` or skip |
| `DocsRevised` | no | `sdd-doc-reviewer` | `sdd_workflow_review_change_docs` |
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
      In `sdd_run_change` OpenRPC definition, add `docs_target_progress` field
      to result schema properties:

      "docs_target_progress": {
        "type": ["array", "null"],
        "items": { "$ref": "#/definitions/DocsTargetProgress" },
        "description": "Per-target docs generation progress. Populated for docs-phase states. Null for other phases."
      }

      Add `DocsTargetProgress` definition to shared `definitions` block:
        required: [crate_name, guide, status]
        properties: crate_name, guide, status (enum: pending/created/reviewed/approved),
                    revision_count (int), sections_updated (string[]), verdict (nullable enum)

      Add `x-docs-phase-routing` extension documenting four new routing entries:
        docs_check: transient, inline resolution (config lookup + crate match)
        docs_created: dispatch sdd_workflow_review_change_docs
        docs_reviewed: verdict routing (APPROVED→merge, REVIEWED/REJECTED→revise)
        docs_revised: dispatch sdd_workflow_review_change_docs

      Add `x-docs-eligible-phases` extension:
        ["docs_check", "docs_created", "docs_reviewed", "docs_revised"]

  - path: crates/cclab-sdd/src/workflow/mod.rs
    action: modify
    description: |
      In route() match arms, add docs-phase dispatch entries:
        StatePhase::DocsCheck => resolve_docs_check() — inline config evaluation,
          returns DocsCheckResult { skip, targets }. If skip, advance to
          ChangeMergeCreated. If match, dispatch sdd_workflow_create_change_docs.
        StatePhase::DocsCreated => dispatch sdd-doc-reviewer agent
          via sdd_workflow_review_change_docs
        StatePhase::DocsReviewed => read verdict from STATE.yaml;
          APPROVED → advance to ChangeMergeCreated;
          REVIEWED/REJECTED → dispatch sdd-doc-writer via sdd_workflow_revise_change_docs
        StatePhase::DocsRevised => dispatch sdd-doc-reviewer agent
          via sdd_workflow_review_change_docs

      In build_response(), populate docs_target_progress field from
      STATE.yaml docs_targets when current phase is in x-docs-eligible-phases.
      Set to null for all non-docs phases.
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


## RPC API

```json
{
  "openrpc": "1.3.2",
  "info": { "title": "SDD Workflow Tools — Docs Phase Extensions", "version": "1.0.0" },
  "methods": [
    {
      "name": "sdd_run_change",
      "summary": "Extend sdd_run_change with docs-phase routing and response fields",
      "x-docs-phase-routing": {
        "docs_check": {
          "transient": true,
          "resolution": "inline",
          "description": "Evaluate [sdd.docs] config + crate match. No agent dispatch.",
          "outcomes": {
            "match": { "next_phase": "docs_created", "dispatch": "sdd_workflow_create_change_docs" },
            "no_match": { "next_phase": "change_merge_created", "dispatch": "sdd_workflow_create_change_merge" },
            "no_config": { "next_phase": "change_merge_created", "dispatch": "sdd_workflow_create_change_merge" }
          }
        },
        "docs_created": {
          "transient": false,
          "agent": "sdd-doc-reviewer",
          "dispatch": "sdd_workflow_review_change_docs",
          "next_phase": "docs_reviewed"
        },
        "docs_reviewed": {
          "transient": false,
          "verdict_routing": {
            "APPROVED": { "next_phase": "change_merge_created", "dispatch": "sdd_workflow_create_change_merge" },
            "REVIEWED": { "next_phase": "docs_revised", "dispatch": "sdd_workflow_revise_change_docs" },
            "REJECTED": { "next_phase": "docs_revised", "dispatch": "sdd_workflow_revise_change_docs" }
          }
        },
        "docs_revised": {
          "transient": false,
          "agent": "sdd-doc-reviewer",
          "dispatch": "sdd_workflow_review_change_docs",
          "next_phase": "docs_reviewed"
        }
      },
      "x-docs-eligible-phases": ["docs_check", "docs_created", "docs_reviewed", "docs_revised"],
      "result": {
        "name": "RunChangeResult",
        "schema": {
          "x-docs-phase-extension": true,
          "properties": {
            "docs_target_progress": {
              "type": ["array", "null"],
              "items": { "$ref": "#/definitions/DocsTargetProgress" },
              "description": "Per-target docs generation progress. Populated for docs-phase states (docs_check, docs_created, docs_reviewed, docs_revised). Null for all other phases."
            }
          }
        }
      }
    }
  ],
  "definitions": {
    "DocsTargetProgress": {
      "type": "object",
      "required": ["crate_name", "guide", "status"],
      "properties": {
        "crate_name": { "type": "string", "description": "Crate name from [[sdd.docs.targets]] config" },
        "guide": { "type": "string", "description": "Output guide file path (resolved from output_dir + guide)" },
        "status": {
          "type": "string",
          "enum": ["pending", "created", "reviewed", "approved"],
          "description": "Current docs generation state for this target"
        },
        "revision_count": { "type": "integer", "default": 0, "description": "Number of CRR revision cycles completed for this target" },
        "sections_updated": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Guide sections updated in the latest write"
        },
        "verdict": {
          "type": ["string", "null"],
          "enum": ["APPROVED", "REVIEWED", "REJECTED", null],
          "description": "Latest review verdict. Null if not yet reviewed."
        }
      }
    }
  }
}
```

# Reviews
