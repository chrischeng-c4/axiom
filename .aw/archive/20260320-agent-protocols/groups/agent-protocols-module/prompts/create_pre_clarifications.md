# Task: Clarify Group 'agent-protocols-module' for Change 'agent-protocols'

## Context

Group: **agent-protocols-module**
Issues: #958_feat-agent-add-protocols-module-domain-contracts-f

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-protocols/groups/agent-protocols-module/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-protocols/groups/agent-protocols-module/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-protocols/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-protocols/issues/issue_*.md` — issues with `group: "agent-protocols-module"` in frontmatter

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
cclab sdd artifact create-pre-clarifications agent-protocols cclab/changes/agent-protocols/payloads/create-pre-clarifications.json
```