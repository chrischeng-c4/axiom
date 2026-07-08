---
id: mcp-utility-tools
type: spec
title: "Utility Tools — OpenRPC Definitions"
version: 1
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Utility Tools

Stateless tools. No phase transitions. Callable anytime.

## sdd_read_artifact
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_read_artifact
summary: Read any SDD artifact by scope string.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: scope
    required: true
    schema:
      type: string
      description: >-
        Change artifacts (require change_id): context_clarifications,
        spec_clarifications, codebase_context, spec_context, knowledge_context,
        gap_*, proposal, tasks, requirements, review_{artifact}, or {spec_id}.
        Main spec: main_spec:{group/id}. Listings: list:main_specs, list:specs.
        Task instructions: task (requires task_type).
  - name: change_id
    required: false
    schema:
      type: string
      description: Required for change artifact reads and task scope
  - name: task_type
    required: false
    schema:
      type: string
      enum: [create_spec, review_spec, revise_spec, implement, code_review, resolve, review_archive]
      description: Required when scope=task
  - name: spec_id
    required: false
    schema:
      type: string
      description: Spec ID (for spec-related task_types)
  - name: description
    required: false
    schema:
      type: string
  - name: iteration
    required: false
    schema:
      type: integer
  - name: dependencies
    required: false
    schema:
      type: array
      items:
        type: string
```

## sdd_write_artifact
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_write_artifact
summary: Unified artifact writer. Routes by (artifact, action).
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: false
    schema:
      type: string
      pattern: '^[a-z0-9-]+$'
  - name: artifact
    required: true
    schema:
      type: string
      enum:
        - change
        - context_clarifications
        - spec_clarifications
        - codebase_context
        - spec_context
        - knowledge_context
        - gap_codebase_spec
        - gap_codebase_knowledge
        - gap_spec_knowledge
        - proposal
        - spec
        - main_spec
        - issues_context
  - name: action
    required: true
    schema:
      type: string
      enum: [create, revise, review, write, fetch]
  - name: caller
    required: false
    schema:
      type: string
      enum: [agent, mainthread]
      default: mainthread
  - name: payload
    required: true
    schema:
      type: object
  - name: issue
    required: false
    schema:
      type: integer
  - name: iteration
    required: false
    schema:
      type: integer
      minimum: 1
      default: 1
```

## sdd_delegate_agent
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_delegate_agent
summary: Dispatch prompt to external LLM agent with post-execution verification.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: false
    schema:
      type: string
      pattern: '^[a-z0-9-]+$'
  - name: agent
    required: true
    schema:
      type: string
      description: "provider:model_id (e.g. gemini:flash, codex:balanced, claude:fast)"
  - name: action
    required: true
    schema:
      type: string
      enum:
        - explore
        - review
        - custom
        - create_spec
        - review_spec
        - revise_spec
        - generate_tasks
        - implement_task
        - review_implementation
        - begin_merge
        - resume_merge
        - review_merge
        - fix_merge
  - name: prompt
    required: true
    schema:
      type: string
result:
  schema:
    type: object
    properties:
      status:
        type: string
        enum: [ok, error]
      change_id:
        type: string
      action:
        type: string
      verification:
        type: object
      usage:
        type: object
        properties:
          tokens_in:
            type: integer
          tokens_out:
            type: integer
          duration_ms:
            type: integer
          cost_usd:
            type: number
```

## sdd_validate_change
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_validate_change
summary: Validate change artifacts.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: true
    schema:
      type: string
      pattern: '^[a-z0-9-]+$'
```

## sdd_analyze_code_for_spec
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_analyze_code_for_spec
summary: Analyze code structure for spec generation.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: path
    required: true
    schema:
      type: string
```

## sdd_platform_sync
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_platform_sync
summary: Sync change status to GitHub/GitLab.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: true
    schema:
      type: string
      pattern: '^[a-z0-9-]+$'
```

## sdd_read_implementation_summary
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_read_implementation_summary
summary: Git diff summary for review.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: true
    schema:
      type: string
      pattern: '^[a-z0-9-]+$'
```

## sdd_list_changed_files
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_list_changed_files
summary: List changed files with stats.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: true
    schema:
      type: string
      pattern: '^[a-z0-9-]+$'
```

## sdd_validate_spec_completeness
<!-- type: rpc-api lang: yaml -->

```yaml
name: sdd_validate_spec_completeness
summary: Validate spec has required sections for code generation.
params:
  - name: project_path
    required: true
    schema:
      type: string
  - name: change_id
    required: true
    schema:
      type: string
  - name: spec_id
    required: true
    schema:
      type: string
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: rpc-api
    impl_mode: hand-written
    description: "Traceability metadata edge for the rpc-api section."

```