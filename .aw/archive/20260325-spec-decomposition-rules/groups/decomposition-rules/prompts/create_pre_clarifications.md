# Task: Clarify Group 'decomposition-rules' for Change 'spec-decomposition-rules'

## Context

Group: **decomposition-rules**


## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-decomposition-rules/groups/decomposition-rules/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-decomposition-rules/groups/decomposition-rules/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-decomposition-rules/user_input.md` — user's description


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
cclab sdd artifact create-pre-clarifications spec-decomposition-rules cclab/changes/spec-decomposition-rules/payloads/create-pre-clarifications.json
```