# Task: Clarify Group 'spec-plan' for Change 'sdd-workflow-cleanup'

## Context

Group: **spec-plan**
Issues: #886_spec-plan-in-reference-context-auto-determine-main

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-workflow-cleanup/groups/spec-plan/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-workflow-cleanup/groups/spec-plan/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-workflow-cleanup/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-workflow-cleanup/issues/issue_*.md` — issues with `group: "spec-plan"` in frontmatter

## Instructions

1. Read requirements.md and pre_clarifications.md for this group
2. The pre_clarifications.md contains pre-generated questions — use these as your starting point
3. Use AskUserQuestion to ask the pre-generated questions to the user
4. After answers, evaluate: did answers raise new questions?
5. If more clarification needed: ask follow-up questions
6. When sufficient: run `cclab sdd artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-pre-clarifications sdd-workflow-cleanup cclab/changes/sdd-workflow-cleanup/payloads/create-pre-clarifications.json
```