# Task: Clarify Group 'mamba-codegen-runtime-fixes' for Change 'mamba-p0-bugfix'

## Context

Group: **mamba-codegen-runtime-fixes**
Issues: #1109_fix-mamba-no-arg-constructor-codegen-verifier-erro, #1114_fix-mamba-sigbus-crash-in-multi-threaded-conforman, #1111_fix-mamba-string-reverse-slice-1-returns-empty-str

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-p0-bugfix/groups/mamba-codegen-runtime-fixes/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-p0-bugfix/groups/mamba-codegen-runtime-fixes/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-p0-bugfix/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-p0-bugfix/issues/issue_*.md` — issues with `group: "mamba-codegen-runtime-fixes"` in frontmatter

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
cclab sdd artifact create-pre-clarifications mamba-p0-bugfix cclab/changes/mamba-p0-bugfix/payloads/create-pre-clarifications.json
```