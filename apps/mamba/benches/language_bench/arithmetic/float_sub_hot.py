"""Float subtraction (-) hot-loop bench — arithmetic perf.

End-user scenario: tight loop over `x - y` float subtract ops, the
foundation of every elapsed-time delta (`now - start`) / centered-
coordinate offset (`pos - origin`) / drawdown-from-peak metric
(`peak - current`) / temperature-delta computer (`reading - baseline`).
CPython routes through float_sub (C-level IEEE-754 sub); mamba's
float should hit a native f64 sub path through its typed bridge.

Distinct from `float_add_hot.py` (+) — same FPU latency but exercises
the sign-flip path; some implementations route a-b as a + (-b).

Bounded context (DDD): language_bench/arithmetic.

Tier: compute (pure FPU sub, no allocation).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `-` is a syntax op — no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars. Use `math.isclose` for FP checksum.
"""

import math
import sys
import time

N = 1000
ITERS = 5000
OFFSET = 1.5

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for i in range(N):
        s = s + (float(i) - OFFSET)
    acc = acc + s
_t1 = time.perf_counter()

print("float_sub_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0.0
for i in range(N):
    per_iter = per_iter + (float(i) - OFFSET)
expected = float(ITERS) * per_iter
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum mismatch: {acc} vs {expected}"
