# Task: Clarify Group 'merge-lens-into-sdd' for Change 'merge-lens-into-sdd'

## Context

Group: **merge-lens-into-sdd**
Issues: #942_refactor-merge-cclab-lens-into-cclab-sdd

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/merge-lens-into-sdd/groups/merge-lens-into-sdd/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/merge-lens-into-sdd/groups/merge-lens-into-sdd/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/merge-lens-into-sdd/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/merge-lens-into-sdd/issues/issue_*.md` — issues with `group: "merge-lens-into-sdd"` in frontmatter

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
cclab sdd artifact create-pre-clarifications merge-lens-into-sdd cclab/changes/merge-lens-into-sdd/payloads/create-pre-clarifications.json
```