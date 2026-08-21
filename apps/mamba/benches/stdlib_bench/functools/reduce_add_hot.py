"""functools.reduce — left-fold sum perf bench.

End-user scenario: `reduce(lambda a, b: a + b, xs, 0)` —
canonical pure-Python left-fold (running totals, aggregate
products, custom monoid reductions). CPython routes through
functools_reduce + per-step PyObject_CallObject; mamba's
functools.reduce inlines the callable when it is a simple
lambda the JIT can specialize.

Bounded context (DDD): stdlib_bench/functools.

Tier: compute.

#2105: print of `acc` happens BEFORE the INTERNAL_TIME_NS marker.
#2097: hoist `reduce` to a local before the hot loop to avoid
module-attribute lookup churn.
"""

import sys
import time
from functools import reduce

_reduce = reduce


def _add(a, b):
    return a + b


N = 1000
xs = list(range(N))
ITERS = 1000

acc = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    acc = acc + _reduce(_add, xs, 0)
_t1 = time.perf_counter()

print("reduce_add_hot:", acc)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

# Per outer iter: sum(range(N)) = N*(N-1)//2.
expected = ITERS * (N * (N - 1) // 2)
diff = acc - expected
assert diff == 0, f"checksum mismatch: {acc} - {expected} = {diff}"
