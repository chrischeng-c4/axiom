"""Hot-loop bench for `gunicorn.__version__` / `gunicorn.SERVER` /
`gunicorn.SERVER_SOFTWARE` / `gunicorn.version_info` module-attribute
reads (#1522).

End-user scenario: gunicorn-using WSGI server bootstraps re-resolve
`__version__` (compat probe), `SERVER` / `SERVER_SOFTWARE`
(per-response server-header constants), and `version_info` (tuple
compat probe) on every call site. Per-call attribute resolution goes
through the `gunicorn` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `__version__`, `SERVER`,
`SERVER_SOFTWARE`, and `version_info` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import gunicorn


_V_BASELINE = gunicorn.__version__
_S_BASELINE = gunicorn.SERVER
_SS_BASELINE = gunicorn.SERVER_SOFTWARE
_VI_BASELINE = gunicorn.version_info

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = gunicorn.__version__
    b = gunicorn.SERVER
    c = gunicorn.SERVER_SOFTWARE
    d = gunicorn.version_info
    if (a is _V_BASELINE
            and b is _S_BASELINE
            and c is _SS_BASELINE
            and d is _VI_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"gunicorn module-attribute read acc drift: acc={acc} expected={ITERS}"
print("gunicorn_type_read_hot:", acc)
