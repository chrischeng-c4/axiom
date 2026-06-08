---
id: merge-tool-openrpc
type: spec
title: "Workflow Tools — OpenRPC Definitions"
version: 3
main_spec_ref: "crates/cclab-sdd/interfaces/tools/workflow-tools.md"
merge_strategy: extend
fill_sections: [overview, requirements, scenarios, rpc-api, changes]
filled_sections: [overview, rpc-api, changes, requirements, scenarios]
create_complete: true
---

# Workflow Tools

Workflow tools (`sdd_workflow_*`) are routers and prompt generators. They read state,
return prompts and `next_actions`. No large payload parameters — those live in artifact tools.

## sdd_run_change

```json
{
  "name": "sdd_run_change",
  "summary": "Workflow bridge. Reads STATE.yaml, returns next tool to call.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } },
    { "name": "description", "required": false, "schema": { "type": "string" } },
    { "name": "issues", "required": false, "schema": { "type": "array", "items": { "type": "string" } } },
    { "name": "git_workflow", "required": false, "schema": { "type": "string", "enum": ["new_branch", "in_place"] } },
    { "name": "last_action", "required": false, "schema": { "type": "string" } }
  ],
  "result": {
    "schema": {
      "type": "object",
      "required": ["change_id", "action", "current_phase", "next_actions"],
      "properties": {
        "change_id": { "type": "string" },
        "action": { "type": "string", "enum": ["init_change", "restructure_input", "create_pre_clarifications", "create_reference_context", "reference_context_lifecycle", "create_post_clarifications", "delegate_to_per_action_tools", "begin_merge", "complete"] },
        "current_phase": { "type": ["string", "null"] },
        "executor": { "type": "array", "const": ["mainthread"] },
        "message": { "type": "string" },
        "next_actions": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "cli": { "type": "string" },
              "args": { "type": "object" }
            }
          }
        }
      }
    }
  }
}
```

## sdd_workflow_init_change

```json
{
  "name": "sdd_workflow_init_change",
  "summary": "Initialize change directory, user_input.md, STATE.yaml.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } },
    { "name": "description", "required": true, "schema": { "type": "string" } },
    { "name": "issues", "required": false, "schema": { "type": "array", "items": { "type": "string" } } },
    { "name": "git_workflow", "required": false, "schema": { "type": "string", "enum": ["new_branch", "in_place"] } },
    { "name": "branch", "required": false, "schema": { "type": "string" } }
  ],
  "result": {
    "schema": {
      "type": "object",
      "required": ["status", "artifacts_written", "next_actions"],
      "properties": {
        "status": { "type": "string", "enum": ["ok", "error"] },
        "artifacts_written": { "type": "array", "items": { "type": "string" } },
        "next_actions": { "type": "array" }
      }
    }
  },
  "x-phase": "→ ChangeInited",
  "x-next": "sdd_workflow_restructure_input"
}
```

## sdd_workflow_restructure_input

```json
{
  "name": "sdd_workflow_restructure_input",
  "summary": "Return prompt to restructure input into groups.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "result": {
    "schema": {
      "type": "object",
      "required": ["status"],
      "properties": {
        "status": { "type": "string" },
        "prompt_path": { "type": "string" },
        "executor": { "type": "array" },
        "agent_completed": { "type": "boolean" },
        "next_actions": { "type": "array" }
      }
    }
  },
  "x-phase": "ChangeInited → InputRestructured"
}
```

## sdd_workflow_create_pre_clarifications

```json
{
  "name": "sdd_workflow_create_pre_clarifications",
  "summary": "Return prompt for per-group user Q&A.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "result": {
    "schema": {
      "type": "object",
      "required": ["status", "group_id"],
      "properties": {
        "status": { "type": "string", "enum": ["ok", "error", "phase_complete"] },
        "prompt_path": { "type": "string" },
        "group_id": { "type": "string" }
      }
    }
  },
  "x-phase": "InputRestructured → PreClarificationsCreated",
  "x-progress": "groups_progress.pre_clarifications"
}
```

## sdd_workflow_create_reference_context

