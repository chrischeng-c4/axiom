# Task: Clarify Group 'issues-cli-crud' for Change 'issues-cli-crud'

## Context

Group: **issues-cli-crud**
Issues: #1179_notation-4-3-issue-authoring-notation-agent-for-ro

## Files to Read

- `/Users/chrischeng/projects/wt/conductor/.score/changes/issues-cli-crud/groups/issues-cli-crud/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/wt/conductor/.score/changes/issues-cli-crud/groups/issues-cli-crud/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/wt/conductor/.score/changes/issues-cli-crud/user_input.md` — user's description
- `/Users/chrischeng/projects/wt/conductor/.score/changes/issues-cli-crud/issues/issue_*.md` — issues with `group: "issues-cli-crud"` in frontmatter

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
score artifact create-pre-clarifications issues-cli-crud .score/changes/issues-cli-crud/payloads/create-pre-clarifications.json
```