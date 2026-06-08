# Task: Clarify Group 'benchmark-suite' for Change 'all-mamba-p0'

## Context

Group: **benchmark-suite**
Issues: #836_benchmark-suite-mamba-vs-cpython-3-12-vs-pypy-perf

## Files to Read

- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/all-mamba-p0/groups/benchmark-suite/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/all-mamba-p0/groups/benchmark-suite/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/all-mamba-p0/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/all-mamba-p0/issues/issue_*.md` — issues with `group: "benchmark-suite"` in frontmatter

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
cclab sdd artifact create-pre-clarifications all-mamba-p0 cclab/changes/all-mamba-p0/payloads/create-pre-clarifications.json
```