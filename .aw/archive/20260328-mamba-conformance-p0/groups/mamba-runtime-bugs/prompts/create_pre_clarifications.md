# Task: Clarify Group 'mamba-runtime-bugs' for Change 'mamba-conformance-p0'

## Context

Group: **mamba-runtime-bugs**
Issues: #1084_fix-mamba-decorator-return-value-is-none-calling-c, #1085_fix-mamba-floor-division-by-zero-doesn-t-raise-zer

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-p0/groups/mamba-runtime-bugs/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-p0/groups/mamba-runtime-bugs/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-p0/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-p0/issues/issue_*.md` — issues with `group: "mamba-runtime-bugs"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-conformance-p0 cclab/changes/mamba-conformance-p0/payloads/create-pre-clarifications.json
```