```json
{
  "name": "sdd_workflow_create_reference_context",
  "summary": "Central router for per-group reference context lifecycle (create/review/revise).",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "PreClarificationsCreated → PostClarificationsCreated",
  "x-progress": "groups_progress.reference_context",
  "x-sub-states": ["Create", "Review", "Revise", "AllDone"]
}
```

## sdd_workflow_create_post_clarifications

```json
{
  "name": "sdd_workflow_create_post_clarifications",
  "summary": "Per-group router for post-clarifications (create-only, no CRR).",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "PostClarificationsCreated → ChangeSpecCreated",
  "x-progress": "groups_progress.post_clarifications"
}
```

## sdd_workflow_create_change_spec

```json
{
  "name": "sdd_workflow_create_change_spec",
  "summary": "Per-spec sub-state router: skeleton → analyze → fill sections → prune → review → revise.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "PostClarificationsCreated → ChangeSpecCreated → ChangeSpecReviewed",
  "x-sub-states": ["Create", "Review", "Revise", "MainthreadMustFix", "AdvanceToImplementation"]
}
```

## sdd_workflow_review_change_spec

```json
{
  "name": "sdd_workflow_review_change_spec",
  "summary": "Return prompt to review a completed spec.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "ChangeSpecCreated → ChangeSpecReviewed"
}
```

## sdd_workflow_revise_change_spec

```json
{
  "name": "sdd_workflow_revise_change_spec",
  "summary": "Return prompt to revise a spec based on review feedback.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "ChangeSpecReviewed → ChangeSpecRevised"
}
```

## sdd_workflow_create_change_implementation

```json
{
  "name": "sdd_workflow_create_change_implementation",
  "summary": "Return prompt to implement code changes based on approved specs.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "ChangeImplementationCreated → ChangeImplementationReviewed"
}
```

## sdd_workflow_review_change_implementation

```json
{
  "name": "sdd_workflow_review_change_implementation",
  "summary": "Return prompt to review implementation against spec.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "ChangeImplementationCreated → ChangeImplementationReviewed"
}
```

## sdd_workflow_revise_change_implementation

```json
{
  "name": "sdd_workflow_revise_change_implementation",
  "summary": "Return prompt to revise implementation based on review.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "x-phase": "ChangeImplementationReviewed → ChangeImplementationRevised"
}
```


## Overview

## Overview
<!-- type: overview lang: markdown -->

Add `sdd_workflow_create_change_merge` OpenRPC definition to workflow-tools.md. This tool is fully programmatic (`x-executor: programmatic`, `x-crr: false`) — it merges change specs into main specs, archives the change directory, and optionally performs post-archive git operations (auto-commit, auto-PR).

The response schema extends the standard workflow response with merge-specific fields: `specs_merged[]`, `archive_path`, `audit_log[]` for merge tracking, plus `git_commit_sha`, `pr_url`, `git_warning` for post-archive git operation results.

Three merge strategies are declared via `x-merge-strategies`: 3-way merge (when `.base.md` sibling exists and git available), overwrite (target exists, no base), and create (target does not exist). Errors use JSON-RPC codes: `-32001` (missing main_spec_ref), `-32002` (root-level path), `-32003` (3-way merge conflict).

No existing tool definitions are modified — this is a new tool addition to the workflow-tools interface spec.
## Overview

## Overview
<!-- type: overview lang: markdown -->

Add `sdd_workflow_create_change_merge` OpenRPC definition to workflow-tools.md. This tool is fully programmatic (`x-executor: programmatic`, `x-crr: false`) — it merges change specs into main specs, archives the change directory, and optionally performs post-archive git operations (auto-commit, auto-PR).

The response schema extends the standard workflow response with merge-specific fields: `specs_merged[]`, `archive_path`, `audit_log[]` for merge tracking, plus `git_commit_sha`, `pr_url`, `git_warning` for post-archive git operation results.

Three merge strategies are declared via `x-merge-strategies`: 3-way merge (when `.base.md` sibling exists and git available), overwrite (target exists, no base), and create (target does not exist). Errors use JSON-RPC codes: `-32001` (missing main_spec_ref), `-32002` (root-level path), `-32003` (3-way merge conflict).

