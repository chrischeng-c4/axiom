"""collections.deque — append/popleft FIFO perf bench.

End-user scenario: `q.append(x)` then `q.popleft()` inside a tight
loop, the canonical FIFO queue primitive that backs every BFS / job
queue / sliding-window aggregator. CPython routes through C-level
_collections.deque (doubly-linked block list); mamba's
collections.deque should hit a native impl through its typed bridge.

Bounded context (DDD): stdlib_bench/collections.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `deque` to a local.

NOTE: mamba's `len(deque)` returns 0 (known gap); checksum derives
total from popleft return values, not len(d).
"""

import collections
import sys
import time

_deque = collections.deque

N = 1000
# Mamba ~235x slower on this path; cap ITERS to keep wall under 10s.
ITERS = 200

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    q = _deque()
    for i in range(N):
        q.append(i)
    s = 0
    for _j in range(N):
        s = s + q.popleft() + (_j & 0)
    total = total + s
_t1 = time.perf_counter()

print("deque_append_popleft_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Sum of 0..N-1 per iter = N*(N-1)/2; total across ITERS:
ref = N * (N - 1) // 2
expected = ITERS * ref
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
