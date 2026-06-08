"""Hot-loop bench for `kombu.__version__` / `kombu.Connection` /
`kombu.Exchange` / `kombu.Queue` module-attribute reads (#1531).

End-user scenario: kombu-using messaging code re-resolves
`__version__` (compat probe), `Connection` (broker connection),
`Exchange` (exchange descriptor), and `Queue` (queue descriptor) on
every call site. Per-call attribute resolution goes through the
`kombu` module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `Connection`,
`Exchange`, and `Queue` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import kombu


_V_BASELINE = kombu.__version__
_C_BASELINE = kombu.Connection
_E_BASELINE = kombu.Exchange
_Q_BASELINE = kombu.Queue

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = kombu.__version__
    b = kombu.Connection
    c = kombu.Exchange
    d = kombu.Queue
    if (a is _V_BASELINE
            and b is _C_BASELINE
            and c is _E_BASELINE
            and d is _Q_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"kombu module-attribute read acc drift: acc={acc} expected={ITERS}"
print("kombu_type_read_hot:", acc)
