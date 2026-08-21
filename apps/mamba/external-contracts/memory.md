# memory — external contract (as-is, 2026-07-15)

Domain map: `tech-design/memory/ARCHITECTURE.md` (EC surface section). Verdict law: `HARNESS.md`.
Jurisdiction is symptom-first: memory owns `gc/` + `stability/` dirs AND the absence of hang/SIGTRAP
anywhere in the 46k corpus — a UAF from a wrong refcount surfaces in unrelated fixtures.

## Positive contract — must RUN and byte-match the python3.12 oracle

| Fixture dir (`tests/cpython/`) | live `.py` | xfail | notes |
|---|---|---|---|
| `behavior/std-libs/gc/` | 48 | 35 | only 13 execute; 34 xfails are blanket "promotion pending", 1 owned (soak, below) |
| `surface/std-libs/gc/` | 25 | 0 | generated `api_*_is_present` probes (manifest below) |
| `_regression/core/stability/` | 1 | 0 | `heap_churn_soak.py`, env-tunable `MAMBA_SOAK_{OUTER,ROWS,WIDTH}` |

Regression anchors owned by symptom (live, not xfail'd — from the fix-family TDs):

- `_regression/builtin-libs/list_methods/reentrancy.py` — escape-analysis flow-sensitivity (`tech-design/memory/object-lifetime.md` §Escape analysis licenses GC-tracking elision)
- `behavior/std-libs/tempfile/temporary_directory_cleanup_on_exit.py` + `real_world/std-libs/errno/translate_oserror_errno_to_name.py` — with-exit retain contract (`tech-design/memory/object-lifetime.md` §With-protocol refcount contract); intermittency class ⇒ single-sample runs prove nothing, run repeatedly

Corpus-wide clause: zero hang / zero SIGTRAP across the full C1 gate is part of this domain's
positive contract (wrong-`NonEscaping` and under-retain bugs are delayed-symptom classes).

## Negative contract — must be REJECTED

`type/std-libs/gc/` — 4 walls, 0 xfail'd (none of the corpus's 1,200 disabled walls are here):
`collect__generation_as_int_wrong.py`, `get_objects__generation_as_typed_wrong.py`,
`set_debug__flags_as_int_wrong.py`, `set_threshold__threshold0_as_int_wrong.py`.
Wall semantics belong to type-system's dimension rule; this domain only owes that the gc-module
argument walls stay red.

## Known contract gaps

- **Cycle collection has no running positive proof**: `behavior/std-libs/gc/self_referential_cycle_soak.py` is xfail'd — `gc.collect()` returns 0 for self-referential Python cycles (tracked: #1123/#1360). The domain's headline mechanism (gc.rs 4-phase collector) is currently pinned only by Rust unit tests.
- **35/48 behavior-gc fixtures skipped**; 34 markers unowned blanket text. Corpus-measured hidden-PASS rate ~20.7% suggests several are free greens; the gc bucket itself is an unmeasured stratum (tracked: #1768 un-xfail campaign). Runner has no xpass detection — xfail = full skip, rots silently (tracked: #1771).
- **No `errors/std-libs/gc/` or `real_world/std-libs/gc/` dirs** — the gc module has zero error-dimension and real-world coverage (tracked: #1770 family).
- **Module-scope hot-loop leak carve-out**: fresh per-iter VRegs bypass rebind release (jit.rs:653); monotonic leak with iteration count, no fixture pins it (tracked: #2111).
- **`__main__` epilogue release sweep deliberately disabled** (jit.rs:488) — entry-body locals are never released; re-enabling historically oscillated the gate (tracked: #1663 T4c5).
- **Generator carve-out**: stdlib "iterators" materialize as eager `List` (rc.rs ObjData) — RSS profile diverges from CPython on large iterations, visible only through C2 pins (tracked: #2182).
- **Safepoint API is no-op stubs** — per-thread GC, no stop-the-world (gc.rs; knowledge, see ARCHITECTURE hazards).
- **The 1627 anchors are fragile-by-nature** (intermittent UAF class, maskable by unrelated walls) — repeat-run discipline is part of the contract (tracked: #1772 family).
- **C2 mem-gate auto-pass**: pins whose mamba RSS ≤ the 26 MB fixed floor pass vacuously (HARNESS.md) — small pins prove nothing about this domain's RSS behavior.

## Verification

```bash
# focused fixture slice (paths relative to tests/cpython; set MAMBA_BIN — stale-binary trap)
MAMBA_BIN=target/release/mamba python3 tests/harness/cpython/tools/sweep.py \
  behavior/std-libs/gc surface/std-libs/gc type/std-libs/gc _regression/core/stability
# anchors, repeated (intermittency): loop the two 1627 fixtures + reentrancy.py ~20x at --jobs 1
# cargo-gate slice (datatest name filter)
cargo test -p mamba --release --test conformance -- std-libs/gc core/stability
# Rust-side proofs (escape analysis, typed-list layout, rc/gc unit tests)
cargo test -p mamba escape_analysis typed_list_layout
cargo test -p mamba --lib runtime::gc runtime::rc
# domain slice of the full gate: memory owns corpus-wide hang/SIGTRAP absence ⇒ full C1 run
cargo test -p mamba --release --test conformance          # ~3 min, 46k+
# C2: memory changes (threshold/hasher/SmallVec/parking_lot) need the full 130-pin sweep
cargo test -p mamba --release --test perf_pin -- perf_pin # incl. typed_list_sum_1075.toml
```

Surface manifest: `tests/harness/cpython/config/manifests/std-libs/cpython312_surface/gc.toml`
(generated — regen via `tools/sync_cpython_surface_manifest.py`, never hand-edit).
