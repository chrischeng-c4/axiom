---
change: sdd-structured-issue
group: structured-issue-format
date: 2026-04-09
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| ? | - | high | init_change routes to restructure_input after creation — skip target is post_clarifications_created, Phase transitions: ChangeInited → InputRestructured → PreClarificationsCreated → ReferenceContext*, Structured issues skip directly from ChangeInited to post_clarifications_created phase |
| ? | - | high | Output: groups/{id}/requirements.md + pre_clarifications.md stub, init_change must replicate group creation when skipping this phase |
| ? | - | high | Output: pre_clarifications.md with status: answered, init_change writes this with Key Decisions from structured issue |
| ? | - | high | Output: reference_context.md with spec table + spec_plan, Structured issue's Reference Context section provides this directly |
| ? | - | medium | Output: post_clarifications.md with scope_summary, Structured issue's Scope + Acceptance Criteria sections replace this |
| ? | - | medium | Issue format, CLI commands — enrich subcommand extends this |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| sdd-structured-issue | create | crates/sdd/logic/structured-issue.md | overview, schema, cli, changes |

