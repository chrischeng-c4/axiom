"""random.choice — uniform-pick perf bench.

End-user scenario: `random.choice(pool)` inside a tight loop, the
canonical uniform-pick primitive that backs every dice-roll /
ab-test arm selection / synthetic-data scaffold / load-balancer
shuffle pick. CPython routes through Python random.choice (uses
_random Mersenne Twister); mamba's random should hit the same
algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/random.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `choice` to a local.
"""

import random
import sys
import time

_choice = random.choice

POOL = list(range(100))
N = 1000
ITERS = 100
random.seed(42)

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0
    for _i in range(N):
        s = s + _choice(POOL) + (_i & 0)
    acc = acc + s
_t1 = time.perf_counter()

print("choice_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Stochastic — only assert non-trivial work happened.
assert acc > 0, "no picks accumulated"
