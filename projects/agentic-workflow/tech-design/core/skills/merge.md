---
id: merge-skill
type: spec
title: "/cclab:sdd:merge Skill"
version: 1
spec_type: algorithm
spec_group: sdd
created_at: 2026-03-17T00:00:00+00:00
updated_at: 2026-03-17T00:00:00+00:00
requirements:
  total: 2
  ids: [R1, R2]
refs: [merge]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# /cclab:sdd:merge Skill

User-invoked skill that merges a completed SDD change: copies specs to `.aw/tech-design/`, archives the change directory.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - User-Only Invocation

```yaml
id: R1
priority: high
```

- `auto-invoke: false` -- LLM must NOT call this skill automatically
- Only the user can trigger merge via `/cclab:sdd:merge`
- `run-change` stops at `implementation_complete` and prompts the user

### R2 - Merge Execution

```yaml
id: R2
priority: high
```

- Resolves change-id (from argument or current branch)
- Calls `cclab sdd workflow create-change-merge <change-id>`
- Reports merged specs and archive path

## Template
<!-- type: doc lang: markdown -->

```markdown
---
name: cclab:sdd:merge
description: Merge completed change -- archive specs and implementation
user-invocable: true
auto-invoke: false
---
```

## Installation
<!-- type: doc lang: markdown -->

Installed to `~/.claude/skills/cclab-sdd-merge/SKILL.md` by `cclab sdd update`.

## Test Plan
<!-- type: doc lang: markdown -->

| Test | Covers |
|------|--------|
| merge_skill_is_user_invoked_only | R1 |
| merge_skill_executes_change_merge_and_reports_paths | R2 |

## Changes (issue-lifecycle-crr)
<!-- type: changelog lang: markdown -->

### Merge Writes to Issue Frontmatter

On successful merge, `close_issue_if_exists()` updates the associated issue file with final state:

| Field | Value | Rationale |
|-------|-------|-----------|
| `state` | `closed` | Issue is resolved |
| `phase` | `change_archived` | Terminal SDD phase |

### Transient Fields Cleared

The following fields are cleared on merge to reflect that the change is complete. They are only meaningful during an active workflow run:

| Cleared Field | Why |
|---------------|-----|
| `git_workflow` | Worktree/branch info no longer relevant |
| `iteration` | Re-proposal counter is session-specific |
| `current_task_id` | No active task after merge |
| `impl_spec_phase` | Per-spec tracking done |
| `task_revisions` | Revision counts done |
| `revision_counts` | CRR counts done |
| `last_action` | No more actions |
| `session_id` | Session over |
| `validation_errors` | Cleared (passed to get here) |

### Fields Retained for Audit Trail

| Retained Field | Why |
|----------------|-----|
| `change_id` | Links closed issue to archived change in `.aw/archive/` |
| `branch` | Records which branch the work was done on |

### Issue File Move

The issue file is moved from `.aw/issues/open/{slug}.md` to `.aw/issues/closed/{slug}.md` as part of the close operation. The matching strategy tries:

1. **Slug match** -- open issue file named `{change_id}.md`
2. **Frontmatter id match** -- scan all open issues for one whose `id` field matches `change_id`

If no matching issue is found (legacy changes without an associated issue), the merge proceeds normally without closing any issue.
