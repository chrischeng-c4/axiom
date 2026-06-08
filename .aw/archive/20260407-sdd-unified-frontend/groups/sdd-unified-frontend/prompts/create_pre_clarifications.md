# Task: Clarify Group 'sdd-unified-frontend' for Change 'sdd-unified-frontend'

## Context

Group: **sdd-unified-frontend**
Issues: #1183_extract-plan-viewer-dashboard-ui-into-packages-ccl

## Files to Read

- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/sdd-unified-frontend/groups/sdd-unified-frontend/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/sdd-unified-frontend/groups/sdd-unified-frontend/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/sdd-unified-frontend/user_input.md` — user's description
- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/sdd-unified-frontend/issues/issue_*.md` — issues with `group: "sdd-unified-frontend"` in frontmatter

## Instructions

1. Read requirements.md and pre_clarifications.md for this group
2. The pre_clarifications.md contains pre-generated questions — use these as your starting point
3. Use AskUserQuestion to ask the pre-generated questions to the user
4. After answers, evaluate: did answers raise new questions?
5. If more clarification needed: ask follow-up questions
6. When sufficient: run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications sdd-unified-frontend .score/changes/sdd-unified-frontend/payloads/create-pre-clarifications.json
```