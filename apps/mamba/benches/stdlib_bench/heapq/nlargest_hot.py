"""heapq.nlargest — top-k over iterable perf bench.

End-user scenario: `heapq.nlargest(k, iterable)` inside a tight loop,
the canonical top-k primitive that backs every leaderboard / hot-list
pick / top-N-by-frequency report. CPython routes through heapq.nlargest
(Python with a C heappushpop hot inner); mamba's heapq should hit the
same algorithm — partial-heap of size k — through its typed bridge.

Bounded context (DDD): stdlib_bench/heapq.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `nlargest` to a local before the hot loop.
"""

import heapq
import sys
import time

_nlargest = heapq.nlargest

N = 1000
xs = [(i * 1103515245 + 12345) & 0xFFFFFFFF for i in range(N)]
K = 10
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    top = _nlargest(K, xs)
    total = total + len(top)
_t1 = time.perf_counter()

print("nlargest_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Each call returns exactly K elements (since N >> K).
expected = ITERS * K
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
