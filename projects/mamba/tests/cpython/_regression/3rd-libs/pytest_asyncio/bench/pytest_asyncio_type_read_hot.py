"""Hot-loop bench for `pytest_asyncio.__version__` /
`pytest_asyncio.fixture` / `pytest_asyncio.is_async_test` /
`pytest_asyncio.Mode` module-attribute reads (#1526).

End-user scenario: pytest-asyncio-using test suites re-resolve
`__version__` (compat probe), `fixture` (async fixture decorator),
`is_async_test` (introspection helper), and `Mode` (auto/strict
config enum) on every call site. Per-call attribute resolution
goes through the `pytest_asyncio` module's attribute table on each
call site. That per-call module-attribute quadruple-read is the
workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `fixture`,
`is_async_test`, and `Mode` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import pytest_asyncio


_V_BASELINE = pytest_asyncio.__version__
_F_BASELINE = pytest_asyncio.fixture
_I_BASELINE = pytest_asyncio.is_async_test
_M_BASELINE = pytest_asyncio.Mode

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = pytest_asyncio.__version__
    b = pytest_asyncio.fixture
    c = pytest_asyncio.is_async_test
    d = pytest_asyncio.Mode
    if (a is _V_BASELINE
            and b is _F_BASELINE
            and c is _I_BASELINE
            and d is _M_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"pytest_asyncio module-attribute read acc drift: acc={acc} expected={ITERS}"
print("pytest_asyncio_type_read_hot:", acc)
