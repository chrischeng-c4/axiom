"""Allocation-shaped hot-loop bench for `random.sample` (Task #40).

Informational fixture. Each iteration draws a k=100 sample from a
1000-element pool. Per-call cost is partial Fisher-Yates plus the
allocation of a length-100 list. Predicted **balanced** with
allocation pressure — falls into #2096 subset A territory if k goes
large.

# tier: compute
"""

import random

random.seed(42)
POP = list(range(1000))
sm = random.sample
ITERS = 1000
K = 100

# Stream-independent invariant: every sample yields exactly K elements
# drawn from [0, 1000). Count iterations where len() and bounds hold.
acc = 0
for _ in range(ITERS):
    out = sm(POP, K)
    if len(out) == K and 0 <= out[0] < 1000:
        acc += 1
print("sample_alloc:", acc)
