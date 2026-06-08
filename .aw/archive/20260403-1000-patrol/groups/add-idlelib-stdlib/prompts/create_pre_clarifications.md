# Task: Clarify Group 'add-idlelib-stdlib' for Change '1000-patrol'

## Context

Group: **add-idlelib-stdlib**
Issues: #1000_add-native-stdlib-idlelib-idle-editor-internals

## Files to Read

- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1000-patrol/groups/add-idlelib-stdlib/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1000-patrol/groups/add-idlelib-stdlib/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1000-patrol/user_input.md` — user's description
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1000-patrol/issues/issue_*.md` — issues with `group: "add-idlelib-stdlib"` in frontmatter

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
cclab sdd artifact create-pre-clarifications 1000-patrol cclab/changes/1000-patrol/payloads/create-pre-clarifications.json
```