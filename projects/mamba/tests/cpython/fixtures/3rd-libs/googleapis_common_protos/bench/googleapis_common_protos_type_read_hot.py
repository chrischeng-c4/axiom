"""Hot-loop bench for `google.rpc.status_pb2` /
`google.rpc.code_pb2` /
`google.rpc.error_details_pb2` /
`google.rpc.context_pb2` module-attribute reads (#1512).

End-user scenario: googleapis-common-protos-using services
re-resolve `google.rpc.status_pb2` (rpc status protos),
`google.rpc.code_pb2` (canonical error codes),
`google.rpc.error_details_pb2` (extended error details),
and `google.rpc.context_pb2` (rpc context protos) on every
call site. Per-call attribute resolution goes through the
`google.rpc` module's attribute table on each call site.
That per-call module-attribute quadruple-read is the workload
measured here.

Tier: `runtime module-attr read` (target mamba/cpython <= 1.0x).

Workload: 20_000 paired reads of `status_pb2`, `code_pb2`,
`error_details_pb2`, and `context_pb2` per iteration (ITERS scaled
so 4 attrs x 20_000 = ~80k attr-reads per run).

DoD: exits 0 under both CPython and mamba; the cross-runtime bench
marker on stderr) and reports the ratio. Floor is 1.0x per #1265
Goal 2.
"""

import google.rpc


_S_BASELINE = google.rpc.status_pb2
_C_BASELINE = google.rpc.code_pb2
_E_BASELINE = google.rpc.error_details_pb2
_CT_BASELINE = google.rpc.context_pb2

ITERS = 20_000

acc = 0
for _ in range(ITERS):
    a = google.rpc.status_pb2
    b = google.rpc.code_pb2
    c = google.rpc.error_details_pb2
    d = google.rpc.context_pb2
    if (a is _S_BASELINE
            and b is _C_BASELINE
            and c is _E_BASELINE
            and d is _CT_BASELINE):
        acc = acc + 1

assert acc - ITERS == 0, f"google.rpc module-attribute read acc drift: acc={acc} expected={ITERS}"
print("googleapis_common_protos_type_read_hot:", acc)
