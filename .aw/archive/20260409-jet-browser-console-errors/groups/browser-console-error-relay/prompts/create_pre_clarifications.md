# Task: Clarify Group 'browser-console-error-relay' for Change 'jet-browser-console-errors'

## Context

Group: **browser-console-error-relay**


## Files to Read

- `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/main/.score/changes/jet-browser-console-errors/user_input.md` — user's description


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
score artifact create-pre-clarifications jet-browser-console-errors .score/changes/jet-browser-console-errors/payloads/create-pre-clarifications.json
```