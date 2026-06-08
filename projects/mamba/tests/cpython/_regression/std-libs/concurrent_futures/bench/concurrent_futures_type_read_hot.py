"""Hot-loop bench for `concurrent.futures.ThreadPoolExecutor` /
`concurrent.futures.ProcessPoolExecutor` /
`concurrent.futures.Future` / `concurrent.futures.as_completed`
module-attribute reads (#1261).

End-user scenario: concurrent.futures-using parallel code
re-resolves `ThreadPoolExecutor` (thread pool), `ProcessPoolExecutor`
(process pool), `Future` (handle class), and `as_completed`
(fan-in iterator) on every call site. Per-call attribute
resolution goes through the `concurrent.futures` module's
attribute table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `ThreadPoolExecutor`,
`ProcessPoolExecutor`, `Future`, and `as_completed` per iteration
(ITERS scaled so 4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import concurrent.futures as cf


_TPE_BASELINE = cf.ThreadPoolExecutor
_PPE_BASELINE = cf.ProcessPoolExecutor
_FUT_BASELINE = cf.Future
_AC_BASELINE = cf.as_completed

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = cf.ThreadPoolExecutor
    b = cf.ProcessPoolExecutor
    c = cf.Future
    d = cf.as_completed
    if (a is _TPE_BASELINE
            and b is _PPE_BASELINE
            and c is _FUT_BASELINE
            and d is _AC_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"concurrent.futures module-attribute read acc drift: acc={acc} expected={ITERS}"
print("concurrent_futures_type_read_hot:", acc)
