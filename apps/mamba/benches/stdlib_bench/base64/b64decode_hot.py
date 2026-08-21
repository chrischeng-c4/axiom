"""base64.b64decode — b64-to-bytes decode perf bench.

End-user scenario: `base64.b64decode(token)` inside a tight loop, the
inverse of the Basic Auth header / JWT signature / data: URL pipeline.
Every API gateway parsing a `Authorization: Basic …` header lands
here. CPython routes through binascii.a2b_base64; mamba's base64
should hit the same native path through its typed bridge.

Bounded context (DDD): stdlib_bench/base64.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `b64decode` to a local before the hot loop.
"""

import base64
import sys
import time

_b64encode = base64.b64encode
_b64decode = base64.b64decode

N = 1000
payloads = [(f"row-{i:08d}-payload-bytes").encode("ascii") for i in range(N)]
encoded = [_b64encode(p) for p in payloads]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for e in encoded:
        total = total + len(_b64decode(e))
_t1 = time.perf_counter()

print("b64decode_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in payloads:
    per_iter = per_iter + len(p)
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
