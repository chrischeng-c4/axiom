---
number: 1124
title: "bug(sdd): reference_context phase never advances groups_progress — workflow stuck loop"
state: open
labels: [type:bug, priority:p1, crate:sdd]
group: "phase-advance-and-timeout"
---

# #1124 — bug(sdd): reference_context phase never advances groups_progress — workflow stuck loop

## Problem

When the `claude-agent:reference-context` agent completes and writes `reference_context.md` + `spec_plan.yaml` via CLI artifact command, `STATE.yaml` `groups_progress.reference_context` remains empty `[]`. The `sdd_run_change` router sees the group as incomplete and re-dispatches `create-reference-context` indefinitely.

## Reproduction

1. Start a new change with one group
2. Let reference-context agent complete (exit_code 0, artifacts written)
3. Run `cclab sdd run-change` — it loops back to `create_reference_context` instead of advancing

## Root Cause (suspected)

The artifact CLI (`sdd artifact create-reference-context` / `review-reference-context`) writes the file but does not update `groups_progress.reference_context` in STATE.yaml. The review step also completed via agent but didn't write `review_reference_context.md` — the agent exited 0 without actually calling the artifact CLI.

## Workaround

Manually edit STATE.yaml: set `phase: reference_context_reviewed` and add group to `groups_progress.reference_context`.

## Expected

After `sdd artifact review-reference-context` with verdict=approved, STATE.yaml should automatically:
1. Add group to `groups_progress.reference_context`
2. Advance phase when all groups are complete
