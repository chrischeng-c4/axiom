---
id: artifact-tools-docs-phase
main_spec_ref: "crates/cclab-sdd/interfaces/tools/artifact-tools.md"
merge_strategy: new
fill_sections: [overview, rpc-api, changes]
filled_sections: [overview, rpc-api, changes]
create_complete: true
---

# Artifact Tools Docs Phase

## Overview

Extend artifact tool interfaces with docs-phase artifact operations. Three new artifact tool pairs (workflow + artifact) follow the existing CRR pattern established by change-spec and change-implementation tools:

1. **`sdd_workflow_create_change_docs` / `sdd_artifact_create_change_docs`** — doc-writer agent creates/updates guide sections from change specs + CLI specs + config specs + scenarios
2. **`sdd_workflow_review_change_docs` / `sdd_artifact_review_change_docs`** — doc-reviewer agent reviews for accuracy (via CLI execution), completeness, audience fit
3. **`sdd_workflow_revise_change_docs` / `sdd_artifact_revise_change_docs`** — doc-writer agent revises based on review feedback

**Scope**: This spec covers the artifact tool interface definitions only (OpenRPC params/result schemas). Workflow orchestration logic is in `docs-phase-logic`. State machine changes are in `state-machine-docs-phase`.

| Tool pair | Agent | Pattern source |
|-----------|-------|----------------|
| create_change_docs | sdd-doc-writer | `create_change_impl` |
| review_change_docs | sdd-doc-reviewer | `review_change_impl` |
| revise_change_docs | sdd-doc-writer | `revise_change_impl` |
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
  - path: crates/cclab-sdd/src/tools/mod.rs
    action: modify
    description: |
      Add module declarations for new docs tool files:
        pub mod create_change_docs;
        pub mod review_change_docs;
        pub mod revise_change_docs;
      Register 6 new tool definitions in all_tools_vec():
        create_change_docs::workflow_definition(),
        create_change_docs::artifact_definition(),
        review_change_docs::workflow_definition(),
        review_change_docs::artifact_definition(),
        revise_change_docs::workflow_definition(),
        revise_change_docs::artifact_definition(),

  - path: crates/cclab-sdd/src/tools/create_change_docs.rs
    action: create
    description: |
      New file. Two tool entry points following create_change_impl.rs pattern:
      - workflow_definition() -> ToolDefinition for sdd_workflow_create_change_docs
      - artifact_definition() -> ToolDefinition for sdd_artifact_create_change_docs
      - execute_workflow(): resolve matched doc targets from [sdd.docs] config,
        build doc-writer prompt (inputs: change specs + existing guide + audience config),
        dispatch sdd-doc-writer agent. Returns prompt_path + executor.
      - execute_artifact(): write sections_content map to guide_path file.
        Merge new sections into existing guide (preserve unchanged sections).
        Update STATE.yaml phase to DocsCreated.

  - path: crates/cclab-sdd/src/tools/review_change_docs.rs
    action: create
    description: |
      New file. Two tool entry points following review_change_impl.rs pattern:
      - workflow_definition() -> ToolDefinition for sdd_workflow_review_change_docs
      - artifact_definition() -> ToolDefinition for sdd_artifact_review_change_docs
      - execute_workflow(): build doc-reviewer prompt with review checklist
        (hard: accuracy/completeness/no-regression; soft: audience-fit/examples/flow),
        dispatch sdd-doc-reviewer agent. Reviewer has Bash (read-only by prompt),
        Read, Glob, Grep — no Write tool.
      - execute_artifact(): write verdict (APPROVED/REVIEWED/REJECTED) + review_notes.
        Store cli_verification_results. Update STATE.yaml phase to DocsReviewed.
        On APPROVED -> next_action points to sdd_workflow_create_change_merge.
        On REVIEWED/REJECTED -> next_action points to sdd_workflow_revise_change_docs.

  - path: crates/cclab-sdd/src/tools/revise_change_docs.rs
    action: create
    description: |
      New file. Two tool entry points following revise_change_impl.rs pattern:
      - workflow_definition() -> ToolDefinition for sdd_workflow_revise_change_docs
      - artifact_definition() -> ToolDefinition for sdd_artifact_revise_change_docs
      - execute_workflow(): build doc-writer prompt with review feedback included,
        dispatch sdd-doc-writer agent for revision.
      - execute_artifact(): delegates to create_change_docs::execute_artifact().
        Increments revision count in STATE.yaml task_revisions.
        Update phase to DocsRevised -> next_action points to review.

  - path: crates/cclab-sdd/src/models/change.rs
    action: modify
    description: |
      Add variants to WorkflowArtifact enum:
        CreateChangeDocs,
        ReviewChangeDocs,
        ReviseChangeDocs,
      Add name() mappings:
        CreateChangeDocs => "create_change_docs"
        ReviewChangeDocs => "review_change_docs"
        ReviseChangeDocs => "revise_change_docs"

  - path: cclab/specs/crates/cclab-sdd/interfaces/tools/artifact-tools.md
    action: modify
    description: |
      Append OpenRPC method definitions for the 6 new docs-phase tools
      to the existing artifact tools spec. Add to the artifact enum in
      sdd_write_artifact: "docs" artifact type with actions create/revise/review.
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
  "info": { "title": "SDD Artifact Tools — Docs Phase Extensions", "version": "1.0.0" },
  "methods": [
    {
      "name": "sdd_workflow_create_change_docs",
      "summary": "Orchestrate docs creation: resolve target guides, build doc-writer prompt, dispatch agent",
      "params": [
        { "name": "project_path", "required": true, "schema": { "type": "string" } },
        { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
      ],
      "result": {
        "name": "CreateDocsWorkflowResult",
        "schema": {
          "type": "object",
          "required": ["status"],
          "properties": {
            "status": { "type": "string", "enum": ["ok", "skip", "error"] },
            "targets": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "crate": { "type": "string" },
                  "guide": { "type": "string" },
                  "sections": { "type": "array", "items": { "type": "string" } }
                }
              },
              "description": "Matched doc targets from [sdd.docs] config"
            },
            "prompt_path": { "type": "string", "description": "Path to generated doc-writer prompt" },
            "executor": { "type": "string", "description": "Agent executor for doc-writer" },
            "skip_reason": { "type": "string", "description": "Present when status=skip (no config or no crate match)" },
            "next_actions": { "type": "array", "items": { "$ref": "#/definitions/NextAction" } }
          }
        }
      }
    },
    {
      "name": "sdd_artifact_create_change_docs",
      "summary": "Write updated guide sections to output_dir for matched doc targets",
      "params": [
        { "name": "project_path", "required": true, "schema": { "type": "string" } },
        { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } },
        { "name": "target_crate", "required": true, "schema": { "type": "string" }, "description": "Crate name from docs target config" },
        { "name": "guide_path", "required": true, "schema": { "type": "string" }, "description": "Output guide file path (relative to project root)" },
        { "name": "sections_content", "required": true, "schema": { "type": "object", "additionalProperties": { "type": "string" } }, "description": "Map of section_name -> markdown content" },
        { "name": "summary", "required": true, "schema": { "type": "string" }, "description": "Brief description of doc changes" }
      ],
      "result": {
        "name": "CreateDocsArtifactResult",
        "schema": {
          "type": "object",
          "required": ["status", "artifacts_written"],
          "properties": {
            "status": { "type": "string", "enum": ["ok", "error"] },
            "artifacts_written": { "type": "array", "items": { "type": "string" } },
            "guide_path": { "type": "string" },
            "sections_updated": { "type": "array", "items": { "type": "string" } },
            "next_actions": { "type": "array", "items": { "$ref": "#/definitions/NextAction" } }
          }
        }
      }
    },
    {
      "name": "sdd_workflow_review_change_docs",
      "summary": "Orchestrate docs review: build doc-reviewer prompt with accuracy checklist, dispatch agent",
      "params": [
        { "name": "project_path", "required": true, "schema": { "type": "string" } },
        { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
      ],
      "result": {
        "name": "ReviewDocsWorkflowResult",
        "schema": {
          "type": "object",
          "required": ["status"],
          "properties": {
            "status": { "type": "string", "enum": ["ok", "error"] },
            "prompt_path": { "type": "string", "description": "Path to generated doc-reviewer prompt" },
            "executor": { "type": "string", "description": "Agent executor for doc-reviewer" },
            "review_checklist": {
              "type": "object",
              "properties": {
                "hard": { "type": "array", "items": { "type": "string" }, "description": "Must-pass criteria (accuracy, completeness, no regression)" },
                "soft": { "type": "array", "items": { "type": "string" }, "description": "REVIEWED criteria (audience fit, examples, flow)" }
              }
            },
            "next_actions": { "type": "array", "items": { "$ref": "#/definitions/NextAction" } }
          }
        }
      },
      "x-review-verification": {
        "method": "cli_execution",
        "description": "Doc-reviewer runs actual CLI commands to verify documented behavior matches implementation"
      }
    },
    {
      "name": "sdd_artifact_review_change_docs",
      "summary": "Write doc review verdict with inline annotations",
      "params": [
        { "name": "project_path", "required": true, "schema": { "type": "string" } },
        { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } },
        { "name": "verdict", "required": true, "schema": { "type": "string", "enum": ["APPROVED", "REVIEWED", "REJECTED"] } },
        { "name": "review_notes", "required": true, "schema": { "type": "string" }, "description": "Structured review with accuracy findings, completeness gaps, audience issues" },
        { "name": "cli_verification_results", "schema": { "type": "array", "items": { "type": "object", "properties": { "command": { "type": "string" }, "expected": { "type": "string" }, "actual": { "type": "string" }, "pass": { "type": "boolean" } } } }, "description": "Results of CLI command verification against documented behavior" }
      ],
      "result": {
        "name": "ReviewDocsArtifactResult",
        "schema": {
          "type": "object",
          "required": ["status", "verdict"],
          "properties": {
            "status": { "type": "string", "enum": ["ok", "error"] },
            "verdict": { "type": "string", "enum": ["APPROVED", "REVIEWED", "REJECTED"] },
            "review_path": { "type": "string" },
            "next_actions": { "type": "array", "items": { "$ref": "#/definitions/NextAction" } }
          }
        }
      }
    },
    {
      "name": "sdd_workflow_revise_change_docs",
      "summary": "Orchestrate docs revision: build doc-writer prompt with review feedback, dispatch agent",
      "params": [
        { "name": "project_path", "required": true, "schema": { "type": "string" } },
        { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
      ],
      "result": {
        "name": "ReviseDocsWorkflowResult",
        "schema": {
          "type": "object",
          "required": ["status"],
          "properties": {
            "status": { "type": "string", "enum": ["ok", "error"] },
            "prompt_path": { "type": "string" },
            "executor": { "type": "string" },
            "revision_count": { "type": "integer", "description": "Current revision number" },
            "next_actions": { "type": "array", "items": { "$ref": "#/definitions/NextAction" } }
          }
        }
      }
    },
    {
      "name": "sdd_artifact_revise_change_docs",
      "summary": "Write revised guide sections based on review feedback. Delegates to create artifact logic.",
      "params": [
        { "name": "project_path", "required": true, "schema": { "type": "string" } },
        { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } },
        { "name": "target_crate", "required": true, "schema": { "type": "string" } },
        { "name": "guide_path", "required": true, "schema": { "type": "string" } },
        { "name": "sections_content", "required": true, "schema": { "type": "object", "additionalProperties": { "type": "string" } } },
        { "name": "summary", "required": true, "schema": { "type": "string" } }
      ],
      "result": {
        "name": "ReviseDocsArtifactResult",
        "schema": {
          "type": "object",
          "required": ["status", "artifacts_written"],
          "properties": {
            "status": { "type": "string", "enum": ["ok", "error"] },
            "artifacts_written": { "type": "array", "items": { "type": "string" } },
            "revision_count": { "type": "integer" },
            "next_actions": { "type": "array", "items": { "$ref": "#/definitions/NextAction" } }
          }
        }
      }
    }
  ],
  "definitions": {
    "NextAction": {
      "type": "object",
      "properties": {
        "tool": { "type": "string" },
        "args": { "type": "object" },
        "cli": { "type": "string" }
      }
    }
  }
}
```

# Reviews
