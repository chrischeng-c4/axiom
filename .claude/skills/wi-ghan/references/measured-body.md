# Measured red reference body for wi_draft_gate

## Goal

Provide a reference body whose acceptance rows fail against the checkout.

## How

### Verified premises

- `.agents/rules/authoring/agent-instruction-ghan.md:21` - `python3` is available in the environment.

### Change points

- `.claude/skills/wi-ghan/scripts/wi_draft_gate.py` - measured reference body.

### Frozen decisions

- The command exits non-zero to serve as a measured red baseline.

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---------|---------|--------|--------------------------------|
| 1 | `python3 -c "import sys; sys.exit(1)"` | exits 1 | exits 0 | command exists and runs but fails |

### Negative control

At the revision these premises were read, `shasum -a 256 .claude/skills/wi-ghan/scripts/wi_draft_gate.py` reports `b8d07f3c8fca541b006ad4b581b7090678a61e223f04f66bf3e8b6f0c9d2c596`; that digest changes once this work lands, so record the post-change digest immediately before mutating. Then delete only the new measurement so the script returns to printing `PASS` on the structural verdict alone, leaving both the suite and the reference body untouched, and rerun row 1. The gate must go red, and the report must quote the failure verbatim, including the failing test name and the assertion values. Restore the file by writing the original bytes back with an editor, never with `cp -p` or any copy that preserves mtime, confirm `shasum -a 256` reports the digest recorded before the mutation, and rerun both rows to return them to green.

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

- none

### Must not do

- none
