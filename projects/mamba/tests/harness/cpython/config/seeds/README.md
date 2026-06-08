# cpython_lib_test — externally-defined conformance denominator

Tracking issue: **#1396**. Folder-convention refactor: **#3729**.

Goal: replace the self-defined "100% conformance / N/N fixtures we wrote
ourselves" metric with a real CPython 3.12 denominator. Files under this
directory are vendored or hand-authored seeds that exercise CPython 3.12
runtime behaviour. Mamba runs each one through `mamba run`, classifies the
outcome, and the runner asserts the outcome matches the seed's parent
directory.

PyPy uses CPython's `Lib/test/` as their conformance suite for the same
reason — it's a denominator we don't control.

## Folder-based contract convention (#3729)

The parent directory name IS the expected outcome. No TOML file pins per-seed
state any more — `cpython_lib_test_baseline.toml` and
`cpython_lib_test_allowlist.toml` were retired by #3729.

```
fixtures/cpython_lib_test/
├── README.md
├── pass/          # must AssertionPass — prints `MAMBA_ASSERTION_PASS: <name> <N> asserts`
├── spec/          # must Fail today; encodes the full CPython 3.12 contract
│                  # mamba is growing into. Promotion = `git mv spec/<f>.py pass/<f>.py`
├── stub/          # must Stub — mamba silently bypassed the entry point
│                  # (e.g. `unittest.main()` swallow)
├── fail/          # must Fail — known broken with no current growth path
├── import_pass/   # must ImportPass — legacy. New seeds go in `pass/` or `spec/`
├── timeout/       # must Timeout — wall-clock budget exceeded
├── executes_assertions_not_stub_pass/   # parent-gate manifest, #2528
└── minimal_unittest_dispatch/           # parent-gate manifest, #2545
```

### Promotion (mamba caught up to a `spec/` seed)

```
git mv tests/cpython/lib_test_seeds/spec/<file>.py \
       tests/cpython/lib_test_seeds/pass/<file>.py
```

The runner surfaces drift bidirectionally: if `spec/<f>.py` starts passing,
the failure message recommends the `git mv` above verbatim. If a `pass/<f>.py`
regresses, the same gate fails and the fix is `git mv pass/<f>.py fail/<f>.py`
(or `stub/`, whichever matches the new outcome).

### Outcome classification (see `tests/cpython_lib_test_runner.rs`)

| Outcome | Signal |
|---------|--------|
| `AssertionPass` | exit 0 + `MAMBA_ASSERTION_PASS` or `[mamba-assertion-pass]` marker |
| `ImportPass`    | exit 0 + no stub marker + no assertion-pass marker |
| `Stub`          | exit 0 + a known stub marker (`unittest.main() called`, `is not implemented in Mamba`) |
| `Fail`          | non-zero exit |
| `Timeout`       | child exceeded 60s wall-clock |

## Phases (against #1396 acceptance)

| Phase | Deliverable | Status |
|-------|-------------|--------|
| **0 — Scaffolding** | Directory structure, README | ✅ |
| **1 — Runner** | `tests/cpython_lib_test_runner.rs` walks every contract dir | ✅ |
| **2 — Contract pinning** | Folder convention — directory IS the contract | ✅ (#3729) |
| **3 — Expansion** | Vendor + author seeds across language and stdlib surface | ongoing |
| **4 — Monitoring loop wired** | Loop summary line includes `cpython_lib_test: assertion_pass / total` | TODO |

## Conformance metric semantics

Numbers from this suite are **externally defined** — they reflect how much
CPython 3.12 surface mamba supports, not how much regression-prevention
coverage we've authored. See `NOTES-NEXT.md` top-of-file caveat (#1398
context) for why the existing self-defined fixture pass rate is
regression-baseline only.

Per-run sidecar JSON is written to
`$CARGO_TARGET_TMPDIR/cpython_lib_test_summary.json` (override with
`MAMBA_CPYTHON_LIB_TEST_SUMMARY_PATH`). Schema is versioned
(`schema_version = 2`, `harness_kind = "runtime"`); CI scrapers gate on
`harness_kind` so this never gets silently summed with the parser-only
`cpython_compat.rs` harness.

## Rules for vendored files

- **Verbatim copy** is preferred for vendored CPython tests — byte-equivalent
  to CPython 3.12's `Lib/test/test_*.py`. Hand-authored seeds (PEP 695 syntax
  drills, sentinels, async-context-manager probes) live in the same dirs but
  are explicitly mamba-authored.
- **CPython 3.12 only**: when 3.13 / 3.14 conformance opens (#1266 / #1267),
  use sibling fixture roots.

## Sentinels (#2539)

Files named `sentinel_*.py` are **mamba-authored**, not vendored. They are
intentionally tiny (one test class, one or two assertions) and they answer
one question: did the unittest runner actually execute assertions?

If both `sentinel_assertion_pass.py` and `sentinel_assertion_fail.py` end up
in the same outcome bucket, the dispatcher is bypassing assertions and the
sentinel pair has done its job.

## Spec seeds (`spec/`)

A `spec/` seed encodes the **full CPython 3.12 contract** for a feature, NOT
the subset mamba already implements. Each unsupported behavior is wrapped in
`try/except` plus an explicit `assert`, so mamba's print-error-but-exit-0 path
becomes a hard AssertionError. This keeps `spec/` failing today AND turns into
a regression-prevention contract the moment mamba implements the missing
piece.

Pattern (real example, `spec/lang_pep695_generics_spec.py`):

```python
_b = None
_err = None
try:
    _b = _Box[int](42)
except Exception as e:
    _err = f"{type(e).__name__}: {e}"
assert _b is not None and _err is None and _b.v - 42 == 0, (
    f"_Box[int](42) constructs a real instance with .v == 42, "
    f"got _b={_b!r}, err={_err!r}"
)
```

When mamba grows `__class_getitem__` for PEP 695 generic classes, this
assertion starts passing — the runner reports drift, and the one-line
promotion is `git mv spec/lang_pep695_generics_spec.py pass/`.
