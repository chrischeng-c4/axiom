# Task: Clarify Group 'test-coverage-gaps' for Change '1035-patrol'

## Context

Group: **test-coverage-gaps**
Issues: #1035_test-mamba-per-module-test-coverage-gaps-lower-res

## Files to Read

- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1035-patrol/groups/test-coverage-gaps/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1035-patrol/groups/test-coverage-gaps/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1035-patrol/user_input.md` — user's description
- `/Users/chrischeng/projects/wt/mamba/cclab/changes/1035-patrol/issues/issue_*.md` — issues with `group: "test-coverage-gaps"` in frontmatter

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
cclab sdd artifact create-pre-clarifications 1035-patrol cclab/changes/1035-patrol/payloads/create-pre-clarifications.json
```