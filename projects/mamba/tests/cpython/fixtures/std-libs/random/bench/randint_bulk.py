"""Scalar per-call hot-loop bench for `random.randint` (Task #40).

100k iterations of `rint(0, 100)`. Module-attr hoist (#2097) so the
JIT can see a stable callee. Per-call discrete-uniform sampling
should be in the same regime as scalar_random (compute-tier, mostly
startup-dominated for small inner work).

# tier: compute
"""

import random

random.seed(42)
rint = random.randint

ITERS = 100_000

# Stream-independent invariant: rint(0, 100) ∈ [0, 100]. Count in-bound
# results so stdout match doesn't depend on PRNG stream parity.
acc = 0
for _ in range(ITERS):
    x = rint(0, 100)
    if 0 <= x <= 100:
        acc += 1
print("randint_bulk:", acc)
