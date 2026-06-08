# Task: Clarify Group 'spec-consolidation-enforcement' for Change 'spec-consolidation'

## Context

Group: **spec-consolidation-enforcement**
Issues: #1039_fix-sdd-enforce-spec-consolidation-prevent-scatter

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-consolidation/groups/spec-consolidation-enforcement/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-consolidation/groups/spec-consolidation-enforcement/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-consolidation/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/spec-consolidation/issues/issue_*.md` — issues with `group: "spec-consolidation-enforcement"` in frontmatter

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
cclab sdd artifact create-pre-clarifications spec-consolidation cclab/changes/spec-consolidation/payloads/create-pre-clarifications.json
```