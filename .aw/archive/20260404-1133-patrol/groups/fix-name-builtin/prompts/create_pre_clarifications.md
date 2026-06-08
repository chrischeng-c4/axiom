# Task: Clarify Group 'fix-name-builtin' for Change '1133-patrol'

## Context

Group: **fix-name-builtin**
Issues: #1133_mamba-name-resolves-to-0-0-instead-of-main-for-ent

## Files to Read

- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1133-patrol/groups/fix-name-builtin/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1133-patrol/groups/fix-name-builtin/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1133-patrol/user_input.md` — user's description
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1133-patrol/issues/issue_*.md` — issues with `group: "fix-name-builtin"` in frontmatter

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
cclab sdd artifact create-pre-clarifications 1133-patrol cclab/changes/1133-patrol/payloads/create-pre-clarifications.json
```