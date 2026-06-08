# 1312-patrol-handoff (covers #1312, #1313, #1546)

**Issues:** #1312, #1313, #1546 — all titled `Adopted: <N>` with empty stub bodies
**Branch:** `issue-1312` (materialization only; #1313 and #1546 flagged but not branched)
**Stop reason:** Adopted-stub cluster — no spec content to drive autopilot

## What these stubs are

Three p0 `phase:td_reviewed` issues created by `score td claim --from-path`:

| Issue | Title | Phase | Projects |
|-------|-------|-------|----------|
| #1312 | Adopted: 1312 | td_reviewed | cue, mamba |
| #1313 | Adopted: 1313 | td_reviewed | (likely same) |
| #1546 | Adopted: 1546 | td_reviewed | (likely same) |

All three bodies are literally:

```
# Adopted: <N>

Issue stub created by `score td claim --from-path`.
```

No `## Problem`, no `## Requirements`, no `## Scope`, no `## Reference Context`, no `## Changes`. No spec file under `.score/tech_design/`. No commits in git history reference these issue numbers.

The `score cb-claim` skill (adopt existing code into score by generating a TD spec via the fillback pipeline) creates these stubs as placeholders during adoption flows — but for #1312/1313/1546 the fillback pipeline apparently never completed, so the placeholder stayed and the phase got bumped to `td_reviewed` without a real TD body.

## Why autopilot bails

- `score td merge` needs a `--spec-path` — there isn't one.
- Handwrite step has no `## Changes` file list to consume.
- All four auto-stop conditions trip: empty Changes, missing Problem/Requirements/Scope/RefCtx (0/4 vs the <3 threshold), no actionable phase.

These are not implementation issues — they are housekeeping artifacts from an aborted adoption pipeline.

## What operator should do

For each of #1312, #1313, #1546, one of two actions:

1. **Close as adopted-complete** if the underlying code already shipped under a different issue/PR — these stubs are no-op trackers and can be closed with comment `Adopted via <real-issue-#>, closing stub`.
2. **Rehydrate the stub** if the adoption is genuinely pending: re-run `score cb-claim --from-path <path>` with non-interactive mode (#2076 / Bug 2 fix wired this up — see commit `61d730d0f impl(score-cli,sdd): non-interactive mode for cb claim / fillback`). That produces a real TD spec and lets autopilot pick it up.

Bulk operation suggestion:

```bash
for n in 1312 1313 1546; do
  gh issue close "$n" --repo chrischeng-c4/cclab --comment "Closing Adopted: $n stub — no associated code/spec, replaced by real CRRR flow"
done
```

…OR, if rehydration is the right path:

```bash
for n in 1312 1313 1546; do
  gh issue edit "$n" --repo chrischeng-c4/cclab --remove-label "phase:td_reviewed" --add-label "phase:created"
  # then re-run score cb-claim with the original --from-path argument (stored in cb-claim history if available)
done
```

## What patrol did

1. Materialized #1312 to `.score/issues/open/1312.md` on branch `issue-1312`.
2. Bulk-flagged #1312, #1313, #1546 with `flagged:needs-human` so the next state-C scan skips all three immediately (no need to wait for 24h handoff-skip per slug — #1313 and #1546 don't have individual handoffs but the label skip-filter catches them).
3. Will FF the materialize commit into `project-mamba` so the #1312 cache survives.

## Tick timing

Started 12:59:27, bailed 13:00:56 — **~1m 29s** for the bail-cluster path. Bulk-flagging 3 stubs in one tick saves 2 future ticks (~3-4 min of patrol wallclock).

## Side observation

After this handoff, the p0 mamba candidate list collapses to just #1260 (already handoff'd for grab-bag-split, 24h skip until 2026-05-14 ~12:30). For the next ~24 h the patrol will drop straight to p1. p1 has 9 candidates; #1234 / #1258 / #1262 are the oldest and merit checking next tick.
