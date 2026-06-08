# Task: Clarify Group 'jet-infra-codesign' for Change 'all-jet-issues'

## Context

Group: **jet-infra-codesign**
Issues: #906_chore-jet-codesign-release-binary-macos-kills-unsi

## Files to Read

- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-jet-issues/groups/jet-infra-codesign/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-jet-issues/groups/jet-infra-codesign/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-jet-issues/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-jet-issues/issues/issue_*.md` — issues with `group: "jet-infra-codesign"` in frontmatter

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
cclab sdd artifact create-pre-clarifications all-jet-issues cclab/changes/all-jet-issues/payloads/create-pre-clarifications.json
```