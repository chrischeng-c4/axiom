"""binascii.hexlify — bytes-to-hex perf bench.

End-user scenario: `hexlify(payload)` inside a tight loop, the
canonical bytes-as-printable primitive that backs every log dump /
debug emit / hash-as-hex render / wire-trace inspector. CPython
routes through binascii.b2a_hex (C-level table-lookup); mamba's
binascii should hit the same native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/binascii.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `hexlify` to a local before the hot loop.
"""

import binascii
import sys
import time

_hexlify = binascii.hexlify

N = 1000
payloads = [(f"hash-{i:08d}").encode("ascii") for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for p in payloads:
        total = total + len(_hexlify(p))
_t1 = time.perf_counter()

print("hexlify_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in payloads:
    per_iter = per_iter + len(_hexlify(p))
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
