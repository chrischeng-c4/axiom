# Task: Clarify Group 'merge-3way' for Change 'sdd-merge-3way'

## Context

Group: **merge-3way**
Issues: #1128_feat-sdd-gen-code-gen-diff-gen-parse-spec-driven-c

## Files to Read

- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-merge-3way/groups/merge-3way/requirements.md` — consolidated requirements
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-merge-3way/groups/merge-3way/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-merge-3way/user_input.md` — user's description
- `/Users/chrischeng/projects/cclab-sdd/cclab/changes/sdd-merge-3way/issues/issue_*.md` — issues with `group: "merge-3way"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sdd-merge-3way cclab/changes/sdd-merge-3way/payloads/create-pre-clarifications.json
```