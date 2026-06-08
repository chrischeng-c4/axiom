"""statistics.mean — arithmetic-mean perf bench.

End-user scenario: `mean(values)` inside a tight loop, the canonical
descriptive-stats primitive that backs every metrics window aggregate /
A/B-test point estimate / latency report. CPython routes through
statistics.mean (pure Python — Fraction-precision sum); mamba's
statistics should hit the same algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/statistics.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `mean` to a local before the hot loop.
"""

import math
import statistics
import sys
import time

_mean = statistics.mean

N = 100
xs = [float(i + 1) for i in range(N)]
ITERS = 1000

acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + _mean(xs)
_t1 = time.perf_counter()

print("mean_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# mean of 1..N = (N+1)/2 = 50.5 for N=100
ref = (N + 1) / 2.0
expected = ITERS * ref
assert math.isclose(acc, expected, rel_tol=1e-9, abs_tol=1e-3), (
    f"checksum mismatch: {acc} vs {expected}"
)
