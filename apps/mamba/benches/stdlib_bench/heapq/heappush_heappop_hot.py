"""heapq.heappush + heappop — priority-queue churn perf bench.

End-user scenario: `heappush(h, x)` then `heappop(h)` inside a tight
loop, the canonical priority-queue primitive that backs every
Dijkstra shortest-path / event-loop scheduler / top-K stream /
median-of-stream / Huffman-coding build. CPython routes through
C-level _heapq.heappush + heappop; mamba's heapq should hit a
native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/heapq.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `heappush`/`heappop` to locals.
"""

import heapq
import sys
import time

_heappush = heapq.heappush
_heappop = heapq.heappop

N = 1000
xs = [(i * 31 + 7) % 1009 for i in range(N)]
ITERS = 200

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    h = []
    for x in xs:
        _heappush(h, x)
    s = 0
    for _i in range(N):
        s = s + _heappop(h) + (_i & 0)
    total = total + s
_t1 = time.perf_counter()

print("heappush_heappop_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = sum(xs)
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
