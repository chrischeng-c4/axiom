# Task: Clarify Group 'code-agent-core' for Change 'change-impl-agent'

## Context

Group: **code-agent-core**
Issues: #925_feat-agent-add-codeagent-spec-to-code-with-server-

## Files to Read

- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-impl-agent/groups/code-agent-core/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-impl-agent/groups/code-agent-core/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-impl-agent/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/change-impl-agent/issues/issue_*.md` — issues with `group: "code-agent-core"` in frontmatter

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
cclab sdd artifact create-pre-clarifications change-impl-agent cclab/changes/change-impl-agent/payloads/create-pre-clarifications.json
```