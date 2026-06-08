# Task: Clarify Group 'jet-resolver-bare-pkg-alias' for Change 'fix-bare-pkg-alias'

## Context

Group: **jet-resolver-bare-pkg-alias**
Issues: #957_jet-install-resolver-fails-on-bare-package-name-as, #883_jet-install-resolver-bugs-fixed-version-conflicts-

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-bare-pkg-alias/groups/jet-resolver-bare-pkg-alias/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-bare-pkg-alias/groups/jet-resolver-bare-pkg-alias/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-bare-pkg-alias/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-bare-pkg-alias/issues/issue_*.md` — issues with `group: "jet-resolver-bare-pkg-alias"` in frontmatter

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
cclab sdd artifact create-pre-clarifications fix-bare-pkg-alias cclab/changes/fix-bare-pkg-alias/payloads/create-pre-clarifications.json
```