# Task: Clarify Group 'stdlib-main-module' for Change 'mamba-stdlib-main'

## Context

Group: **stdlib-main-module**


## Files to Read

- `/Users/chris.cheng/cclab/main/.score/changes/mamba-stdlib-main/groups/stdlib-main-module/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/main/.score/changes/mamba-stdlib-main/groups/stdlib-main-module/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/main/.score/changes/mamba-stdlib-main/user_input.md` — user's description


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
score artifact create-pre-clarifications mamba-stdlib-main .score/changes/mamba-stdlib-main/payloads/create-pre-clarifications.json
```