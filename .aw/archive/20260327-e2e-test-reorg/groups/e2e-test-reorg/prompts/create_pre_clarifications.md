# Task: Clarify Group 'e2e-test-reorg' for Change 'e2e-test-reorg'

## Context

Group: **e2e-test-reorg**


## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/e2e-test-reorg/groups/e2e-test-reorg/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/e2e-test-reorg/groups/e2e-test-reorg/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/e2e-test-reorg/user_input.md` — user's description


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
cclab sdd artifact create-pre-clarifications e2e-test-reorg cclab/changes/e2e-test-reorg/payloads/create-pre-clarifications.json
```