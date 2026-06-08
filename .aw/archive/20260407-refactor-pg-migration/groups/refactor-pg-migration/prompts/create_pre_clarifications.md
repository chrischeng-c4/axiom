# Task: Clarify Group 'refactor-pg-migration' for Change 'refactor-pg-migration'

## Context

Group: **refactor-pg-migration**
Issues: #1196_refactor-remove-conductor-specific-migration-code-

## Files to Read

- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/refactor-pg-migration/groups/refactor-pg-migration/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/refactor-pg-migration/groups/refactor-pg-migration/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/refactor-pg-migration/user_input.md` — user's description
- `/Users/chris.cheng/cclab/wt/conductor/.score/changes/refactor-pg-migration/issues/issue_*.md` — issues with `group: "refactor-pg-migration"` in frontmatter

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
score artifact create-pre-clarifications refactor-pg-migration .score/changes/refactor-pg-migration/payloads/create-pre-clarifications.json
```