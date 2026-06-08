"""struct.pack — int-to-bytes pack perf bench.

End-user scenario: `struct.pack('>I', n)` inside a tight loop,
the canonical fixed-width binary encode that backs every
network header emit / on-disk record write / protocol frame.
CPython routes through pack_into with a precompiled format-
char table; mamba's struct.pack lowers typed format strings to
a native bswap + store on the JIT path.

Bounded context (DDD): stdlib_bench/struct.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `pack` to a local before the hot loop.
"""

import struct
import sys
import time

_pack = struct.pack

N = 1000
xs = [(i * 1103515245 + 12345) & 0xFFFFFFFF for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        b = _pack(">I", x)
        total = total + len(b)
_t1 = time.perf_counter()

print("pack_int_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Every pack >I produces 4 bytes.
expected = ITERS * N * 4
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
