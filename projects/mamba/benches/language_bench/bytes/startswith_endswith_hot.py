"""bytes.startswith / bytes.endswith — prefix/suffix-match perf bench.

End-user scenario: `b.startswith(b"GET ")` / `b.endswith(b".png")`
inside a tight loop, the canonical byte-prefix/suffix-classify primitive
that backs every HTTP-method router / file-extension filter / log-line
prefix matcher / wire-frame demultiplexer. CPython routes through
bytes_startswith / bytes_endswith (C-level memcmp on a single arena);
mamba's bytes should hit a native impl through its typed bridge.

Bounded context (DDD): language_bench/bytes.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `startswith` / `endswith` are bytes methods; no module-attr
hoisting needed. DO NOT hoist `_sw = b.startswith` — mamba force-typed
bound-method hoist silently returns None on some receivers.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

LINES = [
    b"GET /index.html HTTP/1.1",
    b"POST /api/v1/data HTTP/1.1",
    b"PUT /resource/42 HTTP/2.0",
    b"DELETE /tmp HTTP/1.0",
    b"HEAD /favicon.ico HTTP/1.1",
    b"OPTIONS * HTTP/1.1",
    b"PATCH /thing HTTP/1.1",
    b"TRACE /diag HTTP/1.1",
]
ITERS = 30000

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for line in LINES:
        if line.startswith(b"GET "):
            s = s + 1
        if line.startswith(b"POST "):
            s = s + 2
        if line.endswith(b"HTTP/1.1"):
            s = s + 4
        if line.endswith(b"HTTP/2.0"):
            s = s + 8
    acc = acc + s
_t1 = time.perf_counter()

print("startswith_endswith_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for line in LINES:
    if line.startswith(b"GET "):
        per_iter = per_iter + 1
    if line.startswith(b"POST "):
        per_iter = per_iter + 2
    if line.endswith(b"HTTP/1.1"):
        per_iter = per_iter + 4
    if line.endswith(b"HTTP/2.0"):
        per_iter = per_iter + 8
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
