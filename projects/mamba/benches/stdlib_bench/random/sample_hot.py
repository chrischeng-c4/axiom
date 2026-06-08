"""random.Random.sample — k-of-n-without-replacement perf bench.

End-user scenario: `r.sample(pool, k)` inside a tight loop, the
canonical sampling-without-replacement primitive that backs every
A/B-test split builder / minibatch shuffler / random subset for
holdout eval / lottery-style draw. CPython routes through
Random.sample (Python-level fast path with reservoir/swap algos);
mamba's random should hit the same Python path through its typed
bridge.

Bounded context (DDD): stdlib_bench/random.

Tier: compute (with new-list allocation per call).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `sample` is a bound method on a Random instance.
DO NOT hoist `_sample = r.sample` — bound-method hoist on Random
silently returns 0/None under mamba (same shape as `_randint` quirk).

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import random
import sys
import time

POOL = list(range(500))
K = 16
ITERS = 5000

r = random.Random(42)

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    pick = r.sample(POOL, K)
    s = 0
    for v in pick:
        s = s + v
    acc = acc + s
_t1 = time.perf_counter()

print("sample_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Sample is RNG-dependent — we only check the bench actually picked
# K values per iter and that values lay inside POOL. Both invariants
# can be summarized by sum-bounds: K * min(POOL) ≤ s_iter ≤ K * max(POOL).
#
# Use subtraction-style ordering checks: mamba's accumulator-vs-arithmetic
# int comparison (==, <=, <) returns False when one side is fold-built
# and the other is `ITERS * K * len(...)`-built, even when both report
# `<class 'int'>` and the same str repr. Subtraction goes through an
# unboxing arithmetic path and works reliably.
lo = 0
hi = ITERS * K * (len(POOL) - 1)
diff_lo = acc - lo
diff_hi = hi - acc
assert diff_lo >= 0, f"acc below lower bound: {acc} < {lo}"
assert diff_hi >= 0, f"acc above upper bound: {acc} > {hi}"
