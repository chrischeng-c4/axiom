"""Hot-loop bench for `flask.Flask` / `flask.Blueprint` /
`flask.request` / `flask.__version__` module-attribute reads
(#1516).

End-user scenario: flask-using web apps re-resolve `Flask`
(app factory), `Blueprint` (route registration), `request`
(per-request proxy), and `__version__` (compat probe) on every
call site. Per-call attribute resolution goes through the
`flask` module's attribute table on each call site. That
per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `Flask`, `Blueprint`,
`request`, and `__version__` per iteration (ITERS scaled so
4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import flask


_F_BASELINE = flask.Flask
_B_BASELINE = flask.Blueprint
_R_BASELINE = flask.request
_V_BASELINE = flask.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = flask.Flask
    b = flask.Blueprint
    c = flask.request
    d = flask.__version__
    if (a is _F_BASELINE
            and b is _B_BASELINE
            and c is _R_BASELINE
            and d is _V_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"flask module-attribute read acc drift: acc={acc} expected={ITERS}"
print("flask_type_read_hot:", acc)