No existing tool definitions are modified — this is a new tool addition to the workflow-tools interface spec.
## Overview

## Overview
<!-- type: overview lang: markdown -->

Add `sdd_workflow_create_change_merge` OpenRPC definition to workflow-tools.md. This tool is fully programmatic (`x-executor: programmatic`, `x-crr: false`) — it merges change specs into main specs, archives the change directory, and optionally performs post-archive git operations (auto-commit, auto-PR).

The response schema extends the standard workflow response with merge-specific fields: `specs_merged[]`, `archive_path`, `audit_log[]` for merge tracking, plus `git_commit_sha`, `pr_url`, `git_warning` for post-archive git operation results.

Three merge strategies are declared via `x-merge-strategies`: 3-way merge (when `.base.md` sibling exists and git available), overwrite (target exists, no base), and create (target does not exist). Errors use JSON-RPC codes: `-32001` (missing main_spec_ref), `-32002` (root-level path), `-32003` (3-way merge conflict).

No existing tool definitions are modified — this is a new tool addition to the workflow-tools interface spec.
## Overview

## Overview
<!-- type: overview lang: markdown -->

Add `sdd_workflow_create_change_merge` OpenRPC definition to workflow-tools.md. This tool is fully programmatic (`x-executor: programmatic`, `x-crr: false`) — it merges change specs into main specs, archives the change directory, and optionally performs post-archive git operations (auto-commit, auto-PR).

The response schema extends the standard workflow response with merge-specific fields: `specs_merged[]`, `archive_path`, `audit_log[]` for merge tracking, plus `git_commit_sha`, `pr_url`, `git_warning` for post-archive git operation results.

Three merge strategies are declared via `x-merge-strategies`: 3-way merge (when `.base.md` sibling exists and git available), overwrite (target exists, no base), and create (target does not exist). Errors use JSON-RPC codes: `-32001` (missing main_spec_ref), `-32002` (root-level path), `-32003` (3-way merge conflict).

No existing tool definitions are modified — this is a new tool addition to the workflow-tools interface spec.
## RPC API

## sdd_workflow_create_change_merge
<!-- type: rpc-api lang: json -->

```json
{
  "name": "sdd_workflow_create_change_merge",
  "summary": "Programmatic merge: copy change specs to main specs, archive change dir, optionally auto-commit and create PR.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "result": {
    "name": "MergeResult",
    "schema": {
      "type": "object",
      "required": ["status", "specs_merged", "archive_path", "audit_log", "next_actions"],
      "properties": {
        "status": { "type": "string", "enum": ["ok", "error"] },
        "prompt_path": { "type": "string" },
        "executor": { "type": "array", "items": { "type": "string" } },
        "specs_merged": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["spec_id", "target"],
            "properties": {
              "spec_id": { "type": "string" },
              "target": { "type": "string" }
            }
          }
        },
        "archive_path": { "type": "string", "description": "cclab/archive/{YYYYMMDD}-{id}" },
        "audit_log": {
          "type": "array",
          "items": { "type": "string" },
          "description": "[merge] {create|overwrite|3way-merge} {path}"
        },
        "git_commit_sha": { "type": ["string", "null"] },
        "pr_url": { "type": ["string", "null"] },
        "git_warning": { "type": ["string", "null"] },
        "next_actions": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "cli": { "type": "string" },
              "args": { "type": "object" }
            }
          }
        }
      }
    }
  },
  "errors": [
    { "code": -32001, "message": "missing main_spec_ref", "data": { "description": "Spec has null or missing main_spec_ref" } },
    { "code": -32002, "message": "root-level main_spec_ref rejected", "data": { "description": "main_spec_ref must contain '/'" } },
    { "code": -32003, "message": "3-way merge conflict", "data": { "description": "git merge-file detected conflicts — merge aborted, no files written" } }
  ],
  "x-phase": "ChangeImplementationReviewed \u2192 ChangeArchived",
  "x-executor": "programmatic",
  "x-crr": false,
  "x-merge-strategies": {
    "3way": "When .base.md sibling exists AND target exists AND git available: git merge-file",
    "overwrite": "When no .base.md or target already exists: write/overwrite directly",
    "create": "When target does not exist: create new file"
  },
  "x-post-archive-git": {
    "description": "Optional git operations after archive move, controlled by [sdd.repo_platform] config",
    "sequence": ["load_config", "check_auto_commit", "find_git_binary", "git_status", "git_add", "git_commit", "check_auto_pr", "dispatch_pr_agent", "gh_pr_create"]
  }
}
```
## sdd_workflow_create_change_merge
<!-- type: rpc-api lang: json -->

