# Task: Clarify Group 'mamba-py312-conformance' for Change 'mamba-py312-conformance'

## Context

Group: **mamba-py312-conformance**
Issues: #1037_test-mamba-py3-12-behavioral-conformance-every-fun

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-py312-conformance/groups/mamba-py312-conformance/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-py312-conformance/groups/mamba-py312-conformance/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-py312-conformance/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-py312-conformance/issues/issue_*.md` — issues with `group: "mamba-py312-conformance"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-py312-conformance cclab/changes/mamba-py312-conformance/payloads/create-pre-clarifications.json
```