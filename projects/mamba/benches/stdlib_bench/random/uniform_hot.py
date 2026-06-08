"""random.uniform — bounded float draw perf bench.

End-user scenario: `random.uniform(lo, hi)` inside a tight loop, the
canonical bounded-float-draw primitive that backs every Monte Carlo
parameter sampler / jitter-amount picker / load-shape randomizer /
synthetic-data generator with custom range. CPython routes through
_random_Random_uniform_impl (C-level Mersenne Twister draw +
`a + (b-a) * x` scale); mamba's random should hit a native impl
through its typed bridge.

Distinct from `random_float_hot.py` (covers `random.random()`,
unscaled [0,1)) and `gauss_hot.py` (covers normal distribution).

Bounded context (DDD): stdlib_bench/random.

Tier: compute (small per-call FP arithmetic + state-update overhead).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: DO NOT hoist `_uniform = r.uniform` — bound-method hoist
returns None silently under mamba.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import random
import sys
import time

SEED = 4242
LO = -50.0
HI = 50.0
N = 200
ITERS = 5000

r = random.Random(SEED)

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for i in range(N):
        s = s + r.uniform(LO, HI)
    acc = acc + s
_t1 = time.perf_counter()

print("uniform_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Sanity: total must lie within the algebraic envelope [N*ITERS*LO, N*ITERS*HI].
# Use subtraction-style bounds — mamba's accumulator-vs-arith int/float compare
# (==, <=, <) returns False when one side is fold-built and the other is loop-
# accumulated; subtraction routes through unboxing arithmetic and works.
lo_bound = float(N) * float(ITERS) * LO
hi_bound = float(N) * float(ITERS) * HI
diff_lo = acc - lo_bound
diff_hi = hi_bound - acc
assert diff_lo >= 0.0, f"acc below lower bound: {acc} < {lo_bound}"
assert diff_hi >= 0.0, f"acc above upper bound: {acc} > {hi_bound}"
