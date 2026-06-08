# mamba Production Test Gate

> Single source of truth for the **D1–D5 test + harness goal**. Every item below
> has a **verifiable success metric** — a command and its expected output. The
> gate is DONE when all metrics pass. Driven by Claude Code `/goal`; the
> condition at the bottom points here.

## Non-negotiables

1. **Scope is `projects/mamba/tests/**` only.** Editing `projects/mamba/src/**`
   is OUT OF SCOPE — that is the separate runtime line (`mamba-jit-worker` /
   `fix(mamba)` bugs). Sole exception: a `[[test]]` stanza in
   `projects/mamba/Cargo.toml` when a new harness binary is unavoidable.
2. **Test + harness authoring only — never fix runtime to go green.** A fixture
   that can only pass by changing `src/` STAYS RED.
3. **Red is correct.** A fixture failing on today's mamba honestly records that
   mamba is not yet shippable for that behavior. DONE = the gate is **complete
   and reports honestly**, NOT all-green.
4. **The harness does the heavy lifting; test cases stay pure.** A fixture
   describes *what* to test (workload + assertion). Timing, memory, sampling,
   oracle diffing, and result storage are the harness's job — never the
   fixture's.
5. **One case = one file**, exits 0 under CPython 3.12, no shared state — the
   subprocess boundary is the isolation guarantee.

## Dimensions

| Dim | Name | What the harness proves | Priority |
|-----|------|-------------------------|----------|
| **D5** | **Harness** | the gate itself: pure fixtures, external measurement, one results store, pooled isolated collection, one summary command | **FIRST** |
| D1 | Type-strict | mamba's force-typed boundaries vs CPython (`typeerror:` / `no_typeerror:`) | then |
| D2 | Behavior | observable output matches CPython 3.12 exactly | then |
| D3 | Performance | CPU + peak-RSS vs CPython baseline, measured externally | then |
| D4 | Safety & stability | no SIGSEGV / OOM / hang / leak under hostile input | then |

---

## D5 — Harness (land first)

### D5.1 — Pure test cases (no fixture times itself)

- **Goal**: strip all self-timing; a bench fixture = workload + stdout sink +
  `iters` metadata in its `[tool.mamba]` block, as pure as a behavior fixture.
- **Success metric**: only **bench** fixtures self-time. The signal is
  `perf_counter` + the `INTERNAL_TIME_NS` marker; `import time` is excluded, and
  so are **non-bench fixtures** (a behavior/surface fixture testing
  `time.perf_counter` legitimately contains it). `projects/mamba/benches/` is
  outside `tests/` and also out of scope:
  ```
  find projects/mamba/tests/cpython/fixtures -path '*/bench/*.py' \
    -exec grep -l 'perf_counter\|INTERNAL_TIME_NS' {} + | wc -l
  ```
  → **`0`**
- **Current**: DONE — 244 cpython bench fixtures stripped by
  `tools/strip_self_timing.py` (which now only touches the bench dimension).
  22 non-bench fixtures it initially damaged were reverted.
- **Issues**: new (fixture-purity sweep); relates to D5.2.

### D5.2 — Harness owns measurement

- **Goal**: CPU time + peak-RSS come ONLY from external `getrusage(RUSAGE_CHILDREN)`
  + `/usr/bin/time` in the harness; warmup / iteration count / median are
  harness-controlled (env or config), not hardcoded in fixtures.
- **Success metric**:
  ```
  rg -l 'INTERNAL_TIME_NS' projects/mamba/tests/harness | wc -l   # → 0 (harness stops parsing fixture timing)
  rg -n 'getrusage|maximum resident set size' projects/mamba/tests/harness/cpython/*.rs | wc -l   # → >0
  ```
- **Current**: `perf_pin.rs:101` reads `INTERNAL_TIME_NS` as the PRIMARY datum;
  external `getrusage` + `/usr/bin/time` path already half-present.
- **Issues**: relates to #3937 (memory gate).

### D5.3 — Single results store

- **Goal**: one machine-local, gitignored `results.sqlite` (WAL, single-writer)
  at `tests/cpython/.cache/conformance/results.sqlite`, holding every
  `fixture × runtime × dimension` result. CPython rows are content-addressed
  (oracle runs once, re-runs only changed fixtures); mamba rows keyed by mamba
  git sha.
- **Schema (minimum columns)**: `fixture_id, content_hash, runtime,
  runtime_version, dimension, verdict, exit_code, stdout_hash, raised_type,
  cpu_time_ns, peak_rss_bytes, signal, timed_out, recorded_at`.
