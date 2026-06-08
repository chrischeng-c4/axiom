# Task: Clarify Group 'index-path-and-cli-hints' for Change 'sdd-index-path-rename'

## Context

Group: **index-path-and-cli-hints**


## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-index-path-rename/groups/index-path-and-cli-hints/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-index-path-rename/user_input.md` — user's description


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
cclab sdd artifact create-pre-clarifications sdd-index-path-rename cclab/changes/sdd-index-path-rename/payloads/create-pre-clarifications.json
```