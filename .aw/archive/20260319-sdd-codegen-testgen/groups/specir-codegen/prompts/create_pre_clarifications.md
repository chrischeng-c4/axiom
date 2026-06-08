# Task: Clarify Group 'specir-codegen' for Change 'sdd-codegen-testgen'

## Context

Group: **specir-codegen**
Issues: #932_feat-sdd-codegen-last-mile-consume-specir-to-gener

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-testgen/groups/specir-codegen/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-testgen/groups/specir-codegen/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-testgen/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-codegen-testgen/issues/issue_*.md` — issues with `group: "specir-codegen"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sdd-codegen-testgen cclab/changes/sdd-codegen-testgen/payloads/create-pre-clarifications.json
```