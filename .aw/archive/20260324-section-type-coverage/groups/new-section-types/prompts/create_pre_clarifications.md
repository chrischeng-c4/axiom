# Task: Clarify Group 'new-section-types' for Change 'section-type-coverage'

## Context

Group: **new-section-types**
Issues: #1053_sdd-add-e2e-scenario-section-type-for-qa, #1055_sdd-add-qa-section-types-test-fixture-perf-test, #1051_epic-sdd-section-type-coverage-all-roles-fe-be-sre, #1057_sdd-add-backend-mle-agent-section-types-grpc-graph, #1056_sdd-add-sre-section-types-container-deploy-cloud-r, #1054_sdd-add-security-section-types-threat-model-auth-m

## Files to Read

- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/section-type-coverage/groups/new-section-types/requirements.md` — consolidated requirements
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/section-type-coverage/groups/new-section-types/pre_clarifications.md` — pre-generated questions (status: pending)
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/section-type-coverage/user_input.md` — user's description
- `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/section-type-coverage/issues/issue_*.md` — issues with `group: "new-section-types"` in frontmatter

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
cclab sdd artifact create-pre-clarifications section-type-coverage cclab/changes/section-type-coverage/payloads/create-pre-clarifications.json
```