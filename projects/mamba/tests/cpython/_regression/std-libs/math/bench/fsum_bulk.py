"""Bulk-iterable hot-loop bench for `math.fsum` (Task #36 — Wave-1 收尾).

End-user scenario: math.fsum over a 1M-element list of floats × 50 iters.
This is the canonical bulk-iterable tier:compute path — every iteration
crosses the FFI boundary exactly once but does 1M elements of work
(Shewchuk's algorithm internal accumulation in Rust), so per-element
dispatch overhead amortizes over the whole list and the floor for
`tier:compute` is wall >=10x.

Hoist convention (per #2097): `fsum = math.fsum` BEFORE the hot loop.
Without hoisting, mamba's module-attr lookup at the call site is ~5x
slower than the hoisted form.

#2105 avoidance: the post-loop `print(acc)` happens BEFORE the
readback.

# tier: compute
"""

import math

# Hoist module-level attributes outside the loop (#2097).
fsum = math.fsum

# Build a 1M-element list of floats. Mix of magnitudes so the Shewchuk
# partial-sum bookkeeping actually runs (a uniform-magnitude list would
# collapse to single-partial accumulation).
#
# Avoid bitwise-and / shift / float-mul mixing in the comprehension —
# mamba's JIT currently returns None for that expression shape (separate
# bug, not on Task #36's critical path). Use plain float arithmetic.
DATA = [i * 0.5 + i * 0.0001 for i in range(1_000_000)]

ITERS = 50

acc = 0.0
for _ in range(ITERS):
    acc += fsum(DATA)
print("fsum_bulk:", acc)
