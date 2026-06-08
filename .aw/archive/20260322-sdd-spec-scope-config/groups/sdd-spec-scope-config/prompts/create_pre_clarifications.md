# Task: Clarify Group 'sdd-spec-scope-config' for Change 'sdd-spec-scope-config'

## Context

Group: **sdd-spec-scope-config**


## Files to Read

- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-spec-scope-config/groups/sdd-spec-scope-config/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-spec-scope-config/groups/sdd-spec-scope-config/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-spec-scope-config/user_input.md` — user's description


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
cclab sdd artifact create-pre-clarifications sdd-spec-scope-config cclab/changes/sdd-spec-scope-config/payloads/create-pre-clarifications.json
```