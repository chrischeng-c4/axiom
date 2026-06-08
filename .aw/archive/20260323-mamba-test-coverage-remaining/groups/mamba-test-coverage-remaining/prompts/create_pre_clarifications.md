# Task: Clarify Group 'mamba-test-coverage-remaining' for Change 'mamba-test-coverage-remaining'

## Context

Group: **mamba-test-coverage-remaining**
Issues: #1035_test-mamba-per-module-test-coverage-gaps-lower-res

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd-mamba-test/cclab/changes/mamba-test-coverage-remaining/groups/mamba-test-coverage-remaining/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-test/cclab/changes/mamba-test-coverage-remaining/groups/mamba-test-coverage-remaining/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-test/cclab/changes/mamba-test-coverage-remaining/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-test/cclab/changes/mamba-test-coverage-remaining/issues/issue_*.md` — issues with `group: "mamba-test-coverage-remaining"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-test-coverage-remaining cclab/changes/mamba-test-coverage-remaining/payloads/create-pre-clarifications.json
```