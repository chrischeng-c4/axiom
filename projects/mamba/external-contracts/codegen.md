# codegen — external contract (as-is, 2026-07-15)

Substrate rule (`README.md`, this dir): codegen compiles EVERY run-dimension fixture — the whole 46,620-fixture
corpus minus `type/` walls is its ambient positive contract; a compile reject outside `type/` is a defect by
definition. Below are only the proof-bearing subsets this domain owns directly. Map: `../tech-design/codegen/ARCHITECTURE.md`.

## Positive contract — fixtures that must RUN and byte-match the python3.12 oracle

Live counts under `tests/cpython/` (counted 2026-07-15; xfail = acknowledged gap, still part of the contract):

| Fixture set | .py | xfail | proves |
|---|---|---|---|
| `behavior/core/sys_settrace/` | 281 | 275 | trace-event emission |
| `behavior/std-libs/sys_setprofile/` | 29 | 27 | profile-event emission |
| `{behavior 48, surface 8, errors 3, real_world 1}/std-libs/bdb/` | 60 | 35 | debugger baseline |
| `{behavior 27, surface 2}/std-libs/trace/` | 29 | 27 | trace-module baseline |
| `_regression/core/class_system/` | 50 | 8 | `pending_class_*` drain order |
| `_regression/core/mro_super/` | 5 | 0 | register → bases ordering |
| `_regression/core/language/` | 145 | 2 | core-statement lowering |
| `behavior/core/descr/` | 154 | 141 | descriptor dispatch emission (shared w/ object-model) |
| `_regression/core/jit/` | 19 | 0 | 16 `# RUN: jit` in-process pipeline fixtures (`conformance_pipeline`) + 3 oracle fixtures |
| `**/bench/` | 252 | 1 | C2 substrate workloads |
| `perf/` | 74 | 74 | pin-owned; xfail by design (#2239), asserted via pins below |

Direct-owned total (excl. bench/perf): 772 fixtures / 515 xfail. C2: 128 pin TOMLs in
`tests/harness/cpython/config/perf/pins/` (`contract.rs` floor ≥100) — recursion-guard fast path, rc-elision,
loop-carried preseed cost. Shared with memory: gc/stability soaks + corpus-wide absence of hang/SIGTRAP/SIGBUS
proves `JIT_LOCK` discipline and leak boundedness.

## Negative contract — what must be REJECTED

None owned. All 8,769 `type/` walls belong to type-system; the 2 `# RUN: typecheck` pipeline fixtures are
type-system's. Codegen's obligation is the inverse: it must never reject a non-`type/` fixture.

## Known contract gaps

- settrace `'exception'` under-emission — fires once at the raising frame, not per unwound frame; drives 275/281
  sys_settrace xfails (dominant cluster 212/275) and the all-xfail `behavior/std-libs/trace`. tracked: #1535;
  topic: `../tech-design/codegen/tracing-and-frames.md`.
- The bdb+trace "regression guard" named in that topic's EC surface mostly does not execute: 62/89 xfail. tracked: #1768/#1535.
- `behavior/core/descr` is 141/154 xfail; the un-xfail probe measured ~20% hidden-PASS corpus-wide (85% in the
  seed-pass stratum) and codegen's dirs have not been swept yet — stale greens likely. tracked: #1768.
- Ghost bench paths: 26 committed `.gitkeep` bench dirs in the retired lib-first layout
  (`tests/cpython/{std-libs,3rd-libs}/<lib>/bench/`) sit outside the runner allowlist — fixtures authored there
  are never run; `perf_pin.rs:10`'s doc example cites the dead path. tracked: #1767.
- Vacuous parse smokes: `src/driver/tests/behavioral_lang.rs:707-746` walks 5 retired dirs (incl. `core/language`,
  `core/class_system`) and passes while parsing 0 files. tracked: #1767.
- 17 `xfail_zero_conformance.rs:run_fixture` lib tests re-derive cargo-gate verdicts with a pyenv-shim oracle
  (~1.47s/spawn), no oracle cache, no scratch-CWD isolation; the in-process JIT-pipeline execution is the distinct
  signal worth keeping. tracked: #1771.
- `--emit hir|mir` dump tooling has zero fixture coverage, and `check` honors only `Ast` (`driver/mod.rs:139`). tracked: #1771 family.
- Manifests: only `config/manifests/std-libs/bdb.toml` + `core/cpython321_core_lang.toml` cover this domain's
  dirs; sys_settrace/descr/class_system were auto-port-authored with no manifest of their own.

## Verification

```bash
# focused inner loop (seconds; runner-parity verdicts, shared oracle cache; set MAMBA_BIN after a release build)
python3 tests/harness/cpython/tools/sweep.py tests/cpython/behavior/core/sys_settrace \
  tests/cpython/_regression/core/{class_system,mro_super,language,jit} tests/cpython/behavior/core/descr
cargo test -p mamba --release --test conformance_pipeline          # in-process `# RUN: jit` fixtures
cargo test -p mamba --release --test perf_pin -- perf_pin          # C2 slice (opt-in)
cargo test -p mamba --release --test conformance                   # full C1 gate (~3 min); codegen = substrate
```

Verdict semantics, xfail-is-skip rot, sweep load-degradation, and JIT_LOCK hazards: `HARNESS.md` (this dir).
