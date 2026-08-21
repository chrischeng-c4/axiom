# harness-ec — architecture (as-is, 2026-07-15)

EC execution machinery for mamba: how the C1/C2 contracts in `README.md` (same dir) are actually run and where
verdicts come from. Source root: `tests/harness/cpython/`. Fixture spec: `tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`.

## Responsibilities

- Execute C1 functional parity: every fixture under `tests/cpython/**` vs the LIVE CPython 3.12 oracle (byte-diff stdout); goldens are retired (D5.6) — the oracle subprocess IS the expected value (`runner.rs:run_conformance`).
- Execute C2 performance: per-pin external CPU/RSS ratio gates vs a recorded CPython baseline (`perf_pin.rs:run_pin`, D5.2 — harness owns measurement, fixtures never self-time).
- Own the ONE verdict semantics; `tools/sweep.py:classify` mirrors `runner.rs` verdict-for-verdict so the seconds-scale inner loop and the ~3min cargo gate cannot disagree.
- Meta-gate the corpus itself: `contract.rs` (replacement axes cannot silently vanish), `lib_test.rs` (folder-is-contract seed drift), `status.rs` (read-only readiness reporter, `--json`).
- Isolate every fixture run (throwaway CWD + TMPDIR, PYTHONBREAKPOINT=0, rlimits, process-group kill) so `test.support`/TESTFN droppings never land in the repo tree and hangs never wedge a run.

## Key structures & invariants

