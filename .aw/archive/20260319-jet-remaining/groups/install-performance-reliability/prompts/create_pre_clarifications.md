# Task: Clarify Group 'install-performance-reliability' for Change 'jet-remaining'

## Context

Group: **install-performance-reliability**
Issues: #883_jet-install-resolver-bugs-fixed-version-conflicts-, #881_jet-install-cold-install-4-9s-vs-pnpm-3-4s-optimiz

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining/groups/install-performance-reliability/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining/groups/install-performance-reliability/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/jet-remaining/issues/issue_*.md` — issues with `group: "install-performance-reliability"` in frontmatter

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
cclab sdd artifact create-pre-clarifications jet-remaining cclab/changes/jet-remaining/payloads/create-pre-clarifications.json
```