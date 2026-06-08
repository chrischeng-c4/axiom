"""itertools.product — cartesian-product enumeration perf bench.

End-user scenario: `for a, b in product(xs, ys)` inside a tight
loop, the canonical pairwise enumeration primitive that backs every
grid sweep / hyperparameter search / pair-of-things scan / two-axis
report. CPython routes through C-level _itertools.product; mamba's
itertools.product should hit the same algorithm through its typed
bridge.

Bounded context (DDD): stdlib_bench/itertools.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `product` to a local.
"""

import itertools
import sys
import time

_product = itertools.product

A = list(range(20))
B = list(range(50))
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    s = 0
    for a, b in _product(A, B):
        s = s + a + b
    total = total + s
_t1 = time.perf_counter()

print("product_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

per_iter = 0
for a, b in _product(A, B):
    per_iter = per_iter + a + b
expected = ITERS * per_iter
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