```json
{
  "openrpc": "1.3.2",
  "info": { "title": "sdd_workflow_create_change_merge", "version": "1.0.0" },
  "methods": [
    {
      "name": "sdd_workflow_create_change_merge",
      "summary": "Programmatic merge: copy change specs to main specs, archive change dir, optionally auto-commit and create PR.",
      "params": [
        {
          "name": "project_path",
          "required": true,
          "schema": { "type": "string", "description": "Project root path" }
        },
        {
          "name": "change_id",
          "required": true,
          "schema": { "type": "string", "pattern": "^[a-z0-9-]+$", "description": "Change ID" }
        }
      ],
      "result": {
        "name": "MergeResult",
        "schema": {
          "type": "object",
          "required": ["status", "specs_merged", "archive_path", "audit_log", "next_actions"],
          "properties": {
            "status": {
              "type": "string",
              "enum": ["ok", "error"]
            },
            "prompt_path": {
              "type": "string",
              "description": "Relative path to the merge completion prompt file"
            },
            "executor": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Executor chain for this tool response"
            },
            "specs_merged": {
              "type": "array",
              "items": {
                "type": "object",
                "required": ["spec_id", "target"],
                "properties": {
                  "spec_id": { "type": "string", "description": "Spec file stem identifier" },
                  "target": { "type": "string", "description": "Target path under cclab/specs/" }
                }
              },
              "description": "List of specs merged into cclab/specs/"
            },
            "archive_path": {
              "type": "string",
              "description": "Path where change dir was archived (cclab/archive/{YYYYMMDD}-{id})"
            },
            "audit_log": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Ordered log of merge actions: [merge] {create|overwrite|3way-merge} {path}"
            },
            "git_commit_sha": {
              "type": ["string", "null"],
              "description": "SHA of auto-commit if repo_platform.auto_commit=true and dirty paths exist, null otherwise"
            },
            "pr_url": {
              "type": ["string", "null"],
              "description": "URL of created PR if repo_platform.auto_pr=true and commit succeeded, null otherwise"
            },
            "git_warning": {
              "type": ["string", "null"],
              "description": "Warning if git operations were skipped (binary missing, auto_pr without auto_commit)"
            },
            "next_actions": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "cli": { "type": "string" },
                  "args": { "type": "object" }
                }
              }
            }
          }
        }
      },
      "errors": [
        {
          "code": -32001,
          "message": "missing main_spec_ref",
          "data": { "description": "Spec has null or missing main_spec_ref — all specs must specify a subfolder target path" }
        },
        {
          "code": -32002,
          "message": "root-level main_spec_ref rejected",
          "data": { "description": "main_spec_ref must contain '/' — root-level paths are not allowed" }
        },
        {
          "code": -32003,
          "message": "3-way merge conflict",
          "data": { "description": "git merge-file detected conflicts — merge aborted, no files written (all-or-nothing)" }
        }
      ],
      "x-phase": "ChangeImplementationReviewed → ChangeArchived",
      "x-executor": "programmatic",
      "x-crr": false,
      "x-merge-strategies": {
        "3way": "When .base.md sibling exists AND target file exists AND git binary available: use git merge-file for 3-way merge",
        "overwrite": "When no .base.md or target does not exist: write/overwrite target directly",
        "create": "When target does not exist: create new file"
      },
      "x-post-archive-git": {
        "description": "Optional git operations after archive move, controlled by [sdd.repo_platform] config",
        "sequence": ["load_config", "check_auto_commit", "find_git_binary", "git_status", "git_add", "git_commit", "check_auto_pr", "dispatch_pr_agent", "gh_pr_create"]
      }
    }
  ]
}
```
## sdd_workflow_create_change_merge
<!-- type: rpc-api lang: json -->

