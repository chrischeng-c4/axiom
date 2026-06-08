# Task: Clarify Group 'jet-dev-server-v2' for Change 'jet-dev-server-v2'

## Context

Group: **jet-dev-server-v2**
Issues: #1091_jet-dev-browser-compatible-node-js-builtin-polyfil, #1089_jet-dev-implement-optimizedeps-full-cjs-esm-pre-bu, #1092_jet-install-jet-store-symlinks-break-node-js-modul, #1090_jet-dev-ast-based-typescript-type-stripping-replac

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-dev-server-v2/groups/jet-dev-server-v2/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-dev-server-v2/groups/jet-dev-server-v2/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-dev-server-v2/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-dev-server-v2/issues/issue_*.md` — issues with `group: "jet-dev-server-v2"` in frontmatter

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
cclab sdd artifact create-pre-clarifications jet-dev-server-v2 cclab/changes/jet-dev-server-v2/payloads/create-pre-clarifications.json
```