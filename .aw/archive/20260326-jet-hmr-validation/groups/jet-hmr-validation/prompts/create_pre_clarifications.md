# Task: Clarify Group 'jet-hmr-validation' for Change 'jet-hmr-validation'

## Context

Group: **jet-hmr-validation**
Issues: #1119_jet-dev-validate-with-conductor-fe-real-world-reac, #1118_jet-dev-javascript-module-hmr-hot-module-replaceme

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-hmr-validation/groups/jet-hmr-validation/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-hmr-validation/groups/jet-hmr-validation/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-hmr-validation/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-hmr-validation/issues/issue_*.md` — issues with `group: "jet-hmr-validation"` in frontmatter

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
cclab sdd artifact create-pre-clarifications jet-hmr-validation cclab/changes/jet-hmr-validation/payloads/create-pre-clarifications.json
```