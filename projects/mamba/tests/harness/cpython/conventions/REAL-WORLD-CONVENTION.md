# Real-World Conformance Fixture Convention

> Every per-lib conformance issue carries an **R4** acceptance gate:
> *"at least one downstream PyPI consumer runs unchanged under mamba"*.
> This file pins the directory layout, fixture shape, and runner contract
> that satisfies R4 uniformly across all libraries (std-libs and 3rd-libs).

## Scope

Real-world fixtures are **end-user scenarios**, not unit tests of library
internals. The behavior contract is:

- An untouched Python script (`<scenario>.py`) imports the target library
  and performs one meaningful end-user operation.
- The script exits `0` on success under **both** CPython and mamba.
- The script's stdout is allowed to differ in whitespace but must be
  semantically equivalent across the two interpreters.

These complement, not replace, the behavior fixtures already in
`tests/cpython/std-libs/<lib>_*.py` (which are mamba-specific
golden tests).

## Directory layout

```text
projects/mamba/tests/cpython/
├── fixtures/                          # Python test specimens
│   ├── core/                          # language semantics
│   ├── pep/                           # PEP-numbered features
│   ├── builtin-libs/                  # builtin type methods
│   ├── std-libs/                      # importable stdlib modules
│   │   └── <lib>/                     # one-case-per-file dimension subdirs
│   │       ├── surface/<case>.py      # Q1 has-it (one probe each)
│   │       ├── behavior/<case>.py     # Q2 correct (one behavior each)
│   │       ├── errors/<case>.py       # one exception path each
│   │       ├── real_world/<case>.py   # R4 fixtures (exit-0 under both)
│   │       ├── security/<case>.py     # adversarial / untrusted-input
│   │       └── bench/<case>.py        # Q3 fast
│   │       # LEGACY (retired, still discovered): surface.py / behavior.py
│   │       #   / errors.py monoliths at the lib root coexist during the
│   │       #   incremental migration (DUAL-SHAPE — see FIXTURE-LAYOUT.md).
│   ├── 3rd-libs/                      # third-party PyPI libraries (uv-resolved)
│   │   └── <lib>/
│   │       ├── surface.py / behavior.py
│   │       ├── requirements.txt       # mirrors PEP 723 pins
│   │       ├── real_world/<scenario>.py
│   │       └── bench/<scenario>.py
│   └── type-strict/                   # runtime-typing contract (two-golden)
├── conventions/                       # author guides (this file lives here)
└── config/                            # declarative inputs
    ├── perf/pins/                     # perf-pin TOMLs
    └── seeds/                         # spec seeds (lib_test runner)

projects/mamba/tests/harness/
└── cpython/                           # Rust runners (Cargo [[test]])
```

Rules:

- `<lib>` is the importable package name (e.g. `idna`, `json`, `pathlib`).
  Use the lowercase canonical PyPI / stdlib name.
- A leading underscore (`_baseline`, `_synthetic`) marks a non-library
  meta-bucket — pure-Python fixtures that exercise language semantics
  rather than a specific package. Reserved for the bench harness's
  fallback fixtures while mamba is still maturing.
- `<scenario>` is a short snake_case verb phrase describing what the
  script does (e.g. `encode_idn`, `parse_iso_date`, `hash_blob`).
- One scenario per file. If a library needs multiple end-user flows,
  add multiple scenario files in the same `real_world/` or `bench/` dir.
- `std-libs/` is the existing flat directory for legacy behavior fixtures
  AND the new home for per-stdlib-lib subdirs. The two coexist (flat
  files alongside `<lib>/{real_world,bench}/` subdirs).
- `real_world/` fixtures must exit 0 on both runtimes (R4 DoD).
  `bench/` fixtures must exit 0 on both runtimes AND produce equivalent
  output — they additionally need to be timed deterministically (no
  network, no sleeps, fixed loop counts).

## Fixture script shape

Every scenario file follows this structure. The leading `# /// script`
block is **PEP 723 inline script metadata** — `uv run <file>` consumes
it to materialize an ephemeral venv with the declared deps before
running the script under CPython. Stdlib-only fixtures still carry the
block (with an empty `dependencies = []`) so `uv run <any-fixture>`
works uniformly across buckets.

```python
# /// script
# requires-python = ">=3.12"
# dependencies = []           # stdlib-only fixtures keep the block empty
# ///
"""<one-line scenario description>.

DoD: this script must exit 0 under both CPython and mamba.
"""

import <lib>

# 1. Set up one realistic input.
# 2. Call one library entry point.
# 3. assert the result against an expected value.
# 4. print() a short confirmation so a human running it sees signal.

result = <lib>.<operation>(<input>)
assert result == <expected>, f"unexpected {result!r}"
print("ok:", result)
```

