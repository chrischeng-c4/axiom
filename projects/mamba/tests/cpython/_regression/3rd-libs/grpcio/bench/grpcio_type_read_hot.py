"""Hot-loop bench for `grpc.Server` / `grpc.Channel` /
`grpc.insecure_channel` / `grpc.__version__` module-attribute
reads (#1515).

End-user scenario: grpcio-using services re-resolve `Server`
(server bind), `Channel` (secure client), `insecure_channel`
(plain client constructor), and `__version__` (compat probe)
on every call site. Per-call attribute resolution goes through
the `grpc` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `Server`, `Channel`,
`insecure_channel`, and `__version__` per iteration (ITERS
scaled so 4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import grpc


_S_BASELINE = grpc.Server
_C_BASELINE = grpc.Channel
_IC_BASELINE = grpc.insecure_channel
_V_BASELINE = grpc.__version__

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = grpc.Server
    b = grpc.Channel
    c = grpc.insecure_channel
    d = grpc.__version__
    if (a is _S_BASELINE
            and b is _C_BASELINE
            and c is _IC_BASELINE
            and d is _V_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"grpc module-attribute read acc drift: acc={acc} expected={ITERS}"
print("grpcio_type_read_hot:", acc)