| Structure | File:symbol | Invariant |
|---|---|---|
| Directives | `runner.rs:parse_directives` | `# mamba-xfail:` ⇒ full skip; `# mamba-strict-type:` ⇒ strict-type verdict path. Parsed identically in `sweep.py:parse_directives` — the two MUST stay in lockstep. |
| Dimension keyword | `runner.rs:is_type_strict_path` | ANY path component `== "type"` ⇒ strict-type rules (mamba must reject); all other dimensions are positive contracts (a compile reject there = type-system false positive, per README.md dimension rule; cf. `tech-design/type-system/walls-and-widening.md` §Fire/defer semantics). |
| Discovery allowlist | `runner.rs:harness!` regex (l.626) | Dimensions are explicitly enumerated (`behavior|type|surface|_regression|real_world|errors|security|security-matrix|perf|concurrency`); `tests/cpython/.cache/` (oracle-env venv, sweep state) must never be collected. `harness_common.rs:collect_files` skips hidden dirs for the same reason. |
| Oracle cache | `runner.rs:oracle_cache_path` | Content-addressed: `target/cpython-oracle-cache/<2hex>/<sha256(pythonVersion + "\0v1\0" + fixture bytes)>`. Only successful oracle runs cached (INVALID stays live); write-then-rename for parallel safety; `MAMBA_ORACLE_CACHE=0` disables. Same scheme reimplemented in `sweep.py:oracle_cache_path` — shared cache, shared invalidation. |
| Timeout/kill | `harness_common.rs:TimeoutPolicy`, `wait_with_timeout` | One env lookup (`MAMBA_CONFORMANCE_TIMEOUT_SECS`, default 30s); exponential backoff poll 1ms→cap; on timeout kill the whole process GROUP (children are `setpgid` group leaders via `runner.rs:apply_child_limits`). |
| Child rlimits | `runner.rs:apply_child_limits` | RLIMIT_AS/DATA = 1 GiB (`MAMBA_CONFORMANCE_MEM_MB`), RLIMIT_CPU = 2× timeout, core dumps off — for BOTH mamba and the oracle. |
| Oracle interpreter | `harness_common.rs:python3_bin` | Resolved ONCE: `MAMBA_ORACLE_PYTHON` → `tests/cpython/.cache/oracle-env/bin/python3` (uv-materialized, pinned 3p deps) → PATH `sys.executable`. Mirrored by `sweep.py:python3_bin`. |
| Perf pin | `perf_pin.rs:Pin` (TOML in `config/perf/pins/*.toml`) | `floor <= 1.0` and `mem_floor >= 1.0` enforced by `contract.rs:perf_pins_gate_speed_and_memory_against_cpython`; ≥100 pins must exist. |
| Perf baseline | `perf_pin.rs:CpythonPerfBaseline` ← `tools/perf_baseline.py` (SQLite at `tests/cpython/.cache/perf/cpython_baseline.sqlite`) | `fixture_sha256` must match on load (hard assert = stale-baseline gate); cross-host mismatch warns only (#966). Absent baseline ⇒ live python3 measurement unless `MAMBA_REQUIRE_CPYTHON_PERF_BASELINE=1`. |
| RSS floor | `perf_pin.rs:MAMBA_FIXED_RUNTIME_RSS_FLOOR_BYTES` = 26 MB | Mem gate ratio = cpython_rss / max(mamba_rss − 26MB, 0) ≥ mem_floor (#1024 ship-profile fixed runtime floor). |
| Seed contracts | `lib_test.rs:CONTRACT_DIRS` under `config/seeds/{pass,spec,stub,fail,import_pass,timeout}/` | Directory IS the contract; stems unique across dirs (`discover_seeds` errors on dupes); only `AssertionPass` counts for MVP (#2540); promotion = `git mv spec/x.py pass/x.py`. |
| Sweep state | `tests/cpython/.cache/sweep/failures.txt` | `--store` merge rule: `stored' = (stored − covered) ∪ failed-this-run` — partial runs never resurrect or drop uncovered entries (`sweep.py:main`). |
| Tool exec core | `tools/harness_lib.py:run_fixture` | ALL python tools spawn through this: per-call scratch CWD + TMPDIR, PYTHONBREAKPOINT=0, timeout ⇒ `(None, "", "")`; canonical verdict vocabulary `PASS/MAMBA_RED/DIVERGE/ORACLE_SKIP` + `compute_pass_rate`. |
| In-process isolation | `src/conformance/mod.rs:ScratchCwd` (l.109) | RAII chdir into throwaway dir, restore+remove on drop; safe ONLY because `JIT_LOCK` serializes the in-process runner. Subprocess twin: `src/main.rs:maybe_enter_scratch_cwd` (`MAMBA_SCRATCH_CWD=1`, #1558 — opt-in, default `mamba run` keeps user CWD). |

## Control flow

1. **Conformance gate** `cargo test -p mamba --release --test conformance` → datatest_stable `harness!(run_conformance, allowlist)` per fixture.
2. `runner.rs:run_conformance`: `bench/` component ⇒ skip (perf-pin owned, #2239) → read src → `# RUN:` ⇒ skip (pipeline.rs owned) → xfail ⇒ skip → strict_type directive or `/type/` path ⇒ step 3 → else step 4.
3. `runner.rs:run_type_strict`: signal/resource kill ⇒ FAIL; nonzero exit + "TypeError"/"type error" in output ⇒ STRICT_TYPE_OK(compile); exit 0 + line-prefix `typeerror:` (and no `no_typeerror:`) ⇒ STRICT_TYPE_OK(runtime); `no_typeerror:` only ⇒ MAMBA_TYPE_LEAKED; both/neither ⇒ malformed fixture.
4. Oracle: cache hit ⇒ use bytes; miss ⇒ `spawn_python` (sandboxed, rlimited); oracle nonzero ⇒ INVALID fixture (fails the test, never cached). Then `spawn_mamba`; nonzero ⇒ FAIL (with CPU_LIMIT/OOM detail via `resource_failure`); stdout ≠ oracle ⇒ mismatch diff (`format_diff`).
5. **Inner loop** `tools/sweep.py`: select (`--all` | `--failures` | `--sample N` | `--list` | paths, then `--filter`) → `classify` per fixture (verdicts PASS/FAIL/DIVERGE/XFAIL/SKIP/INVALID, runner-parity, shared oracle cache) on ThreadPool (default 2×cores) → summary counts → optional `--store` merge; exit 1 iff any BAD.
6. **Perf gate** (opt-in: `harness=false`, `test=false` target `perf_pin`): per pin TOML → `load_cpython_baseline` (via `perf_baseline.py get`, sha-checked) or live `measure_n("python3",…)` → `measure_n(mamba,…)` — each run `/usr/bin/time -l|-v`-wrapped + `getrusage(RUSAGE_CHILDREN)` delta, median-of-N CPU / min RSS → assert cpu_ratio ≤ floor, adjusted mem_ratio ≥ mem_floor; both failures joined so one run reports all.
7. **Seed gate** `--test conformance_cpython_lib_test`: `discover_seeds` → `run_seed` (60s fixed budget) → classify Fail/Stub/ImportPass/AssertionPass/Timeout by exit status + markers (`MAMBA_ASSERTION_PASS`, stub markers) → drift vs parent dir fails with a `git mv` hint → summary JSON sidecar (`schema_version=2`, `harness_kind="runtime"`; parser-only twin is `grammar.rs`, #2546 — counts must never be summed).
8. **Meta-gates**: `--test conformance_contract` re-walks the tree per `[tool.mamba]` records (all axes present, xfail policy explicit, type walls carry inverse markers, pin floors sane, required tool markers present); `--test cpython_status` reports corpus/migration/baseline readiness without executing fixtures.

## Known hazards

- **Sweep load degradation**: full `--all --store` at 20 jobs on a 10-core box times out ~127 PASSING fixtures ⇒ false FAILs, and `--store` then corrupts `failures.txt`. Tell = "regressions" spanning unrelated libs; verify flagged fixtures in isolation (jobs=1 / direct `mamba run`); copy failures.txt to /tmp before any `--store`.
- **Sweep limits-parity gap** (`sweep.py` docstring): macOS `sh ulimit` cannot express RLIMIT_AS, so sweep applies only the CPU cap — a runaway-allocation fixture can pass one gate and fail the other.
- **Stale binary**: `sweep.py:mamba_bin` defaults to `target/debug/mamba`; after a release build set `MAMBA_BIN` or the canary tests an old binary silently.
- **Grandchild pipe wedge** (`runner.rs:apply_child_limits` comment): a fixture's grandchild survives the child's SIGKILL holding the stdout pipe write-end ⇒ `wait_with_output` blocks forever. Every new spawn site MUST setpgid + group-kill (as `wait_with_timeout` does), or a timeout wedges the whole run at the very end.
- **Allowlist silence**: a new top-level dimension dir not added to the `runner.rs:harness!` regex is simply never executed by the cargo gate — no error, no count. sweep's `rglob` WOULD run it, so the two gates diverge.
- **`.cache` collection**: the materialized oracle-env venv holds thousands of stdlib `.py` files inside `tests/cpython/.cache/`; any new walker must exclude hidden dirs or the corpus count explodes ("the .cache lesson", `runner.rs` l.622).
- **xfail = skip, not expected-fail**: xfail fixtures are never executed (avoids infinite-loop hangs) so they rot silently; the only re-enable is deleting the directive.
- **Oracle cache staleness by env**: the cache key is python-version + fixture bytes only — a fixture whose oracle output depends on installed packages/env serves stale bytes until the cache dir is cleared.
- **pyenv shim cost**: PATH-resolved `python3` may be a bash shim (~470ms/exec vs ~25ms, ≈65% of a full run); resolve through `python3_bin()` once, never `Command::new("python3")` per spawn.
- **Mem-gate auto-pass**: if mamba workload RSS ≤ the 26MB fixed floor, adjusted ratio = ∞ and the mem gate passes vacuously — small pins prove nothing about memory.
- **Build/sweep contention**: each sweep = up to 32 parallel mamba procs; a concurrent `cargo build --release` degrades from ~3.5min to 77–117min. Serialize: build alone, then sweep.
- **Wrong-file verification trap**: when a canary flags fixture X, run EXACTLY the flagged path — `find -path` globs have matched different, out-of-scope files three separate times.

## Extension points

- **New fixture**: manifest `config/manifests/<bucket>/<lib>.toml` → `tools/fixture_gen.py` → fill semantic bodies → `tools/fixture_lint.py`; the cargo harness picks it up with no runner edit (see FIXTURE-LAYOUT.md). New DIMENSION additionally requires the `runner.rs:harness!` regex + FIXTURE-LAYOUT table row + `contract.rs` axis assertions.
- **New directive**: extend `runner.rs:parse_directives` AND `sweep.py:parse_directives` together (parity contract), plus a `contract.rs` marker test if it is gate-bearing.
- **New perf pin**: TOML in `config/perf/pins/` (floor ≤ 1.0, mem_floor ≥ 1.0 or `contract.rs` fails) → `perf_baseline.py record --pin <toml>` → runs automatically via the `perf_pin` datatest target.
- **New seed contract**: drop the `.py` under the matching `config/seeds/<outcome>/` dir; no code change (`lib_test.rs:CONTRACT_DIRS` already walks it). New outcome class = new `(dir, Outcome)` row + classifier arm.
- **New python tool**: spawn via `harness_lib.run_fixture` and emit the canonical verdict vocabulary; never open-code a subprocess loop (the pre-consolidation copies drifted and leaked TESTFN files).
- **New shared Rust primitive**: `harness_common.rs` (sibling-`#[path]` include), only for genuinely duplicated primitives — per-runner classification stays in each runner.

## EC surface

Per `external-contracts/README.md`: this domain is the EXECUTOR of every other domain's EC, and is itself proven by:

| Proof | Artifact |
|---|---|
| C1 gate runs at all, dimensions honored | `cargo test -p mamba --release --test conformance` (~3 min, 46k+ fixtures) |
| Corpus cannot lose a replacement axis | `--test conformance_contract` meta-tests (`fixture_tree_covers_all_replacement_axes`, surface-coverage, pin-floor, tool-marker gates) |
| Verdict parity inner loop | `sweep.py` mirrors runner verdicts; README rule "gate readings are the only progress signal" |
| C2 gate | `--test perf_pin -- perf_pin` (opt-in) + `--test cpython_status` baseline-readiness preflight |
| Seed/drift gate | `--test conformance_cpython_lib_test` + summary JSON sidecar (schema v2, harness_kind=runtime) |
| Harness unit-level | `lib_test.rs` in-file `#[test]`s (discover_seeds duplicate-stem, summary-schema stability) |

Dimension rule (README.md, restated once because it IS this domain's verdict law): behavior/errors/real_world/surface/_regression/security/concurrency fixtures MUST run — only `type/` fixtures are walls.
