# Perf-pin regression gate — baselines & running the enforcing test (#957 / #707 phase 0-1)

Each `*.toml` in this directory declares one perf-pin: `issue`, `lib`,
`fixture` (a `bench/*.py` under `tests/cpython/**`), `floor` (max allowed
`mamba_cpu_ns / cpython_cpu_ns`), optional `mem_floor` (min allowed
`cpython_rss / mamba_rss`), `samples` (median-of-N), and `prereq_imports`
(skip the pin if these Python modules aren't importable in the oracle env).

The gate itself lives in `tests/harness/cpython/perf_pin.rs`, registered as a
`harness=false` cargo test named `perf_pin` in the repo-root `Cargo.toml`. It
measures mamba live via external `getrusage`/`/usr/bin/time` on every run and
compares against a **pre-recorded CPython baseline** looked up by
`fixture_sha256` (so a baseline silently goes stale — and loudly fails — the
moment the fixture file changes).

## Recording the CPython baseline (do this before working on mamba perf)

Two independent tools, two independent purposes:

1. **The pin gate's baseline** (`tests/cpython/.cache/perf/cpython_baseline.sqlite`,
   consumed directly by `perf_pin.rs`):

   ```bash
   cd projects/mamba
   PY=tests/cpython/.cache/oracle-env/bin/python3   # the REAL interpreter — see below
   $PY tests/harness/cpython/tools/perf_baseline.py record --python "$PY"
   # single pin:
   $PY tests/harness/cpython/tools/perf_baseline.py record --pin tests/harness/cpython/config/perf/pins/<name>.toml --python "$PY"
   ```

2. **The broader dev-loop bench table** (`tests/cpython/.cache/perf.db`, all
   250 `bench/*.py` fixtures, for tuning/triage rather than gating):

   ```bash
   cd projects/mamba
   python3 tools/perf_baseline.py record --runtime cpython
   MAMBA_BIN=$PWD/target/release/mamba python3 tools/perf_baseline.py record --runtime mamba
   python3 tools/perf_baseline.py view
   ```

**Always resolve the real interpreter, never the bare `python3.12` on PATH** —
a pyenv shim adds ~190-200ms of wall/CPU overhead per invocation and will
poison every ratio. Verify with:

```bash
python3.12 -c "import sys; print(sys.executable)"
```

and prefer the harness's own resolved oracle interpreter,
`tests/cpython/.cache/oracle-env/bin/python3` (already the real
`pyenv`-resolved 3.12.11 binary, not the shim).

## Running the enforcing gate

```bash
cd projects/mamba
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" \
  cargo build --release -p mamba
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" \
  MAMBA_REQUIRE_CPYTHON_PERF_BASELINE=1 \
  cargo test -p mamba --release --test perf_pin
```

`MAMBA_REQUIRE_CPYTHON_PERF_BASELINE=1` turns a missing/stale baseline row
into a hard `assert!` failure instead of a silent live-fallback measurement —
always set it when the run is meant to *gate*, not just explore.

Note: the compiled test binary's own `--list`/filter names are
`run_pin::<pin-file>.toml`, **not** `perf_pin` — a trailing `-- perf_pin`
filter on `cargo test` matches zero tests (0 passed, N filtered out) and
silently no-ops the whole gate. Either pass no filter, or filter on
`run_pin::`.

## Build profile used for the recorded baseline

Baselines in this repo were recorded/gated against the **official ship
profile** (`opt-level=3, lto=true, codegen-units=1`, root `Cargo.toml`
`[profile.release]`), i.e. plain `cargo build --release -p mamba` with no
profile overrides — this is what should always back a *baseline recording or
enforcing run*. The `CARGO_PROFILE_RELEASE_LTO=false
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16` fast-verify override (documented for
iteration elsewhere in this repo) is fine for day-to-day conformance work but
must NOT be used to record or gate a perf baseline — it changes codegen and
will shift every ratio.

## `samples` convention

All 119 pins are normalized to `samples = 9` (median-of-9) as of #957, to
absorb process-startup/scheduler noise on a shared/contended box. Bump
further only if a specific pin is empirically still noisy at 9.

## Known gaps (out of scope for #957 — measurement infra only)

- **2 pins skip** for missing prereq imports in the oracle env:
  `google_cloud_storage_1510`, `google_cloud_pubsub_1511`.
- **12 pins fail baseline recording** on genuine CPython-side fixture bugs
  (not a mamba or harness defect): 11 `*_type_read_hot.py` 3rd-party-lib
  fixtures raise `AssertionError` (module-attribute-read accumulator drift)
  under real CPython 3.12, and `time_1435`'s `clock_calls_hot.py` raises
  `NameError: name 'b' is not defined` at line 34. See the #957 report for the
  full list; these need real fixture fixes, filed as a follow-up.
- **6 mamba bench fixtures hang indefinitely** under the mamba release binary
  (orphaned 100%-CPU child; `perf_pin.rs`'s `Command::output()` has no
  timeout): `set_add_hot`, `contextmanager_hot`, `chain_islice_hot`,
  `iskeyword_hot_module_attr`, `path_join_hot`, `simplefilter_hot` (the last
  is this dir's `warnings_1445` pin). Only `warnings_1445` overlaps a
  declared pin; the other 5 live in non-pinned bench fixtures. Filed as a P0
  follow-up (real mamba hangs) plus a secondary harness-hardening gap
  (recording/gating tools should launch in a new process group and `killpg`
  on timeout, since `subprocess.run(timeout=)`/`Command` only reaps the
  direct child, not a `/usr/bin/time`-wrapped grandchild).
