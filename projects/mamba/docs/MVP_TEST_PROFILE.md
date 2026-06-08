# MVP Test Profile — `mamba`

Canonical command list a worker must run before claiming mamba MVP test
readiness. Closes #2535 under parent #2527 / epic #2526.

This document is the single source of truth for "what does a passing
mamba MVP run look like". Each command lists:

- **Runtime category** — how long to budget (`<5s`, `<30s`, `<5min`,
  `<30min`, `live-network`).
- **Default vs opt-in** — does `cargo test -p mamba` run it, or does
  the worker need a flag / feature / env var.
- **Failure meaning** — what a non-zero exit actually proves.

The MVP gates map 1:1 to parent issues #2527 through #2533. When a gate
is split across multiple binaries or harnesses, each row links to the
atomic leaf issue that owns it.

---

## Gate 0 — smoke / inventory (parent: #2527)

> "The mamba test surface compiles, links, and is enumerable." This is
> the cheapest possible signal that the next six gates can run at all.

| # | Command | Runtime | Default? | Failure means |
|---|---------|---------|----------|---------------|
| G0.1 | `cargo test -p mamba --no-run` | `<5min` (cold) / `<30s` (warm) | Required | Crate did not compile. Every deeper gate is blocked until this passes. |
| G0.2 | `cargo test -p mamba -- --list` | `<5s` | Required (#2534) | The test binary linked but cannot enumerate tests — fix linkage / loader issues before running anything else. |
| G0.3 | `cargo test -p mamba --test cpython_lib_test_runner -- cpython_lib_test_mvp_gate_requires_pass_dir_entry discover_seeds_rejects_duplicate_stems_across_contract_dirs` | `<5s` | Required (#3729, supersedes #2536/#2537) | Folder-contract drift: no seed file under `tests/cpython/lib_test_seeds/pass/`, or a stem appears in more than one contract dir. Cheap inventory pass — no seed execution. |
| G0.4 | `.agents/skills/score-mamba-test-coverage/scripts/coverage.sh` | `<30s` | Opt-in | Inventory script targets the wrong crate path (#2538). Inventory-only — no test execution. |

### What G0 explicitly does **not** prove

- That any assertion executed (that's Gate 1).
- That mamba is faster than CPython (that's Gate 3).
- That mambalibs builds (Gate 4) or the package manager works (Gate 5).

---

## Gate 1 — CPython Lib/test assertions execute (parent: #2528)

> "When mamba reports a CPython Lib/test fixture as a pass, it really
> ran the asserts." Anything weaker is a `Stub` outcome and never
> counts toward MVP.

| # | Command | Runtime | Default? | Failure means |
|---|---------|---------|----------|---------------|
| G1.1 | `cargo test -p mamba --test cpython_lib_test_runner` | `<5min` | Required | Folder-contract drift (#3729): a seed under `tests/cpython/lib_test_seeds/<dir>/` did not produce the outcome `<dir>` pins. The runner suggests a `git mv` to the matching outcome dir in the failure message. |
| G1.2 | (retired — folded into G1.1 by #3729; folder layout supersedes `cpython_lib_test_allowlist.toml`.) | — | — | — |
| G1.3 | `cargo test -p mamba --test cpython_lib_test_runner -- --ignored cpython_lib_test_mvp_gate_requires_assertion_pass` | `<5s` | Opt-in (#2544, until #2545) | Zero baseline entries have `outcome = "AssertionPass"`. The MVP gate is `MVP-PENDING` until one real unittest dispatch path ships under #2545. |
| G1.4 | `cat "$CARGO_TARGET_TMPDIR/cpython_lib_test_summary.json"` after G1.1 | `<1s` | CI-only (#2543) | Schema-v2 (`harness_kind`, `mvp_status`) sidecar missing or malformed. CI parsers ingest this to bucket counts (#2546). |
| G1.5 | `cargo test -p mamba --test cpython_compat` | `<30s` | Required | Parser-only CPython 3.12 fixture surface regressed. Output is annotated `[cpython_compat:parser-only]` so it is **not** mistaken for runtime execution (#2546). |

### Failure mode that is **not** Gate 1

A `Fail` outcome that is also listed in `cpython_known_failures.toml`
(parser harness) or carries a `# XFAIL` directive is **debt**, owned
by Gate 6 (#2533), not Gate 1. The drift gate still triggers if a known
failure unexpectedly starts passing — that requires removing the
manifest entry in the same commit.

---

## Gate 2 — Py3.12 ecosystem real-world acceptance (parent: #2529)

> "The top-N real-world Py3.12 libraries import and run their own
> smoke under mamba."

| # | Command | Runtime | Default? | Failure means |
|---|---------|---------|----------|---------------|
| G2.1 | `cargo test -p mamba --test 'conformance_real_world*'` | `<5min` | Required | Curated Py3.12 real-world fixture regressed. Pinned per issue (declarative TOML under `tests/cpython/perf/pins/`, dispatched by `tests/perf_pin_runner.rs`). |
| G2.2 | `cargo test -p mamba --test 'cpython_compat'` | `<30s` | Required | Parser-only acceptance dropped on a real-world fixture. Same harness as G1.5 — note the parser-only banner. |

Live-network / package-install ecosystem coverage (e.g. PyPI mirror
hits) is **out of scope for G2** and lives under Gate 5 (#2532), gated
behind the package-manager E2E.

**No-network invariant (#2556).** The G2 default gate has no network
dependency. The smoke test
`ecosystem_fixture_manifest_required_fixtures_are_no_network` scans
every manifest-listed fixture source for known network-touching
patterns (`urllib.request`, `http.client`, `socket.create_connection`,
bare `requests` calls, etc.) and fails the gate up-front if a required
fixture would reach the network at runtime. Live integration checks
must run behind the opt-in `--ignored` runner with `MAMBA_NETWORK=1`
(reserved for a future commit) — they never gate default MVP pass/fail.

---

## Gate 3 — 10× CPython 3.12 performance (parent: #2530)

> "Mamba's average measured wall-time is at least 10× faster than
> CPython 3.12 on the curated perf suite, and per-bench memory does
> not regress below 1.0× CPython."

| # | Command | Runtime | Default? | Failure means |
|---|---------|---------|----------|---------------|
| G3.1 | `cargo test -p mamba --release --test perf_pin_runner -- perf_pin` | `<30min` | Opt-in (release profile required) | A perf-pinned bench fell below its baseline ratio. Mamba's primary success criterion is perf+memory > CPython; conformance is secondary. (The 119 per-pin `<lib>_perf_pin_<N>.rs` binaries were consolidated into `tests/perf_pin_runner.rs` + `tests/cpython/perf/pins/*.toml`; the substring filter `perf_pin` matches every emitted test name.) |
| G3.2 | `cargo test -p mamba --release --test cross_runtime` | `<5min` | Opt-in | Cross-runtime ratio collection drift. **Caveat:** wall-time ratios are Python-startup-dominated, not throughput ratios — interpret per-iter numbers, not totals. |
| G3.3 | `cargo bench -p mamba` | `<30min` | Opt-in (Criterion) | Microbench regression. Criterion HTML reports under `target/criterion/`. |

### Mem regression triage (#2096)

A mem-regression failure under G3 is one of:

- **Subset A** — transient input-clone churn at the FFI/shim boundary
  (fix: shim-borrow refactor, ~150 LOC).
- **Subset B** — `MbObject` 104-byte header × N iters (fix: runtime
  layout change, deferred).

See parent #2096 + the runtime layout notes in
`NOTES-NEXT.md` before touching either.

---

## Gate 4 — `mambalibs` Mode 2 build + import (parent: #2531)

> "Mode 2 mambalibs build deterministically and `from mambalibs import
> <pkg>` resolves to the native artifact."

| # | Command | Runtime | Default? | Failure means |
|---|---------|---------|----------|---------------|
| G4.1 | `cargo test -p mamba --test mambalibs_mode2_e2e` | `<5min` | Required (when present) | Mode 2 build artifact missing, ABI drift, or `from mambalibs import X` resolves to the Mode 1 fallback instead of the native module. |
| G4.2 | `cargo build -p mamba --features mambalibs-mode2` | `<5min` | Opt-in | Feature gate broken at the cfg level. |

This gate is `phase:created` until #2531 lands the actual harness.

---

## Gate 5 — uv-like package manager offline E2E (parent: #2532)

> "The mamba CLI's package-manager subcommands round-trip a small
> manifest end-to-end without touching the network."

| # | Command | Runtime | Default? | Failure means |
|---|---------|---------|----------|---------------|
| G5.1 | `cargo test -p mamba --test pkgmgr_e2e_offline` | `<5min` | Required (when present) | Manifest resolve / lockfile / install round-trip regressed. **No** PyPI / network calls — purely on the vendored fixture index. |
| G5.2 | `cargo test -p mamba --test pkgmgr_live` -- `--ignored` | `live-network` | Opt-in only | Live PyPI mirror probe — only worker-initiated, never default. |

This gate is `phase:created` until #2532 lands the actual harness.

---

## Gate 6 — debt manifest (parent: #2533)

> "Every `#[ignore]`, `# XFAIL`, and `Stub` outcome is named in a debt
> manifest with an owner and a closing issue."

| # | Command | Runtime | Default? | Failure means |
|---|---------|---------|----------|---------------|
| G6.1 | `cargo test -p mamba -- --ignored --list` | `<5s` | Required (when manifest lands) | New ignored tests added without a matching `debt_manifest.toml` entry, or a manifest entry whose seed/test no longer exists. |
| G6.2 | `cargo test -p mamba --test cpython_compat -- --nocapture` | `<30s` | Required | An `# XFAIL` fixture started passing (logged as `[xpass]`) without removing the `cpython_known_failures.toml` entry. |
| G6.3 | `grep -r 'HANDWRITE-BEGIN' projects/mamba/src/` | `<5s` | Required | Surfaces hand-written-with-marker debt for the score standardization loop (`/aw:standardize-run`). |

This gate is `phase:created` until #2533 lands the manifest schema.

---

## Worker checklist — minimum viable MVP run

Before claiming "mamba MVP green", run in order and stop on the first
red. Each command line below maps to a row above:

1. G0.1 → G0.2 → G0.3 (gate 0 smoke + inventory)
2. G1.1 → G1.2 → G1.5 (gate 1 runtime + parser-only)
3. G2.1 → G2.2 (gate 2 ecosystem)
4. **If release-profile is in scope:** G3.1 → G3.2 (gate 3 perf)
5. G4.1 (gate 4 mambalibs, when harness lands)
6. G5.1 (gate 5 pkgmgr, when harness lands)
7. G6.1 → G6.2 (gate 6 debt)

`AssertionPass`-driven MVP gating (G1.3) flips from `MVP-PENDING` to
`MVP-OK` the moment #2545 promotes the first baseline entry. Until then
G1.3 stays `#[ignore]` and does not block the run.

---

## Cross-reference table

| Gate | Parent issue | Authoritative harness file |
|------|--------------|----------------------------|
| 0 | #2527 | `tests/cpython_lib_test_runner.rs` (folder-contract inventory subtests, #3729), `cargo test -- --list` |
| 1 | #2528 | `tests/cpython_lib_test_runner.rs`, `tests/cpython_compat.rs` |
| 2 | #2529 | `tests/cpython_real_world.rs`, `tests/cpython_compat.rs` |
| 3 | #2530 | `tests/perf_pin_runner.rs` + `tests/cpython/perf/pins/*.toml`, `tests/cross_runtime.rs`, `benches/` |
| 4 | #2531 | `tests/mambalibs_mode2_e2e.rs` (planned) |
| 5 | #2532 | `tests/pkgmgr_e2e_offline.rs` (planned) |
| 6 | #2533 | `debt_manifest.toml` (planned) |
| epic | #2526 | this document |
