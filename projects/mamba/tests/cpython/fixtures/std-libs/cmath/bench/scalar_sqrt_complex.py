"""Scalar per-iter `cmath.sqrt(complex(i, i)).real` (Task #38, Wave-2 ship #1).

Predicted regime: startup-dominated (per scout doc — 10-20× wall, 1-3×
internal). 100k iters × ~50 ns inner work ≈ 5 ms; dwarfed by ~200 ms
CPython startup. Mirrors math/scalar_sqrt.py shape (Task #36 cleared
3.21× internal PASS) — same module-scalar callable shape, no callback.

Hoist convention (#2097): `sqrt = cmath.sqrt` BEFORE the hot loop so
each iter is a direct func call instead of an attribute lookup.

# tier: compute
"""

import cmath

# Hoist module-level attribute outside the loop (#2097).
sqrt = cmath.sqrt

ITERS = 100_000

acc_re = 0.0
for i in range(1, ITERS + 1):
    acc_re += sqrt(complex(i, i)).real
print("cmath_scalar_sqrt:", int(acc_re))
