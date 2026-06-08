# Task: Clarify Group 'decorator-full' for Change 'mamba-p1-lang-features'

## Context

Group: **decorator-full**
Issues: #847_decorator-arguments-and-chaining-decorator-args-an

## Files to Read

- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-p1-lang-features/groups/decorator-full/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-p1-lang-features/groups/decorator-full/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-p1-lang-features/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-p1-lang-features/issues/issue_*.md` — issues with `group: "decorator-full"` in frontmatter

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