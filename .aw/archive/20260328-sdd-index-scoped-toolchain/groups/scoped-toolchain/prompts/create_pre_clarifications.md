# Task: Clarify Group 'scoped-toolchain' for Change 'sdd-index-scoped-toolchain'

## Context

Group: **scoped-toolchain**
Issues: #1127_feat-sdd-index-server-scoped-toolchain-binding-aut

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-index-scoped-toolchain/groups/scoped-toolchain/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-index-scoped-toolchain/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-index-scoped-toolchain/issues/issue_*.md` — issues with `group: "scoped-toolchain"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sdd-index-scoped-toolchain cclab/changes/sdd-index-scoped-toolchain/payloads/create-pre-clarifications.json
```