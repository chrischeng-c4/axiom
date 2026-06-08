# 1483-patrol-handoff

**Issue:** #1483 — conformance(mamba/3p): certifi — vendor tests pass + perf ≥1.0× + 95% surface
**Branch:** `issue-1483` (issue CRRR through review approved; TD not started)
**Stop reason:** TD authoring + handwrite is multi-day 3rd-party-vendor infra work, not 1-tick autopilot scope

## What patrol completed (real progress)

Issue CRRR is fully done:

- Requirements rewritten from `**R1 (Behavior)** — ...` (rejected by validator) to canonical `- R1: ...` format. R1–R4 all concrete and measurable: vendor test suite green, ≥1.0× perf floor, ≥95% typeshed surface, real-world downstream consumer.
- Scope corrected to `projects/mamba/src/runtime/stdlib/3p/certifi_mod.rs` (was `stdlib/certifi_mod.rs` — confusing for a 3rd-party lib).
- Reference Context Spec Plan table filled with real entry: `mamba-3p-certifi-conformance` (create action).
- DoD adjusted: removed `pytest projects/.../conformance/stdlib/certifi/` (wrong path, 3rd-party not stdlib).
- Review approved with verdict `approved` and three section findings.
- GitHub state: `phase:reviewed`, `review:1`, `state:OPEN`. Next dispatch is `score td create --slug 1483`.

This is a real artifact — operator can pick up TD CRRR from the current state.

## Why autopilot bails at TD boundary

The spec describes work that is **fundamentally not 1-tick scope**:

1. **Vendor strategy** — needs a design decision: bundle certifi as a Python package in mamba's stdlib path? Generate a Rust shim that exposes `certifi.where()` + `certifi.contents()` reading a bundled CA file? Both have downstream implications for the 3rd-party lib pipeline as a whole (#1263).
2. **Test infrastructure** — mamba does not yet have a `pytest`-under-mamba runner per R1. Building that is a separate workstream that blocks every 3rd-party lib conformance issue.
3. **Bench infrastructure** — `projects/mamba/benches/3p/` does not exist. Creating the directory + cargo workspace wiring + cross-runtime baseline (CPython vs mamba on the same workload) is infra, not per-lib work.
4. **typeshed surface checklist** — generator does not exist; same dependency as #1397 (typeshed surface coverage epic).
5. **Real-world downstream consumer** — `requests` hello-world depends on requests, which depends on urllib3 + charset-normalizer + idna + certifi. The full transitive closure must pass before R4 can be satisfied. Cannot be done lib-by-lib.

Items 2–5 are **shared infrastructure** that every 3rd-party lib conformance issue (#1483 / #1484 / #1486 / … / #1503+, ~50 libs total in p2 queue) blocks on. The right sequencing is: build the infrastructure once (pytest-under-mamba runner, bench harness, typeshed checklist generator, real-world fixture pattern), then per-lib issues become 1-tick handwrites.

## What operator should do

Two paths, in priority order:

### Path A — Build the 3rd-party infrastructure first (preferred)

File 4 prerequisite infra issues, drive each through full CRRR + handwrite, **then** unblock the per-lib backlog:

1. **`infra(mamba/3p): pytest-under-mamba runner for vendor test suites`**
   - DoD: `mamba pytest <path>` works on at least one vendored package's test suite; exits non-zero on test failure.
2. **`infra(mamba/3p): bench harness for CPython vs mamba comparison`**
   - DoD: `cargo bench -p mamba-bench-3p <lib>` produces a `[lib_name] mamba/cpython = X.YYx` line; gating logic per #1265 Goal 2 floor.
3. **`infra(mamba/3p): typeshed surface coverage tool`**
   - Depends on #1397 (typeshed surface coverage epic) being underway.
   - DoD: `mamba-surface-report --package certifi` emits `implemented/total = N/M (P%)`.
4. **`infra(mamba/3p): real-world hello-world fixture convention`**
   - DoD: `projects/mamba/tests/fixtures/conformance/3p/<lib>/real_world/` pattern documented; one example landed (e.g. `certifi/real_world/hello.py` reading the CA bundle path).

After these infra issues land, every existing p2 3rd-party lib issue (#1483, #1484, #1486+) becomes a 1-tick autopilot candidate: write the shim + fixtures, run the harness, commit.

### Path B — Hand-drive #1483 end-to-end now

1. `score td create --slug 1483` to author the TD spec interactively.
2. Make the vendor strategy + infra decisions inline.
3. Hand-write certifi_mod.rs + fixtures + bench + surface checklist.
4. Land 3 gates green.

Estimated effort: 2–3 days of focused work. Not 1-tick.

## What patrol did this tick

1. Materialized #1483 to `.score/issues/open/1483.md` on `issue-1483` branch.
2. Drove Requirements / Scope / Reference Context fill-section + apply + validate (Requirements rewrite was the actual content change; Scope + RefCtx fills were format/placeholder fixes).
3. Drove `score wi review --apply` with `approved` verdict + 3 findings.
4. Drove final `score wi validate` — phase advanced to `reviewed`, state stayed `OPEN` (TD-ready).
5. Bailing here — will FF issue-1483 into project-mamba so the CRRR commits + spec corrections survive; will flag `flagged:needs-human` so next tick skips.

## Tick timing

Started ~14:47, bailed ~14:55 — ~8 min. First non-bail tick of this session; ate the full issue CRRR pipeline. Per-section apply + validate is ~30s each plus payload-author wallclock. The 30-min cron interval comfortably absorbs this.

## Repeated observation — autopilot phase ordering

Across 6 ticks now, 5/6 picks bailed before any CRRR work because top-of-queue is umbrellas/stubs. This one (#1483) is the first non-umbrella, and it ate the full issue CRRR. **Recommendation stands**: amend patrol prompt to prefer phase `reviewed`/`td_created`/etc. over `created`/`none` within priority tier. Most useful follow-up work for autopilot would be picking the next `phase:reviewed` issue (now #1483 itself, until handoff-flagged) and driving TD CRRR through merge — assuming the TD scope is also reasonable.

## Note on the per-lib pattern

The body template that #1483 uses (Problem / R1-R4 / Scope / Refs / DoD-3-gates) is **shared across ~50 p2 3rd-party lib issues** (#1483 through #1503+, all `conformance(mamba/3p): <lib> — vendor tests pass + perf ≥1.0× + 95% surface`). Once the 4 infra issues above land, autopilot can clear this queue at one tick per lib — that's the real throughput win.
