# Task: Clarify Group 'lens-dissolution' for Change 'lens-dissolution'

## Context

Group: **lens-dissolution**
Issues: #944_feat-lens-wire-cross-file-type-propagation-deep-in, #946_feat-lens-agent-context-builder-smart-file-selecti, #949_feat-lens-agent-optimized-output-structured-json-f, #1087_refactor-dissolve-lens-module-into-sdd-top-level-s

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/lens-dissolution/groups/lens-dissolution/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/lens-dissolution/groups/lens-dissolution/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/lens-dissolution/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/lens-dissolution/issues/issue_*.md` — issues with `group: "lens-dissolution"` in frontmatter

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
cclab sdd artifact create-pre-clarifications lens-dissolution cclab/changes/lens-dissolution/payloads/create-pre-clarifications.json
```