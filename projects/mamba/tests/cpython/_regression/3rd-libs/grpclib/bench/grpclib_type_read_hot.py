"""Hot-loop bench for `grpclib.Server` / `grpclib.Channel` /
`grpclib.Stream` / `grpclib.__version__` module-attribute reads (#1514).

End-user scenario: grpclib-using services re-resolve `Server`
(server bind), `Channel` (client connect), `Stream` (rpc stream
helper), and `__version__` (compat probe) on every call site.
Per-call attribute resolution goes through the `grpclib` module's
attribute table on each call site. That per-call module-attribute
quadruple-read is the workload measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `Server`, `Channel`, `Stream`,
and `__version__` per iteration (ITERS scaled so 4 attrs x 20_000
= ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import grpclib


_S_BASELINE = grpclib.Server
_C_BASELINE = grpclib.Channel
_ST_BASELINE = grpclib.Stream
_V_BASELINE = grpclib.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = grpclib.Server
    b = grpclib.Channel
    c = grpclib.Stream
    d = grpclib.__version__
    if (a is _S_BASELINE
            and b is _C_BASELINE
            and c is _ST_BASELINE
            and d is _V_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"grpclib module-attribute read acc drift: acc={acc} expected={ITERS}"
print("grpclib_type_read_hot:", acc)
