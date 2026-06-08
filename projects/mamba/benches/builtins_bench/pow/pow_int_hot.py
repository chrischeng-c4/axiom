"""pow() int exponent — builtin perf bench.

End-user scenario: `pow(b, e)` inside a tight loop with small
positive integer exponent, the canonical exponentiation idiom
that backs every polynomial / power-series / scaling
expression. CPython routes through long_pow with a square-and-
multiply fast path; mamba's mb_pow lowers to a native loop on
the small-int path.

Bounded context (DDD): builtins_bench/pow.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time

N = 1000
xs = [(i & 0x1F) + 1 for i in range(N)]
E = 3
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    for x in xs:
        acc = acc + pow(x, E)
_t1 = time.perf_counter()

print("pow_int_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

ref_per_iter = 0
for x in xs:
    ref_per_iter = ref_per_iter + pow(x, E)
expected = ITERS * ref_per_iter
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
