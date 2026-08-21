"""Integer floor-division (//) hot-loop bench — arithmetic perf.

End-user scenario: tight loop over `n // d` integer divisions, the
foundation of every bucket-index quantizer (`offset // bucket_size`) /
page-number computation (`item_idx // page_size`) / tile-coverage
calculator (`pixel // tile_dim`) / time-bucket rounder (`ts // window`).
CPython routes through long_divrem (C-level multi-precision long
division); mamba's int division should hit a native i64 div path
through its typed bridge.

Distinct from `int_add_hot.py` and `int_mul_hot.py` (single-cycle
operations); floor-div is a multi-cycle CPU op + Python sign-correction
overhead.

Bounded context (DDD): language_bench/arithmetic.

Tier: compute (with branch on negative-result for floor semantics).

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: `//` is a syntax op — no hoisting concern.

NOTE: nested `for _ in range(...)` loops corrupt outer `_` under mamba;
use named loop vars.
"""

import sys
import time

N = 1000
ITERS = 5000
DIVISOR = 7

acc = 0
_t0 = time.perf_counter()
for outer in range(ITERS):
    s = 0
    for i in range(N):
        s = s + (i // DIVISOR)
    acc = acc + s
_t1 = time.perf_counter()

print("int_div_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for i in range(N):
    per_iter = per_iter + (i // DIVISOR)
expected = ITERS * per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
