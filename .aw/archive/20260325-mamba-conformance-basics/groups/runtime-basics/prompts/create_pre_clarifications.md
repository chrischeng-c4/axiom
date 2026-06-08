# Task: Clarify Group 'runtime-basics' for Change 'mamba-conformance-basics'

## Context

Group: **runtime-basics**
Issues: #1037_test-mamba-py3-12-behavioral-conformance-every-fun

## Files to Read

- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-conformance-basics/groups/runtime-basics/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-conformance-basics/groups/runtime-basics/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-conformance-basics/user_input.md` — user's description
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/mamba-conformance-basics/issues/issue_*.md` — issues with `group: "runtime-basics"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-conformance-basics cclab/changes/mamba-conformance-basics/payloads/create-pre-clarifications.json
```