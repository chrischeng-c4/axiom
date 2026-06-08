"""Hot-loop bench for `bdb.Bdb` / `bdb.BdbQuit` / `bdb.Breakpoint`
/ `bdb.set_trace` module-attribute reads (#1261).

End-user scenario: bdb-using debugger code re-resolves `Bdb`
(base debugger class), `BdbQuit` (exception class), `Breakpoint`
(breakpoint object class), and `set_trace` (entry point) on every
call site. Per-call attribute resolution goes through the `bdb`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `Bdb`, `BdbQuit`, `Breakpoint`,
and `set_trace` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import bdb


_B_BASELINE = bdb.Bdb
_BQ_BASELINE = bdb.BdbQuit
_BR_BASELINE = bdb.Breakpoint
_ST_BASELINE = bdb.set_trace

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = bdb.Bdb
    b = bdb.BdbQuit
    c = bdb.Breakpoint
    d = bdb.set_trace
    if (a is _B_BASELINE
            and b is _BQ_BASELINE
            and c is _BR_BASELINE
            and d is _ST_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"bdb module-attribute read acc drift: acc={acc} expected={ITERS}"
print("bdb_type_read_hot:", acc)
