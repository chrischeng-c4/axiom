"""random.Random.gauss — normal-distribution sample perf bench.

End-user scenario: `r.gauss(mu, sigma)` inside a tight loop, the
canonical normal-deviate sampler that backs every Monte-Carlo path
simulator / synthetic-feature jitter / Bayesian-posterior prior draw /
stochastic-policy noise injection. CPython routes through
Random.gauss (Python-level Box-Muller transform with a per-instance
cache); mamba's random should hit the same Python path through its
typed bridge.

Bounded context (DDD): stdlib_bench/random.

Tier: compute (with FP math + cache state).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `gauss` is a bound method on a Random instance.
DO NOT hoist `_gauss = r.gauss` — bound-method hoist on Random
silently returns None/0 under mamba (same shape as `_randint` quirk).

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import math
import random
import sys
import time

MU = 0.0
SIGMA = 1.0
N = 200
ITERS = 2000

r = random.Random(2026)

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for inner in range(N):
        s = s + r.gauss(MU, SIGMA)
    acc = acc + s
_t1 = time.perf_counter()

print("gauss_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Gauss is RNG-dependent — we only sanity-check sample volume + that
# the accumulator stayed finite. With N*ITERS draws from N(0,1), the
# Central Limit Theorem says |acc| ≤ ~6 * sqrt(N*ITERS) almost surely.
# Use subtraction-style ordering (mamba accumulator-vs-arith bug).
abs_acc = acc if acc >= 0.0 else -acc
bound = 6.0 * math.sqrt(float(N * ITERS))
diff = bound - abs_acc
assert diff >= 0.0, f"acc outside 6-sigma CLT bound: |{acc}| > {bound}"