- **Success metric**:
  ```
  # store exists with the table after one collect run
  sqlite3 tests/cpython/.cache/conformance/results.sqlite '.tables' | grep -q results
  # oracle re-run is incremental: second run on unchanged tree reports miss=0
  <collector> --record-oracle | rg 'oracle .*hit=[0-9]+ .*miss=0'
  # gitignored
  git check-ignore tests/cpython/.cache/conformance/results.sqlite
  ```
- **Current**: only the perf dimension has `.cache/perf/cpython_baseline.sqlite`
  (`perf_baseline.py`, `record --missing-only`). Behavior/safety have no store.
- **Issues**: generalizes `perf_baseline.py`; relates to #3937.

### D5.4 — Pool collector (isolated, sandboxed, parallel)

- **Goal**: one worker pool runs each fixture as an isolated, RLIMIT-sandboxed
  child (crash-isolated, no shared state), in parallel, sinking results to the
  store via a single writer thread.
- **Success metric**:
  ```
  # a deliberately-crashing fixture yields a CRASH_SIG verdict and the run still completes
  <collector> --jobs $(nproc) ; echo "exit=$?"      # run completes, no harness-level abort
  sqlite3 …/results.sqlite "SELECT verdict,COUNT(*) FROM results GROUP BY verdict"  # CRASH_SIG/TIMEOUT present, others unaffected
  # no sqlite contention
  <collector> 2>&1 | rg -c 'database is locked'      # → 0
  ```
- **Current**: `runner.rs` already uses `datatest_stable` (per-fixture
  `spawn_mamba`/`spawn_python` child + libtest pool + `apply_child_limits`
  RLIMIT), but re-runs the oracle every time and sinks nothing.
- **Issues**: #3935 (RLIMIT sandbox), #3936 (crash ratchet).

### D5.5 — One summary command

- **Goal**: a single command emits a machine-readable **D1–D5 per-dimension
  PASS/FAIL summary** plus a `--since <sha>` delta, reproducing every legacy
  run.py verdict (DIVERGE / MISSING_RAISE / CRASH_SIG{ABRT,SEGV,BUS} / TIMEOUT /
  OOM / TYPE_LEAKED / SLOW / MEM_REGRESSION).
- **Success metric**:
  ```
  <summary-cmd> --format json | jq -e '.dimensions | keys == ["D1","D2","D3","D4","D5"]'
  <summary-cmd> --since HEAD~1 | rg 'newly_red=|newly_green=|regressions='
  ```
- **Current**: cargo test prints `ok`/`FAILED` per case; no structured summary,
  no delta.
- **Issues**: #3936 (verdicts), #3938 (type-strict verdict), #3934 capstone.

### D5.6 — Retire the legacy Python runner

- **Goal**: retire the legacy **runner** — the second, Python conformance runner
  (`run.py`) — so the Rust harness is the single runner. No `run.py`, no doc
  references to it.
- **Scope clarification (2026-06-01)**: #3934 bundled `regen_golden.py` into this
  criterion as an "orphan golden regenerator". It is **not** a runner and **not**
  an orphan: it maintains the 683 active `.expected` goldens (referenced by
  `runner.rs` doc + `tests/README.md`, and it generates the type-strict
  two-golden form). Deleting it would strand those goldens, and **migrating off
  the 683 goldens is an explicit Non-goal**. So `regen_golden.py`'s retirement is
  **out of D5.6's runner-retirement scope** and deferred to that orthogonal
  golden-migration capstone — it stays in the tree.
- **Success metric**:
  ```
  find projects/mamba/tests -name 'run.py' -not -path '*/fixtures/*' | wc -l   # → 0  (MET)
  rg -rln 'tests/cpython/run\.py' projects/mamba/CONTRIBUTING.md \
     projects/mamba/tests/cpython/conventions | wc -l                          # → 0  (MET)
  ```
- **Capstone DONE (option B)**: `golden_capstone.py` proved the dynamic CPython
  oracle reproduces all 668 cpython goldens (type-strict uses
  `# mamba-strict-type` directives, not goldens). `runner.rs` now runs the live
  oracle for every fixture; all 683 static `.expected` files were deleted.
- **Current**: `run.py` retired ✓, all 683 `.expected` goldens retired ✓, no
  `run.py` doc refs ✓ — the legacy runner and its static goldens are gone.
  `regen_golden.py` retained ONLY because `src/main.rs` (the `mamba --regen`
  handler) still invokes it; `src/` is out of `tests/**` scope, so retiring the
  regenerator + that handler is the final **cross-scope** step (needs explicit
  authorization to touch `src/`).
- **Issues**: #3934 capstone (done; regenerator deletion cross-scope), #3939.

---

## D1 — Type-strict

