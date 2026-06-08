"""itertools.combinations — n-choose-k enumeration perf bench.

End-user scenario: `combinations(items, 2)` inside a tight loop, the
canonical n-choose-k primitive that backs every pair-distance compute /
collinear-test scan / all-pairs feature interaction / minimum spanning
tree edge enumeration. CPython routes through C-level
_itertools.combinations; mamba's itertools should hit the same
algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `combinations` to a local.
"""

import itertools
import sys
import time

_combinations = itertools.combinations

ITEMS = list(range(30))
K = 2
ITERS = 500

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0
    for a, b in _combinations(ITEMS, K):
        s = s + a + b
    total = total + s
_t1 = time.perf_counter()

print("combinations_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for a, b in _combinations(ITEMS, K):
    per_iter = per_iter + a + b
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
