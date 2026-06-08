# Task: Clarify Group 'gen-code-pipeline' for Change 'sdd-gen-code-pipeline'

## Context

Group: **gen-code-pipeline**
Issues: #1128_feat-sdd-gen-code-gen-diff-gen-parse-spec-driven-c

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-gen-code-pipeline/groups/gen-code-pipeline/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-gen-code-pipeline/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-gen-code-pipeline/issues/issue_*.md` — issues with `group: "gen-code-pipeline"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sdd-gen-code-pipeline cclab/changes/sdd-gen-code-pipeline/payloads/create-pre-clarifications.json
```