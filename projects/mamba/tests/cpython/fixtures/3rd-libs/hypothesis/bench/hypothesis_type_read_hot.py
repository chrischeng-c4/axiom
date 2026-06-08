"""Hot-loop bench for `hypothesis.__version__` / `hypothesis.given`
/ `hypothesis.strategies` / `hypothesis.settings` module-attribute
reads (#1527).

End-user scenario: hypothesis-using property-test suites re-resolve
`__version__` (compat probe), `given` (test decorator), `strategies`
(strategy factory namespace), and `settings` (config builder) on
every call site. Per-call attribute resolution goes through the
`hypothesis` module's attribute table on each call site. That
per-call module-attribute quadruple-read is the workload measured
here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `given`,
`strategies`, and `settings` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import hypothesis


_V_BASELINE = hypothesis.__version__
_G_BASELINE = hypothesis.given
_ST_BASELINE = hypothesis.strategies
_SE_BASELINE = hypothesis.settings

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = hypothesis.__version__
    b = hypothesis.given
    c = hypothesis.strategies
    d = hypothesis.settings
    if (a is _V_BASELINE
            and b is _G_BASELINE
            and c is _ST_BASELINE
            and d is _SE_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"hypothesis module-attribute read acc drift: acc={acc} expected={ITERS}"
print("hypothesis_type_read_hot:", acc)
