# Task: Clarify Group 'agent-crr-core' for Change 'agent-crr-review'

## Context

Group: **agent-crr-core**
Issues: #924_feat-agent-add-reviewagent-crr-review-for-specs-an, #926_feat-agent-add-crr-cycle-generic-create-review-rev

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-crr-review/groups/agent-crr-core/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-crr-review/groups/agent-crr-core/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-crr-review/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/agent-crr-review/issues/issue_*.md` — issues with `group: "agent-crr-core"` in frontmatter

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
cclab sdd artifact create-pre-clarifications agent-crr-review cclab/changes/agent-crr-review/payloads/create-pre-clarifications.json
```