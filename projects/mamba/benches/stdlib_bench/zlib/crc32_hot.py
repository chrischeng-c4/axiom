"""zlib.crc32 — checksum perf bench.

End-user scenario: `zlib.crc32(payload)` inside a tight loop, the
canonical fast-checksum primitive that backs every chunk-integrity
check / shard-routing key / cache-validity tag / replication
sentinel. CPython routes through zlib_crc32 (C-level CRC32 with HW
SSE4.2 on x86 / NEON on ARM); mamba's zlib should hit the same
native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/zlib.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `crc32` to a local before the hot loop.
"""

import zlib
import sys
import time

_crc32 = zlib.crc32

N = 1000
payloads = [(f"chunk-{i:08d}-payload").encode("ascii") for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for p in payloads:
        total = total + (_crc32(p) & 0xFFFF)
_t1 = time.perf_counter()

print("crc32_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for p in payloads:
    per_iter = per_iter + (_crc32(p) & 0xFFFF)
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
