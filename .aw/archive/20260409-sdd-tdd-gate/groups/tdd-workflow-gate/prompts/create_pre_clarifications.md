# Task: Clarify Group 'tdd-workflow-gate' for Change 'sdd-tdd-gate'

## Context

Group: **tdd-workflow-gate**


## Files to Read

- `/Users/chris.cheng/cclab/main/.score/changes/sdd-tdd-gate/groups/tdd-workflow-gate/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/main/.score/changes/sdd-tdd-gate/groups/tdd-workflow-gate/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/main/.score/changes/sdd-tdd-gate/user_input.md` — user's description


## Instructions

1. Read requirements.md and pre_clarifications.md for this group
2. The pre_clarifications.md contains pre-generated questions — use these as your starting point
3. Use AskUserQuestion to ask the pre-generated questions to the user
4. After answers, evaluate: did answers raise new questions?
5. If more clarification needed: ask follow-up questions
6. When sufficient: run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications sdd-tdd-gate .score/changes/sdd-tdd-gate/payloads/create-pre-clarifications.json
```