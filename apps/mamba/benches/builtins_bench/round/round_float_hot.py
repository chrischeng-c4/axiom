"""round() on float — builtin perf bench.

End-user scenario: `round(x, ndigits)` inside a tight loop, the
canonical decimal-truncation idiom used by every
display-formatter / metric-emitter / price-rounding path.
CPython routes through builtin_round -> float.__round__ with
banker's-rounding; mamba's mb_round lowers to a native f64 op.

Bounded context (DDD): builtins_bench/round.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.

The accumulator uses repeated `+` on the rounded float to keep
the per-iteration body identical between runtimes; the checksum
is delta-bounded with math.isclose so the inherent FP drift
from accumulating ~1M rounded floats doesn't trip the assert.
"""

import math
import sys
import time

N = 1000
xs = [(i + 0.123456789) for i in range(N)]
ITERS = 1000

acc = 0.0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        acc = acc + round(x, 3)
_t1 = time.perf_counter()

print("round_float_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Reference sums the same rounded values once, then scales.
ref_per_iter = 0.0
for x in xs:
    ref_per_iter = ref_per_iter + round(x, 3)
expected = ITERS * ref_per_iter
assert math.isclose(acc, expected, rel_tol=1e-9, abs_tol=1e-3), (
    f"checksum mismatch: {acc} vs {expected}"
)
