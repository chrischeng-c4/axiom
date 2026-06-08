# Task: Create Pre-Clarifications for Change 'mamba-756-patrol'

## Files to Read

- `/Users/chrischeng/projects/cclab/.score/changes/mamba-756-patrol/user_input.md` — user's description

## Instructions

1. Read user_input.md
2. Identify key decisions and open questions
3. Use AskUserQuestion to ask the user for clarifications
4. When sufficient, run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications mamba-756-patrol .score/changes/mamba-756-patrol/payloads/create-pre-clarifications.json
```