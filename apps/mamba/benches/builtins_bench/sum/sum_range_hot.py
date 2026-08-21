"""sum() over range — builtin perf bench.

End-user scenario: `sum(range(N))` repeated, the most basic
aggregation idiom — used in histogram totals, mean computation,
and quick sanity checks. CPython has a special long-int fast path
in builtin_sum; mamba's mb_sum iterates the range and accumulates.

Bounded context (DDD): builtins_bench/sum.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
ITERS = 10_000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    total = total + sum(range(N))
_t1 = time.perf_counter()

print("sum_range_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * (N * (N - 1) // 2)
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
