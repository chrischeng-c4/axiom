# Task: Clarify Group 'phase-advance-and-timeout' for Change 'sdd-phase-advance-timeout'

## Context

Group: **phase-advance-and-timeout**
Issues: #1126_feat-sdd-agent-execution-timeout-prevent-infinite-, #1124_bug-sdd-reference-context-phase-never-advances-gro

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-phase-advance-timeout/groups/phase-advance-and-timeout/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-phase-advance-timeout/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-phase-advance-timeout/issues/issue_*.md` — issues with `group: "phase-advance-and-timeout"` in frontmatter

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
cclab sdd artifact create-pre-clarifications sdd-phase-advance-timeout cclab/changes/sdd-phase-advance-timeout/payloads/create-pre-clarifications.json
```