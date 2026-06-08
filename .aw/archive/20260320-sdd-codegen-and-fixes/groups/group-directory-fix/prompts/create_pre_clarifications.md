# Task: Clarify Group 'group-directory-fix' for Change 'sdd-codegen-and-fixes'

## Context

Group: **group-directory-fix**
Issues: #956_bug-sdd-payloads-prompts-and-specs-placed-at-chang

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-and-fixes/groups/group-directory-fix/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-and-fixes/groups/group-directory-fix/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-and-fixes/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-and-fixes/issues/issue_*.md` — issues with `group: "group-directory-fix"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sdd-codegen-and-fixes cclab/changes/sdd-codegen-and-fixes/payloads/create-pre-clarifications.json
```