"""Bulk-iterable bench for `statistics.fmean` (Task #41 Wave-2 ship #3).

End-user scenario: a numeric pipeline computes the arithmetic mean of a
1M-element float list, repeated 50 times. Welford summation in mamba's
shim; CPython 3.12 uses a high-precision accumulator (slightly slower than
naive sum). Predicted regime: balanced wall 2-5x, internal 0.6-1.0x
(per scout doc - same bulk-iter shape as math.fsum which cleared 1.02x
internal compute-dominated).

The 1M element list is read every iteration; no per-iter allocation amplifier
(see GH #2111). Memory should remain flat.

Hoist `fmean = statistics.fmean` to dodge module-attr lookup (#2097).

# tier: compute
"""

import statistics

fmean = statistics.fmean

N = 1_000_000
# Deterministic non-trivial data; avoid pure-arithmetic-progression that
# CPython might constant-fold or hardware-prefetch perfectly.
DATA = [i * 0.5 + i * 0.0001 for i in range(N)]

ITERS = 50

acc = 0.0
for _ in range(ITERS):
    acc += fmean(DATA)
print("stat_fmean_bulk:", int(acc))
