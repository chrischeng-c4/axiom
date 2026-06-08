"""bytes.hex — bytes-to-hex via method perf bench.

End-user scenario: `payload.hex()` inside a tight loop, the canonical
modern bytes-as-printable that backs every JSON-serialise-bytes /
log-bytes / hash-as-hex render. Pairs with [[binascii.hexlify]] which
is the legacy spelling. CPython routes through unicodeobject's
bytes_hex; mamba's bytes.hex should hit the same native impl via the
typed bridge.

Bounded context (DDD): stdlib_bench/bytes.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: no module attr to hoist — bound method per payload is the
direct call shape used in production.
"""

import sys
import time

N = 1000
payloads = [(f"hash-{i:08d}").encode("ascii") for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for p in payloads:
        total = total + len(p.hex())
_t1 = time.perf_counter()

print("hex_method_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in payloads:
    per_iter = per_iter + len(p.hex())
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
