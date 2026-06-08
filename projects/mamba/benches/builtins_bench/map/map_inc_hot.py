"""map() over list — builtin perf bench.

End-user scenario: `list(map(fn, xs))` materializing a transformed
list, the canonical functional-style transform that competes with
list comprehensions in everyday Python code (data prep, token
normalization, type coercion). CPython routes through PyIter_Next
+ PyObject_CallOneArg per step; mamba's map lowers to a fused
iter that reuses the per-step callable address.

Bounded context (DDD): builtins_bench/map.

Tier: compute.

#2105: print of `total` happens BEFORE the INTERNAL_TIME_NS marker.
"""

import sys
import time


def inc(x):
    return x + 1


N = 1000
xs = list(range(N))
ITERS = 1000

total = 0
_t0 = time.perf_counter()
for _ in range(ITERS):
    ys = list(map(inc, xs))
    total = total + len(ys)
_t1 = time.perf_counter()

print("map_inc_hot:", total)
print(f"INTERNAL_TIME_NS={int((_t1 - _t0) * 1_000_000_000)}", file=sys.stderr, flush=True)

expected = ITERS * N
diff = total - expected
assert diff == 0, f"checksum mismatch: {total} - {expected} = {diff}"
