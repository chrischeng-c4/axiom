"""base64.b64encode — bytes-to-b64 encode perf bench.

End-user scenario: `base64.b64encode(payload)` inside a tight loop,
the canonical binary-safe wire encoding that backs every Basic Auth
header / JWT signature segment / data: URL embed / SMTP/MIME body.
CPython routes through binascii.b2a_base64 (a C-level table-lookup
loop); mamba's base64 should delegate to a native impl behind the
typed bridge.

Bounded context (DDD): stdlib_bench/base64.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `b64encode` to a local before the hot loop.
"""

import base64
import sys
import time

_b64encode = base64.b64encode

N = 1000
payloads = [(f"row-{i:08d}-payload-bytes").encode("ascii") for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for p in payloads:
        total = total + len(_b64encode(p))
_t1 = time.perf_counter()

print("b64encode_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in payloads:
    per_iter = per_iter + len(_b64encode(p))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
