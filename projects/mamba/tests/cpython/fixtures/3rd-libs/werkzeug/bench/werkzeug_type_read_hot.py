"""Hot-loop bench for `werkzeug.Request` / `werkzeug.Response` /
`werkzeug.Local` / `werkzeug.__version__` module-attribute reads
(#1517).

End-user scenario: werkzeug-using WSGI apps re-resolve `Request`
(req wrapper), `Response` (resp wrapper), `Local` (thread-local
proxy), and `__version__` (compat probe) on every call site.
Per-call attribute resolution goes through the `werkzeug`
module's attribute table on each call site. That per-call
module-attribute quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `Request`, `Response`, `Local`,
and `__version__` per iteration (ITERS scaled so 4 attrs x 20_000
= ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import werkzeug


_RQ_BASELINE = werkzeug.Request
_RS_BASELINE = werkzeug.Response
_L_BASELINE = werkzeug.Local
_V_BASELINE = werkzeug.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = werkzeug.Request
    b = werkzeug.Response
    c = werkzeug.Local
    d = werkzeug.__version__
    if (a is _RQ_BASELINE
            and b is _RS_BASELINE
            and c is _L_BASELINE
            and d is _V_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"werkzeug module-attribute read acc drift: acc={acc} expected={ITERS}"
print("werkzeug_type_read_hot:", acc)
