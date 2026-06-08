"""Hot-loop bench for `psycopg.__version__` / `psycopg.connect` /
`psycopg.Connection` / `psycopg.Cursor` module-attribute reads
(#1532).

End-user scenario: psycopg-using db code re-resolves `__version__`
(compat probe), `connect` (factory function), `Connection` (typed
connection class), and `Cursor` (typed cursor class) on every call
site. Per-call attribute resolution goes through the `psycopg`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `connect`,
`Connection`, and `Cursor` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import psycopg


_V_BASELINE = psycopg.__version__
_CO_BASELINE = psycopg.connect
_CN_BASELINE = psycopg.Connection
_CU_BASELINE = psycopg.Cursor

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = psycopg.__version__
    b = psycopg.connect
    c = psycopg.Connection
    d = psycopg.Cursor
    if (a is _V_BASELINE
            and b is _CO_BASELINE
            and c is _CN_BASELINE
            and d is _CU_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"psycopg module-attribute read acc drift: acc={acc} expected={ITERS}"
print("psycopg_type_read_hot:", acc)