```json
{
  "name": "sdd_workflow_create_change_merge",
  "summary": "Merge change specs into main specs, archive change dir, optionally auto-commit and create PR.",
  "params": [
    { "name": "project_path", "required": true, "schema": { "type": "string" } },
    { "name": "change_id", "required": true, "schema": { "type": "string", "pattern": "^[a-z0-9-]+$" } }
  ],
  "result": {
    "schema": {
      "type": "object",
      "required": ["status", "merged_specs", "archive_path"],
      "properties": {
        "status": { "type": "string", "enum": ["ok", "error"] },
        "merged_specs": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "spec_id": { "type": "string" },
              "target_path": { "type": "string" },
              "merge_strategy": { "type": "string", "enum": ["new", "update", "extend"] }
            }
          },
          "description": "List of specs merged into cclab/specs/"
        },
        "archive_path": {
          "type": ["string", "null"],
          "description": "Path where change dir was archived (cclab/archive/{YYYYMMDD}-{id})"
        },
        "git_commit_sha": {
          "type": ["string", "null"],
          "description": "SHA of auto-commit if repo_platform.auto_commit=true and dirty paths exist, null otherwise"
        },
        "pr_url": {
          "type": ["string", "null"],
          "description": "URL of created PR if repo_platform.auto_pr=true and commit succeeded, null otherwise"
        },
        "git_warning": {
          "type": ["string", "null"],
          "description": "Warning message if git operations were skipped (binary missing, config mismatch)"
        },
        "next_actions": { "type": "array" }
      }
    }
  },
  "x-phase": "ChangeImplementationReviewed → ChangeArchived",
  "x-executor": "programmatic",
  "x-crr": false
}
```


## Changes

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: cclab/specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md
    action: MODIFY
    desc: |
      Append new section after ## sdd_workflow_revise_change_implementation:

      ## sdd_workflow_create_change_merge
      OpenRPC method definition with:
        - params: project_path (required), change_id (required, pattern ^[a-z0-9-]+$)
        - result.required: [status, specs_merged, archive_path, audit_log, next_actions]
        - result.properties: status, prompt_path, executor, specs_merged[], archive_path,
          audit_log[], git_commit_sha, pr_url, git_warning, next_actions[]
        - errors: -32001 (missing main_spec_ref), -32002 (root-level path), -32003 (3way conflict)
        - x-phase: ChangeImplementationReviewed → ChangeArchived
        - x-executor: programmatic
        - x-crr: false
        - x-merge-strategies: {3way, overwrite, create}
        - x-post-archive-git.sequence: 9-step git operations pipeline

      No modifications to any existing tool definitions.
```
## Changes

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: cclab/specs/crates/cclab-sdd/interfaces/tools/workflow-tools.md
    action: MODIFY
    desc: |
      Append new section after ## sdd_workflow_revise_change_implementation:

      ## sdd_workflow_create_change_merge
      OpenRPC method definition with:
        - params: project_path (required), change_id (required, pattern ^[a-z0-9-]+$)
        - result.required: [status, specs_merged, archive_path, audit_log, next_actions]
        - result.properties: status, prompt_path, executor, specs_merged[], archive_path,
          audit_log[], git_commit_sha, pr_url, git_warning, next_actions[]
        - errors: -32001 (missing main_spec_ref), -32002 (root-level path), -32003 (3way conflict)
        - x-phase: ChangeImplementationReviewed → ChangeArchived
        - x-executor: programmatic
        - x-crr: false
        - x-merge-strategies: {3way, overwrite, create}
        - x-post-archive-git.sequence: 9-step git operations pipeline

      No modifications to any existing tool definitions.
```
## Requirements

