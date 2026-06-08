# Task: Clarify Group 'jet-build-aot-pipeline' for Change 'jet-remaining-issues'

## Context

Group: **jet-build-aot-pipeline**
Issues: #797_jet-build-validate-against-real-world-open-source-, #765_feat-jet-aot-production-build-tree-shaking-code-sp

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining-issues/groups/jet-build-aot-pipeline/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining-issues/groups/jet-build-aot-pipeline/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining-issues/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining-issues/issues/issue_*.md` — issues with `group: "jet-build-aot-pipeline"` in frontmatter

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
cclab sdd artifact create-pre-clarifications jet-remaining-issues cclab/changes/jet-remaining-issues/payloads/create-pre-clarifications.json
```