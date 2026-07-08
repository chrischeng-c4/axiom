---
id: revise-artifact-skill
type: spec
title: "/cclab:sdd:revise-artifact Skill"
version: 1
spec_type: algorithm
spec_group: sdd
created_at: 2026-03-17T00:00:00+00:00
updated_at: 2026-03-17T00:00:00+00:00
requirements:
  total: 3
  ids: [R1, R2, R3]
refs: [revise-artifact]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# /cclab:sdd:revise-artifact Skill

User-invoked skill that resets the workflow phase to re-enter the spec → implementation cycle when design issues are found after implementation review.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - User-Only Invocation

```yaml
id: R1
priority: high
```

- `auto-invoke: false` — LLM must NOT call this skill automatically
- Only the user can trigger via `/cclab:sdd:revise-artifact`

### R2 - Phase Reset

```yaml
id: R2
priority: high
```

- CLI command: `cclab sdd revise-artifact <change-id> --description "<what needs to change>"`
- Resets `STATE.yaml` phase to `post_clarifications_created`
- Only allowed from implementation or merge phases
- Appends revision description to `user_input.md`

### R3 - Workflow Continuation

```yaml
id: R3
priority: high
```

- After reset, user runs `/cclab:sdd:run-change` to continue
- Workflow naturally routes to `create-change-spec` (spec CRR cycle)
- Then proceeds to implementation CRR cycle
- Stops again at `implementation_complete` for user decision

## Template
<!-- type: doc lang: markdown -->

```markdown
---
name: cclab:sdd:revise-artifact
description: Revise change-spec and re-implement — fix design issues after review
user-invocable: true
auto-invoke: false
---
```

## CLI
<!-- type: doc lang: markdown -->

```yaml
command: cclab sdd revise-artifact
args:
  - name: change_id
    required: true
  - name: --description
    required: false
    description: What design changes are needed
```

## Installation
<!-- type: doc lang: markdown -->

Installed to `~/.claude/skills/cclab-sdd-revise-artifact/SKILL.md` by `cclab sdd update`.

## Test Plan
<!-- type: doc lang: markdown -->

| Test | Covers |
|------|--------|
| revise_artifact_skill_is_user_invoked_only | R1 |
| revise_artifact_resets_phase_and_appends_description | R2 |
| run_change_continues_after_revise_artifact_reset | R3 |
