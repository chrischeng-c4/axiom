"""Float addition hot-loop bench — language-core arithmetic perf.

End-user scenario: tight accumulator loop over float + float, the
inner loop of every Riemann-sum / Monte Carlo / mean-of-stream
workload. Mamba's force-typed JIT lowers this to a native FADD per
iteration; CPython 3.12 dispatches through PyNumber_Add → float_add,
each side a PyFloat heap object.

Bounded context (DDD): language_bench/arithmetic.

Tier: compute (mamba should beat CPython ≥10×).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

Checksum: closed-form sum-of-arithmetic-progression gives the exact
expected accumulator value. We use `math.isclose` because float
rounding accumulates over 1M adds and the exact bit-level result
varies with summation order; both CPython and mamba should land
inside the tolerance.
"""

import math
import sys
import time

ITERS = 1_000_000
STEP = 0.5

acc = 0.0
_t0 = time.perf_counter()
for i in range(ITERS):
    acc = acc + STEP
_t1 = time.perf_counter()

print("float_add_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Closed form: ITERS additions of STEP, starting from 0.0.
expected = ITERS * STEP
assert math.isclose(acc, expected, rel_tol=1e-9), (
    f"float checksum mismatch: {acc} != {expected}"
)
