"""Scalar per-call hot-loop bench for `random.random` (Task #40).

Same shape as math.scalar_sqrt — a tight 100k loop calling
`random.random()` and accumulating the float. Module-attr hoist
convention (#2097): `r = random.random` BEFORE the hot loop so the
JIT sees a stable callee.

# tier: compute
"""

import random

random.seed(42)
r = random.random

ITERS = 100_000

# Stream-independent invariant: every random() returns a float in [0,1).
# Count in-range results so the stdout match doesn't depend on PRNG
# stream parity between rand_mt::Mt and CPython MT19937 seed-init.
acc = 0
for _ in range(ITERS):
    x = r()
    if 0.0 <= x < 1.0:
        acc += 1
print("scalar_random:", acc)
