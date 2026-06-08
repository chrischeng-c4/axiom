"""Float division (/) hot-loop bench — arithmetic perf.

End-user scenario: tight loop over `x / y` float division ops, the
foundation of every average-of-N reducer (`total / n`) / pixel-aspect
normalizer (`w / h`) / rate-per-second computer (`count / elapsed`) /
unit-conversion divider (`px / dpi`). CPython routes through
float_div (C-level IEEE-754 div + zero-check); mamba's float should
hit a native f64 div path through its typed bridge.

Distinct from `int_div_hot.py` (//) — same divide instruction at the
hardware level but no flooring step, and the result is f64 not i64.
On most pipelines f64 div is the slowest scalar FP op (long latency,
not pipelined) — perf-floor matters.

Bounded context (DDD): language_bench/arithmetic.

Tier: compute (pure FPU div, no allocation).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `/` is a syntax op — no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars. Use `math.isclose` for FP checksum — loop-built
vs fold-built float sums differ in low bits.
"""

import math
import sys
import time

N = 1000
ITERS = 5000
DIVISOR = 7.0

acc = 0.0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0.0
    for i in range(N):
        s = s + (float(i) / DIVISOR)
    acc = acc + s
_t1 = time.perf_counter()

print("float_div_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0.0
for i in range(N):
    per_iter = per_iter + (float(i) / DIVISOR)
expected = float(ITERS) * per_iter
assert math.isclose(acc, expected, rel_tol=1e-9), f"checksum mismatch: {acc} vs {expected}"
