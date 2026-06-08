# Task: Create Pre-Clarifications for Change 'mamba-all-support'

## Files to Read

- `/Users/chris.cheng/cclab/main/.score/worktrees/mamba-all-support/.score/changes/mamba-all-support/user_input.md` — user's description

## Instructions

1. Read user_input.md
2. Identify key decisions and open questions
3. Use AskUserQuestion to ask the user for clarifications
4. When sufficient, run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications mamba-all-support .score/changes/mamba-all-support/payloads/create-pre-clarifications.json
```