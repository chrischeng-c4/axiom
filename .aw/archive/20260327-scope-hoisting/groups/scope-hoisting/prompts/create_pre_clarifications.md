# Task: Clarify Group 'scope-hoisting' for Change 'scope-hoisting'

## Context

Group: **scope-hoisting**
Issues: #1120_jet-build-scope-hoisting-module-concatenation

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/scope-hoisting/groups/scope-hoisting/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/scope-hoisting/groups/scope-hoisting/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/scope-hoisting/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/scope-hoisting/issues/issue_*.md` — issues with `group: "scope-hoisting"` in frontmatter

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
cclab sdd artifact create-pre-clarifications scope-hoisting cclab/changes/scope-hoisting/payloads/create-pre-clarifications.json
```