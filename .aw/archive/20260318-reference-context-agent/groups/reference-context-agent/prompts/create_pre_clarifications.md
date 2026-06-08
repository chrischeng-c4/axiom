# Task: Clarify Group 'reference-context-agent' for Change 'reference-context-agent'

## Context

Group: **reference-context-agent**
Issues: #928_feat-agent-add-referencecontextagent-spec-explorat

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/reference-context-agent/groups/reference-context-agent/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/reference-context-agent/groups/reference-context-agent/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/reference-context-agent/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/reference-context-agent/issues/issue_*.md` — issues with `group: "reference-context-agent"` in frontmatter

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
cclab sdd artifact create-pre-clarifications reference-context-agent cclab/changes/reference-context-agent/payloads/create-pre-clarifications.json
```