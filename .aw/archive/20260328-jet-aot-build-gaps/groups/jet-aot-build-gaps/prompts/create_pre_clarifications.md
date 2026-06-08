# Task: Clarify Group 'jet-aot-build-gaps' for Change 'jet-aot-build-gaps'

## Context

Group: **jet-aot-build-gaps**
Issues: #765_feat-jet-aot-production-build-tree-shaking-code-sp

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-aot-build-gaps/groups/jet-aot-build-gaps/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-aot-build-gaps/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-aot-build-gaps/issues/issue_*.md` — issues with `group: "jet-aot-build-gaps"` in frontmatter

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
cclab sdd artifact create-pre-clarifications jet-aot-build-gaps cclab/changes/jet-aot-build-gaps/payloads/create-pre-clarifications.json
```