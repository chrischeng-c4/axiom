# Task: Clarify Group 'consolidate-grid-crates' for Change 'grid-consolidate'

## Context

Group: **consolidate-grid-crates**


## Files to Read

- `/Users/chris.cheng/cclab/main/.score/changes/grid-consolidate/groups/consolidate-grid-crates/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/main/.score/changes/grid-consolidate/groups/consolidate-grid-crates/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/main/.score/changes/grid-consolidate/user_input.md` — user's description


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
score artifact create-pre-clarifications grid-consolidate .score/changes/grid-consolidate/payloads/create-pre-clarifications.json
```