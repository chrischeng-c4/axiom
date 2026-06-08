# 1749-patrol-handoff

**Issue:** #1749 — conformance(mamba): build the surface-area denominator — auto-extract CPython 3.12 API checklist
**Branch:** `issue-1749` (materialization only, no CRRR started)
**Stop reason:** explicit umbrella issue — auto-stop per autopilot scope rule

## Why patrol bailed

The issue body itself declares it as multi-fire:

> ## Single-fire decomposition
> This issue itself is multi-fire. Tractable single-fire children:
> - Fire A: Land step 1 only — the extraction script + a frozen `cpython312_surface.toml` checked into the repo. Not yet wired to mamba.
> - Fire B: Land step 2 — the mamba probe...
> - Fire C: Land step 3 — diff/report tool...
> - Fire D: Land step 4 — `# covers:` directive parser...
> - Fire E: Land step 5 — CI gate.
> Plan: open child issues for A–E once this one is approved as the umbrella.

Autopilot's strict-scope rule says drive **one issue end-to-end per tick** with a single `## Changes` file list. An umbrella spanning Fire A–E breaks that on day one:

- Fire A touches `projects/mamba/scripts/dump_cpython_surface.py` (new file) + `projects/mamba/conformance/cpython312_surface.toml` (~5000-row generated file).
- Fire B touches `projects/mamba/scripts/probe_mamba_surface.py` + `projects/mamba/conformance/mamba_surface.toml`.
- Fire C touches a NEW Rust binary `mamba-conformance-report` + integrates with `cargo test -p mamba`.
- Fire D touches an unknown number of existing fixtures under `projects/mamba/tests/fixtures/conformance/` to retrofit `# covers:` directives.
- Fire E touches CI config + main report column.

The body has no `## Changes` section at all (it has `## Proposal` with 5 numbered steps spanning the five fires). Section header audit by autopilot gate:

| Required section | Present? | Notes |
|------------------|----------|-------|
| `## Problem` | ✓ | "denominator is self-defined" framing |
| `## Requirements` | ✗ | `## Proposal` covers the same ground but spans 5 fires |
| `## Scope` | ✗ | "Single-fire decomposition" is the *opposite* — it admits the issue is out of scope |
| `## Reference Context` | ✗ | `## Refs` is close but a 3-line link list, not RefCtx |

2/4 by header name; even charitably reading `## Proposal` ≈ Requirements and `## Refs` ≈ RefCtx, the missing Scope is fatal because there are five independent workstreams.

## What this issue actually needs (operator)

The body is excellent material for **five child issues**, each CRRR-able independently. Suggested filing:

1. **`conformance(mamba): Fire A — CPython 3.12 surface extraction script + frozen TOML`**
   - `## Changes`: `projects/mamba/scripts/dump_cpython_surface.py`, `projects/mamba/conformance/cpython312_surface.toml`
   - Runs under CPython 3.12; one row per public name. ~5000 rows expected.
   - DoD: TOML checked in, schema documented in spec, idempotent re-run on the same CPython 3.12 build produces byte-identical output.

2. **`conformance(mamba): Fire B — mamba surface probe + snapshot`**
   - Depends on Fire A's TOML schema.
   - `## Changes`: `projects/mamba/scripts/probe_mamba_surface.py`, `projects/mamba/conformance/mamba_surface.toml`.
   - DoD: probe runs under mamba, emits `present|missing|partial` per row, first ratio falls out.

3. **`conformance(mamba): Fire C — diff/report tool + CONFORMANCE.md`**
   - New Rust binary `mamba-conformance-report` under `projects/mamba/conformance-report/` (per CLI auto-registration convention).
   - DoD: `cargo run -p mamba-conformance-report -- diff` emits `CONFORMANCE.md` with sections per type/module, `[x] / [ ]` checkboxes, fixture-coverage column.

4. **`conformance(mamba): Fire D — # covers: directive parser + first retrofit`**
   - Depends on Fire C's tool integration.
   - `## Changes`: `projects/mamba/conformance-report/src/covers.rs` (parser) + at least one fixture under `projects/mamba/tests/fixtures/conformance/` retrofitted with `# covers:`.
   - DoD: parser warns when claim doesn't actually exercise the API.

5. **`conformance(mamba): Fire E — replace 691/691 metric with surface ratio`**
   - Depends on Fires A–D being green.
   - `## Changes`: `projects/mamba/Cargo.toml` (CI test runner), `.github/workflows/mamba-conformance.yml`, `projects/mamba/CLAUDE.md` (metric language).
   - DoD: CI prints `surface: I/T implemented, F/T fixture-covered`; old `691/691` line removed.

Each child should carry full `## Problem` / `## Requirements` / `## Scope` / `## Reference Context` headers so the autopilot gate passes on first scan.

## Related observation

#1885 ("Goal 3 — 100% test coverage requires building the measurement baseline first") is the **same architectural pattern** for Goal 3 that #1749 is for Goal 1: build the denominator before driving the ratio toward 100%. #1885 also bails the autopilot gate (no Problem/Requirements/Scope/RefCtx headers by name; only `## Why` + `## Acceptance` + `## Refs`). Worth queuing both decompositions in the same operator session.

## What patrol did

1. Materialized #1749 to `.score/issues/open/1749.md` on branch `issue-1749`.
2. Did NOT start CRRR — bailed at umbrella detection per autopilot policy.
3. Will FF the materialize commit into `project-mamba` so the cache survives the branch deletion.
4. Will add `flagged:needs-human` to #1749 so the next state-C scan skips it (24h handoff-skip rule applies regardless).

## Tick timing

Started ~13:47, bailed ~13:50 — handoff-doc-write dominated; the actual bail decision took ~30s (umbrella check via `## Single-fire decomposition` substring on the body). Spec-quality-gate continues to do its job: bad/oversized specs cost <3 min.

## Note on autopilot order — repeated observation

Three autopilot ticks now (#1696, #1260, #1312-cluster, #1749) have bailed before handwrite. All four picks were `phase:created` or `phase:td_reviewed` umbrella/stub issues. The throughput-optimal order is `td_reviewed > td_created > fill_reference_context > fill_scope > fill_requirements > created` within tier — i.e. prefer issues that are *farther through the pipeline* over freshly-created ones. Consider amending the patrol prompt:

> Within priority tier, sort by phase descending: `td_reviewed` > `td_revised` > `td_created` > `fill_reference_context` > `fill_scope` > `fill_requirements` > `created` > `none`. Among equal-phase, oldest issue number first.

`phase:none` issues (like #1749 and #1885 here) are essentially raw user proposals that haven't even started CRRR — they belong last, not first.

After this handoff, p1 list collapses to (#1885 also umbrella, will likely bail) + 7 other p1s that haven't been inspected. Next tick (14:17) should drop to those or skip to p2 3p-conformance issues (#1486+), which are well-formed single-library DoD specs.
