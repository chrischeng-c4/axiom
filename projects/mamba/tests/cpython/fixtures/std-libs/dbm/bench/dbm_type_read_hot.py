"""Hot-loop bench for `dbm.open` / `dbm.whichdb` / `dbm.error`
module-attribute reads (#1261).

End-user scenario: dbm-using key-value-store code re-resolves
`open` (backend selector), `whichdb` (file detector), and `error`
(exception tuple) on every call site. Per-call attribute
resolution goes through the `dbm` module's attribute table on
each call site. That per-call module-attribute triple-read is
the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `open`, `whichdb`, and `error`
per iteration (ITERS scaled so 3 attrs x 20_000 = ~60k attr-reads
per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import dbm


_OPEN_BASELINE = dbm.open
_WD_BASELINE = dbm.whichdb
_E_BASELINE = dbm.error

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = dbm.open
    b = dbm.whichdb
    c = dbm.error
    if (a is _OPEN_BASELINE
            and b is _WD_BASELINE
            and c is _E_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"dbm module-attribute read acc drift: acc={acc} expected={ITERS}"
print("dbm_type_read_hot:", acc)
