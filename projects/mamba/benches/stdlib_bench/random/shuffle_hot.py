"""random.shuffle — in-place permutation perf bench.

End-user scenario: `random.shuffle(xs)` inside a tight loop, the
canonical in-place permutation primitive that backs every minibatch
shuffler / dealer-of-cards / test-data jitter / cross-validation
fold randomizer. CPython routes through Python random.shuffle
(Fisher-Yates over Mersenne Twister); mamba's random should hit
the same algorithm through its typed bridge.

Bounded context (DDD): stdlib_bench/random.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `shuffle` to a local.
"""

import random
import sys
import time

_shuffle = random.shuffle

N = 1000
ITERS = 1000
random.seed(42)
xs = list(range(N))

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    _shuffle(xs)
    acc = acc + xs[0]
_t1 = time.perf_counter()

print("shuffle_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Stochastic — verify in-place shuffle does happen (xs is a permutation).
xs_set = set(xs)
assert len(xs_set) == N, f"shuffle lost elements: {len(xs_set)} vs {N}"
assert min(xs) == 0 and max(xs) == N - 1, "shuffle range damaged"
