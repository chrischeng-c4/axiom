"""collections.Counter — bulk count from iterable perf bench.

End-user scenario: `Counter(xs)` over an int sequence, the
canonical bulk frequency count that backs every histogram /
mode / top-k pipeline. CPython's Counter._count_elements has a
C fast path for sequences; mamba's collections.Counter is a
Python-level wrap over a dict accumulator.

Bounded context (DDD): stdlib_bench/collections.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `Counter` to a local before the hot loop.
"""

import sys
import time
from collections import Counter

_Counter = Counter

N = 1000
xs = [(i & 0x0F) for i in range(N)]
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    c = _Counter(xs)
    total = total + len(c)
_t1 = time.perf_counter()

print("counter_count_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Counter over xs (values 0..15) has exactly 16 distinct keys.
expected = ITERS * 16
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
