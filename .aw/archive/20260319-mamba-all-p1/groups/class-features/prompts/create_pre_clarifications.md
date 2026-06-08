# Task: Clarify Group 'class-features' for Change 'mamba-all-p1'

## Context

Group: **class-features**
Issues: #849_class-features-slots-init-subclass-properties-clas

## Files to Read

- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/groups/class-features/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/groups/class-features/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/issues/issue_*.md` — issues with `group: "class-features"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-all-p1 cclab/changes/mamba-all-p1/payloads/create-pre-clarifications.json
```