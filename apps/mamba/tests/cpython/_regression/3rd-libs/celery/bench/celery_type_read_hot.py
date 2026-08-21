"""Hot-loop bench for `celery.__version__` / `celery.Celery` /
`celery.shared_task` / `celery.signature` module-attribute reads
(#1530).

End-user scenario: celery-using worker code re-resolves `__version__`
(compat probe), `Celery` (app class), `shared_task` (task decorator),
and `signature` (subtask constructor) on every call site. Per-call
attribute resolution goes through the `celery` module's attribute
table on each call site. That per-call module-attribute quadruple-read
is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `Celery`,
`shared_task`, and `signature` per iteration (ITERS scaled so 4 attrs
x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import celery


_V_BASELINE = celery.__version__
_C_BASELINE = celery.Celery
_ST_BASELINE = celery.shared_task
_SIG_BASELINE = celery.signature

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = celery.__version__
    b = celery.Celery
    c = celery.shared_task
    d = celery.signature
    if (a is _V_BASELINE
            and b is _C_BASELINE
            and c is _ST_BASELINE
            and d is _SIG_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"celery module-attribute read acc drift: acc={acc} expected={ITERS}"
print("celery_type_read_hot:", acc)
