# Task: Clarify Group 'sdd-impl-test-split' for Change 'sdd-impl-test-split'

## Context

Group: **sdd-impl-test-split**


## Files to Read

- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-impl-test-split/groups/sdd-impl-test-split/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-impl-test-split/groups/sdd-impl-test-split/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-impl-test-split/user_input.md` — user's description


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
cclab sdd artifact create-pre-clarifications sdd-impl-test-split cclab/changes/sdd-impl-test-split/payloads/create-pre-clarifications.json
```