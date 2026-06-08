# Task: Clarify Group 'scope-summary' for Change 'post-clarifications-scope-summary'

## Context

Group: **scope-summary**


## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/post-clarifications-scope-summary/groups/scope-summary/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/post-clarifications-scope-summary/groups/scope-summary/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/post-clarifications-scope-summary/user_input.md` — user's description


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
cclab sdd artifact create-pre-clarifications post-clarifications-scope-summary cclab/changes/post-clarifications-scope-summary/payloads/create-pre-clarifications.json
```