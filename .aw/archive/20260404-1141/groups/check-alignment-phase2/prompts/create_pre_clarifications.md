# Task: Clarify Group 'check-alignment-phase2' for Change '1141'

## Context

Group: **check-alignment-phase2**
Issues: #1141_feat-sdd-code-spec-coverage-mapping-check-alignmen

## Files to Read

- `/Users/chrischeng/projects/wt/sdd/cclab/changes/1141/groups/check-alignment-phase2/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/wt/sdd/cclab/changes/1141/groups/check-alignment-phase2/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/wt/sdd/cclab/changes/1141/user_input.md` — user's description
- `/Users/chrischeng/projects/wt/sdd/cclab/changes/1141/issues/issue_*.md` — issues with `group: "check-alignment-phase2"` in frontmatter

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
cclab sdd artifact create-pre-clarifications 1141 cclab/changes/1141/payloads/create-pre-clarifications.json
```