"""Hot-loop bench for `alembic.__version__` / `alembic.context` /
`alembic.op` / `alembic.EnvironmentContext` module-attribute reads
(#1524).

End-user scenario: alembic-using migration scripts re-resolve
`__version__` (compat probe), `context` (env.py context object),
`op` (DDL operations facade), and `EnvironmentContext` (migration
env class) on every call site. Per-call attribute resolution goes
through the `alembic` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `context`, `op`,
and `EnvironmentContext` per iteration (ITERS scaled so 4 attrs x
20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import alembic


_V_BASELINE = alembic.__version__
_C_BASELINE = alembic.context
_O_BASELINE = alembic.op
_E_BASELINE = alembic.EnvironmentContext

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = alembic.__version__
    b = alembic.context
    c = alembic.op
    d = alembic.EnvironmentContext
    if (a is _V_BASELINE
            and b is _C_BASELINE
            and c is _O_BASELINE
            and d is _E_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"alembic module-attribute read acc drift: acc={acc} expected={ITERS}"
print("alembic_type_read_hot:", acc)
