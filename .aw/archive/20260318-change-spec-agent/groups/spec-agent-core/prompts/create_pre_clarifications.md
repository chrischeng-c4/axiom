# Task: Clarify Group 'spec-agent-core' for Change 'change-spec-agent'

## Context

Group: **spec-agent-core**
Issues: #923_feat-agent-add-specagent-opinionated-spec-generati

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-spec-agent/groups/spec-agent-core/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-spec-agent/groups/spec-agent-core/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-spec-agent/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-spec-agent/issues/issue_*.md` — issues with `group: "spec-agent-core"` in frontmatter

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
cclab sdd artifact create-pre-clarifications change-spec-agent cclab/changes/change-spec-agent/payloads/create-pre-clarifications.json
```