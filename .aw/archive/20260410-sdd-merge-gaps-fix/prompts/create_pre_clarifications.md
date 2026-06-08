# Task: Create Pre-Clarifications for Change 'sdd-merge-gaps-fix'

## Files to Read

- `/Users/chris.cheng/cclab/main/.score/changes/sdd-merge-gaps-fix/user_input.md` — user's description

## Instructions

1. Read user_input.md
2. Identify key decisions and open questions
3. Use AskUserQuestion to ask the user for clarifications
4. When sufficient, run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications sdd-merge-gaps-fix .score/changes/sdd-merge-gaps-fix/payloads/create-pre-clarifications.json
```