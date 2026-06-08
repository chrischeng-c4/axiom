# Task: Clarify Group 'unify-mamba-config' for Change '1134-mamba-dual-config'

## Context

Group: **unify-mamba-config**
Issues: #1134_mamba-dual-mambaconfig-structs-driver-config-rs-vs

## Files to Read

- `/Users/chris.cheng/cclab/main/.score/changes/1134-mamba-dual-config/groups/unify-mamba-config/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/main/.score/changes/1134-mamba-dual-config/groups/unify-mamba-config/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/main/.score/changes/1134-mamba-dual-config/user_input.md` — user's description
- `/Users/chris.cheng/cclab/main/.score/changes/1134-mamba-dual-config/issues/issue_*.md` — issues with `group: "unify-mamba-config"` in frontmatter

## Instructions

1. Read requirements.md and pre_clarifications.md for this group
2. The pre_clarifications.md contains pre-generated questions — use these as your starting point
3. Use AskUserQuestion to ask the pre-generated questions to the user
4. After answers, evaluate: did answers raise new questions?
5. If more clarification needed: ask follow-up questions
6. When sufficient: run `score artifact create-pre-clarifications` with answers

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
score artifact create-pre-clarifications 1134-mamba-dual-config .score/changes/1134-mamba-dual-config/payloads/create-pre-clarifications.json
```