# Task: Clarify Group 'token-counting-and-compact' for Change 'cclab-agent-p0'

## Context

Group: **token-counting-and-compact**
Issues: #786_feat-agent-add-accurate-token-counting, #876_feat-agent-smart-auto-compact-llm-summarization-ac

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/cclab-agent-p0/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/cclab-agent-p0/issues/issue_*.md` — issues with `group: "token-counting-and-compact"` in frontmatter

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
cclab sdd artifact create-pre-clarifications cclab-agent-p0 cclab/changes/cclab-agent-p0/payloads/create-pre-clarifications.json
```