## Requirements
<!-- type: requirements lang: markdown -->

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Standard workflow tool params | P0 | Tool accepts `project_path` (required, string) and `change_id` (required, string, pattern `^[a-z0-9-]+$`), matching the existing workflow tool param convention in workflow-tools.md |
| R2 | Merge tracking response fields | P0 | Response `required` array includes `specs_merged` (array of `{spec_id, target}`), `archive_path` (string), `audit_log` (array of strings, format `[merge] {create\|overwrite\|3way-merge} {path}`) |
| R3 | Git operation response fields | P0 | Response includes nullable fields: `git_commit_sha` (string\|null), `pr_url` (string\|null), `git_warning` (string\|null) for post-archive git results |
| R4 | JSON-RPC error codes | P0 | Define errors: `-32001` (missing main_spec_ref), `-32002` (root-level main_spec_ref rejected), `-32003` (3-way merge conflict — all-or-nothing abort) |
| R5 | Phase and executor metadata | P0 | Declare `x-phase: ChangeImplementationReviewed → ChangeArchived`, `x-executor: programmatic`, `x-crr: false` |
| R6 | Merge strategy extensions | P1 | Declare `x-merge-strategies` with three strategies: `3way` (`.base.md` sibling + target exists + git available), `overwrite` (no `.base.md` or target exists), `create` (target does not exist) |
| R7 | Post-archive git sequence | P1 | Declare `x-post-archive-git.sequence` listing the ordered steps: load_config → check_auto_commit → find_git_binary → git_status → git_add → git_commit → check_auto_pr → dispatch_pr_agent → gh_pr_create |

### Constraints

- Interface spec only — no business logic. Logic is in `change-merge-git-integration` spec.
- Consistent with existing workflow-tools.md format: method object in JSON code block, no OpenRPC envelope wrapper.
- New tool appended after `sdd_workflow_revise_change_implementation` — no modifications to existing tools.


## Scenarios

## Scenarios
<!-- type: scenarios lang: markdown -->

### S1: Successful merge with multiple specs (R1, R2, R5)

**GIVEN** valid `project_path` and `change_id` with 3 change specs, all having valid `main_spec_ref` with subfolder paths
**WHEN** `sdd_workflow_create_change_merge` is called
**THEN** response has `status: "ok"`, `specs_merged` array with 3 entries (each having `spec_id` + `target`), `archive_path` set to `cclab/archive/{YYYYMMDD}-{id}`, `audit_log` with 3 entries.

### S2: Missing main_spec_ref — error -32001 (R4)

**GIVEN** a change spec with `main_spec_ref: null` or missing `main_spec_ref` field
**WHEN** merge tool processes this spec
**THEN** error code `-32001` is returned. No files are written (all-or-nothing).

### S3: Root-level main_spec_ref — error -32002 (R4)

**GIVEN** a change spec with `main_spec_ref: "flat-spec.md"` (no `/` separator)
**WHEN** merge tool validates the path
**THEN** error code `-32002` is returned. No files are written.

### S4: 3-way merge conflict — error -32003 (R4, R6)

**GIVEN** a change spec with `.base.md` sibling, target exists on disk, and both main and change modified the same lines
**WHEN** `git merge-file` detects conflicts
**THEN** error code `-32003` is returned. No files are written (all-or-nothing abort).

### S5: Merge with auto-commit result (R3)

**GIVEN** `repo_platform.auto_commit = true` and dirty paths exist under `cclab/`
**WHEN** merge succeeds and post-archive git operations run
**THEN** response includes `git_commit_sha: "abc1234..."` (non-null string), `git_warning: null`.

### S6: Merge without git operations (R3)

**GIVEN** `repo_platform.auto_commit = false` or config section absent
**WHEN** merge completes
**THEN** response includes `git_commit_sha: null`, `pr_url: null`, `git_warning: null`. Merge result is identical to no-git-ops behavior.

### S7: Clean 3-way merge (R6)

**GIVEN** `.base.md` sibling exists, target exists with diverged content in non-overlapping regions, git binary available
**WHEN** merge tool processes the spec
**THEN** `audit_log` entry is `[merge] 3way-merge {path}`, merged content preserves both sides' changes.

### S8: No specs to merge (R2)

**GIVEN** change directory has no spec files
**WHEN** `sdd_workflow_create_change_merge` is called
**THEN** response has `status: "ok"`, empty `specs_merged`, `archive_path` set. Change is archived without merge.

# Reviews
