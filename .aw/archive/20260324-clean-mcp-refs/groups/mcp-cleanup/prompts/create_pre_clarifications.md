# Task: Clarify Group 'mcp-cleanup' for Change 'clean-mcp-refs'

## Context

Group: **mcp-cleanup**
Issues: #1047_refactor-sdd-clean-up-stale-mcp-references-all-too

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/clean-mcp-refs/groups/mcp-cleanup/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/clean-mcp-refs/groups/mcp-cleanup/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/clean-mcp-refs/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/clean-mcp-refs/issues/issue_*.md` — issues with `group: "mcp-cleanup"` in frontmatter

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
cclab sdd artifact create-pre-clarifications clean-mcp-refs cclab/changes/clean-mcp-refs/payloads/create-pre-clarifications.json
```