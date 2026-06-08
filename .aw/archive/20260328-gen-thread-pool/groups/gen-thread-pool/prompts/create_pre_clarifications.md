# Task: Clarify Group 'gen-thread-pool' for Change 'gen-thread-pool'

## Context

Group: **gen-thread-pool**
Issues: #1114_fix-mamba-sigbus-crash-in-multi-threaded-conforman

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/gen-thread-pool/groups/gen-thread-pool/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/gen-thread-pool/groups/gen-thread-pool/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/gen-thread-pool/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/gen-thread-pool/issues/issue_*.md` — issues with `group: "gen-thread-pool"` in frontmatter

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
cclab sdd artifact create-pre-clarifications gen-thread-pool cclab/changes/gen-thread-pool/payloads/create-pre-clarifications.json
```