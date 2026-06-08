# Task: Clarify Group 'jet-build-aot-production' for Change 'all-open-jet-issues'

## Context

Group: **jet-build-aot-production**
Issues: #903_jet-build-scope-hoisting-phase-2-true-module-flatt, #882_jet-build-bundle-size-215kb-vs-webpack-192kb-imple, #765_feat-jet-aot-production-build-tree-shaking-code-sp

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-open-jet-issues/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-open-jet-issues/issues/issue_*.md` — issues with `group: "jet-build-aot-production"` in frontmatter

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
cclab sdd artifact create-pre-clarifications all-open-jet-issues cclab/changes/all-open-jet-issues/payloads/create-pre-clarifications.json
```