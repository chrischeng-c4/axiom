---
number: 463
title: "bug(sdd): sdd_run_change skips spec creation phase after proposal_approved"
state: open
labels: [bug, crate:sdd]
---

# #463 — bug(sdd): sdd_run_change skips spec creation phase after proposal_approved

## Bug Description

`sdd_run_change` skips the spec creation phase entirely after `proposal_approved`, jumping directly to `generate_tasks`. This results in empty `specs/` directory and a single generic task instead of spec-driven implementation tasks.

## Steps to Reproduce

1. Create a change with a proposal containing 11 `spec_plan` entries
2. Review and approve the proposal → phase becomes `proposal_approved`
3. Call `sdd_run_change` — it returns:
   ```json
   {
     "action": "generate_tasks",
     "missing_specs_count": 0,
     "spec_count": 0
   }
   ```
4. `missing_specs_count: 0` is incorrect — there are 11 specs in the proposal's `spec_plan`
5. `specs/` directory remains empty

## Expected Behavior

After `proposal_approved`, `sdd_run_change` should:
1. Detect 11 spec_plan entries in proposal.md
2. Return `action: "create_spec"` for each missing spec
3. Only proceed to `generate_tasks` after `all_specs_approved`

## Actual Behavior

- Skips spec creation entirely
- Returns `generate_tasks` with `missing_specs_count: 0`
- Auto-generated tasks.md has only 1 generic task: "Implement changes as described in proposal"

## Impact

- Empty `specs/` directory
- No detailed implementation specs for agents
- Tasks are not properly broken down from specs
- Workflow integrity compromised

## Context

- Change ID: `mamba-p2`
- SDD version: 0.3.11
- Phase flow: `proposal_approved` → (should be `create_spec`) → skipped to `generate_tasks`
