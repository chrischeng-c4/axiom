# 1885-patrol-handoff

**Issue:** #1885 — enhancement(mamba): Goal 3 — 100% test coverage requires building the measurement baseline first
**Branch:** `issue-1885` (materialization only, no CRRR started)
**Stop reason:** explicit umbrella issue — same auto-stop trigger as #1749

## Why patrol bailed

The body self-declares as an umbrella with 4 child issues to file:

> ## Single-fire children to file
> Each becomes its own `priority:p1, project:mamba` issue when this umbrella is approved:
> 1. Coverage baseline — install `cargo-llvm-cov`, commit `COVERAGE-BASELINE.md`
> 2. CI artifact — `cargo llvm-cov --json` step + workflow artifact
> 3. PR gate — reject PRs that drop total coverage by >0.5pp
> 4. Untested file enumeration — `COVERAGE-GAPS.md` auto-sorted list

The umbrella's own acceptance criteria are not implementation criteria — they're meta-criteria ("all four child issues filed and linked back here"). Autopilot cannot satisfy that in one tick; it would have to file 4 GitHub issues, drive at least one of them through full CRRR + handwrite, and amend this umbrella with backlinks. That's a 4–5-tick task at best.

Header audit:

| Required section | Present? | Notes |
|------------------|----------|-------|
| `## Problem` | ✗ | `## Why this issue exists` is the closest, content-equivalent |
| `## Requirements` | ✗ | implicit in `## Single-fire children to file` (which is structurally Scope, not Requirements) |
| `## Scope` | partial | `## Out of scope` is present, but no in-scope file list |
| `## Reference Context` | ✗ | `## Refs` is a 3-link list |

Strictly 0/4 by header name; even charitably 1/4 (`Out of scope` ≈ Scope). Below the gate.

## What this issue actually needs (operator)

File 4 child issues per the umbrella body. Suggested filings (each with full CRRR sections):

1. **`coverage(mamba): install cargo-llvm-cov + commit baseline`**
   - `## Changes`: `Cargo.toml` (workspace dev-dep on cargo-llvm-cov is *not* the install; install is a tools step), `projects/mamba/COVERAGE-BASELINE.md` (new, frozen file).
   - DoD: re-running `cargo llvm-cov --package mamba --html --output-dir target/llvm-cov` on the same git SHA reproduces the same total coverage % to ±0.1pp; `COVERAGE-BASELINE.md` lists per-file coverage in stable sort order.

2. **`coverage(mamba): CI artifact for llvm-cov JSON + PR summary`**
   - Depends on (1).
   - `## Changes`: `.github/workflows/mamba-coverage.yml` (new), `projects/mamba/scripts/coverage_summary.py` (new — converts JSON to GitHub Actions step-summary).
   - DoD: every push to `main` or `project-*` uploads `coverage.json` as a workflow artifact; PR check renders a summary table.

3. **`coverage(mamba): PR gate — reject coverage drops > 0.5pp`**
   - Depends on (1) + (2).
   - `## Changes`: `.github/workflows/mamba-coverage.yml` (extend with gate step), `projects/mamba/scripts/coverage_diff.py` (new — compares HEAD vs baseline).
   - DoD: PR with -0.6pp coverage and no `coverage-allowed-drop` label fails the check; same PR with the label passes; PR with -0.4pp passes regardless.

4. **`coverage(mamba): generate COVERAGE-GAPS.md backlog`**
   - Depends on (1).
   - `## Changes`: `projects/mamba/scripts/coverage_gaps.py` (new), `projects/mamba/COVERAGE-GAPS.md` (new, auto-generated, regenerated on `cargo llvm-cov --json` updates).
   - DoD: file lists every `src/**/*.rs` with <100% coverage, sorted by uncovered-line count desc; autopilot can pick the top entry as its next fire.

After all four merge, optionally close #1885 with a comment listing the four child PR/issue numbers — or amend the umbrella body with the backlinks (matching its acceptance criterion #1).

## What patrol did

1. Materialized #1885 to `.score/issues/open/1885.md` on branch `issue-1885`.
2. Bailed pre-CRRR at umbrella detection (auto-stop: declared multi-fire body + 0/4 required headers).
3. Will FF the materialize commit + handoff into `project-mamba` so the cache survives.
4. Will add `flagged:needs-human` to #1885 so the next state-C scan skips it (24h handoff-skip rule applies regardless).

## Tick timing

Started ~14:17, bailed ~14:19 — ~2 min. Umbrella-detection path is now ~30s once the body is on disk; the rest is handoff-doc-write.

## Repeated observation across 5 autopilot ticks

Bail summary so far (this tick included):

| Slug | Phase | Reason | Tier |
|------|-------|--------|------|
| #1696 | created | bug-narrative body, 0/4 sections | p1 |
| #1260 | created | grab-bag (R5'/R6'/R7') + 2/4 sections | p0 |
| #1312/1313/1546 | td_reviewed | empty `Adopted:` stubs from incomplete fillback | p0 |
| #1749 | none | explicit Fire A–E umbrella | p1 |
| #1885 | none | explicit 4-child umbrella | p1 |

The mamba backlog's top of queue is **dominated by umbrellas and stubs**, not single-fire work-items. The autopilot's strict per-tick scope correctly rejects all of them, but the operator queue is now growing faster than autopilot can clear. **Recommend a one-shot decomposition session** before the next batch of autopilot ticks: file the ~15 child issues (Fire A–E for #1749, four for #1885, three for #1260, plus the Adopted-stub resolutions) so autopilot has clean p1 single-fire work to pick.

Alternatively, drop autopilot to p2 (3p conformance issues #1486+) where each issue is a single library and likely has a clean DoD spec. Quick inspection of #1486 (idna) shows it's an `enhancement(mamba/3p)` per-library format — that's the spec shape autopilot is designed to drive end-to-end.

After this handoff, next-tick p1 candidates (none umbrellas confirmed; need inspection):
- #1234 — enhancement(mamba): C3 pytest runs unmodified (phase:fill_reference_context — partially CRRR'd, BEST candidate to measure happy-path)
- #1258, #1262 — unknown shape
- 5 more unsurveyed p1s

#1234 is the highest-throughput pick because it's mid-CRRR (Requirements + Scope already filled, RefCtx pending) — the bail-rate on freshly-created issues is now empirically 100% (5/5).
