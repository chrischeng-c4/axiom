# Task: Clarify Group 'jet-workspace-protocol' for Change 'jet-workspace-protocol'

## Context

Group: **jet-workspace-protocol**


## Files to Read

- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/jet-workspace-protocol/groups/jet-workspace-protocol/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/jet-workspace-protocol/user_input.md` — user's description


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
cclab sdd artifact create-pre-clarifications jet-workspace-protocol cclab/changes/jet-workspace-protocol/payloads/create-pre-clarifications.json
```