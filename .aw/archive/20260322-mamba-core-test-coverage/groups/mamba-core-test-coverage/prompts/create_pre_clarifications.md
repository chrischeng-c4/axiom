# Task: Clarify Group 'mamba-core-test-coverage' for Change 'mamba-core-test-coverage'

## Context

Group: **mamba-core-test-coverage**
Issues: #1035_test-mamba-per-module-test-coverage-gaps-lower-res

## Files to Read

- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-core-test-coverage/groups/mamba-core-test-coverage/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-core-test-coverage/groups/mamba-core-test-coverage/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-core-test-coverage/user_input.md` — user's description
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-core-test-coverage/issues/issue_*.md` — issues with `group: "mamba-core-test-coverage"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-core-test-coverage cclab/changes/mamba-core-test-coverage/payloads/create-pre-clarifications.json
```