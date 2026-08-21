# exceptions — external contract (as-is, 2026-07-15)

Domain map: `tech-design/exceptions/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Oracle = python3.12 byte-diff; xfail = acknowledged gap, still contract (never executed — see HARNESS.md rot hazard).

## Positive contract — must RUN and byte-match the oracle

All paths under `tests/cpython/`. Live counts (2026-07-15): **506 fixtures, 260 xfail**.

| Fixture dir | .py | xfail |
|---|---|---|
| `_regression/core/exceptions` | 46 | 2 |
| `_regression/core/exception_chaining` | 4 | 0 |
| `_regression/core/exception_control_flow` | 4 | 0 |
| `_regression/core/exception_group` | 10 | 0 |
| `_regression/core/custom_exception` | 4 | 0 |
| `_regression/core/traceback_smoke` | 4 | 0 |
| `behavior/core/exceptions` | 117 | 107 |
| `behavior/core/exception_group` | 50 | 41 |
| `behavior/std-libs/baseexception` (test_baseexception port) | 11 | 10 |
| `behavior/std-libs/except_star` (test_except_star port) | 60 | 59 |
| `behavior/std-libs/exception_hierarchy` (port) | 16 | 16 |
| `behavior/std-libs/exception_variations` (port) | 30 | 0 |
| `behavior/std-libs/traceback` (#1441) | 110 | 25 |
| `errors/std-libs/traceback` | 4 | 0 |
| `surface/std-libs/traceback` | 34 | 0 |
| `real_world/std-libs/traceback` | 1 | 0 |
| `_regression/std-libs/traceback` (bench/ — C2 pin-owned, C1-skipped per #2239) | 1 | 0 |

Adjacent, counted elsewhere: `*/std-libs/inspect_traceback` (6/1/9 — stdlib/inspect, same traceback gaps);
`_regression/core/grammar/test_exceptions` (3) + `_regression/core/language/exceptions` (3) — frontend parse fixtures.

## Negative contract — must be REJECTED

No wall dimension of its own (README domain map). Walls over this domain's surface, owned by type-system per the dimension rule:

- `type/std-libs/traceback` — 29 strict-type walls (10 xfail: force-typed arg enforcement pending, epics #861/#862).
- `type/std-libs/asyncio_exceptions` (2), `type/std-libs/xml_sax__exceptions` (2) — owned by those modules' surfaces.

Runtime rejects (non-BaseException matcher → TypeError via `collect_matcher_targets`; PEP 654 EG constructor validation) are proven inside positive fixtures, not walls.

## Known contract gaps

- **51% xfail (260/506)**; 258 carry `auto-ported/extracted CPython test; mamba promotion pending` — the mass strata are behavior/core/exceptions (107), except_star (59), behavior/core/exception_group (41), traceback (25), exception_hierarchy (16), baseexception (10). Un-xfail campaign: #1768. xfail = full skip, so these rot silently until the directive is deleted.
- **Traceback truth-surface hole**: `e.__traceback__` is None after catch; `format_exc()` returns the literal `"NoneType: None\n"` stub (no frame walk/linecache); `mb_take_uncaught_traceback` prints a fixed one-frame header. Pinned as xfail `_regression/core/exceptions/traceback_attribute_raises.py` + the gotcha header of `tests/harness/cpython/config/manifests/std-libs/traceback.toml`. Live-divergence family: #1770.
- **except-var scope divergence**: `except X as e` keeps `e` bound after the block (CPython unbinds) — xfail `_regression/core/exceptions/except_var_leaks_raises.py`, tracked #1770.
- **Subclass-init shapes OPEN (#1557)**: unbound `Exception.__init__(self,…)` loses attrs; `__new__` args not pre-stored (str() falls to generic repr); composite NoneType-callable crash — red lines in `tech-design/exceptions/construction-and-rendering.md` §Known gaps.
- **Ghost lib test**: `src/driver/tests/behavioral_lang.rs:707-746 test_regression_exceptions_parse` walks the retired `core/exceptions` path; `verify_all_parse` returns silently on a missing dir, so it passes on 0 files — pipeline-ghosts #1767 (tests-EC review journal, false-confidence finding).
- **Unmeasured type-dim xfail stratum**: the 10 force-typed traceback walls (#861/#862) were never probed for free greens (journal finding; un-xfail #1768 / harness modernization #1771).

## Verification

```bash
# inner loop (seconds; paths relative to tests/cpython) — from apps/mamba/tests/harness/cpython/
python3 tools/sweep.py _regression/core/exceptions _regression/core/exception_chaining \
  _regression/core/exception_control_flow _regression/core/exception_group _regression/core/custom_exception \
  _regression/core/traceback_smoke behavior/core/exceptions behavior/core/exception_group \
  behavior/std-libs/baseexception behavior/std-libs/except_star behavior/std-libs/exception_hierarchy \
  behavior/std-libs/exception_variations                        # core-semantics slice
python3 tools/sweep.py behavior/std-libs/traceback errors/std-libs/traceback surface/std-libs/traceback \
  real_world/std-libs/traceback type/std-libs/traceback         # traceback slice incl. the 29 walls
# full C1 gate (~3 min; this domain's slice rides inside it) — never concurrent with a cargo build
cargo test -p mamba --release --test conformance
# C2 slice: tests/harness/cpython/config/perf/pins/traceback_1441.toml (format_exc_hot bench)
cargo test -p mamba --release --test perf_pin -- perf_pin
# manifests: config/manifests/std-libs/traceback.toml (+ generated cpython312_surface/traceback.toml);
# after fixture edits run tools/fixture_lint.py
```
