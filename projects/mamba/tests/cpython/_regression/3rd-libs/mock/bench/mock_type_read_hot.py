"""Hot-loop bench for `mock.__version__` / `mock.Mock` / `mock.MagicMock`
/ `mock.patch` module-attribute reads (#1528).

End-user scenario: mock-using test suites re-resolve `__version__`
(compat probe), `Mock` (mock factory), `MagicMock` (auto-spec mock),
and `patch` (decorator/context manager) on every call site. Per-call
attribute resolution goes through the `mock` module's attribute table
on each call site. That per-call module-attribute quadruple-read is
the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `Mock`, `MagicMock`,
and `patch` per iteration (ITERS scaled so 4 attrs x 20_000 = ~80k
attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import mock


_V_BASELINE = mock.__version__
_M_BASELINE = mock.Mock
_MM_BASELINE = mock.MagicMock
_P_BASELINE = mock.patch

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = mock.__version__
    b = mock.Mock
    c = mock.MagicMock
    d = mock.patch
    if (a is _V_BASELINE
            and b is _M_BASELINE
            and c is _MM_BASELINE
            and d is _P_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"mock module-attribute read acc drift: acc={acc} expected={ITERS}"
print("mock_type_read_hot:", acc)
