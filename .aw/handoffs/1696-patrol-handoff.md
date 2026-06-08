# 1696-patrol-handoff

**Issue:** #1696 — codegen(jit): Cranelift verifier rejects 3-arg call to 2-arg sig in test_bool follow-up
**Branch:** `issue-1696` (materialization only, no CRRR started)
**Stop reason:** spec-quality-gate auto-stop — pre-CRRR

## Why patrol bailed

The autopilot's pre-handwrite spec-quality-gate requires the issue body to carry **at least 3 of {Problem, Requirements, Scope, Reference Context}** so that the TD spec has concrete acceptance criteria to drive code against.

#1696's body is structured as a **deep-dive bug report**, not a CRRR-ready work-item. Section headers present:

- `## Summary` — narrative of the verifier reject
- `## Reproduction` — `cargo build + mamba run test_bool.py` repro steps
- `## Hypothesis` — three competing theories (method-resolution cache, static-vs-bound dispatch, ABI mismatch)
- `## Investigation plan` — diagnostic next-steps (IR dump, instrument resolver, bisect across #1691)
- `## Acceptance` — likely fuzzy "verifier no longer rejects" (not a list of testable invariants)

No `## Requirements`, no `## Scope`, no `## Reference Context`. The CRRR fill ladder cannot run on this — `score wi fill-section Requirements` would have nothing to validate against, and the TD reviewer cannot judge "covers requirements" with no requirements present.

## What this issue actually needs (operator)

This is a **JIT-deep, cross-method-state bug** — the kind that needs investigation before it can be specced:

1. Run the IR-dump diagnostic in the issue's Investigation plan. Identify which call site emits `u0:347` with 3 args. Trace the SymbolId allocation back to the method-resolver pass.
2. Confirm or refute each of the three hypotheses (method-resolution cache miss / static-vs-bound dispatch leak / argument-vector receiver shift).
3. Write a follow-up issue with full CRRR sections:
   - **Problem**: one-line statement of root cause (post-investigation).
   - **Requirements**: testable invariants (e.g. "method resolution MUST register sig matching the call-site argument count for every SymbolId").
   - **Scope**: which files in `projects/mamba/src/lowering/` and `runtime/` the fix touches.
   - **Reference Context**: prior #1691 fix, the test_bool fixture, related Cranelift verifier rules.
4. Run autopilot on the **follow-up issue**, not on this one.

## What patrol did

1. Materialized #1696 to `.score/issues/open/1696.md` (commit `5b3c60bd4 chore(score): materialize #1696 for autopilot`).
2. Did NOT start CRRR — bailed at spec-quality-gate per autopilot policy.
3. Will FF the materialize commit into `project-mamba` so the issue cache survives the branch deletion.

## Tag

Patrol will add `flagged:needs-human` to #1696 on GitHub so the next state-C scan skips it (the 24-hour handoff skip rule kicks in regardless).

## Timing this tick (autopilot probe)

Started 11:52:05, bailed 11:53:58 — **~1m 53s** total for the "auto-stop on bad spec" path. Useful data: bad specs cost ~2 min to triage. The 30-min cron interval would absorb 15 such bails per hour without piling up.
