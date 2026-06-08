# Task: Clarify Group 'class-features' for Change 'mamba-p1-lang-features'

## Context

Group: **class-features**
Issues: #849_class-features-slots-init-subclass-properties-clas

## Files to Read

- `/Users/chrischeng/projects/wt/mamba/cclab/changes/mamba-p1-lang-features/groups/class-features/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/mamba-p1-lang-features/groups/class-features/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/mamba-p1-lang-features/user_input.md` — user's description
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/mamba-p1-lang-features/issues/issue_*.md` — issues with `group: "class-features"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-p1-lang-features cclab/changes/mamba-p1-lang-features/payloads/create-pre-clarifications.json
```