For 3rd-libs fixtures, pin exact versions in the PEP 723 block AND
mirror them into a sibling `requirements.txt`. The PEP 723 block is
what `uv run` consumes; the `requirements.txt` mirrors the same dependency
set for Rust harnesses that need dependency-aware CPython oracle runs:

```python
# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "idna==3.7",
# ]
# ///
```

```text
# 3rd-libs/idna/requirements.txt
idna==3.7
```

The CPython oracle for `3rd-libs` is routed through `uv run` by
default (`--use-uv auto`); other buckets still use plain `python3`.
Mamba does not consume PEP 723 metadata, so the mamba side of every
gate stays `mamba run <file>` regardless of bucket.

Hard constraints:

- **Standalone.** No external network, no filesystem writes outside a
  `tempfile.TemporaryDirectory()`, no system mutation. Anything the
  script needs at runtime must be inline in the file.
- **Deterministic.** No `random` without a fixed seed, no wall-clock
  comparisons, no thread races. Same input → same output every run.
- **English only.** All identifiers, comments, docstrings, and printed
  strings are English (per `feedback_english_only_specs`). Test data
  that is itself non-ASCII (e.g. an IDN input string) is fine as long
  as commentary describing it is English.
- **No third-party imports** beyond the library under test and its own
  transitive deps. The fixture must not pull in `pytest`, `hypothesis`,
  etc.
- **Exit semantics.** Success = exit `0`. Any assertion failure or
  uncaught exception (non-zero exit) is a fixture failure.

## Runner contract

`projects/mamba/tests/harness/cpython/real_world.rs` is the Rust integration
test that walks `tests/cpython/{std-libs,3rd-libs}/*/real_world/`
and shells out for every `*.py` it finds.

For each scenario file the runner:

1. Runs `python3 <file>` — must exit `0`. If `python3` is unavailable,
   the per-file case is skipped (not a failure).
2. Runs `mamba run <file>` (resolved via `CARGO_BIN_EXE_mamba`) — must exit
   `0`.
3. Asserts both exited `0`. Stdout equivalence is not asserted by the
   runner — fixtures self-validate via internal `assert`.

The whole test file is marked `#[ignore]` by default so it does not run
on every `cargo test`; opt in with:

```bash
cargo test -p mamba --test conformance_real_world -- --ignored
```

This keeps the default test suite fast while letting CI / conformance
patrol explicitly invoke the real-world gate.

## Bench harness (Phase 1.C)

`projects/mamba/benches/3p/cross_runtime.rs` is the cross-runtime bench
that walks `tests/cpython/{std-libs,3rd-libs}/*/bench/` and runs
each `*.py` under both `python3` and `mamba run`, picking the minimum
wall-clock per side (best-of-N) and emitting one line per fixture:

```text
[<lib>/<scenario>] mamba/cpython = X.YYx (PASS|FAIL @ floor 1.0x)
```

The bench exits non-zero on any FAIL so CI can gate. Tier-aware
thresholds (`compute>=10x`, `app>=3x`, `dynamic>=1.5x`) from issue
#1265 are documented but not yet enforced by the harness — only the
floor (`>=1.0x`) is gated today.

Run:

```bash
cargo bench -p mamba --bench cross_runtime_3p
cargo bench -p mamba --bench cross_runtime_3p -- --fixture fib_recursive
cargo bench -p mamba --bench cross_runtime_3p -- --iters 7
```

Bench-fixture-specific constraints (in addition to the standalone /
deterministic / English-only rules above):

- **Fixed iteration count.** A bench fixture must run for a deterministic
  number of operations (e.g. `for _ in range(1000):`) so wall-clock
  reflects per-op cost, not workload variance.
- **No randomized input.** Use a hard-coded representative input. If
  randomness is essential, seed it with a literal.
- **No external libs that mamba can't load.** Real-PyPI fixtures (e.g.
  `idna`) can be checked in early; they will report `ERROR` from the
  harness until mamba's pkgmgr lands. Use `3rd-libs/_baseline/bench/` for
  pure-Python fallback fixtures that exercise language semantics only.

## Adding a new fixture (checklist)

1. Pick `<lib>` and `<scenario>`.
2. Create `tests/cpython/{std-libs|3rd-libs}/<lib>/real_world/<scenario>.py`
   following the shape above.
3. Confirm locally: `python3 <file>` exits `0`.
4. Confirm under mamba: `mamba <file>` exits `0` (or document the
   blocker in the conformance issue's R4 section).
5. The runner picks up the new file automatically — no Rust edit needed.

## DoD (per conformance issue R4)

A conformance issue's R4 gate is **green** when at least one fixture
under `tests/cpython/{std-libs|3rd-libs}/<lib>/real_world/` runs
to exit-0 success under both CPython and mamba via the runner above.
