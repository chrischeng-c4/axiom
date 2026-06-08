# 1260-patrol-handoff

**Issue:** #1260 — enhancement(mamba): Perf bottoms — string_concat / list_sort_builtin / int_mul_l
**Branch:** `issue-1260` (materialization only, no CRRR started)
**Stop reason:** spec-quality-gate auto-stop — 2/4 required sections + grab-bag scope

## Why patrol bailed

Two independent reasons, either alone is sufficient:

### 1. Body has 2/4 required CRRR sections

Found:
- `## Updated Requirements` (R5', R6', R7' — concrete enough)
- `## Reference Context`

Missing:
- `## Problem` — no one-line root cause statement; the `## Status` table is a measurement snapshot, not a root cause
- `## Scope` — no in-scope / out-of-scope split; the issue blurs perf-bench tooling, schema changes, AND new bench coverage

Autopilot auto-stop fires at `< 3 of {Problem, Requirements, Scope, Reference Context}`.

### 2. Grab-bag scope crosses multiple feature areas

Even if sections were complete, the spec would cover **three distinct workstreams**:

- **R5'**: Restore tier targets per #1265 — fix codegen for 5 benches violating P4 floor.
- **R6'**: Extend `baseline.json` schema with `tier` field + per-tier enforcement in `mamba bench --check`. Tooling work.
- **R7'**: Build new bench coverage for tier:app (json round-trip, Flask, dict-heavy SM) and tier:dynamic (deepcopy, pickle, reflection). New fixtures + likely new stdlib stubs.

Autopilot's strict-scope rule says "stay within the spec's `## Changes` file list" and "if a file you need is NOT in `## Changes`, escalate to handoff — don't widen scope." A grab-bag spec breaks that on day one because the `## Changes` list would span `mamba/src/lowering/*.rs`, `mamba/bench/*.rs`, `mamba/tests/fixtures/**`, and 4-6 new stdlib stub files. Too wide for a single autopilot tick.

## What this issue needs (operator)

Split into **three child issues**, each CRRR-able independently:

1. `bench(mamba): tag existing baseline.json entries with tier:compute` — pure tooling, single file change, fast win.
2. `perf(mamba): close P4 floor for compute benches (factorial_recursive / fib_recursive / list_sort / string_concat / generator_sum)` — five sub-benches, each merits its own TD; tracking issue wraps them.
3. `bench(mamba): add tier:app + tier:dynamic coverage (json / Flask / dict-SM / deepcopy / pickle / reflection)` — likely depends on stdlib gaps; sequence after stub work.

Each child should carry a full `## Problem`, `## Requirements`, `## Scope`, `## Reference Context` to pass the autopilot gate.

## What patrol did

1. Materialized #1260 to `.score/issues/open/1260.md` on branch `issue-1260`.
2. Did NOT start CRRR — bailed at spec-quality-gate per autopilot policy.
3. Will FF the materialize commit into `project-mamba` so the cache survives the branch deletion.
4. Will add `flagged:needs-human` to #1260 so the next state-C scan skips it (the 24h handoff skip rule also applies).

## Tick timing

Started 12:29:23, bailed 12:30:18 — **~55s** total. Even cheaper than #1696's bail (~2m). Spec-quality-gate is doing its job: bad specs cost <1 min to triage.

## Note on autopilot order

The autopilot prompt's "p0 > p1 > p2 > p3, oldest first within tier" picked #1260 (oldest p0), but the three other p0s (#1312, #1313, #1546) are all `phase:td_reviewed` — they skip steps 1-4 entirely and go straight to handwrite. Throughput-optimal order would be `td_reviewed > td_created > fill_* > created` within tier. Consider amending the patrol prompt if bail-rate on `phase:created` p0s stays high.
