"""List comprehension hot-loop bench — language-core comprehension perf.

End-user scenario: tight loop building `[x*x for x in range(N)]`,
the canonical map-style comprehension that every Python data
pipeline uses. CPython has dedicated LIST_APPEND bytecode + the
PyListObject pre-sized growth strategy; mamba lowers comprehensions
to direct Vec<MbValue>::push.

Bounded context (DDD): language_bench/comprehensions — first member
of the language-core comprehension perf suite.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
ITERS = 1000  # build a 1000-elem list 1000 times = 1M elements total

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    xs = [x * x for x in range(N)]
    total = total + len(xs)
_t1 = time.perf_counter()

print("listcomp_square_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = N * ITERS
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
