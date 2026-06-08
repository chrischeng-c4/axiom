# Task: Clarify Group 'sync-adapter' for Change 'sync-adapter'

## Context

Group: **sync-adapter**
Issues: #959_feat-agent-add-syncadapter-trait-platform-sync-ada

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/sync-adapter/groups/sync-adapter/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/sync-adapter/groups/sync-adapter/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/sync-adapter/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/sync-adapter/issues/issue_*.md` — issues with `group: "sync-adapter"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sync-adapter cclab/changes/sync-adapter/payloads/create-pre-clarifications.json
```