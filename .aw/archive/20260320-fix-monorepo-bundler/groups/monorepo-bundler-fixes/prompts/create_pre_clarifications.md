# Task: Clarify Group 'monorepo-bundler-fixes' for Change 'fix-monorepo-bundler'

## Context

Group: **monorepo-bundler-fixes**
Issues: #962_jet-build-bundler-resolver-doesn-t-walk-up-to-mono

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-monorepo-bundler/groups/monorepo-bundler-fixes/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-monorepo-bundler/groups/monorepo-bundler-fixes/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-monorepo-bundler/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-monorepo-bundler/issues/issue_*.md` — issues with `group: "monorepo-bundler-fixes"` in frontmatter

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
cclab sdd artifact create-pre-clarifications fix-monorepo-bundler cclab/changes/fix-monorepo-bundler/payloads/create-pre-clarifications.json
```