- **Goal**: every `type-strict` fixture pins mamba's force-typed boundary with a
  `typeerror:` / `no_typeerror:` marker; the harness yields STRICT_TYPE_OK /
  MAMBA_TYPE_LEAKED.
- **Success metric**:
  ```
  # every type-strict fixture carries a directive
  find projects/mamba/tests/cpython/fixtures/type-strict -name '*.py' \
    -exec grep -L 'typeerror:\|no_typeerror:' {} + | wc -l     # → 0
  # collector emits the verdict pair
  <summary-cmd> --dimension D1 | rg 'STRICT_TYPE_OK|MAMBA_TYPE_LEAKED'
  ```
- **Issues**: #3938.

## D2 — Behavior

- **Goal**: the seed corpus is complete; each seed is a checked-in self-asserting
  `.py` that exits 0 under CPython 3.12 and pins the correct behavior.
- **Success metric**:
  ```
  # lang + 3p seeds present and lint-clean
  python3 projects/mamba/tests/cpython/tools/fixture_lint.py --bucket pep --bucket 3rd-libs   # exit 0
  aw wi list --label gate:production --state open | rg -c 'AssertionPass seed'   # → 0 (all authored)
  ```
- **Issues**: lang #3340 / #3346–3361, 3p #3458–3472.

## D3 — Performance

- **Goal**: bench fixtures are pure (D5.1) and the harness records a CPython 3.12
  CPU + peak-RSS baseline into the results store; the perf verdict is the
  mamba/cpython ratio vs the pinned floor, computed from the store.
- **Success metric**:
  ```
  <collector> --record-oracle --dimension D3
  sqlite3 …/results.sqlite \
    "SELECT COUNT(*) FROM results WHERE runtime='cpython' AND dimension='bench' AND cpu_time_ns IS NOT NULL"  # = bench-fixture count
  <summary-cmd> --dimension D3 | rg 'ratio|floor'
  ```
- **Issues**: epic #3880.

## D4 — Safety & stability

- **Goal (in-scope, black-box)**: a hostile/adversarial corpus under
  `fixtures/**/security/` + the sandboxed collector report 0 SIGSEGV / OOM / hang;
  cross-run peak-RSS growth detects leaks black-box.
- **Out of scope here (runtime line)**: `count_live` / `audit_all_modules` hooks
  (#3930), miri/ASan on `rc.rs` and `cargo-fuzz` targets (#3931) — they require
  `src/` or `fuzz/`. Track as dependencies, do not implement under this goal.
- **Success metric**:
  ```
  # hostile corpus runs fully sandboxed; harness survives, every case gets a verdict
  <collector> --dimension D4 ; echo "exit=$?"
  <summary-cmd> --dimension D4 | rg 'CRASH_SIG|TIMEOUT|OOM|leak'
  ```
- **Issues**: #3929 (parser resilience, black-box parts); #3930 / #3931 (deps).

---

## Per-turn workflow

1. Pick ONE piece — **harness pieces (D5) first**, then D1–D4. Prefer the piece
   that unblocks the most others (D5.3 store + D5.2 external measurement before
   D5.1 fixture-purity sweep, so perf data never goes dark).
2. Author the fixture / harness capability. Confirm it **RUNS and REPORTS
   correctly** — not that mamba passes it.
3. Run the item's success-metric command; paste its output into the transcript.
4. `commit → git pull --rebase → push → close` any matching issue. One piece per
   commit.

## Anti-cheat

NEVER, to make something green: edit `projects/mamba/src/**`, weaken or delete an
assertion, hardcode a metric back into a fixture, lower a perf floor, or mark
`xfail` / `skip` / `stub`. If a fixture looks wrong, STOP and report. Red is a
valid, honest result.

## The `/goal` condition

```
Follow projects/mamba/tests/PRODUCTION-GATE.md. Build the D1–D5 test gate;
harness (D5) first. Scope = projects/mamba/tests/** ONLY (src/** is out of
scope; sole exception a [[test]] stanza in Cargo.toml). TEST + HARNESS
AUTHORING ONLY — never fix runtime to go green; a red fixture is CORRECT.
DONE when every success metric in that file passes: D5.1 grep→0, D5.2 harness
owns measurement, D5.3 results.sqlite content-addressed store, D5.4 sandboxed
pool collector, D5.5 one D1–D5 summary + --since delta, D5.6 regen_golden.py +
run.py retired, then D1 type-strict / D2 behavior seeds / D3 pure-bench baseline
/ D4 black-box safety gate. Each turn advance ONE piece (harness first), run its
success-metric command, paste the output + remaining-work count, then commit →
pull --rebase → push → close. NEVER edit src, weaken an assertion, hardcode a
metric, lower a floor, or xfail/skip/stub to go green.
```
