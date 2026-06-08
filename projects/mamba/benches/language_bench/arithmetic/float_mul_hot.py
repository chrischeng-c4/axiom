"""Float multiplication (*) hot-loop bench — arithmetic perf.

End-user scenario: tight loop over `x * y` float multiply ops, the
foundation of every scaled-coordinate transform (`x * scale`) /
percent-to-fraction normalizer (`pct * 0.01`) / dt-weighted physics
step (`v * dt`) / depreciation/decay step (`value * factor`). CPython
routes through float_mul (C-level f64 multiply); mamba's float should
hit a native f64 mul path through its typed bridge.

Distinct from `float_add_hot.py` and `float_div_hot.py` — multiply
is single-cycle pipelined on modern CPUs (faster than div, similar
to add for f64). The win signal here is whether mamba's typed bridge
fuses the loop tightly enough to expose that throughput.

Bounded context (DDD): language_bench/arithmetic.

Tier: compute (pure FPU mul, no allocation).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `*` is a syntax op — no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars. Use `math.isclose` for FP checksum.
"""

import math
import sys
import time

N = 1000
ITERS = 5000
FACTOR = 1.0001

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for i in range(N):
        s = s + (float(i) * FACTOR)
    acc = acc + s
_t1 = time.perf_counter()

print("float_mul_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0.0
for i in range(N):
    per_iter = per_iter + (float(i) * FACTOR)
expected = float(ITERS) * per_iter
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum mismatch: {acc} vs {expected}"
