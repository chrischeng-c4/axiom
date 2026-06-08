# Axis 1 — existing cpython_lib_test seed audit

Generated: 2026-05-20
Source: #3334 (epic: #3331)

Audits the 71 entries under `projects/mamba/tests/cpython/lib_test_seeds/seed/` against the current `target/release/mamba` build to classify each as ready-to-promote, skeleton, blocked, or unstable. Live results were collected by running each seed under a 60s wall-clock timeout. The `MAMBA_ASSERTION_PASS:` stdout marker is the AssertionPass proof signal (per #2540).

## Summary

| Bucket | Count | Meaning |
|---|---|---|
| AssertionPass-ready | 22 | exit 0 + marker emitted — promote to allowlist `minimum_outcome = "AssertionPass"` |
| needs-asserts | 30 | exit 0 but no marker — add real asserts per `test_math.py` |
| blocked | 18 | runtime error — fix mamba (file/link runtime-gap issue) |
| unrunnable | 1 | timeout / segfault — file/link runtime-gap issue |

Total: 71

## Per-seed rows

| seed | baseline outcome | live result | bucket | blocker / first error | recommended axis-1 issue |
|---|---|---|---|---|---|
| test_argparse | AssertionPass | exit 0, marker emitted (3 asserts) | AssertionPass-ready | — | #3433 |
| test_bisect | AssertionPass | exit 0, marker emitted (15 asserts) | AssertionPass-ready | — | #3379 |
| test_collections | AssertionPass | exit 0, marker emitted (24 asserts) | AssertionPass-ready | — | #3373 |
| test_copy | AssertionPass | exit 0, marker emitted (20 asserts) | AssertionPass-ready | — | #3381 |
| test_datetime | AssertionPass | exit 0, marker emitted (16 asserts) | AssertionPass-ready | — | #3390 |
| test_decimal | AssertionPass | exit 0, marker emitted (6 asserts) | AssertionPass-ready | — | #3370 |
| test_enum | AssertionPass | exit 0, marker emitted (15 asserts) | AssertionPass-ready | — | #3443 |
| test_fractions | AssertionPass | exit 0, marker emitted (11 asserts) | AssertionPass-ready | — | #3371 |
| test_functools | AssertionPass | exit 0, marker emitted (21 asserts) | AssertionPass-ready | — | #3376 |
| test_heapq | AssertionPass | exit 0, marker emitted (26 asserts) | AssertionPass-ready | — | #3378 |
| test_importlib | AssertionPass | exit 0, marker emitted (3 asserts) | AssertionPass-ready | — | #3447 |
| test_itertools | AssertionPass | exit 0, marker emitted (16 asserts) | AssertionPass-ready | — | #3375 |
| test_json | AssertionPass | exit 0, marker emitted (24 asserts) | AssertionPass-ready | — | #3404 |
| test_logging | AssertionPass | exit 0, marker emitted (13 asserts) | AssertionPass-ready | — | #3434 |
| test_math | AssertionPass | exit 0, marker emitted (39 asserts) | AssertionPass-ready | — | #3367 |
| test_operator | AssertionPass | exit 0, marker emitted (38 asserts) | AssertionPass-ready | — | #3377 |
| test_pathlib | AssertionPass | exit 0, marker emitted (11 asserts) | AssertionPass-ready | — | #3397 |
| test_pickle | AssertionPass | exit 0, marker emitted (22 asserts) | AssertionPass-ready | — | #3403 |
| test_random | AssertionPass | exit 0, marker emitted (27 asserts) | AssertionPass-ready | — | #3372 |
| test_re | AssertionPass | exit 0, marker emitted (26 asserts) | AssertionPass-ready | — | #3384 |
| test_statistics | AssertionPass | exit 0, marker emitted (24 asserts) | AssertionPass-ready | — | #3369 |
| test_unittest | AssertionPass | exit 0, marker emitted (19 asserts) | AssertionPass-ready | — | #3450 |
| sentinel_assertion_fail | Stub | exit 0, no marker | needs-asserts | — | — |
| sentinel_assertion_pass | Stub | exit 0, no marker | needs-asserts | — | — |
| test_atexit | Stub | exit 0, no marker | needs-asserts | — | #3430 |
| test_base64 | Stub | exit 0, no marker | needs-asserts | — | #3409 |
| test_bool | Stub | exit 0, no marker | needs-asserts | — | — |
| test_calendar | Stub | exit 0, no marker | needs-asserts | — | #3391 |
| test_codeop | Stub | exit 0, no marker | needs-asserts | — | — |
| test_csv | Stub | exit 0, no marker | needs-asserts | — | #3405 |
| test_difflib | Stub | exit 0, no marker | needs-asserts | — | #3386 |
| test_errno | Stub | exit 0, no marker | needs-asserts | — | — |
| test_float | Stub | exit 0, no marker | needs-asserts | — | — |
| test_fnmatch | Stub | exit 0, no marker | needs-asserts | — | #3401 |
| test_glob | Stub | exit 0, no marker | needs-asserts | — | #3400 |
| test_html | Stub | exit 0, no marker | needs-asserts | — | #3420 |
| test_int | Stub | exit 0, no marker | needs-asserts | — | — |
| test_keyword | Stub | exit 0, no marker | needs-asserts | — | — |
| test_list | Stub | exit 0, no marker | needs-asserts | — | — |
| test_locale | Stub | exit 0, no marker | needs-asserts | — | #3454 |
| test_pkgutil | Stub | exit 0, no marker | needs-asserts | — | #3448 |
| test_sched | Stub | exit 0, no marker | needs-asserts | — | — |
| test_secrets | Stub | exit 0, no marker | needs-asserts | — | #3413 |
| test_set | Stub | exit 0, no marker | needs-asserts | — | — |
| test_slice | Stub | exit 0, no marker | needs-asserts | — | — |
| test_strftime | Stub | exit 0, no marker | needs-asserts | — | — |
| test_string | Stub | exit 0, no marker | needs-asserts | — | #3383 |
| test_struct | Stub | exit 0, no marker | needs-asserts | — | #3387 |
| test_syntax | Stub | exit 0, no marker | needs-asserts | — | — |
| test_textwrap | Stub | exit 0, no marker | needs-asserts | — | #3385 |
| test_tuple | Stub | exit 0, no marker | needs-asserts | — | — |
| test_unpack | Stub | exit 0, no marker | needs-asserts | — | — |
| test_bytes | Fail | exit 1 | blocked | error: syntax error at 12738..12743: expected ,, got identifier | — |
| test_class | Fail | exit 1 | blocked | error: type error at 2068..2074: undefined name: `method` | — |
| test_complex | Fail | exit 1 | blocked | error: type error at 4494..4500: arithmetic requires numeric types | — |
| test_descr | Fail | exit 1 | blocked | error: type error at 125078..125092: arithmetic requires numeric types | — |
| test_dict | Fail | exit 1 | blocked | error: codegen error: codegen error: define: Duplicate definition of identifier: _mb_1000334 | — |
| test_exceptions | Fail | exit 1 | blocked | error: syntax error at 45522..45523: unexpected token: newline | — |
| test_generators | Fail | exit 1 | blocked | error: syntax error at 41792..41793: expected in, got [ | — |
| test_grammar | Fail | exit 1 | blocked | error: syntax error at 3561..3562: unexpected token: ) | — |
| test_hmac | Fail | exit 1 | blocked | error: AttributeError: 'dict' object has no attribute 'main' | #3412 |
| test_imaplib | Fail | exit 1 | blocked | error: type error at 38781..38796: undefined name: `SecureTCPServer` | — |
| test_iter | Fail | exit 1 | blocked | error: type error at 38702..38708: called value is not a function | — |
| test_range | Fail | exit 1 | blocked | error: type error at 17878..17880: argument type mismatch: expected `int`, got `list[Any]` | — |
| test_socketserver | Fail | exit 1 | blocked | error: AttributeError: 'NoneType' object has no attribute 'requires' | — |
| test_time | Fail | exit 1 | blocked | error: AttributeError: 'NoneType' object has no attribute 'ceil' | #3392 |
| test_typing | Fail | exit 1 | blocked | error: syntax error at 61012..61013: expected ), got [ | #3441 |
| test_uuid | Fail | exit 1 | blocked | error: syntax error at 1945..1946: unexpected token: , | #3414 |
| test_with | Fail | exit 1 | blocked | error: type error at 28002..28008: called value is not a function | — |
| test_yield_from | Fail | exit 1 | blocked | error: codegen error: codegen error: define: Duplicate definition of identifier: _mb_4000203 | — |
| test_poplib | Fail | panic/signal (exit 101) | unrunnable | thread 'main' (219386380) panicked at projects/mamba/src/codegen/cranelift/jit.rs:841:77: | — |

## Action items

### AssertionPass-ready seeds (22) — promote in next iteration

These already exit 0 and emit the `MAMBA_ASSERTION_PASS:` marker. Under the folder-based contract convention (#3729), the promotion path is `git mv tests/cpython/lib_test_seeds/<src>/<stem>.py tests/cpython/lib_test_seeds/pass/<stem>.py`. (Historical note: previously this required dual edits to `cpython_lib_test_baseline.toml` + `cpython_lib_test_allowlist.toml`; both files were retired by #3729 — the parent directory IS the contract.)

- `test_argparse` (3 asserts) — #3433, baseline=AssertionPass
- `test_bisect` (15 asserts) — #3379, baseline=AssertionPass
- `test_collections` (24 asserts) — #3373, baseline=AssertionPass
- `test_copy` (20 asserts) — #3381, baseline=AssertionPass
- `test_datetime` (16 asserts) — #3390, baseline=AssertionPass
- `test_decimal` (6 asserts) — #3370, baseline=AssertionPass
- `test_enum` (15 asserts) — #3443, baseline=AssertionPass
- `test_fractions` (11 asserts) — #3371, baseline=AssertionPass
- `test_functools` (21 asserts) — #3376, baseline=AssertionPass
- `test_heapq` (26 asserts) — #3378, baseline=AssertionPass
- `test_importlib` (3 asserts) — #3447, baseline=AssertionPass
- `test_itertools` (16 asserts) — #3375, baseline=AssertionPass
- `test_json` (24 asserts) — #3404, baseline=AssertionPass
- `test_logging` (13 asserts) — #3434, baseline=AssertionPass
- `test_math` (39 asserts) — #3367, baseline=AssertionPass
- `test_operator` (38 asserts) — #3377, baseline=AssertionPass
- `test_pathlib` (11 asserts) — #3397, baseline=AssertionPass
- `test_pickle` (22 asserts) — #3403, baseline=AssertionPass
- `test_random` (27 asserts) — #3372, baseline=AssertionPass
- `test_re` (26 asserts) — #3384, baseline=AssertionPass
- `test_statistics` (24 asserts) — #3369, baseline=AssertionPass
- `test_unittest` (19 asserts) — #3450, baseline=AssertionPass

### Blocked seeds (18) — need runtime fix

These exit non-zero on the current `target/release/mamba`. Until the underlying runtime gap is closed, they cannot reach AssertionPass. Grouped by coarse error category; each cluster should be tracked by a mamba runtime-gap issue (file new if no matching one exists — out of scope for this audit doc).

**syntax error (parser gap)** (6 seeds):

- `test_bytes` — error: syntax error at 12738..12743: expected ,, got identifier (no axis-1 issue mapped, baseline=Fail)
- `test_exceptions` — error: syntax error at 45522..45523: unexpected token: newline (no axis-1 issue mapped, baseline=Fail)
- `test_generators` — error: syntax error at 41792..41793: expected in, got [ (no axis-1 issue mapped, baseline=Fail)
- `test_grammar` — error: syntax error at 3561..3562: unexpected token: ) (no axis-1 issue mapped, baseline=Fail)
- `test_typing` — error: syntax error at 61012..61013: expected ), got [ (#3441, baseline=Fail)
- `test_uuid` — error: syntax error at 1945..1946: unexpected token: , (#3414, baseline=Fail)

**AttributeError (missing module attribute / 'NoneType' has no attribute)** (3 seeds):

- `test_hmac` — error: AttributeError: 'dict' object has no attribute 'main' (#3412, baseline=Fail)
- `test_socketserver` — error: AttributeError: 'NoneType' object has no attribute 'requires' (no axis-1 issue mapped, baseline=Fail)
- `test_time` — error: AttributeError: 'NoneType' object has no attribute 'ceil' (#3392, baseline=Fail)

**type error: undefined name (missing builtin/symbol)** (2 seeds):

- `test_class` — error: type error at 2068..2074: undefined name: `method` (no axis-1 issue mapped, baseline=Fail)
- `test_imaplib` — error: type error at 38781..38796: undefined name: `SecureTCPServer` (no axis-1 issue mapped, baseline=Fail)

**type error: arithmetic on non-numeric (Complex/numeric tower)** (2 seeds):

- `test_complex` — error: type error at 4494..4500: arithmetic requires numeric types (no axis-1 issue mapped, baseline=Fail)
- `test_descr` — error: type error at 125078..125092: arithmetic requires numeric types (no axis-1 issue mapped, baseline=Fail)

**codegen error (duplicate identifier)** (2 seeds):

- `test_dict` — error: codegen error: codegen error: define: Duplicate definition of identifier: _mb_1000334 (no axis-1 issue mapped, baseline=Fail)
- `test_yield_from` — error: codegen error: codegen error: define: Duplicate definition of identifier: _mb_4000203 (no axis-1 issue mapped, baseline=Fail)

**type error: called value is not a function** (2 seeds):

- `test_iter` — error: type error at 38702..38708: called value is not a function (no axis-1 issue mapped, baseline=Fail)
- `test_with` — error: type error at 28002..28008: called value is not a function (no axis-1 issue mapped, baseline=Fail)

**type error: argument type mismatch** (1 seeds):

- `test_range` — error: type error at 17878..17880: argument type mismatch: expected `int`, got `list[Any]` (no axis-1 issue mapped, baseline=Fail)

### Needs-asserts seeds (30)

These exit 0 but emit no `MAMBA_ASSERTION_PASS:` marker — they are skeletons. Each needs real assertions plus the marker line, modelled on `tests/cpython/lib_test_seeds/seed/test_math.py`. This is the primary work item for each linked axis-1 stdlib issue.

- `sentinel_assertion_fail` — no axis-1 issue mapped, baseline=Stub
- `sentinel_assertion_pass` — no axis-1 issue mapped, baseline=Stub
- `test_atexit` — #3430, baseline=Stub
- `test_base64` — #3409, baseline=Stub
- `test_bool` — no axis-1 issue mapped, baseline=Stub
- `test_calendar` — #3391, baseline=Stub
- `test_codeop` — no axis-1 issue mapped, baseline=Stub
- `test_csv` — #3405, baseline=Stub
- `test_difflib` — #3386, baseline=Stub
- `test_errno` — no axis-1 issue mapped, baseline=Stub
- `test_float` — no axis-1 issue mapped, baseline=Stub
- `test_fnmatch` — #3401, baseline=Stub
- `test_glob` — #3400, baseline=Stub
- `test_html` — #3420, baseline=Stub
- `test_int` — no axis-1 issue mapped, baseline=Stub
- `test_keyword` — no axis-1 issue mapped, baseline=Stub
- `test_list` — no axis-1 issue mapped, baseline=Stub
- `test_locale` — #3454, baseline=Stub
- `test_pkgutil` — #3448, baseline=Stub
- `test_sched` — no axis-1 issue mapped, baseline=Stub
- `test_secrets` — #3413, baseline=Stub
- `test_set` — no axis-1 issue mapped, baseline=Stub
- `test_slice` — no axis-1 issue mapped, baseline=Stub
- `test_strftime` — no axis-1 issue mapped, baseline=Stub
- `test_string` — #3383, baseline=Stub
- `test_struct` — #3387, baseline=Stub
- `test_syntax` — no axis-1 issue mapped, baseline=Stub
- `test_textwrap` — #3385, baseline=Stub
- `test_tuple` — no axis-1 issue mapped, baseline=Stub
- `test_unpack` — no axis-1 issue mapped, baseline=Stub

### Unrunnable seeds (1)

Timeouts, panics, or signals. Each needs a mamba runtime-gap or stability issue before the seed is meaningful.

- `test_poplib` — exit 101: thread 'main' (219386380) panicked at projects/mamba/src/codegen/cranelift/jit.rs:841:77: (no axis-1 issue mapped, baseline=Fail)

## Methodology

- Binary: `target/release/mamba` (current build on `project-mamba`).
- Per seed: `mamba run <seed>` wrapped in a 60s wall-clock timeout (perl `alarm`-based shim; macOS has no `timeout(1)`).
- Concurrency: 6 workers via `xargs -P 6`. Raw stdout+stderr captured to `/tmp/axis1-audit-raw/<seed>.txt` (not committed).
- AssertionPass marker: any stdout line matching `MAMBA_ASSERTION_PASS:\s+\S+\s+(\d+)\s+asserts` (per #2540).
- First-error extraction: first line matching `(error:|Error|Traceback|panicked|signal|SIGSEGV|SIGBUS|NotImplemented|not implemented|Exception)`.
- Axis-1 issue mapping: title-substring match against open issues with label `axis:1` (see #3367–#3454 range).

