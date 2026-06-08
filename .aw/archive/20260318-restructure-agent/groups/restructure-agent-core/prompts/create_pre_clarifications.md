# Task: Clarify Group 'restructure-agent-core' for Change 'restructure-agent'

## Context

Group: **restructure-agent-core**
Issues: #900_feat-agent-restructure-agent-llm-based-prompt-refi

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/restructure-agent/groups/restructure-agent-core/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/restructure-agent/groups/restructure-agent-core/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/restructure-agent/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/restructure-agent/issues/issue_*.md` — issues with `group: "restructure-agent-core"` in frontmatter

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
cclab sdd artifact create-pre-clarifications restructure-agent cclab/changes/restructure-agent/payloads/create-pre-clarifications.json
```