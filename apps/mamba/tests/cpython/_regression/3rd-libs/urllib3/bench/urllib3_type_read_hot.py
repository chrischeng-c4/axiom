"""Hot-loop bench for `urllib3.__version__` / `urllib3.PoolManager`
/ `urllib3.Retry` / `urllib3.Timeout` module-attribute reads
(#1482).

End-user scenario: urllib3-using HTTP client code re-resolves
`__version__` (compat probe), `PoolManager` (connection pool
factory), `Retry` (retry policy), and `Timeout` (timeout policy) on
every call site. Per-call attribute resolution goes through the
`urllib3` module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `PoolManager`,
`Retry`, and `Timeout` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import urllib3


_V_BASELINE = urllib3.__version__
_PM_BASELINE = urllib3.PoolManager
_R_BASELINE = urllib3.Retry
_T_BASELINE = urllib3.Timeout

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = urllib3.__version__
    b = urllib3.PoolManager
    c = urllib3.Retry
    d = urllib3.Timeout
    if (a is _V_BASELINE
            and b is _PM_BASELINE
            and c is _R_BASELINE
            and d is _T_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"urllib3 module-attribute read acc drift: acc={acc} expected={ITERS}"
print("urllib3_type_read_hot:", acc)
