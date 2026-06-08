# Task: Clarify Group 'resolver-compat' for Change 'resolver-compat'

## Context

Group: **resolver-compat**
Issues: #960_jet-install-resolver-fails-on-hyphen-range-syntax-, #957_jet-install-resolver-fails-on-bare-package-name-as, #883_jet-install-resolver-bugs-fixed-version-conflicts-

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/resolver-compat/groups/resolver-compat/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/resolver-compat/groups/resolver-compat/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/resolver-compat/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/resolver-compat/issues/issue_*.md` — issues with `group: "resolver-compat"` in frontmatter

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
cclab sdd artifact create-pre-clarifications resolver-compat cclab/changes/resolver-compat/payloads/create-pre-clarifications.json
```