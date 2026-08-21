"""bisect.bisect_left — sorted-index search perf bench.

End-user scenario: `bisect_left(sorted_xs, target)` inside a tight
loop, the canonical sorted-search primitive that backs every
range-bucket assigner / percentile-rank lookup / sorted-key index
seek / step-function evaluator. CPython routes through C-level
_bisect.bisect_left; mamba's bisect should hit the same algorithm
through its typed bridge.

Bounded context (DDD): stdlib_bench/bisect.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `bisect_left` to a local.
"""

import bisect
import sys
import time

_bisect_left = bisect.bisect_left

N = 1000
sorted_xs = list(range(N))
targets = [(i * 13 + 7) % N for i in range(N)]
# Mamba ~9x slower on this path; cap ITERS to keep wall under ~5s.
ITERS = 200

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0
    for t in targets:
        s = s + _bisect_left(sorted_xs, t)
    total = total + s
_t1 = time.perf_counter()

print("bisect_left_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for t in targets:
    per_iter = per_iter + _bisect_left(sorted_xs, t)
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
