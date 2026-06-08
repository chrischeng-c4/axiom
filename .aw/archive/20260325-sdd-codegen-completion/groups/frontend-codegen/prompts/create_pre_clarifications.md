# Task: Clarify Group 'frontend-codegen' for Change 'sdd-codegen-completion'

## Context

Group: **frontend-codegen**
Issues: #937_feat-sdd-frontend-codegen-wireframe-component-desi

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-completion/groups/frontend-codegen/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-completion/groups/frontend-codegen/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-completion/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-completion/issues/issue_*.md` — issues with `group: "frontend-codegen"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sdd-codegen-completion cclab/changes/sdd-codegen-completion/payloads/create-pre-clarifications.json
```